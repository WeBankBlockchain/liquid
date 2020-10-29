// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{ExternalFnABIBuilder, ParamABI};
use liquid_macro::seq;
use liquid_prelude::{string::String, vec::Vec};
use liquid_primitives::types::address;

pub trait GenerateComponents<T = ()> {
    fn generate_components() -> Vec<ParamABI> {
        Vec::new()
    }
}

pub trait TyName {
    fn ty_name() -> String;
}

pub trait GenerateOutputs {
    fn generate_outputs(builder: &mut ExternalFnABIBuilder);
}

macro_rules! impl_primitive_tys {
    ($( $t:ty ),*) => {
        $(
            impl GenerateComponents for $t {}

            impl TyName for $t {
                fn ty_name() -> String {
                    std::str::from_utf8(&<$t as liquid_ty_mapping::MappingToSolidityType>::MAPPED_TYPE_NAME)
                    .unwrap()
                    .trim_end_matches(char::from(0)).into()
                }
            }
        )*
    };
}

impl_primitive_tys!(
    bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    String,
    address,
    ()
);

impl<T> TyName for Vec<T>
where
    T: TyName,
{
    fn ty_name() -> String {
        let mut sub_ty = <T as TyName>::ty_name();
        sub_ty.push_str("[]");
        sub_ty
    }
}

impl<T> GenerateComponents for Vec<T>
where
    T: GenerateComponents,
{
    fn generate_components() -> Vec<ParamABI> {
        <T as GenerateComponents>::generate_components()
    }
}

impl<T> GenerateOutputs for T
where
    T: TyName + GenerateComponents,
{
    fn generate_outputs(builder: &mut ExternalFnABIBuilder) {
        builder.output(
            <T as GenerateComponents>::generate_components(),
            <T as TyName>::ty_name(),
        );
    }
}

macro_rules! impl_for_tuple {
    ($first:tt,) => {
        impl<$first> GenerateOutputs for ($first,)
        where
            $first: GenerateOutputs,
        {
            fn generate_outputs(
                builder: &mut ExternalFnABIBuilder,
            ) {
                <$first as GenerateOutputs>::generate_outputs(builder);
            }
        }
    };
    ($first:tt, $($rest:tt,)+) => {
        impl<$first, $($rest),+> GenerateOutputs for ($first, $($rest),+)
        where
            $first: GenerateOutputs,
            $($rest: GenerateOutputs),*
        {
            fn generate_outputs(builder: &mut ExternalFnABIBuilder) {
                <$first as GenerateOutputs>::generate_outputs(builder);
                $(
                    <$rest as GenerateOutputs>::generate_outputs(builder);
                )*
            }
        }

        impl_for_tuple!($($rest,)+);
    }
}

seq!(N in 0..16 {
    impl_for_tuple!(#(T#N,)*);
});

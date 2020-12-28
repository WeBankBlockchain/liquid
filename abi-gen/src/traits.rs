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

use crate::*;
use cfg_if::cfg_if;
use liquid_macro::seq;
use liquid_prelude::{string::String, vec::Vec};
use liquid_primitives::types::*;
#[cfg(feature = "contract")]
use liquid_primitives::__LIQUID_GETTER_INDEX_PLACEHOLDER;

pub trait GenerateParamABI {
    fn generate_ty_name() -> String;

    fn generate_param_abi(name: String) -> ParamABI;
}

pub trait FnOutputBuilder {
    fn output(&mut self, param_abi: ParamABI);
}

pub trait GenerateOutputs {
    fn generate_outputs<B>(builder: &mut B)
    where
        B: FnOutputBuilder;
}

macro_rules! impl_for_primitive_tys {
    ($( $t:ty, )*) => {
        $(
            impl GenerateParamABI for $t {
                fn generate_ty_name() -> String {
                    std::str::from_utf8(
                        &<Self as liquid_ty_mapping::MappingToSolidityType>::MAPPED_TYPE_NAME,
                    )
                    .unwrap()
                    .trim_end_matches(char::from(0))
                    .into()
                }

                fn generate_param_abi(name: String) -> ParamABI {
                    TrivialABI::new(Self::generate_ty_name(), name).into()
                }
            }

            impl GenerateOutputs for $t
            {
                fn generate_outputs<B>(builder: &mut B)
                where
                    B: FnOutputBuilder,
                {
                    let param_abi = <Self as GenerateParamABI>::generate_param_abi("".into());
                    builder.output(param_abi);
                }
            }
        )*
    };
}

impl_for_primitive_tys!(
    bool, u8, u16, u32, u64, u128, u256, i8, i16, i32, i64, i128, i256, String, Address,
    Bytes,
);

seq!(N in 1..=32 {
    impl_for_primitive_tys!(#(Bytes#N,)*);
});

impl<T> GenerateParamABI for Vec<T>
where
    T: GenerateParamABI,
{
    fn generate_ty_name() -> String {
        let mut sub_ty = <T as GenerateParamABI>::generate_ty_name();
        sub_ty.push_str("[]");
        sub_ty
    }

    fn generate_param_abi(name: String) -> ParamABI {
        let param_abi = <T as GenerateParamABI>::generate_param_abi(name.clone());
        let components = match param_abi {
            ParamABI::Composite(composite_abi) => composite_abi.components,
            _ => Vec::new(),
        };

        CompositeABI {
            trivial: TrivialABI {
                name,
                ty: Self::generate_ty_name(),
            },
            components,
        }
        .into()
    }
}

impl<T> GenerateOutputs for Vec<T>
where
    T: GenerateParamABI,
{
    fn generate_outputs<B>(builder: &mut B)
    where
        B: FnOutputBuilder,
    {
        let param_abi = <Self as GenerateParamABI>::generate_param_abi("".into());
        builder.output(param_abi);
    }
}

impl<T, const N: usize> GenerateParamABI for [T; N]
where
    T: GenerateParamABI,
{
    fn generate_ty_name() -> String {
        let mut sub_ty = <T as GenerateParamABI>::generate_ty_name();
        sub_ty.push_str(&format!("[{}]", N));
        sub_ty
    }

    fn generate_param_abi(name: String) -> ParamABI {
        let param_abi = <T as GenerateParamABI>::generate_param_abi(name.clone());
        let components = match param_abi {
            ParamABI::Composite(composite_abi) => composite_abi.components,
            _ => Vec::new(),
        };

        CompositeABI {
            trivial: TrivialABI {
                name,
                ty: Self::generate_ty_name(),
            },
            components,
        }
        .into()
    }
}

impl<T, const N: usize> GenerateOutputs for [T; N]
where
    T: GenerateParamABI,
{
    fn generate_outputs<B>(builder: &mut B)
    where
        B: FnOutputBuilder,
    {
        let param_abi = <Self as GenerateParamABI>::generate_param_abi("".into());
        builder.output(param_abi);
    }
}

#[cfg(feature = "contract")]
impl GenerateParamABI for __LIQUID_GETTER_INDEX_PLACEHOLDER {
    fn generate_ty_name() -> String {
        String::new()
    }

    fn generate_param_abi(_: String) -> ParamABI {
        ParamABI::None
    }
}

macro_rules! impl_generate_outputs_for_tuple {
    ($first:tt,) => {
        impl<$first> GenerateOutputs for ($first,)
        where
            $first: GenerateParamABI
        {
            fn generate_outputs<B>(builder: &mut B)
            where
                B: FnOutputBuilder,
            {
                builder.output(
                    {
                        let param_abi = <$first as GenerateParamABI>::generate_param_abi("".into());
                        param_abi.into()
                    }
                );
            }
        }
    };
    ($first:tt, $($rest:tt,)+) => {
        impl<$first, $($rest,)+> GenerateOutputs for ($first, $($rest,)+)
        where
            $first: GenerateParamABI,
            $(
                $rest: GenerateParamABI,
            )*
        {
            fn generate_outputs<B>(builder: &mut B)
            where
                B: FnOutputBuilder
            {
                builder.output(
                    {
                        let param_abi = <$first as GenerateParamABI>::generate_param_abi("".into());
                        param_abi.into()
                    }
                );
                $(
                    builder.output(
                        {
                            let param_abi = <$rest as GenerateParamABI>::generate_param_abi("".into());
                            param_abi.into()
                        }
                    );
                )+
            }
        }

        impl_generate_outputs_for_tuple!($($rest,)+);
    }
}

seq!(N in 0..16 {
    impl_generate_outputs_for_tuple!(#(T#N,)*);
});

cfg_if! {
    if #[cfg(not(feature = "solidity-compatible"))] {
        impl<T> GenerateParamABI for Option<T>
        where
            T: GenerateParamABI
        {
            fn generate_ty_name() -> String {
                String::from("option")
            }

            fn generate_param_abi(name: String) -> ParamABI {
                OptionABI {
                    trivial: TrivialABI::new(Self::generate_ty_name(), name),
                    some: Box::new(<T as GenerateParamABI>::generate_param_abi("".into()).into())
                }
                .into()
            }
        }

        impl<T, E> GenerateParamABI for Result<T, E>
        where
            T: GenerateParamABI,
            E: GenerateParamABI,
        {
            fn generate_ty_name() -> String {
                String::from("result")
            }

            fn generate_param_abi(name: String) -> ParamABI {
                ResultABI {
                    trivial: TrivialABI::new(Self::generate_ty_name(), name),
                    ok: Box::new(<T as GenerateParamABI>::generate_param_abi("".into()).into()),
                    err: Box::new(<E as GenerateParamABI>::generate_param_abi("".into()).into()),
                }
                .into()
            }
        }

        macro_rules! impl_generate_param_abi_for_tuple {
            ($first:tt,) => {
                impl<$first> GenerateParamABI for ($first,)
                where
                    $first: GenerateParamABI
                {
                    fn generate_ty_name() -> String {
                        String::from("tuple")
                    }

                    fn generate_param_abi(name: String) -> ParamABI {
                        let param_abis = vec![<$first as GenerateParamABI>::generate_param_abi("".to_owned())];
                        CompositeABI {
                            trivial: TrivialABI::new(Self::generate_ty_name(), name),
                            components: param_abis,
                        }
                        .into()
                    }
                }
            };
            ($first:tt, $($rest:tt,)+) => {
                impl<$first, $($rest),+> GenerateParamABI for ($first, $($rest),+)
                where
                    $first: GenerateParamABI,
                    $($rest: GenerateParamABI),*
                {
                    fn generate_ty_name() -> String {
                        String::from("tuple")
                    }

                    fn generate_param_abi(name: String) -> ParamABI {
                        let mut param_abis = vec![<$first as GenerateParamABI>::generate_param_abi("".to_owned())];
                        $(
                            param_abis.push(<$rest as GenerateParamABI>::generate_param_abi("".to_owned()));
                        )+
                        CompositeABI {
                            trivial: TrivialABI::new(Self::generate_ty_name(), name),
                            components: param_abis,
                        }
                        .into()
                    }
                }

                impl_generate_param_abi_for_tuple!($($rest,)+);
            }
        }

        seq!(N in 0..16 {
            impl_generate_param_abi_for_tuple!(#(T#N,)*);
        });
    }
}

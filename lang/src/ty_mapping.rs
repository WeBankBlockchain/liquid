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

use crate::traits::{
    THIS_IS_NOT_A_VALID_LIQUID_INPUT_TYPE, THIS_IS_NOT_A_VALID_LIQUID_OUTPUT_TYPE,
};
use liquid_env::types::String;
use liquid_macro::seq;

pub trait SolTypeName {
    const NAME: &'static [u8];
}

pub trait SolTypeNameLen {
    const LEN: usize;
}

macro_rules! mapping_type_to_sol {
    ($origin_ty:ty, $mapped_ty:ident) => {
        impl SolTypeName for $origin_ty {
            const NAME: &'static [u8] = stringify!($mapped_ty).as_bytes();
        }

        impl SolTypeNameLen for $origin_ty {
            const LEN: usize = stringify!($mapped_ty).len();
        }

        impl THIS_IS_NOT_A_VALID_LIQUID_INPUT_TYPE for $origin_ty {}
        impl THIS_IS_NOT_A_VALID_LIQUID_OUTPUT_TYPE for $origin_ty {}
    };
}

mapping_type_to_sol!(u8, uint8);
mapping_type_to_sol!(u16, uint16);
mapping_type_to_sol!(u32, uint32);
mapping_type_to_sol!(u64, uint64);
mapping_type_to_sol!(u128, uint128);
mapping_type_to_sol!(i8, int8);
mapping_type_to_sol!(i16, int8);
mapping_type_to_sol!(i32, int32);
mapping_type_to_sol!(i128, int128);
mapping_type_to_sol!(bool, bool);
mapping_type_to_sol!(String, string);

impl SolTypeName for () {
    const NAME: &'static [u8] = b"";
}

impl SolTypeNameLen for () {
    const LEN: usize = 0;
}

impl THIS_IS_NOT_A_VALID_LIQUID_INPUT_TYPE for () {}
impl THIS_IS_NOT_A_VALID_LIQUID_OUTPUT_TYPE for () {}

macro_rules! impl_len {
    ($first:tt,) => {
        impl<$first> SolTypeNameLen for ($first,)
        where
            $first: SolTypeNameLen,
        {
            const LEN: usize = <$first as SolTypeNameLen>::LEN;
        }
    };
    ($first:tt, $($rest:tt,)+) => {
        impl <$first, $($rest),+> SolTypeNameLen for ($first, $($rest,)+)
        where
        $first: SolTypeNameLen,
        $($rest: SolTypeNameLen,)+
        {
            const LEN: usize = <$first as SolTypeNameLen>::LEN $(+ <$rest as SolTypeNameLen>::LEN + 1)+;
        }

        impl_len!($($rest,)+);
    };
}

seq!(N in 0..16 {
    impl_len! {
        #(T#N,)*
    }
});

pub const fn concat<T, E, const N: usize>() -> [u8; N]
where
    T: SolTypeName,
    E: SolTypeName,
{
    let a = <T as SolTypeName>::NAME;
    let b = <E as SolTypeName>::NAME;

    let mut ret = [0u8; N];
    let mut i = 0;
    while i < a.len() {
        ret[i] = a[i];
        i += 1;
    }

    ret[i] = 0x2c; //`,`
    i += 1;

    let mut j = 0;
    while j < b.len() {
        ret[i + j] = b[j];
        j += 1;
    }

    ret
}

pub const fn composite<const N: usize>(name: &[u8], params: &[u8]) -> [u8; N] {
    let mut ret = [0u8; N];

    let mut i = 0;
    while i < name.len() {
        ret[i] = name[i];
        i += 1;
    }

    ret[i] = 0x28; // `(`
    i += 1;

    let mut j = 0;
    while j < params.len() {
        ret[i + j] = params[j];
        j += 1;
    }

    ret[i + j] = 0x29; // `)`
    ret
}

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

use crate::traits::The_Type_You_Used_Here_Must_Be_An_Valid_Liquid_Data_Type;
use liquid_core::env::types::{Address, String, Vec};
use liquid_macro::seq;

/// The generic type parameter `T` is just used for evade orphan rule in Rust.
pub trait SolTypeName<T = ()> {
    const NAME: &'static [u8];

    fn no_need_for_implement() -> Option<T> {
        None
    }
}

/// The generic type parameter `T` is just used for evade orphan rule in Rust.
pub trait SolTypeNameLen<T = ()> {
    const LEN: usize;

    fn no_need_for_implement() -> Option<T> {
        None
    }
}

macro_rules! mapping_type_to_sol {
    ($origin_ty:ty, $mapped_ty:ident) => {
        impl SolTypeName for $origin_ty {
            const NAME: &'static [u8] = stringify!($mapped_ty).as_bytes();
        }

        impl SolTypeNameLen for $origin_ty {
            const LEN: usize = stringify!($mapped_ty).len();
        }

        impl SolTypeName for Vec<$origin_ty> {
            const NAME: &'static [u8] = concat!(stringify!($mapped_ty), "[]").as_bytes();
        }

        impl SolTypeNameLen for Vec<$origin_ty> {
            const LEN: usize = stringify!($mapped_ty).len() + 2;
        }

        impl The_Type_You_Used_Here_Must_Be_An_Valid_Liquid_Data_Type for $origin_ty {}
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
mapping_type_to_sol!(Address, address);

impl SolTypeName for () {
    const NAME: &'static [u8] = b"";
}

impl SolTypeNameLen for () {
    const LEN: usize = 0;
}

impl The_Type_You_Used_Here_Must_Be_An_Valid_Liquid_Data_Type for () {}

pub struct DynamicArraySuffix;

impl SolTypeName for DynamicArraySuffix {
    const NAME: &'static [u8] = b"[]";
}

impl SolTypeNameLen for DynamicArraySuffix {
    const LEN: usize = 2;
}

impl<T> The_Type_You_Used_Here_Must_Be_An_Valid_Liquid_Data_Type for Vec<T> where
    T: The_Type_You_Used_Here_Must_Be_An_Valid_Liquid_Data_Type
{
}

macro_rules! impl_len_for_tuple {
    (($first:tt, $generic:tt),) => {
        impl<$first, $generic> SolTypeNameLen<$generic> for ($first,)
        where
            $first: SolTypeNameLen<$generic>,
        {
            const LEN: usize = <$first as SolTypeNameLen<$generic>>::LEN;
        }
    };
    (($first:tt,$first_generic:tt), $(($rest:tt,$rest_generic:tt),)+) => {
        impl <$first,$first_generic, $($rest),+, $($rest_generic),+> SolTypeNameLen<($first_generic, $($rest_generic,)*)> for ($first, $($rest,)+)
        where
        $first: SolTypeNameLen<$first_generic>,
        $($rest: SolTypeNameLen<$rest_generic>,)+
        {
            const LEN: usize = <$first as SolTypeNameLen<$first_generic>>::LEN $(+ <$rest as SolTypeNameLen<$rest_generic>>::LEN + 1)+;
        }

        impl_len_for_tuple!($(($rest, $rest_generic),)+);
    };
}

seq!(N in 0..16 {
    impl_len_for_tuple! {
        #((T#N,G#N),)*
    }
});

pub const fn concat<T, E, P, Q, const N: usize>(need_comma: bool) -> [u8; N]
where
    T: SolTypeName<P>,
    E: SolTypeName<Q>,
{
    let a = <T as SolTypeName<P>>::NAME;
    let b = <E as SolTypeName<Q>>::NAME;

    let mut ret = [0u8; N];
    let mut i = 0;
    while i < a.len() {
        ret[i] = a[i];
        i += 1;
    }

    if need_comma {
        ret[i] = 0x2c; //`,`
        i += 1;
    }

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

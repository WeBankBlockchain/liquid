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

use crate::traits::TypeToString;
use liquid_macro::seq;
use liquid_prelude::string::String;
use liquid_primitives::types::*;

macro_rules! primitive_type_to_string {
    ($($origin_ty:ty),*) => {
        $(
            impl TypeToString for $origin_ty {
                fn type_to_string() -> String {
                    stringify!($origin_ty).into()
                }
            }
        )*
    };
    ($($origin_ty:ty => $mapping_ty:ty),*) => {
        $(
            impl TypeToString for $origin_ty {
                fn type_to_string() -> String {
                    stringify!($mapping_ty).into()
                }
            }
        )*
    };
}

primitive_type_to_string!(bool);
primitive_type_to_string!(
    String => string,
    Address => string,
    Bytes => bytes,
    Hash => hash,
    u8 => uint8,
    u16 => uint16,
    u32 => uint32,
    u64 => uint64,
    u128 => uint128,
    u256 => uint256,
    i8 => int8,
    i16 => int16,
    i32 => int32,
    i64 => int64,
    i128 => int128,
    i256 => int256,
    FixedPointU64F16 => Fixed64x16
);
seq!(N in 1..=32 {
    primitive_type_to_string!(Bytes#N => bytes#N);
});

impl TypeToString for () {
    fn type_to_string() -> String {
        "".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive() {
        assert_eq!(u8::type_to_string(), "uint8");
        assert_eq!(u16::type_to_string(), "uint16");
        assert_eq!(u32::type_to_string(), "uint32");
        assert_eq!(u64::type_to_string(), "uint64");
        assert_eq!(u128::type_to_string(), "uint128");
        assert_eq!(u256::type_to_string(), "uint256");

        assert_eq!(i8::type_to_string(), "int8");
        assert_eq!(i16::type_to_string(), "int16");
        assert_eq!(i32::type_to_string(), "int32");
        assert_eq!(i64::type_to_string(), "int64");
        assert_eq!(i128::type_to_string(), "int128");
        assert_eq!(i256::type_to_string(), "int256");

        assert_eq!(String::type_to_string(), "string");
        assert_eq!(Address::type_to_string(), "string");
        assert_eq!(Bytes::type_to_string(), "bytes");
        assert_eq!(Hash::type_to_string(), "hash");
        assert_eq!(FixedPointU64F16::type_to_string(), "Fixed64x16");

        seq!(N in 1..=32 {
            assert_eq!(
                Bytes#N::type_to_string(),
                stringify!(bytes#N)
            );
        });
    }
}

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

primitive_type_to_string!(
    u8, u16, u32, u64, u128, u256, i8, i16, i32, i64, i128, i256, bool
);
primitive_type_to_string!(String => string, Address => address, Bytes => bytes, Hash => hash);
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
        assert_eq!(u8::type_to_string(), "u8");
        assert_eq!(u16::type_to_string(), "u16");
        assert_eq!(u32::type_to_string(), "u32");
        assert_eq!(u64::type_to_string(), "u64");
        assert_eq!(u128::type_to_string(), "u128");
        assert_eq!(u256::type_to_string(), "u256");

        assert_eq!(i8::type_to_string(), "i8");
        assert_eq!(i16::type_to_string(), "i16");
        assert_eq!(i32::type_to_string(), "i32");
        assert_eq!(i64::type_to_string(), "i64");
        assert_eq!(i128::type_to_string(), "i128");
        assert_eq!(i256::type_to_string(), "i256");

        assert_eq!(String::type_to_string(), "string");
        assert_eq!(Address::type_to_string(), "address");
        assert_eq!(Bytes::type_to_string(), "bytes");
        assert_eq!(Hash::type_to_string(), "hash");

        seq!(N in 1..=32 {
            assert_eq!(
                Bytes#N::type_to_string(),
                stringify!(bytes#N)
            );
        });
    }
}

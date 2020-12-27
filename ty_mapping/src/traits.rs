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

use liquid_macro::seq;
use liquid_prelude::{string::String, vec::Vec};
use liquid_primitives::types::*;

pub const MAX_LENGTH_OF_MAPPED_TYPE_NAME: usize = 256;

pub trait MappingToSolidityType {
    const MAPPED_TYPE_NAME: [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME];
}

const fn extend(origin: &'static [u8]) -> [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME] {
    let mut ret = [0u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME];
    let mut i = 0;
    while i < origin.len() {
        ret[i] = origin[i];
        i += 1;
    }
    ret
}

macro_rules! mapping_type_to_sol {
    ($origin_ty:ty, $mapped_ty:ident) => {
        impl MappingToSolidityType for $origin_ty {
            const MAPPED_TYPE_NAME: [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME] =
                extend(stringify!($mapped_ty).as_bytes());
        }
    };
}

mapping_type_to_sol!(u8, uint8);
mapping_type_to_sol!(u16, uint16);
mapping_type_to_sol!(u32, uint32);
mapping_type_to_sol!(u64, uint64);
mapping_type_to_sol!(u128, uint128);
mapping_type_to_sol!(u256, uint256);
mapping_type_to_sol!(i8, int8);
mapping_type_to_sol!(i16, int16);
mapping_type_to_sol!(i32, int32);
mapping_type_to_sol!(i64, int64);
mapping_type_to_sol!(i128, int128);
mapping_type_to_sol!(i256, int256);
mapping_type_to_sol!(bool, bool);
mapping_type_to_sol!(String, string);
mapping_type_to_sol!(Address, address);
mapping_type_to_sol!(Bytes, bytes);
seq!(N in 1..=32 {
    mapping_type_to_sol!(Bytes#N, bytes#N);
});

pub const fn concat<T, E>() -> [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME]
where
    T: MappingToSolidityType,
    E: MappingToSolidityType,
{
    let a = &<T as MappingToSolidityType>::MAPPED_TYPE_NAME;
    let b = &<E as MappingToSolidityType>::MAPPED_TYPE_NAME;

    let mut ret = [0u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME];
    let mut i = 0;
    while a[i] != 0 {
        ret[i] = a[i];
        i += 1;
    }

    ret[i] = b',';
    i += 1;

    let mut j = 0;
    while b[j] != 0 {
        ret[i + j] = b[j];
        j += 1;
    }
    ret
}

pub const fn len<T>() -> usize
where
    T: MappingToSolidityType,
{
    let name = &<T as MappingToSolidityType>::MAPPED_TYPE_NAME;
    let mut i = 0;
    while i < MAX_LENGTH_OF_MAPPED_TYPE_NAME && name[i] != 0 {
        i += 1;
    }
    i
}

pub const fn append_dynamic_array_suffix<T>() -> [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME]
where
    T: MappingToSolidityType,
{
    let name = <T as MappingToSolidityType>::MAPPED_TYPE_NAME;
    let mut ret = [0u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME];

    let mut i = 0;
    while name[i] != 0 {
        ret[i] = name[i];
        i += 1;
    }

    ret[i] = b'[';
    ret[i + 1] = b']';
    ret
}

#[allow(clippy::manual_swap)]
pub const fn append_static_array_suffix<T>(
    mut n: usize,
) -> [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME]
where
    T: MappingToSolidityType,
{
    let name = &<T as MappingToSolidityType>::MAPPED_TYPE_NAME;
    let mut ret = [0u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME];

    let mut i = 0;
    while name[i] != 0 {
        ret[i] = name[i];
        i += 1;
    }

    ret[i] = b'[';
    i += 1;

    if n != 0 {
        let mut digits_num = 0;
        while n != 0 {
            ret[i] = 0x30 + ((n % 10) as u8);
            i += 1;
            digits_num += 1;
            n /= 10;
        }

        let mut j = 0;
        loop {
            let front = i - digits_num + j;
            let back = i - j - 1;
            if front >= back {
                break;
            }

            let tmp = ret[front];
            ret[front] = ret[back];
            ret[back] = tmp;
            j += 1;
        }
    } else {
        ret[i] = b'0';
        i += 1;
    }

    ret[i] = b']';
    ret
}

pub const fn composite<T, const N: usize>(prefix: &[u8]) -> [u8; N]
where
    T: MappingToSolidityType,
{
    let mut ret = [0u8; N];

    let mut i = 0;
    while i < prefix.len() {
        ret[i] = prefix[i];
        i += 1;
    }

    ret[i] = b'(';
    i += 1;

    let params = &<T as MappingToSolidityType>::MAPPED_TYPE_NAME;
    let mut j = 0;
    while params[j] != 0 {
        ret[i + j] = params[j];
        j += 1;
    }

    ret[i + j] = b')';
    ret
}

impl<T> MappingToSolidityType for Vec<T>
where
    T: MappingToSolidityType,
{
    const MAPPED_TYPE_NAME: [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME] =
        append_dynamic_array_suffix::<T>();
}

impl<T, const N: usize> MappingToSolidityType for [T; N]
where
    T: MappingToSolidityType,
{
    const MAPPED_TYPE_NAME: [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME] =
        append_static_array_suffix::<T>(N);
}

macro_rules! impl_type_mapping_for_tuples {
    ($first:tt,) => {
        impl<$first> MappingToSolidityType for ($first,)
        where
            $first: MappingToSolidityType,
        {
            const MAPPED_TYPE_NAME: [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME] =
                <$first as MappingToSolidityType>::MAPPED_TYPE_NAME;
        }
    };
    ($first:tt, $( $rest:tt, )+) => {
        impl<$first, $( $rest, )+> MappingToSolidityType for ($first, $( $rest, )+)
        where
            $first: MappingToSolidityType,
            $( $rest: MappingToSolidityType, )+
        {
            const MAPPED_TYPE_NAME:[u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME] =
                concat::<$first, ($( $rest, )+ )>();
        }

        impl_type_mapping_for_tuples!($( $rest, )+);
    };
}

seq!(N in 0..16 {
    impl_type_mapping_for_tuples!(#(T#N,)*);
});

impl MappingToSolidityType for () {
    const MAPPED_TYPE_NAME: [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME] =
        [0u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME];
}

#[cfg(feature = "contract")]
impl MappingToSolidityType for liquid_primitives::__LIQUID_GETTER_INDEX_PLACEHOLDER {
    const MAPPED_TYPE_NAME: [u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME] =
        [0u8; MAX_LENGTH_OF_MAPPED_TYPE_NAME];
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_to_solidity_type<T: MappingToSolidityType>() -> &'static str {
        std::str::from_utf8(&<T as MappingToSolidityType>::MAPPED_TYPE_NAME)
            .unwrap()
            .trim_end_matches(char::from(0))
    }

    #[test]
    fn test_primitive() {
        assert_eq!(map_to_solidity_type::<u8>(), "uint8");
        assert_eq!(map_to_solidity_type::<u16>(), "uint16");
        assert_eq!(map_to_solidity_type::<u32>(), "uint32");
        assert_eq!(map_to_solidity_type::<u64>(), "uint64");
        assert_eq!(map_to_solidity_type::<u128>(), "uint128");
        assert_eq!(map_to_solidity_type::<u256>(), "uint256");

        assert_eq!(map_to_solidity_type::<i8>(), "int8");
        assert_eq!(map_to_solidity_type::<i16>(), "int16");
        assert_eq!(map_to_solidity_type::<i32>(), "int32");
        assert_eq!(map_to_solidity_type::<i64>(), "int64");
        assert_eq!(map_to_solidity_type::<i128>(), "int128");
        assert_eq!(map_to_solidity_type::<i256>(), "int256");

        assert_eq!(map_to_solidity_type::<String>(), "string");
        assert_eq!(map_to_solidity_type::<Address>(), "address");

        seq!(N in 1..=32 {
            assert_eq!(
                map_to_solidity_type::<Bytes#N>(),
                stringify!(bytes#N)
            );
        });
    }

    #[test]
    fn test_dynamic_array() {
        assert_eq!(map_to_solidity_type::<Vec<u8>>(), "uint8[]");
        assert_eq!(map_to_solidity_type::<Vec<u16>>(), "uint16[]");
        assert_eq!(map_to_solidity_type::<Vec<u32>>(), "uint32[]");
        assert_eq!(map_to_solidity_type::<Vec<u64>>(), "uint64[]");
        assert_eq!(map_to_solidity_type::<Vec<u128>>(), "uint128[]");
        assert_eq!(map_to_solidity_type::<Vec<u256>>(), "uint256[]");

        assert_eq!(map_to_solidity_type::<Vec<i8>>(), "int8[]");
        assert_eq!(map_to_solidity_type::<Vec<i16>>(), "int16[]");
        assert_eq!(map_to_solidity_type::<Vec<i32>>(), "int32[]");
        assert_eq!(map_to_solidity_type::<Vec<i64>>(), "int64[]");
        assert_eq!(map_to_solidity_type::<Vec<i128>>(), "int128[]");
        assert_eq!(map_to_solidity_type::<Vec<i256>>(), "int256[]");

        assert_eq!(map_to_solidity_type::<Vec<String>>(), "string[]");
        assert_eq!(map_to_solidity_type::<Vec<Address>>(), "address[]");
        assert_eq!(map_to_solidity_type::<Vec<Vec<i8>>>(), "int8[][]");

        seq!(N in 1..=32 {
            assert_eq!(
                map_to_solidity_type::<Vec<Bytes#N>>(),
                stringify!(bytes#N[]).replace(" ", ""),
            );
        });
    }

    #[test]
    fn test_fixed_size_array() {
        assert_eq!(map_to_solidity_type::<[u8; 1024]>(), "uint8[1024]");
        assert_eq!(map_to_solidity_type::<[u16; 1024]>(), "uint16[1024]");
        assert_eq!(map_to_solidity_type::<[u32; 1024]>(), "uint32[1024]");
        assert_eq!(map_to_solidity_type::<[u64; 1024]>(), "uint64[1024]");
        assert_eq!(map_to_solidity_type::<[u128; 1024]>(), "uint128[1024]");
        assert_eq!(map_to_solidity_type::<[u256; 1024]>(), "uint256[1024]");

        assert_eq!(map_to_solidity_type::<[i8; 1024]>(), "int8[1024]");
        assert_eq!(map_to_solidity_type::<[i16; 1024]>(), "int16[1024]");
        assert_eq!(map_to_solidity_type::<[i32; 1024]>(), "int32[1024]");
        assert_eq!(map_to_solidity_type::<[i64; 1024]>(), "int64[1024]");
        assert_eq!(map_to_solidity_type::<[i128; 1024]>(), "int128[1024]");
        assert_eq!(map_to_solidity_type::<[i256; 1024]>(), "int256[1024]");

        assert_eq!(map_to_solidity_type::<[String; 0]>(), "string[0]");
        assert_eq!(map_to_solidity_type::<[Address; 0]>(), "address[0]");
        assert_eq!(map_to_solidity_type::<Vec<[u8; 65535]>>(), "uint8[65535][]");

        seq!(N in 1..=32 {
            assert_eq!(
                map_to_solidity_type::<[Bytes#N; (N as usize)]>(),
                stringify!(bytes#N[N]).replace(" ", "").replace("u64", ""),
            );
        });
    }

    #[test]
    fn test_concat() {
        assert_eq!(map_to_solidity_type::<(u8,)>(), "uint8");
        assert_eq!(map_to_solidity_type::<(u8, String)>(), "uint8,string");
        assert_eq!(
            map_to_solidity_type::<(Vec<u8>, String)>(),
            "uint8[],string"
        );
        assert_eq!(
            map_to_solidity_type::<(Vec<u8>, [Address; 1024])>(),
            "uint8[],address[1024]"
        )
    }

    #[test]
    fn test_len() {
        assert_eq!(len::<(u8, String)>(), 12);
    }

    #[test]
    fn test_composite() {
        assert_eq!(
            composite::<(), 10>(&[b'g', b'e', b't', b't', b'u', b'p', b'l', b'e']),
            [b'g', b'e', b't', b't', b'u', b'p', b'l', b'e', b'(', b')']
        );
    }
}

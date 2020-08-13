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

use core::cmp::Ordering;
use liquid_abi_codec::{
    peek, DecodeResult, Error, IsDynamic, Mediate, MediateDecode, MediateEncode, Word,
    WORD_SIZE,
};
use liquid_prelude::{
    string::{String, ToString},
    vec::Vec,
};
use liquid_ty_mapping::{SolTypeName, SolTypeNameLen};

pub const ADDRESS_LENGTH: usize = 20;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, scale::Decode, scale::Encode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Address([u8; ADDRESS_LENGTH]);

impl Address {
    pub fn new(address: [u8; ADDRESS_LENGTH]) -> Self {
        Self(address)
    }

    pub fn empty() -> Self {
        Self([0u8; ADDRESS_LENGTH])
    }
}

impl IsDynamic for Address {
    fn is_dynamic() -> bool {
        false
    }
}

impl MediateEncode for Address {
    fn encode(&self) -> Mediate {
        let mut buf = [0x00; WORD_SIZE];
        buf[(WORD_SIZE - ADDRESS_LENGTH)..].copy_from_slice(&self.0);
        Mediate::Raw([buf].to_vec())
    }
}

impl MediateDecode for Address {
    fn decode(slices: &[Word], offset: usize) -> Result<DecodeResult<Self>, Error> {
        let slice = peek(slices, offset)?;

        if !slice[..(WORD_SIZE - ADDRESS_LENGTH)]
            .iter()
            .all(|x| *x == 0)
        {
            Err("Invalid address representation".into())
        } else {
            let new_offset = offset + 1;
            let mut address = [0u8; ADDRESS_LENGTH];
            address[..].copy_from_slice(&slice[(WORD_SIZE - ADDRESS_LENGTH)..]);
            Ok(DecodeResult {
                value: Self(address),
                new_offset,
            })
        }
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        let mut ret = String::with_capacity(ADDRESS_LENGTH * 2 + 2);
        ret.push_str("0x");

        for digit in self.0.iter() {
            let low = digit & 0x0fu8;
            let high = digit >> 4;
            ret.push(core::char::from_digit(high.into(), 16).unwrap());
            ret.push(core::char::from_digit(low.into(), 16).unwrap());
        }
        ret
    }
}

impl Address {
    pub fn from_string<Q>(origin: Q) -> Self
    where
        Q: AsRef<str>,
    {
        let addr_str: &str = origin.as_ref();

        if !addr_str.is_ascii() {
            panic!("invalid address representation");
        }

        if addr_str.len() != ADDRESS_LENGTH * 2 + 2 {
            panic!("invalid address representation");
        }

        if !addr_str.starts_with("0x") && !addr_str.starts_with("0X") {
            panic!("invalid address representation");
        }

        let mut address = [0u8; ADDRESS_LENGTH];
        let bytes = addr_str.as_bytes();
        for i in 0..ADDRESS_LENGTH {
            let high = (bytes[2 + i * 2] as char).to_digit(16).unwrap();
            let low = (bytes[3 + i * 2] as char).to_digit(16).unwrap();
            let digit = (high << 4) + low;
            address[i] = digit as u8;
        }

        Self(address)
    }

    pub fn from_bytes(bytes: &[u8; ADDRESS_LENGTH]) -> Self {
        Self(*bytes)
    }
}

impl PartialEq<[u8; ADDRESS_LENGTH]> for Address {
    fn eq(&self, rhs: &[u8; ADDRESS_LENGTH]) -> bool {
        self.0.eq(rhs)
    }
}

impl PartialOrd<[u8; ADDRESS_LENGTH]> for Address {
    fn partial_cmp(&self, other: &[u8; ADDRESS_LENGTH]) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

const ADDRESS_MAPPED_TYPE: &str = "address";
const ADDRESS_ARRAY_MAPPED_TYPE: &str = "address[]";

#[cfg(feature = "liquid-abi-gen")]
use liquid_abi_gen::{GenerateComponents, TyName};

#[cfg(feature = "liquid-abi-gen")]
impl GenerateComponents for Address {}

#[cfg(feature = "liquid-abi-gen")]
impl TyName for Address {
    fn ty_name() -> String {
        String::from(ADDRESS_MAPPED_TYPE)
    }
}

pub type Timestamp = u64;
pub type BlockNumber = u64;

impl SolTypeName for Address {
    const NAME: &'static [u8] = ADDRESS_MAPPED_TYPE.as_bytes();
}

impl SolTypeNameLen for Address {
    const LEN: usize = ADDRESS_MAPPED_TYPE.len();
}

impl SolTypeName<Address> for Vec<Address> {
    const NAME: &'static [u8] = ADDRESS_ARRAY_MAPPED_TYPE.as_bytes();
}

impl SolTypeNameLen<Address> for Vec<Address> {
    const LEN: usize = ADDRESS_MAPPED_TYPE.len() + 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ADDR: [u8; ADDRESS_LENGTH] = [
        0x3e, 0x9A, 0xFa, 0xA4, 0xa0, 0x62, 0xA4, 0x9d, 0x64, 0xb8, 0xAb, 0x05, 0x7B,
        0x3C, 0xb5, 0x18, 0x92, 0xe1, 0x7E, 0xcb,
    ];

    #[test]
    fn test_address_abi_codec() {
        let origin = TEST_ADDR.clone();
        let mut encoded = [0u8; 32];
        encoded[(WORD_SIZE - ADDRESS_LENGTH)..].copy_from_slice(&origin);

        use liquid_abi_codec::{Decode, Encode};
        let address = (Address(origin.clone()),);
        assert_eq!(address.encode(), encoded.to_vec());

        let decoded = <(Address,) as Decode>::decode(&mut &encoded[..]).unwrap();
        assert_eq!((decoded.0).0, origin);
    }

    #[test]
    fn test_copy() {
        let origin = TEST_ADDR.clone();
        let mut address_0 = Address(origin);
        let address_1 = address_0;
        let address_2 = address_1;

        assert_eq!(address_0, address_1);
        assert_eq!(address_1, address_2);

        (address_0.0)[ADDRESS_LENGTH - 1] = 0x00;

        assert_ne!(address_0, address_1);
        assert_eq!(address_1, address_2);
    }

    #[test]
    fn test_cmp() {
        let small_addr = [0u8; ADDRESS_LENGTH];
        let big_addr = TEST_ADDR.clone();

        assert_eq!(small_addr < big_addr, true);
        assert_eq!(small_addr <= big_addr, true);
        assert_eq!(small_addr <= big_addr, true);
        assert_eq!(small_addr > big_addr, false);
        assert_eq!(small_addr >= big_addr, false);
        assert_eq!(small_addr != big_addr, true);
        assert_eq!(small_addr == small_addr, true);
    }

    #[test]
    fn string_convert() {
        let address = Address(TEST_ADDR.clone());
        let addr_str = "0x3e9afaa4a062a49d64b8ab057b3cb51892e17ecb";
        assert_eq!(address.to_string(), addr_str);
        assert_eq!(Address::from_string(addr_str), address);

        let addr_str = String::from(addr_str);
        assert_eq!(Address::from_string(&addr_str), address);
    }

    #[test]
    #[should_panic]
    fn invalid_addr_length() {
        let _ = Address::from_string("0x3e9afaa4a062a49d64b8ab057b3cb51892e1");
    }

    #[test]
    #[should_panic]
    fn invalid_addr_start() {
        let _ = Address::from_string("0b3e9afaa4a062a49d64b8ab057b3cb51892e17ecb");
    }

    #[test]
    #[should_panic]
    fn invalid_addr_str_encode() {
        let _ = Address::from_string("羞答答小白虎头李荣浩");
    }
}

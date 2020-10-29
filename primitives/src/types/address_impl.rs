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

use crate::Error;
use liquid_prelude::{
    str::FromStr,
    string::{String, ToString},
};

pub const ADDRESS_LENGTH: usize = 20;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, scale::Decode, scale::Encode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct address(pub [u8; ADDRESS_LENGTH]);

impl address {
    pub fn new(addr: [u8; ADDRESS_LENGTH]) -> Self {
        Self(addr)
    }

    pub fn empty() -> Self {
        Default::default()
    }
}

impl Default for address {
    fn default() -> Self {
        Self([0; ADDRESS_LENGTH])
    }
}

impl ToString for address {
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

impl FromStr for address {
    type Err = Error;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if !s.is_ascii() {
            return Err("invalid address representation".into());
        }

        if s.starts_with("0x") || s.starts_with("0X") {
            if s.len() > ADDRESS_LENGTH * 2 + 2 {
                return Err("invalid address representation".into());
            }
            s = &s[2..];
        } else if s.len() > ADDRESS_LENGTH * 2 {
            return Err("invalid address representation".into());
        }

        let mut addr = [0u8; ADDRESS_LENGTH];
        let bytes = s.as_bytes();
        let padding_len = ADDRESS_LENGTH * 2 - bytes.len();
        for i in 0..ADDRESS_LENGTH {
            let (low, high) = if i * 2 + 1 < padding_len {
                (0, 0)
            } else {
                (
                    (bytes[i * 2 + 1 - padding_len] as char)
                        .to_digit(16)
                        .unwrap(),
                    if i * 2 < padding_len {
                        0
                    } else {
                        (bytes[i * 2 - padding_len] as char).to_digit(16).unwrap()
                    },
                )
            };

            let digit = (high << 4) + low;
            addr[i] = digit as u8;
        }
        Ok(Self(addr))
    }
}

impl From<[u8; ADDRESS_LENGTH]> for address {
    fn from(bytes: [u8; ADDRESS_LENGTH]) -> Self {
        Self(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ADDR: [u8; ADDRESS_LENGTH] = [
        0x3e, 0x9A, 0xFa, 0xA4, 0xa0, 0x62, 0xA4, 0x9d, 0x64, 0xb8, 0xAb, 0x05, 0x7B,
        0x3C, 0xb5, 0x18, 0x92, 0xe1, 0x7E, 0xcb,
    ];

    #[test]
    fn test_copy() {
        let origin = TEST_ADDR.clone();
        let mut address_0 = address(origin);
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
        let addr = address(TEST_ADDR.clone());
        let addr_str = "0x3e9afaa4a062a49d64b8ab057b3cb51892e17ecb";
        assert_eq!(addr.to_string(), addr_str);
        assert_eq!(addr_str.parse::<address>().unwrap(), addr);

        let addr_str = String::from(addr_str);
        assert_eq!(addr_str.parse::<address>().unwrap(), addr);
    }

    #[test]
    fn padding_1() {
        let addr: address = "0x12".parse().unwrap();
        assert_eq!(
            addr,
            "0x0000000000000000000000000000000000000012"
                .parse()
                .unwrap()
        );
        assert_eq!(
            addr.to_string(),
            "0x0000000000000000000000000000000000000012"
        );
    }

    #[test]
    fn padding_2() {
        let addr: address = "0x121".parse().unwrap();
        assert_eq!(
            addr,
            "0x0000000000000000000000000000000000000121"
                .parse()
                .unwrap()
        );
        assert_eq!(
            addr.to_string(),
            "0x0000000000000000000000000000000000000121"
        );
    }

    #[test]
    #[should_panic(expected = "invalid address representation")]
    fn invalid_addr_start() {
        let _: address = "0b3e9afaa4a062a49d64b8ab057b3cb51892e17ecb"
            .parse()
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "invalid address representation")]
    fn invalid_addr_str_encode() {
        let _: address = "羞答答小白虎头李荣浩".parse().unwrap();
    }
}

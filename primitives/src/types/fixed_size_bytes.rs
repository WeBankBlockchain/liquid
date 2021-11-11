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

use crate::{
    types::{int256::i256, uint256::u256},
    Error,
};
use liquid_macro::seq;
use liquid_prelude::str::FromStr;

seq!(N in 1..=32 {
    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, scale::Decode, scale::Encode)]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub struct Bytes#N(pub [u8; N as usize]);

    impl Bytes#N {
        pub const LEN: usize = (N as usize);
    }

    impl core::ops::Shl<usize> for Bytes#N {
        type Output = Self;

        fn shl(mut self, mid: usize) -> Self::Output {
            let external_shift = mid / 8;
            let internal_shift = mid % 8;

            if external_shift >= (N as usize) {
                return Self(Default::default());
            }

            if external_shift > 0 {
                for i in 0..(N as usize - external_shift) {
                    self.0[i] = self.0[i + external_shift];
                }

                for i in (N as usize - external_shift)..(N as usize) {
                    self.0[i] = Default::default();
                }
            }

            if internal_shift > 0 {
                self.0[0] <<= internal_shift;
                let mask = u8::MAX << (8 - internal_shift);

                for i in 1..(N as usize) {
                    let carry = self.0[i] & mask;
                    let carry = carry >> (8 - internal_shift);
                    self.0[i - 1] |= carry;
                    self.0[i] <<= internal_shift;
                }
            }

            self
        }
    }

    impl core::ops::Shr<usize> for Bytes#N {
        type Output = Self;

        fn shr(mut self, mid: usize) -> Self::Output {
            let external_shift = mid / 8;
            let internal_shift = mid % 8;

            if external_shift >= (N as usize) {
                return Self(Default::default());
            }

            if external_shift > 0 {
                for i in (external_shift..(N as usize)).rev() {
                    self.0[i] = self.0[i - external_shift];
                }

                for i in 0..external_shift {
                    self.0[i] = Default::default();
                }
            }

            self.0[(N as usize) - 1] >>= internal_shift;
            let mask = (1 << internal_shift) - 1;
            for i in (0..(N as usize - 1)).rev() {
                let carry = self.0[i] & mask;
                let carry = carry << (8 - internal_shift);
                self.0[i + 1] |= carry;
                self.0[i] >>= internal_shift;
            }

            self
        }
    }

    impl core::ops::BitAnd for Bytes#N {
        type Output = Self;

        fn bitand(self, rhs: Self) -> Self::Output {
            let mut buf = [0u8; N as usize];
            for (i, b) in buf.iter_mut().enumerate().take(N as usize) {
                *b = self.0[i] & rhs.0[i];
            }
            Self(buf)
        }
    }

    impl core::ops::BitOr for Bytes#N {
        type Output = Self;

        fn bitor(self, rhs: Self) -> Self::Output {
            let mut buf = [0u8; N as usize];
            for (i, b) in buf.iter_mut().enumerate().take(N as usize) {
                *b = self.0[i] | rhs.0[i];
            }
            Self(buf)
        }
    }

    impl core::ops::BitXor for Bytes#N {
        type Output = Self;

        fn bitxor(self, rhs: Self) -> Self::Output {
            let mut buf = [0u8; N as usize];
            for (i, b) in buf.iter_mut().enumerate().take(N as usize) {
                *b = self.0[i] ^ rhs.0[i];
            }
            Self(buf)
        }
    }

    impl FromStr for Bytes#N {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let bytes = s.as_bytes();
            if bytes.len() > (N as usize) {
                return Err("the string is unable to be converted to fix-sized bytes".into());
            }

            let mut ret = [0u8; N as usize];
            ret[..bytes.len()].copy_from_slice(bytes);
            Ok(Self(ret))
        }
    }

    impl From<[u8; N as usize]> for Bytes#N {
        fn from(bytes: [u8; N as usize]) -> Self {
            Self(bytes)
        }
    }

    impl Default for Bytes#N {
        fn default() -> Self {
            Self(Default::default())
        }
    }
});

pub type Byte = Bytes1;

macro_rules! impl_bytes_from_integer {
    ($( {$t1:ty, $t2:ty} as Bytes{$s:expr => $e:expr} ),+) => {
        $(
            seq!(N in $s..=$e {
                impl From<$t1> for Bytes#N {
                    fn from(i: $t1) -> Self {
                        let mut ret = [0u8; (N as usize)];
                        ret[..$s].copy_from_slice(&i.to_be_bytes());
                        Self(ret)
                    }
                }

                impl From<$t2> for Bytes#N {
                    fn from(i: $t2) -> Self {
                        let mut ret = [0u8; (N as usize)];
                        ret[..$s].copy_from_slice(&i.to_be_bytes());
                        Self(ret)
                    }
                }
            });
        )+
    };
}

impl_bytes_from_integer!(
    {u8, i8} as Bytes{1 => 32},
    {u16, i16} as Bytes{2 => 32},
    {u32, i32} as Bytes{4 => 32},
    {u64, i64} as Bytes{8 => 32},
    {u128, i128} as Bytes{16 => 32},
    {u256, i256} as Bytes{32 => 32}
);

macro_rules! impl_bytes_from_bytes {
    ($( $t:tt as Bytes{$s:expr => $e:expr}, )+) => {
        $(
            seq!(N in $s..=$e {
                impl From<$t> for Bytes#N {
                    fn from(origin: $t) -> Self {
                        let mut buf = [0u8; (N as usize)];
                        buf[..($t::LEN)].copy_from_slice(&origin.0);
                        Self(buf)
                    }
                }

                impl From<&$t> for Bytes#N {
                    fn from(origin: &$t) -> Self {
                        let mut buf = [0u8; (N as usize)];
                        buf[..($t::LEN)].copy_from_slice(&origin.0);
                        Self(buf)
                    }
                }
            });
        )+
    };
}

seq!(N in 1..=32 {
    impl_bytes_from_bytes!(
        #(Bytes#N as Bytes{+1#N => 32},)*
    );
});

seq!(N in 1..=32 {
    impl core::ops::Index<usize> for Bytes#N
    {
        type Output = u8;

        fn index(&self, index: usize) -> &Self::Output {
            if index >= (N as usize) {
                panic!("expected `index` to be within {}", N);
            }
            &self.0[index]
        }
    }

    impl core::ops::IndexMut<usize> for Bytes#N
    {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            if index >= (N as usize) {
                panic!("expected `index` to be within {}", N);
            }
            &mut self.0[index]
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes1_from_str() {
        let b1: Bytes1 = "1".parse().unwrap();
        let b2: Bytes1 = Bytes1::from_str("1").unwrap();
        let b3: Bytes1 = "".parse().unwrap();
        assert_eq!(b1, b2);
        assert_eq!(b1[0], b'1');
        assert_eq!(b3[0], 0);
    }

    #[test]
    #[should_panic]
    fn bytes1_from_str_panic() {
        let _: Bytes1 = "01".parse().unwrap();
    }

    #[test]
    fn bytes1_from_int() {
        let b1: Bytes1 = 255u8.into();
        let b2: Bytes1 = (-1i8).into();
        assert_eq!(b1[0], 255);
        assert_eq!(b2[0], 255);
    }

    #[test]
    fn bytes32_from_str() {
        let b1: Bytes32 = "abcd".parse().unwrap();
        let b2: Bytes32 = Bytes32::from_str("abcd").unwrap();
        assert_eq!(b1, b2);
        assert_eq!(b1[0], b'a');
        assert_eq!(b1[1], b'b');
        assert_eq!(b1[2], b'c');
        assert_eq!(b1[3], b'd');
    }

    #[test]
    #[should_panic(expected = "the string is unable to be converted to fix-sized bytes")]
    fn bytes32_from_str_panic() {
        let _: Bytes1 = "abcdabcdabcdabcdabcdabcdabcdabcdabcdabcd".parse().unwrap();
    }

    #[test]
    fn bytes32_from_int() {
        let i: i256 = 1024.into();
        let u: u256 = 1024.into();

        let b1: Bytes32 = i.into();
        let b2: Bytes32 = u.into();
        assert_eq!(b1, b2);
    }

    #[test]
    #[should_panic(expected = "expected `index` to be within 32")]
    fn bytes32_index_out_of_bounds() {
        let b: Bytes32 = 0x10086.into();
        let _ = b[32];
    }
    #[test]

    fn test_ops() {
        let b1: Bytes1 = 0b10101010u8.into();
        let b2: Bytes1 = 0b01010101u8.into();

        assert_eq!(b1 & b2, 0b00000000u8.into());
        assert_eq!(b1 | b2, 0b11111111u8.into());
        assert_eq!(b1 ^ b2, 0b11111111u8.into());
        assert_eq!(b2 << 1, 0b10101010u8.into());
        assert_eq!(b1 >> 1, 0b01010101u8.into());

        let b3: Bytes32 = b1.into();
        let b4: Bytes32 = b2.into();
        assert_eq!(b3[0], b1[0]);
        assert_eq!(b4[0], b2[0]);

        assert_eq!(b3 << 1024, 0.into());

        let b5 = b4 >> 1;
        assert_eq!(b5[0], 0b00101010u8);
        assert_eq!(b5[1], 0b10000000u8);

        let b6 = b4 >> 13;
        assert_eq!(b6[0], 0);
        assert_eq!(b6[1], 2);
        assert_eq!(b6[2], 168);
        assert_eq!(b6 << 13, b4);

        let b7: Bytes2 = 1u16.into();
        assert_eq!(b7, 0b0000000000000001u16.into());
        assert_eq!(b7 << 8, 0b0000000100000000u16.into())
    }
}

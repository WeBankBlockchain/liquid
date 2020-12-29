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

pub use crate::types::int256::i256;
use core::{
    default::Default,
    fmt,
    ops::{Add, AddAssign, Deref, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};
use liquid_prelude::vec::{from_elem, Vec};
use num::{
    bigint::{ParseBigIntError, ToBigInt},
    pow,
    traits::ops::checked::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
    BigUint, Bounded, Num, Zero,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Zero)]
#[allow(non_camel_case_types)]
pub struct u256(pub BigUint);

impl u256 {
    pub fn from_le_bytes(slice: &[u8]) -> Self {
        Self(BigUint::from_bytes_le(slice))
    }

    pub fn from_be_bytes(slice: &[u8]) -> Self {
        Self(BigUint::from_bytes_be(slice))
    }

    /// Converts value to a signed 256 bit integer
    pub fn to_int256(&self) -> Option<i256> {
        self.0
            .to_bigint()
            .filter(|value| value.bits() <= 255)
            .map(i256)
    }

    pub fn to_be_bytes(&self) -> [u8; 32] {
        let bytes = self.to_bytes_be();
        let mut res = [0u8; 32];
        res[32 - bytes.len()..].copy_from_slice(&bytes);
        res
    }
}

impl Bounded for u256 {
    fn min_value() -> Self {
        // -2**255
        u256::zero()
    }
    fn max_value() -> Self {
        lazy_static! {
            static ref MAX_VALUE: BigUint =
                pow(BigUint::from(2u32), 256) - BigUint::from(1u32);
        }
        Self(MAX_VALUE.clone())
    }
}

impl FromStr for u256 {
    type Err = ParseBigIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix("0x") {
            Some(sub_str) => Ok(BigUint::from_str_radix(sub_str, 16).map(Self)?),
            None => Ok(BigUint::from_str_radix(&s, 10).map(Self)?),
        }
    }
}

impl fmt::Display for u256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0.to_str_radix(10))
    }
}

#[cfg(feature = "std")]
impl fmt::Debug for u256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "u256({})", &self.0.to_str_radix(10))
    }
}

impl Deref for u256 {
    type Target = BigUint;

    fn deref(&self) -> &BigUint {
        &self.0
    }
}

impl From<[u8; 32]> for u256 {
    fn from(n: [u8; 32]) -> Self {
        Self(BigUint::from_bytes_be(&n))
    }
}

impl<'a> From<&'a [u8]> for u256 {
    fn from(n: &'a [u8]) -> Self {
        Self(BigUint::from_bytes_be(n))
    }
}

macro_rules! uint_impl_from_uint {
    ($T:ty) => {
        impl From<$T> for u256 {
            #[inline]
            fn from(n: $T) -> Self {
                u256(BigUint::from(n))
            }
        }
    };
}

macro_rules! uint_impl_from_int {
    ($T:ty) => {
        impl From<$T> for u256 {
            #[inline]
            fn from(n: $T) -> Self {
                if n >= 0 {
                    u256(BigUint::from(n as u128))
                } else {
                    panic!("attempt to convert negative {} to u256", n);
                }
            }
        }
    };
}

// These implementations are pretty much guaranteed to be panic-free.
uint_impl_from_uint!(u8);
uint_impl_from_uint!(u16);
uint_impl_from_uint!(u32);
uint_impl_from_uint!(u64);
uint_impl_from_uint!(u128);
uint_impl_from_uint!(usize);
uint_impl_from_int!(i8);
uint_impl_from_int!(i16);
uint_impl_from_int!(i32);
uint_impl_from_int!(i64);
uint_impl_from_int!(i128);
uint_impl_from_int!(isize);

/// A macro that forwards an unary operator trait i.e. Add
macro_rules! forward_op {
    (impl $trait_:ident for $type_:ident { fn $method:ident }) => {
        impl $trait_<$type_> for $type_ {
            type Output = $type_;

            fn $method(self, $type_(b): $type_) -> $type_ {
                let $type_(a) = self;
                let res = a.$method(&b);
                if res.bits() > 256 {
                    panic!("attempt to {} with overflow", stringify!($method));
                }
                $type_(res)
            }
        }
    };
}

/// A macro that forwards a checked operator i.e. CheckedAdd
macro_rules! forward_checked_op {
    (impl $trait_:ident for $type_:ident { fn $method:ident }) => {
        impl $trait_ for $type_ {
            fn $method(&self, $type_(b): &$type_) -> Option<$type_> {
                let $type_(a) = self;
                a.$method(&b)
                    .filter(|value| value.bits() <= 256)
                    .map($type_)
            }
        }
    };
}

/// A macro that forwards a assignment operator i.e. AddAssign
macro_rules! forward_assign_op {
    (impl $trait_:ident for $type_:ident { fn $method:ident }) => {
        impl $trait_ for $type_ {
            fn $method(&mut self, $type_(b): $type_) {
                let a = &mut self.0;
                a.$method(b);
                if a.bits() > 256 {
                    panic!("attempt to {} with overflow", stringify!($method));
                }
            }
        }
    };
}

forward_op! { impl Add for u256 { fn add } }
forward_checked_op! { impl CheckedAdd for u256 { fn checked_add } }
forward_assign_op! { impl AddAssign for u256 { fn add_assign } }

forward_op! { impl Sub for u256 { fn sub } }
forward_checked_op! { impl CheckedSub for u256 { fn checked_sub } }
forward_assign_op! { impl SubAssign for u256 { fn sub_assign } }

forward_op! { impl Mul for u256 { fn mul } }
forward_checked_op! { impl CheckedMul for u256 { fn checked_mul } }
forward_assign_op! { impl MulAssign for u256 { fn mul_assign } }

forward_op! { impl Div for u256 { fn div } }
forward_checked_op! { impl CheckedDiv for u256 { fn checked_div } }
forward_assign_op! { impl DivAssign for u256 { fn div_assign } }

impl scale::Encode for u256 {
    fn size_hint(&self) -> usize {
        let bits = self.0.bits() as usize;
        ((bits + 7) >> 3) + 1
    }

    fn encode(&self) -> Vec<u8> {
        let size = self.size_hint();
        debug_assert!(size < 34 && size > 0);

        let mut buf = Vec::with_capacity(size);
        buf.push(size as u8);
        buf.extend(self.0.to_bytes_be());
        buf
    }
}

impl scale::Decode for u256 {
    fn decode<I: scale::Input>(value: &mut I) -> Result<Self, scale::Error> {
        let size = value.read_byte()?;
        let mut buf = from_elem(0, (size - 1) as usize);
        value.read(buf.as_mut_slice())?;
        Ok(Self::from_be_bytes(&buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u256_codec() {
        let origin: u256 = 0x1234567890abcdefu64.into();
        let encoded = scale::Encode::encode(&origin);
        let decoded: u256 = scale::Decode::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(origin, decoded);
    }

    #[test]
    fn create_from_32_bytes() {
        let lhs: [u8; 32] = [
            0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42,
            0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42,
            0x42, 0x42, 0x42, 0x42, 0x42, 0x42,
        ];
        let lhs: u256 = lhs.into();
        assert_eq!(
            lhs,
            "0x4242424242424242424242424242424242424242424242424242424242424242"
                .parse()
                .unwrap()
        );
    }

    #[test]
    fn into_array() {
        let val = u256::from(1024u16);
        let data: [u8; 32] = val.to_be_bytes();
        assert_eq!(
            data,
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 4, 0
            ]
        );
    }

    #[test]
    fn check_display() {
        let val = u256::max_value();
        assert_eq!(
        format!("{}", val),
        "115792089237316195423570985008687907853269984665640564039457584007913129639935"
    );
        assert_eq!(
        val.to_string(),
        "115792089237316195423570985008687907853269984665640564039457584007913129639935"
    );
    }
}

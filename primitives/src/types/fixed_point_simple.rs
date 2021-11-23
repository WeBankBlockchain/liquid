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

#[allow(unused_imports)]
use core::{default::Default, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign}, panic, str::FromStr, cmp::Ordering, fmt, ops::Shr};
use liquid_prelude::vec::{from_elem, Vec};
pub use fixed::{FixedU64, types::extra::{U16}};
use num::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct FixedPointU64F16{
    symbol: u8, 
    value: FixedU64<U16>,
}

impl FixedPointU64F16 {
    pub fn from_be_bytes(slice: &[u8]) -> Self {
        let mut bytes:[u8; 8] = [3;8];
        for n in 1..9{
            bytes[n-1] = slice[n];
        }
        Self{
            symbol:slice[0], 
            value: FixedU64::<U16>::from_be_bytes(bytes)
        }
    }

    pub fn from_user(str: &str) -> Self{
        let mut ans:FixedPointU64F16 = Default::default();
        if str.chars().nth(0) == Some('-') {
            ans.symbol = 255;
            ans.value = FixedU64::<U16>::from_str(&str[1..]).unwrap();
        }
        else {
            ans.symbol = 0;
            ans.value = FixedU64::<U16>::from_str(&str[..]).unwrap();
        }
        if ans.value == FixedU64::<U16>::from_num(0){
            ans.symbol = 0;
        }
        if ans > FixedPointU64F16:: max_value() {
            panic!("the value exceed MAX")
        }
        ans
    }
}

impl core::ops::Shl<usize> for FixedPointU64F16 {
    type Output = Self;

    fn shl(self, rhs: usize) -> Self::Output {
        Self {
            symbol: self.symbol,
            value: self.value << rhs,
        }
    }
}

impl Shr<usize> for FixedPointU64F16 {
    type Output = Self;

    fn shr(self, rhs: usize) -> Self::Output {
        Self {
            symbol: self.symbol,
            value: self.value >> rhs,
        }
    }
}


impl Bounded for FixedPointU64F16 {
    fn min_value() -> Self {
        FixedPointU64F16 {
            symbol: 255,
            value: FixedU64::<U16>::from_num(1099511627775.99998)
        }
    }

    fn max_value() -> Self {
        FixedPointU64F16 { 
            symbol: 0, 
            value: FixedU64::<U16>::from_num(1099511627775.99998) 
        }
    }
}

impl Ord for FixedPointU64F16 {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        if self.symbol > other.symbol {
            return Ordering::Less;
        }
        else if self.symbol < other.symbol {
            return Ordering::Greater;
        }
        else if self.symbol == 0 {
            return self.value.cmp(&other.value)
        }
        else {
            return other.value.cmp(&self.value)
        }
    }
}

impl PartialOrd for FixedPointU64F16 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for FixedPointU64F16 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut symbol = "";
        if self.symbol == 255 {
            symbol = "-";
        }
        write!(f, "{}{}", &symbol, self.value)
    }
}

impl fmt::Debug for FixedPointU64F16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut symbol = "";
        if self.symbol == 255 {
            symbol = "-";
        }
        write!(f, "FixedPointU64F16({}{})", &symbol, self.value)
    }
}

impl Add for FixedPointU64F16 {
    type Output = FixedPointU64F16;
    fn add(self, rhs: Self) -> Self::Output {
        let mut ans = self;
        if self.symbol == rhs.symbol {
            ans.value = rhs.value + ans.value;
        }
        else if ans.value == rhs.value {
            ans.value = FixedU64::<U16>::from_num(0);
        }
        else if ans.value > rhs.value {
            ans.value = ans.value - rhs.value;            
        }
        else {
            ans.symbol = rhs.symbol;
            ans.value = rhs.value - ans.value;
        }
        if ans > FixedPointU64F16::max_value() {
            panic!("the value exceed MAX")
        }
        ans
    }
}

impl Sub for FixedPointU64F16 {
    type  Output = FixedPointU64F16;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut ans=self;
        if ans.symbol != rhs.symbol {
            ans.value = ans.value + rhs.value;
        }
        else if ans.value == rhs.value {
            ans.symbol = 0;
            ans.value = FixedU64::<U16>::from_num(0);
        }
        else if ans.value > rhs.value {
            ans.value = ans.value - rhs.value;
        }
        else {
            ans.symbol = rhs.symbol;
            ans.value = rhs.value - ans.value;
        }
        ans
    }
}

// impl SubAssign for FixedPointU64F16 {
//     fn sub_assign(&mut self, rhs: Self) {
//         // copy trait???
//         // let mut a = self;
//         // (*self) = a.sub(rhs);
//         *self = self.sub(rhs);
//     }
// }

macro_rules! forward_op {
    (impl $trait_: ident for $type_:ident {fn $method:ident}) => {
        impl $trait_ for $type_{
            type Output = $type_;
            fn $method(self, b: $type_) -> $type_ {
                let mut ans = self;
                ans.symbol = ans.symbol ^ b.symbol;
                ans.value = ans.value.$method(b.value);
                ans
            }
        }
    };
}
forward_op!{impl Mul for FixedPointU64F16 { fn mul} }
forward_op!{impl Div for FixedPointU64F16 { fn div} }

macro_rules! forward_checked_op {
    (impl $trait_:ident for $type_:ident { fn $method: ident} $methodinvoke: ident) => {
        impl $trait_ for $type_ {
            fn $method(&self, b: &$type_) -> Option<$type_> {
                Some(self.$methodinvoke(*b))
            }
        }
    }
}
forward_checked_op! { impl CheckedAdd for FixedPointU64F16 { fn checked_add } add}
forward_checked_op! { impl CheckedSub for FixedPointU64F16 { fn checked_sub } sub}
forward_checked_op! { impl CheckedMul for FixedPointU64F16 { fn checked_mul } mul}
forward_checked_op! { impl CheckedDiv for FixedPointU64F16 { fn checked_div } div}

macro_rules! forward_assign_op {
    (impl $trait_: ident for $type_: ident { fn $method: ident} $methodinvoke: ident) => {
        impl $trait_ for $type_{
            fn $method(&mut self, rhs: Self) {
                *self = self.$methodinvoke(rhs)
            }
        }
    }
}

forward_assign_op!{impl AddAssign for FixedPointU64F16 { fn add_assign} add}
forward_assign_op!{impl SubAssign for FixedPointU64F16 { fn sub_assign} sub}
forward_assign_op!{impl MulAssign for FixedPointU64F16 { fn mul_assign} mul}
forward_assign_op!{impl DivAssign for FixedPointU64F16 { fn div_assign} div}




impl scale::Encode for FixedPointU64F16 {
    fn encode(&self) -> Vec<u8> {
        let mut buf:Vec<u8> = Vec::with_capacity(8 as usize);
        buf.extend(self.symbol.encode());
        let bytes:[u8; 8 ]= self.value.to_be_bytes();
        buf.extend(bytes.encode());
        buf
    }
}

impl scale::Decode for FixedPointU64F16 {
    fn decode<I: scale::Input>(value: &mut I) -> Result<Self, scale::Error> {
        let mut buf = from_elem(0, 9 as usize);
        value.read(buf.as_mut_slice())?;
        Ok(Self::from_be_bytes(&buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_point_encode1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("12345678.987");
        let encoded = scale::Encode::encode(&origin2);
        assert_eq!(encoded, [0,0,0,0,188,97,78,252,172]);
    }

    #[test]
    fn fixed_point_encode2() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("-0");
        let encoded = scale::Encode::encode(&origin2);
        assert_eq!(encoded, [0,0,0,0,0,0,0,0,0]);
    }

    #[test]
    fn fixed_point_encode3() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("0");
        let encoded = scale::Encode::encode(&origin2);
        assert_eq!(encoded, [0,0,0,0,0,0,0,0,0]);
    }

    #[test]
    fn fixed_point_codec1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("-0");
        let encoded = scale::Encode::encode(&origin2);
        let decoded: FixedPointU64F16  = scale::Decode::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(origin2, decoded);
    }

    #[test]
    fn fixed_point_codec2() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("0");
        let encoded = scale::Encode::encode(&origin2);
        let decoded: FixedPointU64F16  = scale::Decode::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(origin2, decoded);
    }

    #[test]
    fn fixed_point_codec3() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("1");
        let encoded = scale::Encode::encode(&origin2);
        let decoded: FixedPointU64F16  = scale::Decode::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(origin2, decoded);
    }

    #[test]
    fn fixed_point_codec4() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("1099511627775.99998");
        let encoded = scale::Encode::encode(&origin2);
        let decoded: FixedPointU64F16  = scale::Decode::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(origin2, decoded);
    }

    #[test]
    fn fixed_point_codec5() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("1.25");
        let encoded = scale::Encode::encode(&origin2);
        let decoded: FixedPointU64F16  = scale::Decode::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(origin2, decoded);
    }

    #[test]
    fn fixed_point_add1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("-1.25");
        let origin = FixedPointU64F16::from_user("2.15");
        let result = origin+origin2;
        assert_eq!(result, FixedPointU64F16 { symbol: 0, value: FixedU64::<U16>::from_num(0.9)});
    }

    #[test]
    fn fixed_point_add_assign1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("-1.25");
        let mut origin = FixedPointU64F16::from_user("-2.15");
        origin += origin2;
        assert_eq!(origin, FixedPointU64F16::from_user("-3.4"));
    }

    #[test]
    fn fixed_point_mul1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("-1.2");
        let mut origin = FixedPointU64F16::from_user("2.1");
        origin = origin.mul(origin2);
        // 多么不准确的实现
        assert_eq!(origin, FixedPointU64F16::from_user("-2.51999"));
    }

    #[test]
    fn fixed_point_mul_assign1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("-1.2");
        let mut origin = FixedPointU64F16::from_user("-2.1");
        origin *= origin2;
        // 多么不准确的实现
        assert_eq!(origin, FixedPointU64F16::from_user("2.51999"));
    }

    #[test]
    fn fixed_point_cmp1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("1.2");
        let origin = FixedPointU64F16::from_user("-2.1");
        assert_eq!(origin.cmp(&origin2), Ordering::Less)
    }

    #[test]
    fn fixed_point_shl1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("1.25");
        assert_eq!(origin2<<1, FixedPointU64F16::from_user("2.5"))
    }

    #[test]
    fn fixed_point_shr1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("1.25");
        assert_eq!(origin2>>1, FixedPointU64F16::from_user("0.625"))
    }

    #[test]
    fn fixed_point_checkadd1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("1.2");
        let origin = FixedPointU64F16::from_user("-2.1");
        let origin3= origin.checked_add(&origin2);
        assert_eq!(origin3, Some(FixedPointU64F16::from_user("-0.90001")))
    }
}

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
use core::{default::Default, ops::{Add, AddAssign, Div, DivAssign, Deref, Mul, MulAssign, Sub, SubAssign}, panic, str::FromStr};
use liquid_prelude::vec::{from_elem, Vec};
pub use fixed::{FixedU64, types::extra::{U16}};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
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
        if ans.value > FixedU64::<U16>::from_num(1099511627775.99998) {
            panic!("the value exceed MAX")
        }
        // println!("{:?}", ans);     
        ans
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
        return ans;
    }
}


// impl AddAssign for FixedPointU64F16{
//     fn add_assign(&mut self, rhs: Self) {
//         let mut a = self;
//         a = &mut a.add(rhs);
//         // println!("haha: {:?}", a);
//     }
// }

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
        return ans;
    }
}

// impl SubAssign for FixedPointU64F16 {
//     fn sub_assign(&mut self, rhs: Self) {
//         let a = &self;
//         a.sub(rhs);
//     }
// }

impl Mul for FixedPointU64F16{
    type Output = FixedPointU64F16;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut ans = self;
        ans.symbol = ans.symbol | rhs.symbol;
        ans.value = ans.value.mul(rhs.value);
        return ans;
    }    
}

// impl MulAssign for FixedPointU64F16 {
//     fn mul_assign(&mut self, rhs: Self) {
//         let ans = &self;
//         ans.mul(rhs);
//     }
// }

impl Div for FixedPointU64F16 {
    type Output = FixedPointU64F16;
    fn div(self, rhs: Self) -> Self::Output {
        let mut ans = self;
        ans.symbol = ans.symbol | rhs.symbol;
        ans.value = ans.value.div(rhs.value);
        return ans;
    }
}

// impl DivAssign for FixedPointU64F16 {
//     fn div_assign(&mut self, rhs: Self) {
//         let ans = &self;
//         ans.div(rhs);
//     }
// }


// macro_rules! forward_op {
//     (impl $trait_: ident for $type_:ident {fn $method:ident}) => {
//         impl $trait_ for $type_{
//             type Output = $type_;
//             fn $method(self, $type_(b): $type_) -> $type_ {
//                 let $type_(ans) = self;
//                 ans.symbol = ans.symbol | b.symbol;
//                 ans.value = ans.value.$method(b.value);
//                 $type_(ans)
//             }
//         }
//     };
// }
// forward_op!{impl Mul for FixedPointU64F16 { fn mul} }


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
        println!("{:?}", origin2);
        let encoded = scale::Encode::encode(&origin2);
        assert_eq!(encoded, [0,0,0,0,188,97,78,252,172]);
    }

    #[test]
    fn fixed_point_encode2() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("-0");
        println!("{:?}", origin2);
        let encoded = scale::Encode::encode(&origin2);
        assert_eq!(encoded, [0,0,0,0,0,0,0,0,0]);
    }

    #[test]
    fn fixed_point_encode3() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("0");
        println!("{:?}", origin2);
        let encoded = scale::Encode::encode(&origin2);
        assert_eq!(encoded, [0,0,0,0,0,0,0,0,0]);
    }

    #[test]
    fn fixed_point_codec1() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("-0");
        println!("{:?}", origin2);
        let encoded = scale::Encode::encode(&origin2);
        println!("{:?}", encoded);
        let decoded: FixedPointU64F16  = scale::Decode::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(origin2, decoded);
    }

    #[test]
    fn fixed_point_codec2() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("0");
        println!("{:?}", origin2);
        let encoded = scale::Encode::encode(&origin2);
        println!("{:?}", encoded);
        let decoded: FixedPointU64F16  = scale::Decode::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(origin2, decoded);
    }

    #[test]
    fn fixed_point_codec3() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("1");
        println!("{:?}", origin2);
        let encoded = scale::Encode::encode(&origin2);
        println!("{:?}", encoded);
        let decoded: FixedPointU64F16  = scale::Decode::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(origin2, decoded);
    }

    #[test]
    fn fixed_point_codec4() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("1099511627775.99998");
        println!("{:?}", origin2);
        let encoded = scale::Encode::encode(&origin2);
        println!("{:#?}", encoded);
        let decoded: FixedPointU64F16  = scale::Decode::decode(&mut encoded.as_slice()).unwrap();
        assert_eq!(origin2, decoded);
    }

    #[test]
    fn fixed_point_codec5() {
        let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("1.25");
        println!("{:?}", origin2);
        let encoded = scale::Encode::encode(&origin2);
        println!("{:#?}", encoded);
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

    // #[test]
    // fn fixed_point_add_assign1() {
    //     let origin2:FixedPointU64F16 = FixedPointU64F16::from_user("-1.25");
    //     let mut origin = FixedPointU64F16::from_user("2.15");
    //     origin.add_assign(origin2);
    //     println!("{:?}", origin);
    //     assert_eq!(origin, FixedPointU64F16 { symbol: 0, value: FixedU64::<U16>::from_num(0.9)});
    // }
}

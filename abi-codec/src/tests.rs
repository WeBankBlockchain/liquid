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

use crate::codec::{Decode, Encode};
use hex_literal::hex;
use liquid_primitives::types::{i256, u256, Address};

macro_rules! test_encode_decode {
    (name: $name:ident,type: $t:ty,value: $value:expr,data: $data:expr) => {
        paste::item! {
            #[test]
            fn [<encode_ $name>]() {
                let encoded = <$t>::encode(&$value);
                let expected = hex!($data).to_vec();
                assert_eq!(encoded, expected);
            }

            #[test]
            fn [<decode_ $name>]() {
                let data = hex!($data);
                let decoded = <$t>::decode(&mut &data[..]);
                assert_eq!(Ok($value), decoded);
            }
        }
    };
}

macro_rules! test_decode_fail {
    (name: $name:ident,type: $t:ty,data: $data:expr) => {
        paste::item! {
            #[test]
            fn [<decode_fail_ $name>]() {
                let data = hex!($data);
                let decoded = <$t>::decode(&mut &data[..]);
                assert!(decoded.is_err());
            }
        }
    };
}

test_encode_decode! {
    name: bool_0,
    type: (bool,),
    value: (false,),
    data: "0000000000000000000000000000000000000000000000000000000000000000"
}

test_encode_decode! {
    name: bool_1,
    type: (bool,),
    value: (true,),
    data: "0000000000000000000000000000000000000000000000000000000000000001"
}

test_decode_fail! {
    name: bool_0,
    type: (bool,),
    data: "0000000000000000000000000000000000000000000000000000000000000002"
}

test_decode_fail! {
    name: bool_1,
    type: (bool,),
    data: "1000000000000000000000000000000000000000000000000000000000000000"
}

test_decode_fail! {
    name: bool_2,
    type: (bool,),
    data: "0123456789ABCDEF"
}

test_encode_decode! {
    name: uint8,
    type: (u8,),
    value: (127u8,),
    data: "000000000000000000000000000000000000000000000000000000000000007F"
}

test_decode_fail! {
    name: uint8,
    type: (u8,),
    data: "000000000000001000000000000000000000000000000000000000000000007F"
}

test_encode_decode! {
    name: uint16,
    type: (u16,),
    value: (!u16::pow(2, 15),),
    data: "0000000000000000000000000000000000000000000000000000000000007FFF"
}

test_decode_fail! {
    name: uint16,
    type: (u16,),
    data: "0000000000000010000000000000000000000000000000000000000000007FFF"
}

test_encode_decode! {
    name: uint32,
    type: (u32,),
    value: (!u32::pow(2, 31),),
    data: "000000000000000000000000000000000000000000000000000000007FFFFFFF"
}

test_decode_fail! {
    name: uint32,
    type: (u32,),
    data: "000000000000001000000000000000000000000000000000000000007FFFFFFF"
}

test_encode_decode! {
    name: uint64,
    type: (u64,),
    value: (!u64::pow(2, 63),),
    data: "0000000000000000000000000000000000000000000000007FFFFFFFFFFFFFFF"
}

test_decode_fail! {
    name: uint64,
    type: (u64,),
    data: "0000000000000010000000000000000000000000000000007FFFFFFFFFFFFFFF"
}

test_encode_decode! {
    name: uint128,
    type: (u128,),
    value: (!u128::pow(2, 127),),
    data: "000000000000000000000000000000007FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
}

test_decode_fail! {
    name: uint128,
    type: (u128,),
    data: "000000000000001000000000000000007FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
}

test_encode_decode! {
    name: int8_0,
    type: (i8,),
    value: (-1,),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
}

test_encode_decode! {
    name: int8_1,
    type: (i8,),
    value: (i8::MIN,),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF80"
}

test_encode_decode! {
    name: int8_2,
    type: (i8,),
    value: (0,),
    data: "0000000000000000000000000000000000000000000000000000000000000000"
}

test_encode_decode! {
    name: int8_3,
    type: (i8,),
    value: (1,),
    data: "0000000000000000000000000000000000000000000000000000000000000001"
}

test_decode_fail! {
    name: int8,
    type: (i8,),
    data: "F000000000000000000000000000000000000000000000000000000000000001"
}

test_encode_decode! {
    name: int16_0,
    type: (i16,),
    value: (-1,),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
}

test_encode_decode! {
    name: int16_1,
    type: (i16,),
    value: (i16::MIN,),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8000"
}

test_encode_decode! {
    name: int16_2,
    type: (i16,),
    value: (0,),
    data: "0000000000000000000000000000000000000000000000000000000000000000"
}

test_encode_decode! {
    name: int16_3,
    type: (i16,),
    value: (1,),
    data: "0000000000000000000000000000000000000000000000000000000000000001"
}

test_decode_fail! {
    name: int16,
    type: (i16,),
    data: "F000000000000000000000000000000000000000000000000000000000000001"
}

test_encode_decode! {
    name: int32_0,
    type: (i32,),
    value: (-1,),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
}

test_encode_decode! {
    name: int32_1,
    type: (i32,),
    value: (i32::MIN,),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF80000000"
}

test_encode_decode! {
    name: int32_2,
    type: (i32,),
    value: (0,),
    data: "0000000000000000000000000000000000000000000000000000000000000000"
}

test_encode_decode! {
    name: int32_3,
    type: (i32,),
    value: (1,),
    data: "0000000000000000000000000000000000000000000000000000000000000001"
}

test_decode_fail! {
    name: int32,
    type: (i32,),
    data: "F000000000000000000000000000000000000000000000000000000000000001"
}

test_encode_decode! {
    name: int64_0,
    type: (i64,),
    value: (-1,),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
}

test_encode_decode! {
    name: int64_1,
    type: (i64,),
    value: (i64::MIN,),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8000000000000000"
}

test_encode_decode! {
    name: int64_2,
    type: (i64,),
    value: (0,),
    data: "0000000000000000000000000000000000000000000000000000000000000000"
}

test_encode_decode! {
    name: int64_3,
    type: (i64,),
    value: (1,),
    data: "0000000000000000000000000000000000000000000000000000000000000001"
}

test_decode_fail! {
    name: int64,
    type: (i64,),
    data: "F000000000000000000000000000000000000000000000000000000000000001"
}

test_encode_decode! {
    name: int128_0,
    type: (i128,),
    value: (-1,),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
}

test_encode_decode! {
    name: int128_1,
    type: (i128,),
    value: (i128::MIN,),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF80000000000000000000000000000000"
}

test_encode_decode! {
    name: int128_2,
    type: (i128,),
    value: (0,),
    data: "0000000000000000000000000000000000000000000000000000000000000000"
}

test_encode_decode! {
    name: int128_3,
    type: (i128,),
    value: (1,),
    data: "0000000000000000000000000000000000000000000000000000000000000001"
}

test_decode_fail! {
    name: int128,
    type: (i128,),
    data: "F000000000000000000000000000000000000000000000000000000000000001"
}

test_encode_decode! {
    name: uint8_uint8_bool,
    type: (u8, u8, bool),
    value: (1, 2, true),
    data: "0000000000000000000000000000000000000000000000000000000000000001
    0000000000000000000000000000000000000000000000000000000000000002
    0000000000000000000000000000000000000000000000000000000000000001"
}

test_encode_decode! {
    name: int32_uint8_bool,
    type: (i32, i8, bool),
    value: (-1, 2, true),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    0000000000000000000000000000000000000000000000000000000000000002
    0000000000000000000000000000000000000000000000000000000000000001"
}

test_encode_decode! {
    name: int32_int128_bool_uint64,
    type: (i32, i128, bool, u64),
    value: (-1, -42, false, 42),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFD6
    0000000000000000000000000000000000000000000000000000000000000000
    000000000000000000000000000000000000000000000000000000000000002A"
}

test_decode_fail! {
    name: int32_int128_bool_uint64,
    type: (i32, i128, bool, u64),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFD6
    0000000000000000000000000000000000000000000000000000000000000000
    FF0000000000000000000000000000000000000000000000000000000000002A"
}

test_encode_decode! {
    name: unit,
    type: (),
    value: (),
    data: ""
}

test_encode_decode! {
    name: string_0,
    type: (String,),
    value: ("Hello, World".to_owned(),),
    data: "0000000000000000000000000000000000000000000000000000000000000020
    000000000000000000000000000000000000000000000000000000000000000c
    48656c6c6f2c20576f726c640000000000000000000000000000000000000000"
}

test_encode_decode! {
    name: string_1,
    type: (String,),
    value: ("こんにちは，世界".to_owned(),),
    data: "0000000000000000000000000000000000000000000000000000000000000020
    0000000000000000000000000000000000000000000000000000000000000018
    e38193e38293e381abe381a1e381afefbc8ce4b896e7958c0000000000000000"
}

test_encode_decode! {
    name: string_2,
    type: (String,),
    value: ("落霞与孤鹜齐飞，秋水共长天一色。渔舟唱晚，响穷彭蠡之滨，雁阵惊寒，声断衡阳之浦。".to_owned(),),
    data: "0000000000000000000000000000000000000000000000000000000000000020
    0000000000000000000000000000000000000000000000000000000000000078
    e890bde99c9ee4b88ee5ada4e9b99ce9bd90e9a39eefbc8ce7a78be6b0b4e585
    b1e995bfe5a4a9e4b880e889b2e38082e6b894e8889fe594b1e6999aefbc8ce5
    938de7a9b7e5bdade8a0a1e4b98be6bba8efbc8ce99b81e998b5e6838ae5af92
    efbc8ce5a3b0e696ade8a1a1e998b3e4b98be6b5a6e380820000000000000000"
}

test_encode_decode! {
    name: string_3,
    type: (String,),
    value: ("煙婲﹃样嘚男孒，薱：莪狠傻..対伱　动ろ情".to_owned(),),
    data: "0000000000000000000000000000000000000000000000000000000000000020
    000000000000000000000000000000000000000000000000000000000000003b
    e78599e5a9b2efb983e6a0b7e5989ae794b7e5ad92efbc8ce896b1efbc9ae88e
    aae78ba0e582bb2e2ee5afbee4bcb1e38080e58aa8e3828de683850000000000"
}

test_encode_decode! {
    name: string_4,
    type: (String,),
    value: ("🐴的，🌶💉💦🐮🍺".to_owned(),),
    data: "0000000000000000000000000000000000000000000000000000000000000020
    000000000000000000000000000000000000000000000000000000000000001e
    f09f90b4e79a84efbc8cf09f8cb6f09f9289f09f92a6f09f90aef09f8dba0000"
}

test_encode_decode! {
    name: string_uint32_string,
    type: (String, u32, String),
    value: ("Hello, World".to_owned(), 42, "Bye, World".to_owned()),
    data: "0000000000000000000000000000000000000000000000000000000000000060
    000000000000000000000000000000000000000000000000000000000000002a
    00000000000000000000000000000000000000000000000000000000000000a0
    000000000000000000000000000000000000000000000000000000000000000c
    48656c6c6f2c20576f726c640000000000000000000000000000000000000000
    000000000000000000000000000000000000000000000000000000000000000a
    4279652c20576f726c6400000000000000000000000000000000000000000000"
}

test_encode_decode! {
    name: bool_uint32_string_int8,
    type: (bool, u32, String, i8),
    value: (true, 42, "Hello, World".to_owned(), -42),
    data: "0000000000000000000000000000000000000000000000000000000000000001
    000000000000000000000000000000000000000000000000000000000000002a
    0000000000000000000000000000000000000000000000000000000000000080
    ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd6
    000000000000000000000000000000000000000000000000000000000000000c
    48656c6c6f2c20576f726c640000000000000000000000000000000000000000"
}

test_encode_decode! {
    name: vec_uint8,
    type: (Vec<i8>,),
    value: (vec![0, 1, 2, 3],),
    data: "0000000000000000000000000000000000000000000000000000000000000020
    0000000000000000000000000000000000000000000000000000000000000004
    0000000000000000000000000000000000000000000000000000000000000000
    0000000000000000000000000000000000000000000000000000000000000001
    0000000000000000000000000000000000000000000000000000000000000002
    0000000000000000000000000000000000000000000000000000000000000003"
}

test_encode_decode! {
    name: address,
    type: (Address,),
    value: (Address::from([
        0x3e, 0x9A, 0xFa, 0xA4, 0xa0, 0x62, 0xA4, 0x9d, 0x64, 0xb8, 0xAb, 0x05, 0x7B,
        0x3C, 0xb5, 0x18, 0x92, 0xe1, 0x7E, 0xcb,
    ]),),
    data: "0000000000000000000000003e9afaa4a062a49d64b8ab057b3cb51892e17ecb"
}

test_encode_decode! {
    name: int256,
    type: (i256,),
    value: (i256::from(-1),),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
}

test_encode_decode! {
    name: uint256,
    type: (u256,),
    value: (u256::from([0xff; 32]),),
    data: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
}

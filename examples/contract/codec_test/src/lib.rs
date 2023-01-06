#![cfg_attr(not(feature = "std"), no_std)]

use liquid::{storage, InOut};
use liquid_lang as liquid;

#[liquid::contract]
mod hello_world {
    use super::*;

    #[liquid(event)]
    struct Foo {
        s: String,
        i: i32,
        u: u256,
        a: Address,
        b: ([String; 2], u64, i64),
    }

    #[derive(InOut, Clone)]
    #[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
    pub struct User {
        name: String,
        age: u8,
        others: (String, u64, i64, Address),
    }

    #[derive(InOut, Clone)]
    #[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
    pub enum Message {
        Level(u32),
        Msg(String),
    }

    #[liquid(storage)]
    struct HelloWorld {
        name: storage::Value<String>,
        v_str: storage::Value<String>,
        v_u8: storage::Value<u8>,
        v_u16: storage::Value<u16>,
        v_u32: storage::Value<u32>,
        v_u64: storage::Value<u64>,
        v_u128: storage::Value<u128>,
        v_u256: storage::Value<u256>,
        v_i8: storage::Value<i8>,
        v_i16: storage::Value<i16>,
        v_i32: storage::Value<i32>,
        v_i64: storage::Value<i64>,
        v_i128: storage::Value<i128>,
        v_i256: storage::Value<i256>,
        v_bool: storage::Value<bool>,
        v_addr: storage::Value<Address>,
        v_bytes: storage::Value<bytes>,
        v_bytes1: storage::Value<bytes1>,
        v_bytes16: storage::Value<bytes16>,
        v_bytes32: storage::Value<bytes32>,
        v_user: storage::Value<User>,
        v_tuple: storage::Value<(u32, i32, String)>,
        v_array: storage::Value<[i32; 5]>,
        // v_vec: storage::Value<Vec<[(u8, Address); 2]>>,
        v_enum: storage::Value<Message>,
        /* v_s_vec: storage::Vec<(String, u64)>,
         * v_s_mapping: storage::Mapping<String, u64>,
         */
    }

    #[liquid(methods)]
    impl HelloWorld {
        pub fn new(&mut self) {
            self.name.initialize(String::from("Test"));
            self.v_str.initialize(String::from("Alice"));
            self.v_u8.initialize(8);
            self.v_u16.initialize(16);
            self.v_u32.initialize(32);
            self.v_u64.initialize(64);
            self.v_u128.initialize(128);
            self.v_u256.initialize(256.into());
            self.v_i8.initialize(8);
            self.v_i16.initialize(16);
            self.v_i32.initialize(32);
            self.v_i64.initialize(64);
            self.v_i128.initialize(128);
            self.v_i256.initialize(256.into());
            self.v_bool.initialize(false);
            self.v_addr.initialize("0x".into());
            self.v_bytes.initialize(bytes::new());
            self.v_bytes1.initialize("a".parse().unwrap());
            self.v_bytes16.initialize("b".parse().unwrap());
            self.v_bytes32.initialize("abc".parse().unwrap());
            self.v_user.initialize(User {
                name: String::from("Mark"),
                age: 100,
                others: (String::from("haha"), 123, -123, "0x".into()),
            });
            self.v_tuple.initialize((1u32, -1i32, String::from("aaaa")));
            self.v_array.initialize([1, 2, -1, -5, 100]);
            // self.v_vec.initialize(Vec::new());
            self.v_enum
                .initialize(Message::Msg(String::from("message")));
            // self.v_s_vec.initialize();
            // self.v_s_mapping.initialize();
        }

        pub fn get(&self) -> String {
            self.name.clone()
        }

        pub fn set(&mut self, name: String) {
            self.name.set(name)
        }

        pub fn get_str(&self) -> String {
            // self.env().emit(Foo {
            //     s: String::from("get string"),
            //     i: 32,
            //     u: 256.into(),
            //     a: String::from("0x1").into(),
            // });
            self.v_str.clone()
        }

        pub fn set_str(&mut self, v: String) {
            self.env().emit(Foo {
                s: String::from("set string"),
                i: 32,
                u: 256.into(),
                a: String::from("0x2").into(),
                b: ([String::from("ab c"), String::from("aaaa")], 22u64, -1i64),
            });
            self.v_str.set(v)
        }

        pub fn get_u8(&self) -> u8 {
            self.v_u8.clone()
        }

        pub fn set_u8(&mut self, v: u8) {
            self.v_u8.set(v)
        }

        pub fn get_u16(&self) -> u16 {
            self.v_u16.clone()
        }

        pub fn set_u16(&mut self, v: u16) {
            self.v_u16.set(v)
        }

        pub fn get_u32(&self) -> u32 {
            self.v_u32.clone()
        }

        pub fn set_u32(&mut self, v: u32) {
            self.v_u32.set(v)
        }

        pub fn get_u64(&self) -> u64 {
            self.v_u64.clone()
        }

        pub fn set_u64(&mut self, v: u64) {
            self.v_u64.set(v)
        }

        pub fn get_u128(&self) -> u128 {
            self.v_u128.clone()
        }

        pub fn set_u128(&mut self, v: u128) {
            self.v_u128.set(v)
        }

        pub fn get_u256(&self) -> u256 {
            self.v_u256.clone()
        }

        pub fn set_u256(&mut self, v: u256) {
            self.v_u256.set(v)
        }

        pub fn get_i8(&self) -> i8 {
            self.v_i8.clone()
        }

        pub fn set_i8(&mut self, v: i8) {
            self.v_i8.set(v)
        }

        pub fn get_i16(&self) -> i16 {
            self.v_i16.clone()
        }

        pub fn set_i16(&mut self, v: i16) {
            self.v_i16.set(v)
        }

        pub fn get_i32(&self) -> i32 {
            self.v_i32.clone()
        }

        pub fn set_i32(&mut self, v: i32) {
            self.v_i32.set(v)
        }

        pub fn get_i64(&self) -> i64 {
            self.v_i64.clone()
        }

        pub fn set_i64(&mut self, v: i64) {
            self.v_i64.set(v)
        }

        pub fn get_i128(&self) -> i128 {
            self.v_i128.clone()
        }

        pub fn set_i128(&mut self, v: i128) {
            self.v_i128.set(v)
        }

        pub fn get_i256(&self) -> i256 {
            self.v_i256.clone()
        }

        pub fn set_i256(&mut self, v: i256) {
            self.v_i256.set(v)
        }

        pub fn get_bool(&self) -> bool {
            self.v_bool.clone()
        }

        pub fn set_bool(&mut self, v: bool) {
            self.v_bool.set(v)
        }

        pub fn get_addr(&self) -> Address {
            self.v_addr.clone()
        }

        pub fn set_addr(&mut self, v: Address) {
            self.v_addr.set(v)
        }

        pub fn get_bytes(&self) -> bytes {
            self.v_bytes.get().clone()
        }

        pub fn set_bytes(&mut self, v: bytes) {
            self.v_bytes.set(v)
        }

        pub fn get_bytes1(&self) -> bytes1 {
            self.v_bytes1.clone()
        }

        pub fn set_bytes1(&mut self, v: bytes1) {
            self.v_bytes1.set(v)
        }

        pub fn get_bytes16(&self) -> bytes16 {
            self.v_bytes16.clone()
        }

        pub fn set_bytes16(&mut self, v: bytes16) {
            self.v_bytes16.set(v)
        }

        pub fn get_bytes32(&self) -> bytes32 {
            self.v_bytes32.clone()
        }

        pub fn set_bytes32(&mut self, v: bytes32) {
            self.v_bytes32.set(v)
        }

        pub fn get_user(&self) -> User {
            self.v_user.clone()
        }

        pub fn set_user(&mut self, v: User) {
            self.v_user.set(v)
        }

        pub fn get_tuple(&self) -> (u32, i32, String) {
            self.v_tuple.get().clone()
        }

        pub fn set_tuple(&mut self, v: (u32, i32, String)) {
            self.v_tuple.set(v)
        }

        pub fn get_array(&self) -> [i32; 5] {
            self.v_array.get().clone()
        }

        pub fn set_array(&mut self, v: [i32; 5]) {
            self.v_array.set(v)
        }

        // pub fn get_vec(&self) -> Vec<[(u8, Address); 2]> {
        //     self.v_vec.get().clone()
        // }

        // pub fn set_vec(&mut self, v: Vec<[(u8, Address); 2]>) {
        //     self.v_vec.set(v)
        // }

        pub fn get_enum(&self) -> Message {
            self.v_enum.get().clone()
        }

        pub fn set_enum(&mut self, v: Message) {
            self.v_enum.set(v)
        }

        // pub fn vec_get(&self, i: u32) -> (String, u64) {
        //     self.v_s_vec[i].clone()
        // }

        // pub fn vec_push(&mut self, v: (String, u64)) {
        //     self.v_s_vec.push(v)
        // }

        // pub fn vec_len(& self) -> u32 {
        //     self.v_s_vec.len()
        // }

        // pub fn mapping_get(&self, k: String) -> u64 {
        //     self.v_s_mapping[&k].clone()
        // }

        // pub fn mapping_insert(&mut self, k: String, v: u64) {
        //     self.v_s_mapping.insert(k, v);
        // }

        // pub fn mapping_len(&mut self) -> u32 {
        //     self.v_s_mapping.len()
        // }

        // pub fn u8_add(&self, v1: u8, v2: u8) -> u8 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn u8_sub(&self, v1: u8, v2: u8) -> u8 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn u8_mul(&self, v1: u8, v2: u8) -> u8 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn u8_div(&self, v1: u8, v2: u8) -> u8 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn u16_add(&self, v1: u16, v2: u16) -> u16 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn u16_sub(&self, v1: u16, v2: u16) -> u16 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn u16_mul(&self, v1: u16, v2: u16) -> u16 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn u16_div(&self, v1: u16, v2: u16) -> u16 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn u32_add(&self, v1: u32, v2: u32) -> u32 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn u32_sub(&self, v1: u32, v2: u32) -> u32 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn u32_mul(&self, v1: u32, v2: u32) -> u32 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn u32_div(&self, v1: u32, v2: u32) -> u32 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn u64_add(&self, v1: u64, v2: u64) -> u64 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn u64_sub(&self, v1: u64, v2: u64) -> u64 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn u64_mul(&self, v1: u64, v2: u64) -> u64 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn u64_div(&self, v1: u64, v2: u64) -> u64 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn u128_add(&self, v1: u128, v2: u128) -> u128 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn u128_sub(&self, v1: u128, v2: u128) -> u128 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn u128_mul(&self, v1: u128, v2: u128) -> u128 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn u128_div(&self, v1: u128, v2: u128) -> u128 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn u256_add(&self, v1: u256, v2: u256) -> u256 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn u256_sub(&self, v1: u256, v2: u256) -> u256 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn u256_mul(&self, v1: u256, v2: u256) -> u256 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn u256_div(&self, v1: u256, v2: u256) -> u256 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn i8_add(&self, v1: i8, v2: i8) -> i8 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn i8_sub(&self, v1: i8, v2: i8) -> i8 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn i8_mul(&self, v1: i8, v2: i8) -> i8 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn i8_div(&self, v1: i8, v2: i8) -> i8 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn i16_add(&self, v1: i16, v2: i16) -> i16 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn i16_sub(&self, v1: i16, v2: i16) -> i16 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn i16_mul(&self, v1: i16, v2: i16) -> i16 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn i16_div(&self, v1: i16, v2: i16) -> i16 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn i32_add(&self, v1: i32, v2: i32) -> i32 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn i32_sub(&self, v1: i32, v2: i32) -> i32 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn i32_mul(&self, v1: i32, v2: i32) -> i32 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn i32_div(&self, v1: i32, v2: i32) -> i32 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn i64_add(&self, v1: i64, v2: i64) -> i64 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn i64_sub(&self, v1: i64, v2: i64) -> i64 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn i64_mul(&self, v1: i64, v2: i64) -> i64 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn i64_div(&self, v1: i64, v2: i64) -> i64 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn i128_add(&self, v1: i128, v2: i128) -> i128 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn i128_sub(&self, v1: i128, v2: i128) -> i128 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn i128_mul(&self, v1: i128, v2: i128) -> i128 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn i128_div(&self, v1: i128, v2: i128) -> i128 {
        //     let rst = v1 / v2;
        //     rst
        // }

        // pub fn i256_add(&self, v1: i256, v2: i256) -> i256 {
        //     let rst = v1 + v2;
        //     rst
        // }

        // pub fn i256_sub(&self, v1: i256, v2: i256) -> i256 {
        //     let rst = v1 - v2;
        //     rst
        // }

        // pub fn i256_mul(&self, v1: i256, v2: i256) -> i256 {
        //     let rst = v1 * v2;
        //     rst
        // }

        // pub fn i256_div(&self, v1: i256, v2: i256) -> i256 {
        //     let rst = v1 / v2;
        //     rst
        // }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // #[test]
        // fn test_get_str() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_str(), "Alice");
        // }

        // #[test]
        // fn test_set_str() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_str(String::from("Bob").clone());
        //     assert_eq!(contract.get_str(), "Bob");
        // }

        // #[test]
        // fn test_get_u8() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_u8(), 8);
        // }

        // #[test]
        // fn test_set_u8() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_u8(255);
        //     assert_eq!(contract.get_u8(), 255);
        // }

        // #[test]
        // fn test_get_u16() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_u16(), 16);
        // }

        // #[test]
        // fn test_set_u16() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_u16(65535);
        //     assert_eq!(contract.get_u16(), 65535);
        // }

        // #[test]
        // fn test_get_u32() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_u32(), 32);
        // }

        // #[test]
        // fn test_set_u32() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_u32(4294967295);
        //     assert_eq!(contract.get_u32(), 4294967295);
        // }

        // #[test]
        // fn test_get_u64() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_u64(), 64);
        // }

        // #[test]
        // fn test_set_u64() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_u64(18446744073709551615);
        //     assert_eq!(contract.get_u64(), 18446744073709551615);
        // }

        // #[test]
        // fn test_get_u128() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_u128(), 128);
        // }

        // #[test]
        // fn test_set_u128() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_u128(18446744073709551615);
        //     assert_eq!(contract.get_u128(), 18446744073709551615);
        // }

        // #[test]
        // fn test_get_u256() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_u256(), 256.into());
        // }

        // #[test]
        // fn test_set_u256() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_u256(18446744073709551615u64.into());
        //     assert_eq!(contract.get_u256(), 18446744073709551615u64.into());
        // }

        // #[test]
        // fn test_get_i8() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_i8(), 8);
        // }

        // #[test]
        // fn test_set_i8() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_i8(127);
        //     assert_eq!(contract.get_i8(), 127);
        // }

        // #[test]
        // fn test_get_i16() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_i16(), 16);
        // }

        // #[test]
        // fn test_set_i16() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_i16(-32768);
        //     assert_eq!(contract.get_i16(), -32768);
        // }

        // #[test]
        // fn test_get_i32() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_i32(), 32);
        // }

        // #[test]
        // fn test_set_i32() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_i32(-2147483648);
        //     assert_eq!(contract.get_i32(), -2147483648);
        // }

        // #[test]
        // fn test_get_i64() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_i64(), 64);
        // }

        // #[test]
        // fn test_set_i64() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_i64(-9223372036854775808);
        //     assert_eq!(contract.get_i64(), -9223372036854775808);
        // }

        // #[test]
        // fn test_get_i128() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_i128(), 128);
        // }

        // #[test]
        // fn test_set_i128() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_i128(-9223372036854775808);
        //     assert_eq!(contract.get_i128(), -9223372036854775808);
        // }

        // #[test]
        // fn test_get_i256() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_i256(), 256.into());
        // }

        // #[test]
        // fn test_set_i256() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_i256((-9223372036854775808i64).into());
        //     assert_eq!(contract.get_i256(), (-9223372036854775808i64).into());
        // }

        // #[test]
        // fn test_get_bool() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_bool(), false);
        // }

        // #[test]
        // fn test_set_bool() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_bool(true);
        //     assert_eq!(contract.get_bool(), true);
        // }

        // #[test]
        // fn test_get_addr() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_addr(), String::from("0x").into());
        // }

        // #[test]
        // fn test_set_addr() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_addr(String::from("0x999").into());
        //     assert_eq!(contract.get_addr(), String::from("0x999").into());
        // }

        // #[test]
        // fn test_get_bytes() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_bytes().len(), 0);
        // }

        // #[test]
        // fn test_set_bytes() {
        //     let mut contract = HelloWorld::new();
        //     let mut b = bytes::new();
        //     b.push(1);
        //     contract.set_bytes(b);
        //     assert_eq!(contract.get_bytes().len(), 1);
        // }

        // #[test]
        // fn test_get_bytes1() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_bytes1(), "a".parse().unwrap());
        // }

        // #[test]
        // fn test_set_bytes1() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_bytes1("b".parse().unwrap());
        //     assert_eq!(contract.get_bytes1(), "b".parse().unwrap());
        // }

        // #[test]
        // fn test_get_bytes16() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_bytes16(), "b".parse().unwrap());
        // }

        // #[test]
        // fn test_set_bytes16() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_bytes16("cc".parse().unwrap());
        //     assert_eq!(contract.get_bytes16(), "cc".parse().unwrap());
        // }

        // #[test]
        // fn test_get_bytes32() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_bytes32(), "abc".parse().unwrap());
        // }

        // #[test]
        // fn test_set_bytes32() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_bytes32("aaaaa".parse().unwrap());
        //     assert_eq!(contract.get_bytes32(), "aaaaa".parse().unwrap());
        // }

        // #[test]
        // fn test_get_user() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_user(), User{
        //         name: String::from("Mark"),
        //         age: 100,
        //         others: (String::from("haha"), 123, -123, "0x".into()),
        //     });
        // }

        // #[test]
        // fn test_set_user() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_user(User{
        //         name: String::from("Han"),
        //         age: 100,
        //         others: (String::from("9961"), 119, -119, "0x11".into()),
        //     });
        //     assert_eq!(contract.get_user(), User{
        //         name: String::from("Han"),
        //         age: 100,
        //         others: (String::from("9961"), 119, -119, "0x11".into()),
        //     });
        // }

        // #[test]
        // fn test_get_tuple() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_tuple(), (1u32, -1i32, String::from("aaaa")));
        // }

        // #[test]
        // fn test_set_tuple() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_tuple((3u32, -3i32, String::from("bbbbb")));
        //     assert_eq!(contract.get_tuple(), (3u32, -3i32, String::from("bbbbb")));
        // }

        // #[test]
        // fn test_get_array() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_array(), [1, 2, -1, -5, 100]);
        // }

        // #[test]
        // fn test_set_array() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_array([1, 22, -11, -999, 100]);
        //     assert_eq!(contract.get_array(), [1, 22, -11, -999, 100]);
        // }

        // #[test]
        // fn test_get_vec() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_vec().len(), 0);
        // }

        // #[test]
        // fn test_set_vec() {
        //     let mut contract = HelloWorld::new();
        //     let mut vec = Vec::new();
        //     vec.push([(1u8, "0x1".into()), (2u8, "0x2".into())]);
        //     contract.set_vec(vec);
        //     assert_eq!(contract.get_vec().len(), 1);
        // }

        // #[test]
        // fn test_get_enum() {
        //     let contract = HelloWorld::new();
        //     assert_eq!(contract.get_enum(), Message::Msg(String::from("message")));
        // }

        // #[test]
        // fn test_set_enum() {
        //     let mut contract = HelloWorld::new();

        //     contract.set_enum(Message::Msg(String::from("message111")));
        //     assert_eq!(contract.get_enum(), Message::Msg(String::from("message111")));
        // }

        // #[test]
        // fn test_s_vec() {
        //     let mut contract = HelloWorld::new();

        //     contract.vec_push((String::from("apple"), 1u64));
        //     contract.vec_push((String::from("orange"), 10u64));
        //     let t = contract.vec_get(0u32);
        //     assert_eq!((String::from("apple"), 1u64), t);
        //     assert_eq!(contract.vec_len(), 2u32);
        // }

        // #[test]
        // fn test_s_mapping() {
        //     let mut contract = HelloWorld::new();

        //     contract.mapping_insert("mark".to_string(), 1u64);
        //     contract.mapping_insert("meimei".to_string(), 99u64);
        //     let t = contract.mapping_get("mark".to_string());
        //     assert_eq!(1u64, t);
        //     assert_eq!(contract.v_s_mapping.len(), 2u32);
        // }

        // #[test]
        // fn test_u8_add() {
        //     let contract = HelloWorld::new();

        //     let t = contract.u8_add(12u8, 125u8);
        //     assert_eq!(137u8, t);
        // }

        // #[test]
        // fn test_u8_sub() {
        //     let contract = HelloWorld::new();

        //     let t = contract.u8_sub(122u8, 22u8);
        //     assert_eq!(100u8, t);
        // }

        // #[test]
        // fn test_u8_mul() {
        //     let contract = HelloWorld::new();

        //     let t = contract.u8_mul(12u8, 12u8);
        //     assert_eq!(144u8, t);
        // }

        // #[test]
        // fn test_u8_dev() {
        //     let contract = HelloWorld::new();

        //     let t = contract.u8_div(100u8, 3u8);
        //     assert_eq!(33u8, t);
        // }
    }
}

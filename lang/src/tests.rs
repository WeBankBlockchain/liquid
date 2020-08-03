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

use crate as liquid_lang;
use crate::{
    ty_mapping::{SolTypeName, SolTypeNameLen},
    InOut,
};
use hex_literal::hex;
use liquid_abi_codec::{Decode, Encode, IsDynamic};
use pretty_assertions::assert_eq;

macro_rules! test_encode_decode {
    ($t:ty, $var:tt, $e:expr) => {
        assert_eq!(<$t as Decode>::decode(&mut &hex!($e)[..]).unwrap(), $var);
        assert_eq!(<$t as Encode>::encode(&$var), hex!($e).to_vec());
    };
}

#[derive(InOut, PartialEq, Debug, Clone)]
struct T0 {
    a: u128,
    b: bool,
}

#[test]
#[allow(non_snake_case)]
fn test_T0() {
    assert_eq!(<T0 as IsDynamic>::is_dynamic(), false);
    assert_eq!(<T0 as SolTypeName>::NAME, b"(uint128,bool)");
    assert_eq!(<T0 as SolTypeNameLen>::LEN, 14);

    let t0 = T0 { a: 0, b: true };
    test_encode_decode!(T0, t0, "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001");

    let t0 = T0 { a: 42, b: false };
    test_encode_decode!(T0, t0, "000000000000000000000000000000000000000000000000000000000000002a0000000000000000000000000000000000000000000000000000000000000000");
}

#[test]
#[allow(non_snake_case)]
#[should_panic]
fn test_T0_panic() {
    let t0 = T0 { a: 0, b: true };
    test_encode_decode!(T0, t0, "00000001a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001");
}

#[derive(InOut, PartialEq, Debug, Clone)]
struct T1 {
    a: u128,
    b: String,
    c: bool,
}

#[test]
#[allow(non_snake_case)]
fn test_T1() {
    assert_eq!(<T1 as IsDynamic>::is_dynamic(), true);
    assert_eq!(<T1 as SolTypeName<_>>::NAME, b"(uint128,string,bool)");
    assert_eq!(<T1 as SolTypeNameLen<_>>::LEN, 21);

    let t1 = T1 {
        a: 42,
        b: "Hello,World".to_owned(),
        c: false,
    };
    test_encode_decode!(T1, t1, "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b48656c6c6f2c576f726c64000000000000000000000000000000000000000000");
}

#[derive(InOut, PartialEq, Debug, Clone)]
struct T2 {
    a: T0,
    b: T1,
}

#[test]
#[allow(non_snake_case)]
fn test_T2() {
    assert_eq!(<T2 as IsDynamic>::is_dynamic(), true);
    assert_eq!(
        <T2 as SolTypeName<_>>::NAME,
        "((uint128,bool),(uint128,string,bool))"
            .to_string()
            .as_bytes()
    );
    assert_eq!(<T2 as SolTypeNameLen<_>>::LEN, 38);

    let t2 = T2 {
        a: T0 { a: 0, b: true },
        b: T1 {
            a: 42,
            b: "Hello,World".to_owned(),
            c: false,
        },
    };
    test_encode_decode!(T2, t2, "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b48656c6c6f2c576f726c64000000000000000000000000000000000000000000");
}

use liquid_core::env::types::Vec;

#[test]
fn test_dynamic_array() {
    type Array = Vec<T2>;
    assert_eq!(<Array as IsDynamic>::is_dynamic(), true);
    assert_eq!(
        <Array as SolTypeName<_>>::NAME,
        "((uint128,bool),(uint128,string,bool))[]"
            .to_string()
            .as_bytes()
    );
    assert_eq!(<Array as SolTypeNameLen<_>>::LEN, 40);

    let array = [T2 {
        a: T0 { a: 0, b: true },
        b: T1 {
            a: 42,
            b: "Hello,World".to_owned(),
            c: false,
        },
    }]
    .to_vec();
    test_encode_decode!(Array, array, "000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b48656c6c6f2c576f726c64000000000000000000000000000000000000000000");
}

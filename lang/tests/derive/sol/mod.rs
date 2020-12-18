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

use serial_test::serial;

#[test]
#[serial]
fn compile_tests() {
    let t = trybuild::TestCases::new();

    t.pass("tests/derive/sol/ui/pass/01-state.rs");
    t.pass("tests/derive/sol/ui/pass/02-nested.rs");
    t.compile_fail("tests/derive/sol/ui/fail/01-empty-struct.rs");
    t.compile_fail("tests/derive/sol/ui/fail/02-enum.rs");
    t.compile_fail("tests/derive/sol/ui/fail/03-not-public.rs");
    t.compile_fail("tests/derive/sol/ui/fail/04-generic.rs");
    t.compile_fail("tests/derive/sol/ui/fail/05-invalid-field-type.rs");
}

#[cfg(all(test, feature = "solidity-compatible"))]
mod codec_tests {
    use hex_literal::hex;
    use liquid_abi_codec::{Decode, Encode, TypeInfo};
    use liquid_lang::InOut;
    use liquid_ty_mapping::MappingToSolidityType;
    use pretty_assertions::assert_eq;

    macro_rules! test_encode_decode {
        ($t:ty, $var:tt, $e:expr) => {
            assert_eq!(<$t as Decode>::decode(&mut &hex!($e)[..]).unwrap(), $var);
            assert_eq!(<$t as Encode>::encode(&$var), hex!($e).to_vec());
        };
    }

    fn map_to_solidity_type<T: MappingToSolidityType>() -> &'static str {
        std::str::from_utf8(&<T as MappingToSolidityType>::MAPPED_TYPE_NAME)
            .unwrap()
            .trim_end_matches(char::from(0))
    }

    #[derive(InOut, PartialEq, Debug, Clone)]
    pub struct T0 {
        a: u128,
        b: bool,
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_T0() {
        assert_eq!(<T0 as TypeInfo>::is_dynamic(), false);
        assert_eq!(<T0 as TypeInfo>::size_hint(), 64);
        assert_eq!(map_to_solidity_type::<T0>(), "(uint128,bool)");

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
    pub struct T1 {
        a: u128,
        b: String,
        c: bool,
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_T1() {
        assert_eq!(<T1 as TypeInfo>::is_dynamic(), true);
        assert_eq!(map_to_solidity_type::<T1>(), "(uint128,string,bool)");

        let t1 = T1 {
            a: 42,
            b: "Hello,World".to_owned(),
            c: false,
        };
        test_encode_decode!(T1, t1, "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b48656c6c6f2c576f726c64000000000000000000000000000000000000000000");
    }

    #[test]
    #[allow(non_snake_case)]
    #[should_panic]
    fn test_T1_size_hint() {
        let _ = <T1 as TypeInfo>::size_hint();
    }

    #[derive(InOut, PartialEq, Debug, Clone)]
    pub struct T2 {
        a: T0,
        b: T1,
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_T2() {
        assert_eq!(<T2 as TypeInfo>::is_dynamic(), true);
        assert_eq!(
            map_to_solidity_type::<T2>(),
            "((uint128,bool),(uint128,string,bool))"
        );

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

    #[test]
    #[allow(non_snake_case)]
    #[should_panic]
    fn test_T2_size_hint() {
        let _ = <T2 as TypeInfo>::size_hint();
    }

    use liquid_prelude::vec::Vec;

    #[test]
    fn test_dynamic_array() {
        type Array = Vec<T2>;
        assert_eq!(<Array as TypeInfo>::is_dynamic(), true);
        assert_eq!(
            map_to_solidity_type::<Array>(),
            "((uint128,bool),(uint128,string,bool))[]"
        );

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

    #[test]
    #[should_panic]
    fn test_dynamic_array_size_hint() {
        type Array = Vec<T0>;
        let _ = <Array as TypeInfo>::size_hint();
    }
}

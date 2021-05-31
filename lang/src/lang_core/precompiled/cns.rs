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

use crate::lang_core::{
    env::call,
    precompiled::{ReturnDataWrapper, CNS_ADDRESS},
};
use liquid_prelude::string::String;
use liquid_primitives::types::{u256, Address};
use scale::{Decode, Encode};

pub struct Cns;

impl Cns {
    pub fn insert(
        name: String,
        version: String,
        addr: String,
        abi: String,
    ) -> Option<u256> {
        let mut input_data = if cfg!(feature = "gm") {
            [0xb8, 0xea, 0xa0, 0x8d]
        } else {
            [0xa2, 0x16, 0x46, 0x4b]
        }
        .to_vec();

        input_data.extend(&(name, version, addr, abi).encode());
        let ret = call::<ReturnDataWrapper>(&CNS_ADDRESS, &input_data).ok()?;
        <u256 as Decode>::decode(&mut ret.data.as_slice()).ok()
    }

    pub fn get_contract_address(name: String, version: String) -> Option<Address> {
        let mut input_data = if cfg!(feature = "gm") {
            [0xf1, 0xa3, 0x1b, 0xfa]
        } else {
            [0xf8, 0x5f, 0x81, 0x26]
        }
        .to_vec();
        input_data.extend(&(name, version).encode());
        let ret = call::<ReturnDataWrapper>(&CNS_ADDRESS, &input_data).ok()?;
        <Address as Decode>::decode(&mut ret.data.as_slice()).ok()
    }
}

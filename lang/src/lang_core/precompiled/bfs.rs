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
    precompiled::{ReturnDataWrapper, BFS_ADDRESS},
};
use liquid_prelude::string::String;
use liquid_primitives::types::Address;
use scale::{Decode, Encode};

pub struct Bfs;

impl Bfs {
    pub fn insert(name: String, version: String, addr: Address, abi: String) -> bool {
        let mut input_data = if cfg!(feature = "gm") {
            [0x48, 0xfd, 0x6f, 0x59]
        } else {
            [0xe1, 0x9c, 0x2f, 0xcf]
        }
        .to_vec();

        input_data.extend(&(name, version, addr, abi).encode());
        let ret = call::<ReturnDataWrapper>(&BFS_ADDRESS, &input_data);
        match ret {
            Ok(ret) => ret.data.len() == 1 && ret.data[0] == 0,
            _ => false,
        }
    }

    pub fn get_contract_address(name: String, version: String) -> Option<Address> {
        let name_version = String::from("/apps/") + &name + &String::from("/") + &version;
        let mut input_data = if cfg!(feature = "gm") {
            [0xe1, 0xb8, 0x25, 0xad]
        } else {
            [0x1d, 0x05, 0xa8, 0x36]
        }
        .to_vec();
        input_data.extend(&(name_version).encode());
        let ret = call::<ReturnDataWrapper>(&BFS_ADDRESS, &input_data).ok()?;
        <Address as Decode>::decode(&mut ret.data.as_slice()).ok()
    }
}

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

use crate::lang_core::env::api;
use liquid_primitives::types::{timestamp, Address};

pub struct EnvAccess;

impl EnvAccess {
    pub fn get_caller(self) -> Address {
        api::get_caller()
    }

    pub fn get_tx_origin(self) -> Address {
        api::get_tx_origin()
    }

    pub fn now(self) -> timestamp {
        api::now()
    }

    pub fn get_address(self) -> Address {
        api::get_address()
    }

    pub fn is_contract(self, account: &Address) -> bool {
        match api::get_external_code_size(account) {
            0 => true,
            _ => false,
        }
    }
}

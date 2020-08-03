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

use crate::env::{types::Address, CallData, Result};

pub trait Env {
    fn set_storage<V>(&mut self, key: &[u8], value: &V)
    where
        V: scale::Encode;

    fn get_storage<R>(&mut self, key: &[u8]) -> Result<R>
    where
        R: scale::Decode;

    fn remove_storage(&mut self, key: &[u8]);

    fn get_call_data(&mut self) -> Result<CallData>;

    fn finish<V>(&mut self, return_value: &V)
    where
        V: liquid_abi_codec::Encode;

    fn revert<V>(&mut self, revert_into: &V)
    where
        V: liquid_abi_codec::Encode;

    fn get_caller(&mut self) -> Address;
}

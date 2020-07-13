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

mod db;

use self::db::ContractStorage;
use crate::{engine::OnInstance, CallData, Env, Result};
use core::cell::RefCell;
use liquid_primitives::Key;

#[allow(dead_code)]
pub struct EnvInstance {
    contract_storage: ContractStorage,
}

impl EnvInstance {
    pub fn new() -> Self {
        Self {
            contract_storage: ContractStorage::new(),
        }
    }
}

impl Env for EnvInstance {
    fn set_storage<V>(&mut self, key: Key, value: &V)
    where
        V: scale::Encode,
    {
        self.contract_storage.set_storage(key, value);
    }

    fn get_storage<R>(&mut self, key: Key) -> Result<R>
    where
        R: scale::Decode,
    {
        self.contract_storage.get_storage::<R>(key)
    }

    fn get_call_data(&mut self) -> Result<CallData> {
        unimplemented!();
    }

    fn finish<V>(&mut self, _: &V)
    where
        V: liquid_abi_coder::Encode,
    {
        unimplemented!();
    }
}

impl OnInstance for EnvInstance {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        thread_local!(
            static INSTANCE: RefCell<EnvInstance> = RefCell::new(
                EnvInstance::new()
            )
        );

        INSTANCE.with(|instance| f(&mut instance.borrow_mut()))
    }
}

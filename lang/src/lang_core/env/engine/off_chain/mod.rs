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
pub mod test_api;

use self::db::{Block, ContractStorage, Event, ExecContext};
use crate::lang_core::env::{
    backend::Env, calldata::CallData, engine::OnInstance, error::Result, CallMode,
};
use cfg_if::cfg_if;
use core::cell::RefCell;
use liquid_primitives::{types::Address, Topics};

pub struct EnvInstance {
    contract_storage: ContractStorage,
    blocks: Vec<Block>,
    exec_contexts: Vec<ExecContext>,
    events: Vec<Event>,
}

impl Default for EnvInstance {
    fn default() -> Self {
        let mut blocks = Vec::new();
        blocks.push(Block::new(0));

        Self {
            contract_storage: ContractStorage::new(),
            blocks,
            exec_contexts: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl EnvInstance {
    pub fn current_exec_context(&self) -> &ExecContext {
        self.exec_contexts
            .last()
            .expect("there must be at least one execution context in test environment")
    }

    pub fn last_exec_context(&self) -> &ExecContext {
        self.exec_contexts
            .first()
            .expect("there must be at least one execution context in test environment")
    }

    pub fn current_block(&self) -> &Block {
        self.blocks
            .last()
            .expect("there must be at least one block in test environment")
    }

    pub fn get_events(&self) -> std::slice::Iter<Event> {
        self.events.iter()
    }
}

impl Env for EnvInstance {
    fn set_storage<V>(&mut self, key: &[u8], value: &V)
    where
        V: scale::Encode,
    {
        self.contract_storage.set_storage(key, value);
    }

    fn get_storage<R>(&mut self, key: &[u8]) -> Result<R>
    where
        R: scale::Decode,
    {
        self.contract_storage.get_storage::<R>(key)
    }

    fn remove_storage(&mut self, key: &[u8]) {
        self.contract_storage.remove_storage(key);
    }

    fn get_call_data(&mut self, _: CallMode) -> Result<CallData> {
        unimplemented!();
    }

    fn get_caller(&mut self) -> Address {
        self.current_exec_context().caller()
    }

    fn get_tx_origin(&mut self) -> Address {
        self.last_exec_context().caller()
    }

    fn now(&mut self) -> u64 {
        self.current_block().timestamp()
    }

    fn get_block_number(&mut self) -> u64 {
        self.current_block().block_number()
    }

    fn get_address(&mut self) -> Address {
        unimplemented!()
    }

    cfg_if! {
        if #[cfg(feature = "solidity-compatible")] {
            fn emit<E>(&mut self, event: E)
            where
                E: Topics + liquid_abi_codec::Encode,
            {
                self.events.push(Event::new(event));
            }

            fn call<R>(&mut self, _addr: &Address, _data: &[u8]) -> Result<R>
            where
                R: liquid_abi_codec::Decode + liquid_abi_codec::TypeInfo,
            {
                unimplemented!();
            }

            fn finish<V>(&mut self, _: &V)
            where
                V: liquid_abi_codec::Encode,
            {
                unimplemented!();
            }

            fn revert<V>(&mut self, msg: &V)
            where
                V: liquid_abi_codec::Encode,
            {
                // Ensure that the type of `V` can only be String.
                panic!(<String as liquid_abi_codec::Decode>::decode(
                    &mut msg.encode().as_slice()
                )
                .unwrap());
            }
        } else {
            fn emit<E>(&mut self, event: E)
            where
                E: Topics + scale::Encode,
            {
                self.events.push(Event::new(event));
            }

            fn call<R>(&mut self, _addr: &Address, _data: &[u8]) -> Result<R>
            where
                R: scale::Decode,
            {
                unimplemented!();
            }

            fn finish<V>(&mut self, _: &V)
            where
                V: scale::Encode,
            {
                unimplemented!();
            }

            fn revert<V>(&mut self, msg: &V)
            where
                V: scale::Encode,
            {
                // Ensure that the type of `V` can only be String.
                panic!(<String as liquid_abi_codec::Decode>::decode(
                    &mut msg.encode().as_slice()
                )
                .unwrap());
            }
        }
    }
}

impl OnInstance for EnvInstance {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        thread_local!(
            static INSTANCE: RefCell<EnvInstance> = RefCell::new(
                Default::default(),
            )
        );

        INSTANCE.with(|instance| f(&mut instance.borrow_mut()))
    }
}

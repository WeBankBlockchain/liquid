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
use liquid_primitives::{types::address::Address, Topics};
use std::{collections::HashMap, str};

struct AssetInfo {
    issuer: Address,
    fungible: bool,
    total_supply: u64,
    _description: String,
    supplied: u64,
}

pub struct EnvInstance {
    contract_storage: ContractStorage,
    blocks: Vec<Block>,
    exec_contexts: Vec<ExecContext>,
    events: Vec<Event>,
    assets_info: HashMap<String, AssetInfo>,
    fungible_asset: HashMap<String, HashMap<Address, u64>>,
    not_fungible_asset: HashMap<String, HashMap<Address, HashMap<u64, String>>>,
}

impl Default for EnvInstance {
    fn default() -> Self {
        let blocks = vec![Block::new(0)];
        Self {
            contract_storage: ContractStorage::new(),
            blocks,
            exec_contexts: Vec::new(),
            events: Vec::new(),
            assets_info: HashMap::new(),
            fungible_asset: HashMap::new(),
            not_fungible_asset: HashMap::new(),
        }
    }
}

impl EnvInstance {
    pub fn current_exec_context(&self) -> &ExecContext {
        self.exec_contexts
            .last()
            .expect("there must be at least one execution context in test environment")
    }

    pub fn first_exec_context(&self) -> &ExecContext {
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
        self.first_exec_context().caller()
    }

    fn get_address(&mut self) -> Address {
        self.current_exec_context().self_address()
    }

    fn get_external_code_size(&self, _account: &Address) -> u32 {
        unimplemented!();
    }

    fn now(&mut self) -> u64 {
        self.current_block().timestamp()
    }

    fn get_block_number(&mut self) -> u64 {
        self.current_block().block_number()
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
                panic!("{}", <String as liquid_abi_codec::Decode>::decode(
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
                panic!("{}", <String as scale::Decode>::decode(
                    &mut msg.encode().as_slice()
                )
                .unwrap());
            }
        }
    }

    fn register_asset(
        &mut self,
        asset_name: &[u8],
        issuer: &Address,
        fungible: bool,
        total: u64,
        description: &[u8],
    ) -> bool {
        let asset_name = str::from_utf8(asset_name).unwrap();
        if self.assets_info.contains_key(asset_name) {
            return false;
        }
        self.assets_info.insert(
            asset_name.to_string(),
            AssetInfo {
                issuer: *issuer,
                fungible,
                total_supply: total,
                _description: str::from_utf8(description).unwrap().to_string(),
                supplied: 0,
            },
        );
        if fungible {
            self.fungible_asset
                .insert(asset_name.to_string(), HashMap::new());
        } else {
            self.not_fungible_asset
                .insert(asset_name.to_string(), HashMap::new());
        }
        true
    }

    fn issue_fungible_asset(
        &mut self,
        to: &Address,
        asset_name: &[u8],
        amount: u64,
    ) -> bool {
        let asset_name = str::from_utf8(asset_name).unwrap();
        if !self.assets_info.contains_key(asset_name) {
            return false;
        }
        let caller = self.get_caller();
        let mut asset_info = self.assets_info.get_mut(asset_name).unwrap();
        if !asset_info.fungible {
            return false;
        }
        if asset_info.issuer != caller {
            return false;
        }
        if asset_info.total_supply - asset_info.supplied < amount {
            return false;
        }
        asset_info.supplied += amount;
        let account_balance = self
            .fungible_asset
            .entry(asset_name.to_string())
            .or_insert_with(HashMap::new)
            .entry(*to)
            .or_insert(0);
        *account_balance += amount;
        true
    }

    fn issue_not_fungible_asset(
        &mut self,
        to: &Address,
        asset_name: &[u8],
        uri: &[u8],
    ) -> u64 {
        let asset_name = str::from_utf8(asset_name).unwrap();
        if !self.assets_info.contains_key(asset_name) {
            return 0;
        }
        let caller = self.get_caller();
        let mut asset_info = self.assets_info.get_mut(asset_name).unwrap();
        if asset_info.fungible {
            return 0;
        }
        if asset_info.issuer != caller {
            return 0;
        }
        if asset_info.total_supply == asset_info.supplied {
            return 0;
        }
        asset_info.supplied += 1;
        let tokens = self
            .not_fungible_asset
            .entry(asset_name.to_string())
            .or_insert_with(HashMap::new)
            .entry(*to)
            .or_insert_with(HashMap::new);
        tokens.insert(
            asset_info.supplied,
            str::from_utf8(uri).unwrap().to_string(),
        );
        asset_info.supplied
    }

    fn transfer_asset(
        &mut self,
        to: &Address,
        asset_name: &[u8],
        amount_or_id: u64,
        from_self: bool,
    ) -> bool {
        let asset_name = str::from_utf8(asset_name).unwrap();
        if !self.assets_info.contains_key(asset_name) {
            return false;
        }
        let from = {
            if from_self {
                self.get_address()
            } else {
                self.get_caller()
            }
        };
        let asset_info = self.assets_info.get(asset_name).unwrap();
        if asset_info.fungible {
            let amount = amount_or_id;
            let from_balance = self
                .fungible_asset
                .get_mut(asset_name)
                .unwrap()
                .entry(from)
                .or_insert(0);
            if *from_balance >= amount {
                *from_balance -= amount;
                let to_balance = self
                    .fungible_asset
                    .get_mut(asset_name)
                    .unwrap()
                    .entry(*to)
                    .or_insert(0);
                *to_balance += amount;
                return true;
            }
            false
        } else {
            let token_id = amount_or_id;
            let from_balance = self
                .not_fungible_asset
                .get_mut(asset_name)
                .unwrap()
                .entry(from)
                .or_insert_with(HashMap::new);
            if !from_balance.contains_key(&token_id) {
                return false;
            }
            let token_uri = from_balance.remove(&token_id).unwrap();
            let to_balance = self
                .not_fungible_asset
                .get_mut(asset_name)
                .unwrap()
                .entry(*to)
                .or_insert_with(HashMap::new);
            to_balance.insert(token_id, token_uri);
            true
        }
    }

    fn get_asset_balance(&self, to: &Address, asset_name: &[u8]) -> u64 {
        let asset_name = str::from_utf8(asset_name).unwrap();
        if !self.assets_info.contains_key(asset_name) {
            return 0;
        }
        let asset_info = self.assets_info.get(asset_name).unwrap();
        if asset_info.fungible {
            *self
                .fungible_asset
                .get(asset_name)
                .unwrap()
                .get(to)
                .unwrap_or(&0)
        } else {
            match self.not_fungible_asset.get(asset_name).unwrap().get(to) {
                None => 0,
                Some(tokens) => tokens.len() as u64,
            }
        }
    }

    fn get_not_fungible_asset_info(
        &mut self,
        account: &Address,
        asset_name: &[u8],
        asset_id: u64,
    ) -> String {
        let asset_name = str::from_utf8(asset_name).unwrap();
        let ret = String::new();
        if !self.assets_info.contains_key(asset_name) {
            return ret;
        }
        let asset_info = self.assets_info.get_mut(asset_name).unwrap();
        if asset_info.fungible {
            return ret;
        }
        match self
            .not_fungible_asset
            .get(asset_name)
            .unwrap()
            .get(account)
        {
            None => String::new(),
            Some(tokens) => tokens.get(&asset_id).unwrap_or(&ret).clone(),
        }
    }

    fn get_not_fungible_asset_ids(
        &mut self,
        account: &Address,
        asset_name: &[u8],
    ) -> Vec<u64> {
        let asset_name = str::from_utf8(asset_name).unwrap();
        let mut ret = Vec::new();
        if !self.assets_info.contains_key(asset_name) {
            return ret;
        }
        let asset_info = self.assets_info.get_mut(asset_name).unwrap();
        if asset_info.fungible {
            return ret;
        }
        let accounts = self.not_fungible_asset.get(asset_name).unwrap();
        match accounts.get(account) {
            None => (),
            Some(tokens) => {
                for key in tokens.keys() {
                    ret.push(*key)
                }
            }
        }
        ret
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

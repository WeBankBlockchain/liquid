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

mod buffer;
pub mod ext;

use self::buffer::StaticBuffer;
use super::OnInstance;
use crate::env::{
    backend::Env,
    calldata::CallData,
    error::{EnvError, Result},
    CallMode,
};
use cfg_if::cfg_if;
use core::convert::TryInto;
use liquid_prelude::{string::String, vec::Vec};
use liquid_primitives::{types::address::*, Topics};

/// The on-chain environment
pub struct EnvInstance {
    buffer: StaticBuffer,
}

impl OnInstance for EnvInstance {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        static mut INSTANCE: EnvInstance = EnvInstance {
            buffer: StaticBuffer::new(),
        };

        f(unsafe { &mut INSTANCE })
    }
}

impl EnvInstance {
    fn reset_buffer(&mut self) {
        self.buffer.clear();
    }

    fn encode_into_buffer_scale<V>(&mut self, value: &V)
    where
        V: scale::Encode,
    {
        self.reset_buffer();
        scale::Encode::encode_to(value, &mut self.buffer);
    }

    #[cfg(feature = "solidity-compatible")]
    fn encode_into_buffer_abi<V>(&mut self, value: &V)
    where
        V: liquid_abi_codec::Encode,
    {
        self.reset_buffer();
        liquid_abi_codec::Encode::encode_to(value, &mut self.buffer);
    }

    fn decode_from_buffer_scale<R>(&mut self) -> Result<R>
    where
        R: scale::Decode,
    {
        let len = self.buffer.len();
        scale::Decode::decode(&mut &self.buffer[..len]).map_err(Into::into)
    }

    #[cfg(feature = "solidity-compatible")]
    fn decode_from_buffer_abi<R>(&mut self) -> Result<R>
    where
        R: liquid_abi_codec::Decode,
    {
        let len = self.buffer.len();
        liquid_abi_codec::Decode::decode(&mut &self.buffer[..len]).map_err(Into::into)
    }
}

impl Env for EnvInstance {
    fn set_storage<V>(&mut self, key: &[u8], value: &V)
    where
        V: scale::Encode,
    {
        self.encode_into_buffer_scale(value);
        ext::set_storage(key, &self.buffer[..]);
    }

    fn get_storage<R>(&mut self, key: &[u8]) -> Result<R>
    where
        R: scale::Decode,
    {
        let size = ext::get_storage(key, &mut self.buffer[..])?;
        self.buffer.resize(size as usize);
        self.decode_from_buffer_scale()
    }

    fn remove_storage(&mut self, key: &[u8]) {
        ext::set_storage(key, &[]);
    }

    fn get_call_data(&mut self, mode: CallMode) -> Result<CallData> {
        let call_data_size = ext::get_call_data_size();
        if mode == CallMode::Call {
            // The call data of external methods must have a selector.
            if call_data_size < 4 {
                return Err(EnvError::UnableToReadCallData);
            }
        }

        let mut call_data_buf =
            liquid_prelude::vec::from_elem(0u8, call_data_size as usize);
        ext::get_call_data(call_data_buf.as_mut_slice());

        if mode == CallMode::Call {
            #[cfg(feature = "solidity-compatible")]
            use liquid_abi_codec::Decode;
            #[cfg(not(feature = "solidity-compatible"))]
            use scale::Decode;

            CallData::decode(&mut call_data_buf.as_slice()).map_err(Into::into)
        } else {
            Ok(CallData {
                selector: [0x00; 4],
                data: call_data_buf,
            })
        }
    }

    cfg_if! {
        if #[cfg(feature = "solidity-compatible")] {
            fn emit<Event>(&mut self, event: Event)
            where
                Event: Topics + liquid_abi_codec::Encode,
            {
                self.encode_into_buffer_abi(&event);
                let topics = event.topics();
                ext::log(&self.buffer[..self.buffer.len()], &topics);
            }

            fn call<R>(&mut self, addr: &Address, data: &[u8]) -> Result<R>
            where
                R: liquid_abi_codec::Decode + liquid_abi_codec::TypeInfo,
            {
                let status = ext::call(&addr.0, data);
                if status != 0 {
                    return Err(EnvError::FailToCallForeignContract);
                }
                if core::mem::size_of::<R>() == 0 {
                    // The `R` is unit type.
                    self.buffer.clear();
                    self.decode_from_buffer_abi()
                } else {
                    let return_data_size = if <R as liquid_abi_codec::TypeInfo>::is_dynamic() {
                        ext::get_return_data_size()
                    } else {
                        <R as liquid_abi_codec::TypeInfo>::size_hint()
                    };
                    if return_data_size <= StaticBuffer::CAPACITY as u32 {
                        if return_data_size != 0 {
                            ext::get_return_data(&mut self.buffer[..]);
                        }
                        self.buffer.resize(return_data_size as usize);
                        self.decode_from_buffer_abi()
                    } else {
                        let mut return_data_buffer =
                            liquid_prelude::vec::from_elem(0u8, return_data_size as usize);
                        ext::get_return_data(&mut return_data_buffer);
                        liquid_abi_codec::Decode::decode(&mut return_data_buffer.as_slice())
                            .map_err(Into::into)
                    }
                }
            }

            fn finish<V>(&mut self, return_value: &V)
            where
                V: liquid_abi_codec::Encode,
            {
                let encoded = return_value.encode();
                ext::finish(&encoded);
            }

            fn revert<V>(&mut self, revert_info: &V)
            where
                V: liquid_abi_codec::Encode,
            {
                let encoded = revert_info.encode();
                ext::revert(&encoded);
            }
        } else {
            fn emit<Event>(&mut self, event: Event)
            where
                Event: Topics + scale::Encode,
            {
                self.encode_into_buffer_scale(&event);
                let topics = event.topics();
                ext::log(&self.buffer[..self.buffer.len()], &topics);
            }

            fn call<R>(&mut self, addr: &Address, data: &[u8]) -> Result<R>
            where
                R: scale::Decode,
            {
                let status = ext::call(&addr.0, data);
                if status != 0 {
                    return Err(EnvError::FailToCallForeignContract);
                }
                if core::mem::size_of::<R>() == 0 {
                    // The `R` is unit type.
                    self.buffer.clear();
                    self.decode_from_buffer_scale()
                } else {
                    // TODO: Optimize the performance of getting return data size
                    let return_data_size = ext::get_return_data_size();
                    if return_data_size <= StaticBuffer::CAPACITY as u32 {
                        if return_data_size != 0 {
                            ext::get_return_data(&mut self.buffer[..]);
                        }
                        self.buffer.resize(return_data_size as usize);
                        self.decode_from_buffer_scale()
                    } else {
                        let mut return_data_buffer =
                            liquid_prelude::vec::from_elem(0u8, return_data_size as usize);
                        ext::get_return_data(&mut return_data_buffer);
                        scale::Decode::decode(&mut return_data_buffer.as_slice())
                            .map_err(Into::into)
                    }
                }
            }

            fn finish<V>(&mut self, return_value: &V)
            where
                V: scale::Encode,
            {
                let encoded = return_value.encode();
                ext::finish(&encoded);
            }

            fn revert<V>(&mut self, revert_info: &V)
            where
                V: scale::Encode,
            {
                let encoded = revert_info.encode();
                ext::revert(&encoded);
            }
        }
    }

    fn get_caller(&mut self) -> Address {
        self.buffer.resize(ADDRESS_LENGTH);
        ext::get_caller(&mut self.buffer[..ADDRESS_LENGTH]);
        let mut addr = [0u8; ADDRESS_LENGTH];
        addr.copy_from_slice(&self.buffer[..ADDRESS_LENGTH]);
        Address::new(addr)
    }

    fn get_tx_origin(&mut self) -> Address {
        self.buffer.resize(ADDRESS_LENGTH);
        ext::get_tx_origin(&mut self.buffer[..ADDRESS_LENGTH]);
        let mut addr = [0u8; ADDRESS_LENGTH];
        addr.copy_from_slice(&self.buffer[..ADDRESS_LENGTH]);
        Address::new(addr)
    }

    fn get_address(&mut self) -> Address {
        self.buffer.resize(ADDRESS_LENGTH);
        ext::get_caller(&mut self.buffer[..ADDRESS_LENGTH]);
        let mut addr = [0u8; ADDRESS_LENGTH];
        addr.copy_from_slice(&self.buffer[..ADDRESS_LENGTH]);
        Address::new(addr)
    }

    fn now(&mut self) -> u64 {
        ext::get_block_timestamp() as u64
    }

    fn get_block_number(&mut self) -> u64 {
        ext::get_block_number() as u64
    }
    fn get_external_code_size(&self, account: &Address) -> u32 {
        ext::get_external_code_size(&account.0)
    }
    fn register_asset(
        &mut self,
        asset_name: &[u8],
        issuer: &Address,
        fungible: bool,
        total: u64,
        description: &[u8],
    ) -> bool {
        ext::register_asset(asset_name, &issuer.0, fungible, total, description)
    }

    fn issue_fungible_asset(
        &mut self,
        to: &Address,
        asset_name: &[u8],
        amount: u64,
    ) -> bool {
        ext::issue_fungible_asset(&to.0, asset_name, amount)
    }

    fn issue_not_fungible_asset(
        &mut self,
        to: &Address,
        asset_name: &[u8],
        uri: &[u8],
    ) -> u64 {
        ext::issue_not_fungible_asset(&to.0, asset_name, uri)
    }

    fn transfer_asset(
        &mut self,
        to: &Address,
        asset_name: &[u8],
        amount_or_id: u64,
        from_self: bool,
    ) -> bool {
        ext::transfer_asset(&to.0, asset_name, amount_or_id, from_self)
    }

    fn get_asset_balance(&self, to: &Address, asset_name: &[u8]) -> u64 {
        ext::get_asset_balance(&to.0, asset_name)
    }

    fn get_not_fungible_asset_info(
        &mut self,
        account: &Address,
        asset_name: &[u8],
        asset_id: u64,
    ) -> String {
        let size = ext::get_not_fungible_asset_info(
            &account.0,
            asset_name,
            asset_id,
            &mut self.buffer[..],
        );
        if size <= 0 {
            return String::new();
        }
        self.buffer.resize(size as usize);
        String::from_utf8(self.buffer[..].to_vec()).unwrap()
    }

    fn get_not_fungible_asset_ids(
        &mut self,
        account: &Address,
        asset_name: &[u8],
    ) -> Vec<u64> {
        let mut ret = Vec::new();
        if let Ok(size) =
            ext::get_not_fungible_asset_ids(&account.0, asset_name, &mut self.buffer[..])
        {
            self.buffer.resize(size as usize);
            let mut start: usize = 0;
            while start < size as usize {
                ret.push(u64::from_le_bytes(
                    self.buffer[start..start + core::mem::size_of::<u64>()]
                        .try_into()
                        .unwrap(),
                ));
                ext::print64(*ret.last().unwrap());
                start += core::mem::size_of::<u64>();
            }
        }
        ret
    }
}

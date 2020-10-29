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
use crate::env::{CallData, CallMode, Env, EnvError, Result};
use liquid_abi_codec::Decode;
use liquid_primitives::{types::address_impl::*, Topics};

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
            CallData::decode(&mut call_data_buf.as_slice()).map_err(Into::into)
        } else {
            Ok(CallData {
                selector: [0x00; 4],
                data: call_data_buf,
            })
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

    fn emit<Event>(&mut self, event: Event)
    where
        Event: Topics + liquid_abi_codec::Encode,
    {
        self.encode_into_buffer_abi(&event);
        let topics = event.topics();
        ext::log(&self.buffer[..self.buffer.len()], &topics);
    }

    fn get_caller(&mut self) -> address {
        self.buffer.resize(ADDRESS_LENGTH);
        ext::get_caller(&mut self.buffer[..ADDRESS_LENGTH]);
        let mut addr = [0u8; ADDRESS_LENGTH];
        addr.copy_from_slice(&self.buffer[..ADDRESS_LENGTH]);
        address::new(addr)
    }

    fn now(&mut self) -> u64 {
        ext::get_block_timestamp() as u64
    }

    fn get_block_number(&mut self) -> u64 {
        ext::get_block_number() as u64
    }

    fn call<R>(&mut self, addr: &address, data: &[u8]) -> Result<R>
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
}

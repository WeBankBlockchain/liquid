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

//! External C API to communicate with FISCO BCOS WASM Runtime

use crate::env::{EnvError, Result};

mod sys {
    extern "C" {
        pub fn setStorage(
            key_offset: u32,
            key_length: u32,
            value_offset: u32,
            value_length: u32,
        );

        pub fn getStorage(key_offset: u32, key_length: u32, result_offset: u32) -> u32;

        pub fn getCallDataSize() -> u32;

        pub fn getCallData(result_offset: u32);

        pub fn finish(data_offset: u32, data_length: u32);

        pub fn revert(data_offset: u32, data_length: u32);

        pub fn getCaller(data_offset: u32);

        pub fn getBlockTimestamp() -> u64;

        pub fn getBlockNumber() -> u64;
    }
}

pub fn set_storage(key: &[u8], encoded_value: &[u8]) {
    unsafe {
        sys::setStorage(
            key.as_ptr() as u32,
            key.len() as u32,
            encoded_value.as_ptr() as u32,
            encoded_value.len() as u32,
        )
    }
}

pub fn get_storage(key: &[u8], result_offset: &mut [u8]) -> Result<u32> {
    let size = unsafe {
        sys::getStorage(
            key.as_ptr() as u32,
            key.len() as u32,
            result_offset.as_mut_ptr() as u32,
        )
    };
    match size {
        0 => Err(EnvError::UnableToReadFromStorage),
        _ => Ok(size),
    }
}

pub fn get_call_data_size() -> u32 {
    unsafe { sys::getCallDataSize() }
}

pub fn get_call_data(result_offset: &mut [u8]) {
    unsafe {
        sys::getCallData(result_offset.as_mut_ptr() as u32);
    }
}

pub fn finish(return_value: &[u8]) {
    unsafe {
        sys::finish(return_value.as_ptr() as u32, return_value.len() as u32);
    }
}

pub fn revert(revert_info: &[u8]) {
    unsafe {
        sys::revert(revert_info.as_ptr() as u32, revert_info.len() as u32);
    }
}

pub fn get_caller(result_offset: &mut [u8]) {
    unsafe {
        sys::getCaller(result_offset.as_mut_ptr() as u32);
    }
}

pub fn get_block_timestamp() -> u64 {
    unsafe { sys::getBlockTimestamp() }
}

pub fn get_block_number() -> u64 {
    unsafe { sys::getBlockNumber() }
}

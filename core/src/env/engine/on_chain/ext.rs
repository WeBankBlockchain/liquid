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

//! External C API to communicate with FISCO BCOS Wasm runtime

use crate::env::{EnvError, Result};
use liquid_primitives::types::Hash;

mod sys {
    #[link(wasm_import_module = "bcos")]
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

        pub fn log(
            data_offset: u32,
            data_length: u32,
            number_of_topics: u32,
            topic1: u32,
            topic2: u32,
            topic3: u32,
            topic4: u32,
        );

        pub fn getCaller(data_offset: u32);

        pub fn getBlockTimestamp() -> u64;

        pub fn getBlockNumber() -> u64;

        pub fn call(address_offset: u32, data_offset: u32, data_length: u32) -> u32;

        pub fn getReturnDataSize() -> u32;

        pub fn getReturnData(result_offset: u32);
    }

    #[link(wasm_import_module = "debug")]
    /// For debug using, unnecessary to implement them in environment API.
    extern "C" {
        pub fn print32(i: i32);

        pub fn printMem(data_offset: u32, data_length: u32);
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

pub fn get_storage(key: &[u8], result: &mut [u8]) -> Result<u32> {
    let size = unsafe {
        sys::getStorage(
            key.as_ptr() as u32,
            key.len() as u32,
            result.as_mut_ptr() as u32,
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

pub fn get_call_data(result: &mut [u8]) {
    unsafe {
        sys::getCallData(result.as_mut_ptr() as u32);
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

pub fn log(data: &[u8], topics: &[Hash]) {
    match topics.len() {
        4 => unsafe {
            sys::log(
                data.as_ptr() as u32,
                data.len() as u32,
                4,
                topics[0].as_ptr() as u32,
                topics[1].as_ptr() as u32,
                topics[2].as_ptr() as u32,
                topics[3].as_ptr() as u32,
            );
        },
        3 => unsafe {
            sys::log(
                data.as_ptr() as u32,
                data.len() as u32,
                3,
                topics[0].as_ptr() as u32,
                topics[1].as_ptr() as u32,
                topics[2].as_ptr() as u32,
                0,
            );
        },
        2 => unsafe {
            sys::log(
                data.as_ptr() as u32,
                data.len() as u32,
                2,
                topics[0].as_ptr() as u32,
                topics[1].as_ptr() as u32,
                0,
                0,
            );
        },
        1 => unsafe {
            sys::log(
                data.as_ptr() as u32,
                data.len() as u32,
                1,
                topics[0].as_ptr() as u32,
                0,
                0,
                0,
            );
        },
        0 => unsafe {
            sys::log(data.as_ptr() as u32, data.len() as u32, 0, 0, 0, 0, 0);
        },
        _ => unreachable!(),
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

pub fn call(address: &[u8], data: &[u8]) -> u32 {
    unsafe {
        sys::call(
            address.as_ptr() as u32,
            data.as_ptr() as u32,
            data.len() as u32,
        )
    }
}

pub fn get_return_data_size() -> u32 {
    unsafe { sys::getReturnDataSize() }
}

pub fn get_return_data(result: &mut [u8]) {
    unsafe {
        sys::getReturnData(result.as_ptr() as u32);
    }
}

pub fn print32(i: i32) {
    unsafe {
        sys::print32(i);
    }
}

pub fn print_mem(data_offset: u32, data_length: u32) {
    unsafe {
        sys::printMem(data_offset, data_length);
    }
}

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

mod cache_entry;
mod cell;
mod chunk;
mod collections;
mod traits;
mod value;

pub use self::{
    collections::{IterableMapping, Mapping, Vec},
    traits::*,
    value::Value,
};

use self::{
    cache_entry::CacheEntry,
    cell::{CachedCell, TypedCell},
    chunk::{CachedChunk, TypedChunk},
};

#[cfg(feature = "std")]
mod mutable_call_flag {
    use core::cell::RefCell;

    thread_local! {
        static MUTABLE_CALL_HAPPENS: RefCell<bool> = RefCell::new(false);
    }

    pub fn mutable_call_happens() {
        MUTABLE_CALL_HAPPENS.with(|flag| *flag.borrow_mut() = true);
    }

    pub fn reset_mutable_call_flag() {
        MUTABLE_CALL_HAPPENS.with(|flag| *flag.borrow_mut() = false);
    }

    pub fn has_mutable_call_happens() -> bool {
        MUTABLE_CALL_HAPPENS.with(|flag| *flag.borrow())
    }
}

#[cfg(not(feature = "std"))]
mod mutable_call_flag {
    use lazy_static::lazy_static;
    use spin::Mutex;

    lazy_static! {
        static ref MUTABLE_CALL_HAPPENS: Mutex<bool> = Mutex::new(false);
    }

    pub fn mutable_call_happens() {
        *MUTABLE_CALL_HAPPENS.lock() = true;
    }

    pub fn reset_mutable_call_flag() {
        *MUTABLE_CALL_HAPPENS.lock() = false;
    }

    pub fn has_mutable_call_happens() -> bool {
        *MUTABLE_CALL_HAPPENS.lock()
    }
}

pub use mutable_call_flag::*;

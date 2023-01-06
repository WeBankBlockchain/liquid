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

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(incomplete_features)]
#![feature(associated_type_defaults)]
#![feature(specialization)]

mod dispatch_error;
mod env_access;
pub mod intrinsics;
mod lang_core;
#[cfg(feature = "std")]
pub mod mock;
mod traits;

pub use dispatch_error::{DispatchError, DispatchResult, DispatchRetInfo};
pub use env_access::EnvAccess;
pub use traits::*;

pub mod storage {
    pub use super::lang_core::storage::*;
}

pub mod env {
    pub use super::lang_core::env::*;
}

pub mod precompiled {
    pub use super::lang_core::precompiled::*;
}

pub use liquid_lang_macro::InOut;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "collaboration")] {
        pub trait Fetch {
            type Target;
            fn fetch(&self) -> Self::Target;
        }

        pub use liquid_lang_macro::collaboration;
    } else if #[cfg(feature = "contract")] {
        pub use liquid_lang_macro::{contract, interface};
    }
}

use liquid_prelude::string::String;

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let high = byte >> 4;
        let low = byte & 0x0f;
        s.push((HEX_DIGITS[high as usize]) as char);
        s.push((HEX_DIGITS[low as usize]) as char);
    }
    s
}

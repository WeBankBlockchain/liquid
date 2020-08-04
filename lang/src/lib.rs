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

#![allow(incomplete_features)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(associated_type_defaults)]
#![feature(const_panic)]
#![cfg_attr(not(feature = "std"), no_std)]

mod contract;
mod dispatch_error;
mod dispatcher;
mod env_access;
pub mod intrinsics;
pub mod selector_conflict_detect;
mod traits;
pub mod ty_mapping;

#[cfg(test)]
mod tests;

pub use contract::Contract;
pub use dispatch_error::{DispatchError, DispatchResult, DispatchRetInfo};
pub use env_access::EnvAccess;
pub use liquid_lang_derive::{InOut, State};
pub use liquid_lang_macro::contract;
pub use traits::*;

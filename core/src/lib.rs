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

// This extern crate definition is required since otherwise rustc
// is not recognizing its allocator and panic handler definitions.

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(associated_type_defaults)]

#[cfg(not(feature = "std"))]
extern crate liquid_alloc;

#[cfg(feature = "std")]
extern crate chrono;

pub mod env;
pub mod storage;

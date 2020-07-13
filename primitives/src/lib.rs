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

#![feature(const_fn)]
#![feature(const_mut_refs)]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod hash;

/// Typeless generic key into contract storage
pub type Key = &'static str;
pub type Selector = [u8; 4];

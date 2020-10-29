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
#![feature(const_fn)]
#![feature(const_mut_refs)]
#![feature(min_const_generics)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;

pub mod hash;
pub mod types;

/// Typeless generic key into contract storage
pub type Key = &'static str;
pub type Selector = [u8; 4];

#[cfg(feature = "std")]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Error(&'static str);

#[cfg(not(feature = "std"))]
pub struct Error;

impl From<&'static str> for Error {
    #[cfg(feature = "std")]
    fn from(s: &'static str) -> Error {
        Error(s)
    }

    #[cfg(not(feature = "std"))]
    fn from(_: &'static str) -> Error {
        Error
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

use liquid_prelude::vec::Vec;
pub trait Topics {
    fn topics(&self) -> Vec<types::hash>;
}

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

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;

use cfg_if::cfg_if;
use liquid_prelude::vec::Vec;

pub mod hash;
pub mod types;

/// Typeless generic key into contract storage
pub type Key = &'static str;
pub type Selector = [u8; 4];

#[cfg(feature = "std")]
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Error(&'static str);

#[cfg(not(feature = "std"))]
#[derive(Debug)]
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

pub trait Topics {
    fn topics(&self) -> Vec<types::Hash>;
}

cfg_if! {
    if #[cfg(feature = "contract")] {
        #[allow(non_camel_case_types)]
        pub struct __Liquid_Getter_Index_Placeholder;

        #[cfg(not(feature = "solidity-compatible"))]
        impl scale::Decode for __Liquid_Getter_Index_Placeholder {
            fn decode<I: scale::Input>(_: &mut I) -> Result<Self, scale::Error> {
                Ok(Self {})
            }
        }
    }
}

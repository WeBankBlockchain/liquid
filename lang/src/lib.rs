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
#![feature(const_fn)]
#![feature(associated_type_defaults)]
#![feature(const_panic)]
#![feature(min_const_generics)]
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

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(feature = "contract", feature = "collaboration"))] {
        compile_error! {
            "compilation feature `contract` and `collaboration` can not be \
             enabled simultaneously"
        }
    } else if #[cfg(all(feature = "collaboration", feature = "solidity-compatible"))] {
        compile_error! {
            "compilation feature `collaboration` and `solidity-compatible` can not be \
             enabled simultaneously"
        }
    } else if #[cfg(all(feature = "solidity-compatible", feature = "solidity-interface"))]{
        compile_error! {
            "it's unnecessary to enable `solidity-interface` feature when \
             `solidity-compatible` is enabled"
        }
    } else if #[cfg(feature = "collaboration")] {
        pub struct ContractId<T>
        where
            T: You_Should_Use_An_Valid_Contract_Type
        {
            pub __liquid_index: u32,
            pub __liquid_marker: ::core::marker::PhantomData<fn() -> T>,
        }

        impl<T> Copy for ContractId<T>
        where
            T: You_Should_Use_An_Valid_Contract_Type
        {
        }

        impl<T> Clone for ContractId<T>
        where
        T: You_Should_Use_An_Valid_Contract_Type
        {
            fn clone(&self) -> ContractId<T> {
                *self
            }
        }

        impl<T> scale::Encode for ContractId<T>
        where
            T: You_Should_Use_An_Valid_Contract_Type
        {
            fn encode(&self) -> Vec<u8> {
                <u32 as scale::Encode>::encode(&self.__liquid_index)
            }
        }

        impl<T> scale::Decode for ContractId<T>
        where
            T: You_Should_Use_An_Valid_Contract_Type
        {
            fn decode<I: scale::Input>(value: &mut I) -> Result<Self, scale::Error> {
                let __liquid_index = <u32 as scale::Decode>::decode(value)?;
                Ok(Self {
                    __liquid_index,
                    __liquid_marker: Default::default()
                })
            }
        }

        impl<T> ::core::cmp::PartialEq for ContractId<T>
        where
            T: You_Should_Use_An_Valid_Contract_Type
        {
            fn eq(&self, other: &Self) -> bool {
                self.__liquid_index == other.__liquid_index
            }
        }

        impl<T> You_Should_Use_An_Valid_Field_Type for ContractId<T>
        where
            T: You_Should_Use_An_Valid_Contract_Type
        {
        }

        impl<T> You_Should_Use_An_Valid_Input_Type for ContractId<T>
        where
            T: You_Should_Use_An_Valid_Contract_Type
        {
        }

        impl<T> You_Should_Use_An_Valid_Return_Type for ContractId<T>
        where
            T: You_Should_Use_An_Valid_Contract_Type
        {
        }

        #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Default, scale::Encode, scale::Decode)]
        #[allow(non_camel_case_types)]
        pub struct Contract_Constructing_Is_Forbidden(pub bool);

        impl Clone for Contract_Constructing_Is_Forbidden {
            fn clone(&self) -> Self {
                Self(true)
            }
        }

        impl You_Should_Use_An_Valid_Field_Type for Contract_Constructing_Is_Forbidden {}

        pub use liquid_lang_macro::{collaboration, InOut};
    } else if #[cfg(all(feature = "contract", feature = "solidity-compatible"))] {
        pub use liquid_lang_macro::{contract, interface, InOut, State};
    } else if #[cfg(all(feature = "contract", not(feature = "solidity-compatible")))] {
        pub use liquid_lang_macro::{contract, interface, InOut};
    }
}

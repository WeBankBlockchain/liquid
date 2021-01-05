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

#![allow(unused_imports)]
#![allow(unused_macros)]

extern crate proc_macro;

#[macro_use]
mod error;
mod common;
mod derive;
mod utils;

use cfg_if::cfg_if;
use derive::wrapper;
use proc_macro::TokenStream;

cfg_if! {
    if #[cfg(all(not(feature = "contract"), not(feature = "collaboration")))] {
        compile_error! {
            "one of compilation feature `contract` and `collaboration` must \
             be enabled"
        }
    } else if #[cfg(all(feature = "contract", feature = "collaboration"))] {
        compile_error! {
            "compilation feature `contract` and `collaboration` can not be \
             enabled simultaneously"
        }
    } else if #[cfg(all(feature = "collaboration", feature = "solidity-compatible"))] {
        compile_error! {
            "compilation feature `collaboration` and `solidity-compatible` can not be \
             enabled simultaneously"
        }
    } else if #[cfg(all(feature = "collaboration", feature = "solidity-interface"))] {
        compile_error! {
            "compilation feature `collaboration` and `solidity-interface` can not be \
             enabled simultaneously"
        }
    } else if #[cfg(all(feature = "solidity-compatible", feature = "solidity-interface"))]{
        compile_error! {
            "it's unnecessary to enable `solidity-interface` feature when \
             `solidity-compatible` is enabled"
        }
    } else if #[cfg(all(feature = "contract", not(feature = "solidity-compatible")))] {
        compile_error! {
            "up till now, compilation feature `contract` and `solidity-compatible` must \
             be enabled simultaneously"
        }
    } else if #[cfg(feature = "collaboration")] {
        mod collaboration;
        use derive::codec;

        #[proc_macro_derive(InOut)]
        pub fn inout_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
            wrapper::generate_wrapper(codec::generate(input.into())).into()
        }

        #[proc_macro_attribute]
        pub fn collaboration(attr: TokenStream, item: TokenStream) -> TokenStream {
            collaboration::generate(attr.into(), item.into()).into()
        }
    } else if #[cfg(feature = "contract")] {
        mod contract;
        use contract::GenerateMode;

        cfg_if! {
            if #[cfg(feature = "solidity-compatible")] {
                use derive::{in_out, state};

                #[proc_macro_derive(InOut)]
                pub fn inout_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
                    wrapper::generate_wrapper(in_out::generate(input.into())).into()
                }

                #[proc_macro_derive(State)]
                pub fn state_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
                    wrapper::generate_wrapper(state::generate(input.into())).into()
                }
            } else {
                use derive::codec;

                #[proc_macro_derive(InOut)]
                pub fn inout_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
                    wrapper::generate_wrapper(codec::generate(input.into())).into()
                }
            }
        }

        #[proc_macro_attribute]
        pub fn interface(attr: TokenStream, item: TokenStream) -> TokenStream {
            contract::generate(attr.into(), item.into(), GenerateMode::Interface).into()
        }

        #[proc_macro_attribute]
        pub fn contract(attr: TokenStream, item: TokenStream) -> TokenStream {
            contract::generate(attr.into(), item.into(), GenerateMode::Contract).into()
        }
    }
}

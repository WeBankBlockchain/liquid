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
#![allow(dead_code)]

extern crate proc_macro;

#[macro_use]
mod error;
mod derive;
mod traits;
mod utils;

use cfg_if::cfg_if;
use derive::wrapper;
use proc_macro::TokenStream;

cfg_if! {
    if #[cfg(feature = "collaboration")] {
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

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

extern crate proc_macro;

#[macro_use]
mod error;

mod in_out;
mod state;
mod utils;
mod wrapper;

#[proc_macro_derive(InOut)]
pub fn in_out_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    wrapper::generate_wrapper(in_out::generate(input.into())).into()
}

#[proc_macro_derive(State)]
pub fn state_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    wrapper::generate_wrapper(state::generate(input.into())).into()
}

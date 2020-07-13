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

use crate::{codegen::GenerateCode, ir::Contract};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[derive(From)]
pub struct EnvTypes<'a> {
    #[allow(dead_code)]
    contract: &'a Contract,
}

impl<'a> GenerateCode for EnvTypes<'a> {
    fn generate_code(&self) -> TokenStream2 {
        quote! {
            type Address = liquid_env::types::Address;
            type String = liquid_env::types::String;
        }
    }
}

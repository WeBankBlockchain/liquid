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

mod contracts;
mod dispatch;
mod path_visitor;
mod storage;
mod utils;

use crate::{collaboration::ir::Collaboration, traits::GenerateCode, utils as macro_utils};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use contracts::Contracts;
use dispatch::Dispatch;
use storage::Storage;

impl GenerateCode for Collaboration {
    fn generate_code(&self) -> TokenStream2 {
        let ident = &self.ident;
        let rust_items = &self.rust_items;
        let types = macro_utils::generate_primitive_types();
        let storage = Storage::from(self).generate_code();
        let contracts = Contracts::from(self).generate_code();
        let dispatch = Dispatch::from(self).generate_code();

        quote! {
            mod #ident {
                use liquid_lang::intrinsics::*;
                use liquid_lang::ContractId;
                use liquid_macro::create;
                #types

                #contracts
                mod __liquid_private {
                    use super::*;

                    #storage
                    #dispatch
                }
                #(#rust_items)*
            }
        }
    }
}

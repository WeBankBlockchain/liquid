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

mod abi_gen;
mod contract_id;
mod contracts;
mod dispatch;
mod path_visitor;
mod rights;
mod storage;

use crate::{
    collaboration::ir::Collaboration, common::GenerateCode, utils as macro_utils,
};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;

use abi_gen::AbiGen;
use contract_id::ContractId;
use contracts::Contracts;
use dispatch::Dispatch;
use heck::CamelCase;
use rights::Rights;
use storage::Storage;

impl GenerateCode for Collaboration {
    fn generate_code(&self) -> TokenStream2 {
        let mod_ident = &self.mod_ident;
        let rust_items = &self.rust_items;
        let types = macro_utils::generate_primitive_types();
        let storage = Storage::from(self).generate_code();
        let contracts = Contracts::from(self).generate_code();
        let dispatch = Dispatch::from(self).generate_code();
        let rights = Rights::from(self).generate_code();
        let contract_id = ContractId::generate_code();
        let abi_gen = AbiGen::from(self).generate_code();

        quote! {
            mod #mod_ident {
                #[allow(unused_imports)]
                use liquid_lang::intrinsics::*;
                #[allow(unused_imports)]
                use liquid_macro::sign;
                #[allow(unused_imports)]
                use liquid_lang::Env;
                #[allow(unused_imports)]
                use liquid_lang::{ContractVisitor, ContractName};
                #types
                #contract_id

                #contracts
                #rights
                mod __liquid_private {
                    use super::*;

                    #storage
                    #dispatch
                }

                use __liquid_private::__liquid_acquire_storage_instance;
                use __liquid_private::__liquid_acquire_authorizers_guard;
                use __liquid_private::__liquid_authorization_check;

                #abi_gen
                #(#rust_items)*
            }

            #[cfg(feature = "liquid-abi-gen")]
            pub use crate::#mod_ident::__LIQUID_ABI_GEN;
        }
    }
}

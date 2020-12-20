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

use crate::{
    collaboration::{codegen::utils, ir::*},
    traits::GenerateCode,
    utils::filter_non_liquid_attributes,
};
use derive_more::From;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[derive(From)]
pub struct Rights<'a> {
    collaboration: &'a Collaboration,
}

impl<'a> GenerateCode for Rights<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let all_item_rights = &self.collaboration.all_item_rights;
        let impls = all_item_rights.iter().map(|item_rights| {
            let contract_ident = &item_rights.ty;
            let rights = &item_rights.rights;
            let fns = rights.iter().map(|right| {
                let attrs = filter_non_liquid_attributes(&right.attrs);
                let sig = &right.sig;
                let fn_ident = &sig.ident;
                let inputs = &sig.inputs;
                let output = &sig.output;
                let body = &right.body;
                let stmts = &body.stmts;
                let clone_error = format!(
                    "the exercising of right `{}` must be based on an existing `{}` \
                     contract, not a cloned one",
                    fn_ident, contract_ident
                );

                quote_spanned! { right.span =>
                    #(#attrs)*
                    pub fn #fn_ident (#inputs) #output {
                        if self.__liquid_forbids_constructing_contract.0 {
                            liquid_lang::env::revert(&#clone_error.to_owned())
                        }
                        #(#stmts)*
                    }
                }
            });

            quote! {
                impl #contract_ident {
                    #(#fns)*
                }
            }
        });

        let contracts = &self.collaboration.contracts;
        let envs = contracts.iter().map(|contract| {
            let ident = &contract.ident;

            quote! {
                impl #ident {
                    #[allow(unused)]
                    pub fn env(&self) -> liquid_lang::EnvAccess {
                        liquid_lang::EnvAccess {}
                    }
                }
            }
        });

        quote! {
            #(#impls)*
            #(#envs)*
        }
    }
}

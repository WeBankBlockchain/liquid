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
    collaboration::{
        codegen::{path_visitor::PathVisitor, utils},
        ir::*,
    },
    common::GenerateCode,
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
            let contract_ident = &item_rights.ident;
            let contract_ident_str = contract_ident.to_string();
            let rights = &item_rights.rights;
            let fns = rights.iter().map(|right| {
                let owners = &right.owners;
                let selectors = owners.iter().map(|owner| {
                    let from = &owner.from;
                    let with = &owner.with;
                    let span = from.span();
                    let ident = match from {
                        SelectFrom::This(ident) => quote! { self.#ident },
                        SelectFrom::Argument(ident) => quote! { #ident },
                    };

                    match with {
                        None => {
                            quote_spanned! { span => &#ident }
                        }
                        Some(SelectWith::Func(path)) => {
                            quote_spanned! { path.span() =>
                                #path(#ident)
                            }
                        }
                        Some(SelectWith::Obj(ast)) => {
                            let mut path_visitor =
                                PathVisitor::new(Some(ident), &ast.arena);
                            let stmts = path_visitor.eval(ast.root);
                            quote_spanned! { span =>
                                #stmts
                            }
                        }
                        _ => unreachable!(),
                    }
                });
                let attrs = filter_non_liquid_attributes(&right.attrs).collect::<Vec<_>>();
                let sig = &right.sig;
                let fn_ident = &sig.ident;
                let fn_ident_str = fn_ident.to_string();
                let inputs = &sig.inputs;
                let output = &sig.output;
                let body = &right.body;
                let stmts = &body.stmts;
                let (self_ref, rm_from_ptrs) = if sig.is_self_ref() {
                    (quote! { self }, quote! {})
                } else {
                    (quote! { &self }, quote! { ptrs.swap_remove(pos); })
                };

                quote_spanned! { right.span =>
                    #(#attrs)*
                    #[cfg_attr(feature = "std", allow(dead_code))]
                    pub fn #fn_ident(#inputs) #output {
                        loop {
                            #[cfg(test)]
                            if __liquid_acquire_storage_instance().__liquid_under_exec {
                                break;
                            }

                            // Validity check.
                            let self_addr = #self_ref as *const #contract_ident;
                            let ptrs = <ContractId::<#contract_ident> as FetchContract<#contract_ident>>::fetch_ptrs();
                            let pos = ptrs.iter().position(|&ptr| ptr == self_addr);
                            #[allow(unused_variables)]
                            if let Some(pos) = pos {
                                #rm_from_ptrs
                                break;
                            } else {
                                let mut error_info = String::from("DO NOT excise right on an inexistent `");
                                error_info.push_str(#contract_ident_str);
                                error_info.push_str("`contract");
                                liquid_lang::env::revert(&error_info);
                                unreachable!();
                            }
                        }
                        let __liquid_authorizers = __liquid_acquire_authorizers();
                        let __liquid_count = __liquid_authorizers.len();
                        {
                            // Authorization checking.
                            #[allow(unused_imports)]
                            use liquid_lang::{Can_Not_Select_Any_Account_Address_From_It, AcquireSigners};
                            #[allow(unused_mut)]
                            let mut owners = liquid_prelude::collections::BTreeSet::<address>::new();
                            #(owners.extend((#selectors).acquire_addrs());)*
                            if !__liquid_authorization_check(&owners) {
                                let mut error_info = String::from("exercising right `");
                                error_info.push_str(#fn_ident_str);
                                error_info.push_str("` of contract `");
                                error_info.push_str(#contract_ident_str);
                                error_info.push_str("` is not permitted");
                                liquid_lang::env::revert(&error_info);
                                unreachable!();
                            }
                            let signers = <#contract_ident as AcquireSigners>::acquire_signers(#self_ref);
                            __liquid_authorizers.extend(signers);
                            __liquid_authorizers.extend(owners);
                            __liquid_authorizers.sort();
                            __liquid_authorizers.dedup();
                        }
                        let result = {
                            #(#stmts)*
                        };
                        __liquid_authorizers.truncate(__liquid_count);
                        result
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
                impl liquid_lang::Env for #ident {}
            }
        });

        quote! {
            #(#impls)*
            #(#envs)*
        }
    }
}

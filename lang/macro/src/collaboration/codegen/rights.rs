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
    collaboration::{codegen::path_visitor::PathVisitor, ir::*},
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
            let mated_name = &item_rights.mated_name;
            let contract_ident_str = contract_ident.to_string();
            let rights = &item_rights.rights;
            let fns = rights.iter().map(|right| {
                let owners = &right.owners;
                let selectors = owners.iter().map(|owner| {
                    let from = &owner.from;
                    let with = &owner.with;
                    let span = from.span();
                    let from_ident = match from {
                        SelectFrom::This(ident) => quote_spanned! { ident.span() =>
                            &self.#ident
                        },
                        SelectFrom::Argument(ident) => quote_spanned! { ident.span() =>
                            &#ident
                        },
                    };
                    match with {
                        None => {
                            quote_spanned! { span => #from_ident }
                        }
                        Some(SelectWith::Func(path)) => {
                            quote_spanned! { path.span() =>
                                #path(#from_ident)
                            }
                        }
                        Some(SelectWith::Obj(ast)) => {
                            let mut path_visitor =
                                PathVisitor::new(Some(from_ident), &ast.arena);
                            let stmts = path_visitor.eval(ast.root);
                            quote_spanned! { span =>
                                #stmts
                            }
                        }
                        _ => unreachable!(),
                    }
                });

                let attrs =
                    filter_non_liquid_attributes(&right.attrs).collect::<Vec<_>>();
                let sig = &right.sig;
                let fn_ident = &sig.ident;
                let fn_ident_str = fn_ident.to_string();
                let inputs = &sig.inputs;
                let output = &sig.output;
                let body = &right.body;
                let stmts = &body.stmts;
                let self_ref = if sig.is_self_ref() {
                    quote! { self }
                } else {
                    quote! { &self }
                };

                quote_spanned! { right.span =>
                    #(#attrs)*
                    #[cfg_attr(feature = "std", allow(dead_code))]
                    pub fn #fn_ident(#inputs) #output {
                        let mut __liquid_guard = __liquid_acquire_authorizers_guard();
                        {
                            // Authorization checking.
                            #[allow(unused_imports)]
                            use liquid_lang::AcquireSigners;
                            #[allow(unused_mut)]
                            let mut owners =
                                liquid_prelude::collections::BTreeSet::<&'_ Address>::new();
                            #(owners.extend(liquid_lang::acquire_addrs(#selectors));)*
                            if !__liquid_authorization_check(&owners) {
                                let mut error_info = String::from("exercising right `");
                                error_info.push_str(#fn_ident_str);
                                error_info.push_str("` of contract `");
                                error_info.push_str(#contract_ident_str);
                                error_info.push_str("` is not permitted");
                                liquid_lang::env::revert(&error_info);
                                unreachable!();
                            }
                            let signers = <#mated_name as AcquireSigners>::acquire_signers(#self_ref);
                            let authorizers = __liquid_guard.authorizers();
                            authorizers.extend(
                                signers
                                    .into_iter()
                                    .map(|signer| signer.clone())
                            );
                            authorizers.extend(
                                owners
                                    .into_iter()
                                    .map(|owner| owner.clone())
                            );
                            authorizers.sort();
                            authorizers.dedup();
                        }
                        #(#stmts)*
                    }
                }
            });

            quote! {
                impl #mated_name {
                    #(#fns)*
                }
            }
        });

        let contracts = &self.collaboration.contracts;
        let envs = contracts.iter().map(|contract| {
            let mated_name = &contract.mated_name;

            quote! {
                impl liquid_lang::Env for #mated_name {}
            }
        });

        quote! {
            #(#impls)*
            #(#envs)*
        }
    }
}

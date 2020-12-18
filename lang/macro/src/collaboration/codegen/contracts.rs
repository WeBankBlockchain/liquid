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
        codegen::path_visitor::PathVisitor,
        ir::{Collaboration, SelectFrom, SelectWith, Selector},
    },
    traits::GenerateCode,
    utils::filter_non_liquid_attributes,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[derive(From)]
pub struct Contracts<'a> {
    collaboration: &'a Collaboration,
}

impl<'a> GenerateCode for Contracts<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let structs = self.generate_structs();
        let impls = self.generate_impls();

        quote! {
            #structs
            #impls
        }
    }
}

impl<'a> Contracts<'a> {
    fn generate_structs(&self) -> TokenStream2 {
        let contracts = &self.collaboration.contracts;
        let structs = contracts.iter().map(|contract| {
            let ident = &contract.ident;
            let attrs = filter_non_liquid_attributes(&contract.attrs);
            let fields = contract.fields.named.iter().map(|field| {
                let ident = field.ident.as_ref().unwrap();
                let attrs = filter_non_liquid_attributes(&field.attrs);
                let ty = &field.ty;
                let span = field.span();

                quote_spanned! { span =>
                    #(#attrs)*
                    pub #ident: #ty,
                }
            });

            quote_spanned! { contract.span =>
                #(#attrs)*
                #[derive(liquid_lang::InOut)]
                pub struct #ident {
                    #(#fields)*
                }
            }
        });

        quote! {
            #(#structs)*
        }
    }

    fn generate_impls(&self) -> TokenStream2 {
        let contracts = &self.collaboration.contracts;
        let impls = contracts.iter().map(|contract| {
            let ident = &contract.ident;
            let field_signers = &contract.field_signers;

            let selectors = field_signers.iter().map(|selector| {
                let from = &selector.from;
                let with = &selector.with;
                let field_ident = match from {
                    SelectFrom::This(ident) => ident,
                    _ => unreachable!(),
                };

                match with {
                    None => {
                        quote_spanned! { field_ident.span() =>
                            &self.#field_ident
                        }
                    }
                    Some(SelectWith::Func(path)) => {
                        quote_spanned! { path.span() =>
                            #path(&self.#field_ident)
                        }
                    }
                    Some(SelectWith::Obj(ast)) => {
                        let mut path_visitor = PathVisitor::new(
                            Some(quote! { self.#field_ident }),
                            &ast.arena,
                        );
                        let stmts = path_visitor.eval(ast.root);
                        quote_spanned! { field_ident.span() =>
                            #stmts
                        }
                    }
                }
            });

            quote! {
                impl liquid_lang::AcquireSigners for #ident {
                    fn acquire_signers(&self) -> liquid_prelude::vec::Vec<address> {
                        use liquid_lang::Can_Not_Select_Any_Account_Address_From_It;

                        let mut addresses = Vec::new();
                        #(addresses.extend((#selectors).acquire_addrs());)*
                        addresses
                    }
                }

                impl liquid_lang::You_Should_Use_An_Valid_Contract_Type for #ident {}
            }
        });

        quote! {
            #(#impls)*
        }
    }
}

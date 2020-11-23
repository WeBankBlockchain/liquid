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
    codegen::GenerateCode,
    ir::{utils, Contract, Function, FunctionKind},
};
use derive_more::From;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{punctuated::Punctuated, spanned::Spanned, Token};

#[derive(From)]
pub struct Storage<'a> {
    contract: &'a Contract,
}

impl<'a> GenerateCode for Storage<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let span = self.contract.storage.span();
        let storage_struct = self.generate_storage_struct();
        let function_impls = self.generate_functions();
        let constants = self.generate_constants();

        quote_spanned! { span =>
            mod __liquid_storage {
                #[allow(unused_imports)]
                use super::*;

                #storage_struct
            }

            pub use __liquid_storage::Storage;

            const _: () = {
                #function_impls
                #constants
            };
        }
    }
}

impl<'a> Storage<'a> {
    fn generate_storage_struct(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();
        let attrs = utils::filter_non_liquid_attributes(&storage.attrs);

        let mut fields = storage.fields.clone();
        fields.named.iter_mut().for_each(|field| {
            field.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: Default::default(),
            });

            let ty = &field.ty;
            field.ty = syn::parse2::<syn::Type>(quote_spanned!( span => <#ty as liquid_core::storage::You_Should_Use_A_Container_To_Wrap_Your_State_Field_In_Storage>::T)).unwrap();
        });

        let field_idents = fields
            .named
            .iter()
            .map(|field| {
                field
                    .ident
                    .clone()
                    .expect("unnamed fields are not allowed in liquid")
            })
            .collect::<Vec<_>>();

        let keys = field_idents
            .iter()
            .map(|ident| syn::LitStr::new(ident.to_string().as_str(), Span::call_site()))
            .collect::<Punctuated<syn::LitStr, Token![,]>>();
        let keys_count = keys.len();

        let bind_stats = field_idents.iter().enumerate().map(|(i, ident)| {
            quote_spanned! { span =>
                #ident: liquid_core::storage::Bind::bind_with(Self::STORAGE_KEYS[#i].as_bytes()),
            }
        });

        quote_spanned! { span =>
            #(#attrs)*
            #[cfg_attr(test, derive(Debug))]
            pub struct Storage
                #fields

            impl liquid_core::storage::Flush for Storage {
                fn flush(&mut self) {
                    #(liquid_core::storage::Flush::flush(&mut self.#field_idents);)*
                }
            }

            impl Storage {
                #[allow(unused)]
                const STORAGE_KEYS: [&'static str; #keys_count] = [ #keys ];

                #[allow(unused)]
                pub fn env(&self) -> liquid_lang::EnvAccess {
                    liquid_lang::EnvAccess {}
                }
            }

            impl liquid_core::storage::New for Storage {
                fn new() -> Self {
                    Self {
                        #(#bind_stats)*
                    }
                }
            }
        }
    }

    fn generate_constructor(&self) -> TokenStream2 {
        let constructor = &self.contract.constructor;
        let span = constructor.span();
        let attrs = utils::filter_non_liquid_attributes(constructor.attrs.iter());
        let ident = &constructor.sig.ident;
        let inputs = &constructor.sig.inputs;
        let output = &constructor.sig.output;
        let body = &constructor.body;

        quote_spanned! { span =>
            #(#attrs)*
            pub fn #ident(#inputs) #output
                #body
        }
    }

    fn generate_function(&self, function: &Function) -> TokenStream2 {
        let span = function.span();
        let vis = if let FunctionKind::Normal = function.kind {
            quote_spanned! {span =>}
        } else {
            quote_spanned! {span => pub}
        };
        let attrs = utils::filter_non_liquid_attributes(function.attrs.iter())
            .collect::<Vec<_>>();
        let sig = &function.sig;
        let ident = &sig.ident;
        let inputs = &sig.inputs;
        let output = &sig.output;
        let body = &function.body;
        let stmts = &body.stmts;
        let is_mut = sig.is_mut();

        if is_mut {
            quote_spanned! { span =>
                #[cfg(not(test))]
                #(#attrs)*
                #vis fn #ident(#inputs) #output
                    #body

                #[cfg(test)]
                #(#attrs)*
                #vis fn #ident(#inputs) #output {
                    let result = (move || {
                        #(#stmts)*
                    })();
                    liquid_core::storage::reset_mutable_call_records();
                    result
                }
            }
        } else {
            quote_spanned! { span =>
                #(#attrs)*
                #vis fn #ident(#inputs) #output {
                    let result = (move || {
                        #(#stmts)*
                    })();
                    if liquid_core::storage::has_mutable_call_happens() {
                        liquid_core::env::revert(&String::from(
                            "attempt to call mutable external interfaces in an immutable \
                             transaction, all writes will be discarded",
                        ));
                    }
                    result
                }
            }
        }
    }

    fn generate_functions(&self) -> TokenStream2 {
        let storage = &self.contract.storage;
        let span = storage.span();
        let constructor = self.generate_constructor();
        let functions = self
            .contract
            .functions
            .iter()
            .map(|func| self.generate_function(func));

        quote_spanned!(span =>
            impl Storage {
                #constructor
                #(#functions)*
            }
        )
    }

    fn generate_constants(&self) -> TokenStream2 {
        let constants = &self.contract.constants;

        quote! {
            impl Storage {
                #(#constants)*
            }
        }
    }
}

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

use crate::{collaboration::ir::Collaboration, traits::GenerateCode};
use derive_more::From;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{punctuated::Punctuated, Token};

#[derive(From)]
pub struct Storage<'a> {
    collaboration: &'a Collaboration,
}

impl<'a> GenerateCode for Storage<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let storage_struct = self.generate_storage_struct();

        quote! {
            mod __liquid_storage {
                #[allow(unused_imports)]
                use super::*;
                #storage_struct
            }
            pub use __liquid_storage::Storage;
            pub use __liquid_storage::__liquid_acquire_storage_instance;
        }
    }
}

impl<'a> Storage<'a> {
    fn generate_storage_struct(&self) -> TokenStream2 {
        let contracts = &self.collaboration.contracts;
        let (field_idents, fields): (Vec<_>, Vec<_>) = contracts
            .iter()
            .map(|contract| {
                use heck::SnakeCase;

                let contract_ident = &contract.ident;
                let field_ident = Ident::new(&format!("__liquid_{}", contract_ident.to_string().to_snake_case()), Span::call_site());
                (
                    field_ident.clone(),
                    quote! {
                        // The 2nd field in value is used to mark whether the contract is abolished.
                        pub #field_ident: liquid_lang::storage::Mapping<u32, (#contract_ident, bool)>,
                    },
                )
            })
            .unzip();

        let keys = field_idents
            .iter()
            .map(|ident| syn::LitStr::new(ident.to_string().as_str(), Span::call_site()))
            .collect::<Punctuated<syn::LitStr, Token![,]>>();
        let keys_count = keys.len();

        let bind_stats = field_idents.iter().enumerate().map(|(i, ident)| {
            quote! {
                #ident: liquid_lang::storage::Bind::bind_with(Self::STORAGE_KEYS[#i].as_bytes()),
            }
        });

        quote! {
            pub struct Storage {
                pub __liquid_authorizers: liquid_prelude::collections::BTreeSet<address>,
                #(#fields)*
            }

            impl liquid_lang::storage::Flush for Storage {
                fn flush(&mut self) {
                    #(liquid_lang::storage::Flush::flush(&mut self.#field_idents);)*
                }
            }

            impl Storage {
                #[allow(unused)]
                const STORAGE_KEYS: [&'static str; #keys_count] = [ #keys ];
            }

            impl liquid_lang::storage::New for Storage {
                fn new() -> Self {
                    Self {
                        __liquid_authorizers: {
                            let mut addrs = liquid_prelude::collections::BTreeSet::new();
                            addrs.insert(liquid_lang::env::get_caller());
                            addrs
                        },
                        #(#bind_stats)*
                    }
                }
            }

            pub fn __liquid_acquire_storage_instance() -> &'static mut Storage {
                use liquid_lang::storage::New;
                use spin::Once;
                static mut STORAGE_INSTANCE: Once<Storage> = Once::INIT;

                unsafe {
                    STORAGE_INSTANCE.call_once(Storage::new);
                    STORAGE_INSTANCE.get_mut().unwrap()
                }
            }
        }
    }
}

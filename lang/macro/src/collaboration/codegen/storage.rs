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

use crate::{collaboration::ir::Collaboration, common::GenerateCode};
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
            pub use __liquid_storage::*;
        }
    }
}

impl<'a> Storage<'a> {
    fn generate_storage_struct(&self) -> TokenStream2 {
        let contracts = &self.collaboration.contracts;
        let ptrs_field_names = contracts
            .iter()
            .map(|contract| {
                let storage_field_name = &contract.storage_field_name;
                Ident::new(
                    &format!("{}_ptrs", storage_field_name.to_string()),
                    Span::call_site(),
                )
            })
            .collect::<Vec<_>>();
        let (storage_field_idents, fields): (Vec<_>, Vec<_>) = contracts
            .iter()
            .enumerate()
            .map(|(i, contract)| {
                let contract_ident = &contract.ident;
                let storage_field_name = &contract.storage_field_name;
                let ptrs_field_name = &ptrs_field_names[i];
                (
                    storage_field_name,
                    quote! {
                        // The 2nd field in value is used to mark whether the contract is abolished.
                        pub #storage_field_name: liquid_lang::storage::Mapping<u32, (#contract_ident, bool)>,
                        pub #ptrs_field_name: Vec<*const #contract_ident>,
                    },
                )
            })
            .unzip();

        let keys = storage_field_idents
            .iter()
            .map(|ident| syn::LitStr::new(ident.to_string().as_str(), Span::call_site()))
            .collect::<Punctuated<syn::LitStr, Token![,]>>();
        let keys_count = keys.len();

        let bind_stats = storage_field_idents.iter().enumerate().map(|(i, ident)| {
            quote! {
                #ident: liquid_lang::storage::Bind::bind_with(Self::STORAGE_KEYS[#i].as_bytes()),
            }
        });
        let ptrs_inits = ptrs_field_names.iter().map(|ident| {
            quote! {
                #ident: Vec::new(),
            }
        });

        quote! {
            pub struct Storage {
                pub __liquid_authorizers: liquid_prelude::vec::Vec<address>,
                #(#fields)*
            }

            impl liquid_lang::storage::Flush for Storage {
                fn flush(&mut self) {
                    #(liquid_lang::storage::Flush::flush(&mut self.#storage_field_idents);)*
                }
            }

            impl Storage {
                #[allow(unused)]
                const STORAGE_KEYS: [&'static str; #keys_count] = [ #keys ];
            }

            impl liquid_lang::storage::New for Storage {
                fn new() -> Self {
                    let mut storage = Self {
                        __liquid_authorizers: liquid_prelude::vec::Vec::new(),
                        #(#bind_stats)*
                        #(#ptrs_inits)*
                    };
                    #[cfg(test)]
                    {
                        #(storage.#storage_field_idents.initialize();)*
                    }
                    storage
                }
            }

            pub fn __liquid_acquire_authorizers() -> &'static mut liquid_prelude::vec::Vec<address> {
                let storage = __liquid_acquire_storage_instance();
                &mut storage.__liquid_authorizers
            }

            pub fn __liquid_authorization_check(parties: &liquid_prelude::collections::BTreeSet<address>) -> bool {
                let authorizers = __liquid_acquire_authorizers();
                if authorizers.is_empty() {
                    let caller = liquid_lang::env::get_caller();
                    for party in parties {
                        if *party != caller {
                            return false;
                        }
                    }
                    true
                } else {
                    for party in parties {
                        if !authorizers.contains(party) {
                            return false;
                        }
                    }
                    true
                }
            }

            #[cfg(not(test))]
            pub fn __liquid_acquire_storage_instance() -> &'static mut Storage {
                use liquid_lang::storage::New;
                use spin::Once;
                static mut STORAGE_INSTANCE: Once<Storage> = Once::INIT;

                unsafe {
                    STORAGE_INSTANCE.call_once(Storage::new);
                    STORAGE_INSTANCE.get_mut().unwrap()
                }
            }

            #[cfg(test)]
            pub fn __liquid_acquire_storage_instance() -> &'static mut Storage {
                thread_local!(
                    static STORAGE_INSTANCE: ::core::cell::RefCell<Storage> = ::core::cell::RefCell::new(
                        <Storage as liquid_lang::storage::New>::new(),
                    )
                );

                STORAGE_INSTANCE.with(|instance| unsafe {
                    &mut (*instance.as_ptr())
                })
            }
        }
    }
}

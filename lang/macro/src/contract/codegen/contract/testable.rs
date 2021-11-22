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
    common::GenerateCode,
    contract::ir::{Contract, FnArg},
    utils as lang_utils,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[derive(From)]
pub struct Testable<'a> {
    contract: &'a Contract,
}

impl<'a> GenerateCode for Testable<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let testable_storage = self.generate_testable_storage();
        let constructor = self.generate_constructor();

        quote! {
            #[cfg(test)]
            mod __liquid_testable {
                use super::*;

                #testable_storage

                impl TestableStorage {
                    #constructor
                }
            }

            #[cfg(test)]
            pub use __liquid_testable::TestableStorage;
        }
    }
}

impl<'a> Testable<'a> {
    fn generate_testable_storage(&self) -> TokenStream2 {
        let attrs =
            lang_utils::filter_non_liquid_attributes(self.contract.storage.attrs.iter());

        quote! {
            #(#attrs)*
            #[derive(Debug)]
            pub struct TestableStorage {
                contract: Storage
            }

            impl From<Storage> for TestableStorage {
                fn from(contract: Storage) -> Self {
                    Self {
                        contract
                    }
                }
            }

            impl core::ops::Deref for TestableStorage {
                type Target = Storage;

                fn deref(&self) -> &Self::Target {
                    &self.contract
                }
            }

            impl core::ops::DerefMut for TestableStorage {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut self.contract
                }
            }
        }
    }

    fn generate_constructor(&self) -> TokenStream2 {
        let constructor = &self.contract.constructor;
        let attrs = &constructor.attrs;
        let sig = &constructor.sig;
        let ident = &sig.ident;
        let args = sig.inputs.iter().skip(1);
        let arg_idents = sig.inputs.iter().skip(1).map(|arg| match arg {
            FnArg::Typed(ident_type) => &ident_type.ident,
            _ => unreachable!(),
        });

        quote! {
            #(#attrs)*
            pub fn #ident(#(#args,)*) -> Self {
                let mut contract = <Storage as liquid_lang::storage::New>::new();
                contract.#ident(#(#arg_idents,)*);
                Self {
                    contract
                }
            }
        }
    }
}

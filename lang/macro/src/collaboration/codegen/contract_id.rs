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
    common::GenerateCode,
    utils::filter_non_liquid_attributes,
};

use proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub struct ContractId;

impl ContractId {
    fn generate_contract_id() -> TokenStream2 {
        quote! {
            pub struct ContractId<T>
            where
                T: liquid_lang::You_Should_Use_An_Valid_Contract_Type,
            {
                pub __liquid_id: u32,
                pub __liquid_marker: core::marker::PhantomData<fn() -> T>,
            }

            #[cfg(test)]
            impl<T> ::core::fmt::Debug for ContractId<T>
            where
                T: liquid_lang::You_Should_Use_An_Valid_Contract_Type,
            {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    write!(f, "ContractId({})", self.__liquid_id)
                }
            }

            // https://github.com/rust-lang/rust/issues/41481
            impl<T> ::core::clone::Clone for ContractId<T>
            where
                T: liquid_lang::You_Should_Use_An_Valid_Contract_Type,
            {
                fn clone(&self) -> Self {
                    Self {
                        __liquid_id: self.__liquid_id,
                        __liquid_marker: Default::default(),
                    }
                }
            }

            impl<T> ::core::marker::Copy for ContractId<T>
            where
                T: liquid_lang::You_Should_Use_An_Valid_Contract_Type,
            {
            }

            impl<T> ::core::cmp::PartialEq for ContractId<T>
            where
                T: liquid_lang::You_Should_Use_An_Valid_Contract_Type,
            {
                fn eq(&self, other: &Self) -> bool {
                    self.__liquid_id == other.__liquid_id
                }
            }

            impl<T> scale::Encode for ContractId<T>
            where
                T: liquid_lang::You_Should_Use_An_Valid_Contract_Type,
            {
                fn encode(&self) -> liquid_prelude::vec::Vec<u8> {
                    <u32 as scale::Encode>::encode(&self.__liquid_id)
                }
            }

            impl<T> liquid_lang::You_Should_Use_An_Valid_Field_Type for ContractId<T>
            where
                T: liquid_lang::You_Should_Use_An_Valid_Contract_Type,
            {
            }

            impl<T> liquid_lang::You_Should_Use_An_Valid_Input_Type for ContractId<T>
            where
                T: liquid_lang::You_Should_Use_An_Valid_Contract_Type
            {
            }

            impl<T> liquid_lang::You_Should_Use_An_Valid_Return_Type for ContractId<T>
            where
                T: liquid_lang::You_Should_Use_An_Valid_Contract_Type
            {
            }
        }
    }

    pub fn generate_code() -> TokenStream2 {
        let contract_id = Self::generate_contract_id();

        quote! {
            mod __liquid_contract_id {
                #[allow(unused_imports)]
                use super::*;
                #contract_id
            }

            pub use __liquid_contract_id::ContractId;
        }
    }
}

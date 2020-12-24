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
use derive_more::From;
use heck::SnakeCase;
use proc_macro2::{Ident, Literal, Span, TokenStream as TokenStream2};
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
                        let mut path_visitor =
                            PathVisitor::new(Some(quote! { self.#field_ident }), &ast.arena);
                        let stmts = path_visitor.eval(ast.root);
                        quote_spanned! { field_ident.span() =>
                            #stmts
                        }
                    }
                    Some(SelectWith::Inherited(field_ty)) => {
                        quote_spanned! { field_ident.span() =>
                            <#field_ty as liquid_lang::AcquireSigners>::acquire_signers(&self.#field_ident)
                        }
                    }
                }
            });

            let ident_str = ident.to_string();
            let storage_field_name = &contract.storage_field_name;
            let ptrs_field_name = Ident::new(&format!("{}_ptrs", storage_field_name.to_string()), Span::call_site());

            let no_available_signers_error = format!("no available signers to sign this `{}` contract", ident_str);
            let unauthorized_signing_error = format!("signing of contract `{}` is not permitted", ident_str);

            quote! {
                impl liquid_lang::AcquireSigners for #ident {
                    fn acquire_signers(&self) -> liquid_prelude::collections::BTreeSet::<address> {
                        use liquid_lang::Can_Not_Select_Any_Account_Address_From_It;

                        let mut signers = liquid_prelude::collections::BTreeSet::new();
                        #(signers.extend((#selectors).acquire_addrs());)*
                        signers
                    }
                }

                impl liquid_lang::You_Should_Use_An_Valid_Contract_Type for #ident {}

                impl liquid_lang::FetchContract<#ident> for ContractId<#ident> {
                    const CONTRACT_NAME: &'static str = #ident_str;
                    fn fetch_collection() -> &'static mut liquid_lang::storage::Mapping<u32, (#ident, bool)> {
                        let storage = __liquid_acquire_storage_instance();
                        &mut storage.#storage_field_name
                    }

                    fn fetch_ptrs() -> &'static mut Vec<*const #ident> {
                        let storage = __liquid_acquire_storage_instance();
                        &mut storage.#ptrs_field_name
                    }
                }

                impl scale::Decode for ContractId<#ident>
                {
                    fn decode<I: scale::Input>(input: &mut I) -> ::core::result::Result<Self, scale::Error> {
                        let __liquid_id = <u32 as scale::Decode>::decode(input)?;
                        let (contract, _) = <Self as FetchContract<#ident>>::fetch(__liquid_id);
                        let ptrs = <Self as FetchContract<#ident>>::fetch_ptrs();
                        ptrs.push(contract as *const #ident);
                        Ok(Self { 
                            __liquid_id,
                            __liquid_marker: Default::default(),
                        })
                    }
                }

                impl ::core::convert::AsRef<#ident> for ContractId<#ident>
                {
                    fn as_ref(&self) -> &#ident {
                        let (contract, _) = self.abolishment_check();
                        contract
                    }
                }
    
                impl ::core::convert::AsMut<#ident> for ContractId<#ident>
                {
                    fn as_mut(&mut self) -> &mut #ident {
                        let (contract, _) = self.abolishment_check();
                        contract
                    }
                }

                impl ContractId<#ident> {
                    fn abolishment_check(&self) -> (&'static mut #ident, &'static mut bool) {
                        use liquid_prelude::string::ToString;
    
                        let (contract, abolished) = <Self as FetchContract<#ident>>::fetch(self.__liquid_id);
                        if *abolished {
                            let mut error_info = String::from("the contract `");
                            error_info.push_str(<Self as FetchContract<#ident>>::CONTRACT_NAME);
                            error_info.push_str("` with id `");
                            error_info.push_str(&self.__liquid_id.to_string());
                            error_info.push_str("` had been abolished already");
                            liquid_lang::env::revert(&error_info);
                            unreachable!();
                        }
                        (contract, abolished)
                    }
                }
    
                impl ContractId<#ident> {
                    pub fn take(self) -> #ident {
                        let (contract, abolished) = self.abolishment_check();
                        let signers = <#ident as liquid_lang::AcquireSigners>::acquire_signers(contract);
                        if !__liquid_authorization_check(&signers) {
                            let mut error_info = String::from("the contract `");
                            error_info.push_str(<Self as FetchContract<#ident>>::CONTRACT_NAME);
                            error_info.push_str("` with id `");
                            error_info.push_str(&self.__liquid_id.to_string());
                            error_info.push_str("` is not allowed to be abolished");
                            liquid_lang::env::revert(&error_info);
                            unreachable!();
                        }
                        *abolished = true;
                        let encoded = <#ident as scale::Encode>::encode(contract);
                        let decoded = <#ident as scale::Decode>::decode(&mut encoded.as_slice()).unwrap();
                        let ptrs = <Self as FetchContract<#ident>>::fetch_ptrs();
                        println!("origin {:p}", &decoded as *const #ident);
                        ptrs.push(&decoded as *const #ident);
                        decoded
                    }

                    pub fn fetch(&self) -> #ident {
                        let (contract, _) = <Self as FetchContract<#ident>>::fetch(self.__liquid_id);
                        let encoded = <#ident as scale::Encode>::encode(contract);
                        let decoded = <#ident as scale::Decode>::decode(&mut encoded.as_slice()).unwrap();
                        decoded
                    }

                    pub fn is_abolished(&self) -> bool {
                        let (_, abolished) = <Self as FetchContract<#ident>>::fetch(self.__liquid_id);
                        *abolished
                    }

                    #[cfg(test)]
                    pub fn exec<F, R>(&self, f: F) -> R
                    where
                        F: FnOnce(#ident) -> R
                    {
                        let (contract, abolished) = self.abolishment_check();
                        let encoded = <#ident as scale::Encode>::encode(contract);
                        let decoded = <#ident as scale::Decode>::decode(&mut encoded.as_slice()).unwrap();
                        let ptr = &decoded as *const #ident;
                        println!("origin {:p}", ptr);
                        let ptrs = <Self as FetchContract<#ident>>::fetch_ptrs();
                        let len = ptrs.len();
                        ptrs.push(ptr);
                        println!("{:?}", ptrs);
                        let result = f(decoded);
                        ptrs.swap_remove(len);
                        *abolished = true;
                        result
                    }
                }

                impl ContractId<#ident> {
                    const UNAUTHORIZED_SIGNING_ERROR: &'static str = #unauthorized_signing_error;
                    const NO_AVAILABLE_SIGNERS_ERROR: &'static str = #no_available_signers_error;
                }
            }
        });

        quote! {
            #(#impls)*
        }
    }
}

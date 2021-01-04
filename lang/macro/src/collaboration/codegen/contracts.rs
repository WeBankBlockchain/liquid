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
        let acquire_signers = self.generate_acquire_signers();
        let codecs = self.generate_codecs();
        let contract_visitors = self.generate_contract_visitors();
        let fns = self.generate_fns();
        let constants = self.generate_constants();

        quote! {
            #(#structs)*
            #(#acquire_signers)*
            #(#codecs)*
            #(#contract_visitors)*
            #(#fns)*
            #(#constants)*
        }
    }
}

impl<'a> Contracts<'a> {
    fn generate_structs(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        let contracts = &self.collaboration.contracts;
        contracts.iter().map(|contract| {
            let ident = &contract.ident;
            let mated_name = &contract.mated_name;
            let attrs = filter_non_liquid_attributes(&contract.attrs).collect::<Vec<_>>();
            let fields = contract
                .fields
                .named
                .iter()
                .map(|field| {
                    let ident = field.ident.as_ref().unwrap();
                    let attrs = filter_non_liquid_attributes(&field.attrs);
                    let ty = &field.ty;
                    let span = field.span();

                    quote_spanned! { span =>
                        #(#attrs)*
                        pub #ident: #ty,
                    }
                })
                .collect::<Vec<_>>();

            quote_spanned! { contract.span =>
                #(#attrs)*
                #[derive(liquid_lang::InOut)]
                pub struct #ident {
                    #(#fields)*
                }

                impl liquid_lang::ContractType for #ident {
                    type T = #ident;
                }

                #[allow(non_camel_case_types)]
                #[derive(liquid_lang::InOut)]
                pub struct #mated_name {
                    #(#fields)*
                }

                impl liquid_lang::ContractType for #mated_name {
                    type T = #ident;
                }

                impl ::core::convert::AsRef<#ident> for #mated_name {
                    fn as_ref(&self) -> &#ident {
                        let ptr = self as *const #mated_name;
                        unsafe {
                            let ptr = ::core::mem::transmute::<_, *const #ident>(ptr);
                            &(*ptr)
                        }
                    }
                }

                impl ::core::convert::AsMut<#ident> for #mated_name {
                    fn as_mut(&mut self) -> &mut #ident {
                        let ptr = self as *mut #mated_name;
                        unsafe {
                            let ptr = ::core::mem::transmute::<_, *mut #ident>(ptr);
                            &mut (*ptr)
                        }
                    }
                }
            }
        })
    }

    fn generate_acquire_signers(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        let contracts = &self.collaboration.contracts;
        contracts.iter().map(|contract| {
            let span = contract.span;
            let ident = &contract.ident;
            let field_signers = &contract.field_signers;
            let mated_name = &contract.mated_name;
            let signers = field_signers.iter().map(|selector| {
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
                            PathVisitor::new(Some(quote! { &self.#field_ident }), &ast.arena);
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

            let acquire_signers = quote_spanned! { span =>
                fn acquire_signers(&self) -> liquid_prelude::collections::BTreeSet::<&address> {
                    #[allow(unused_imports)]
                    let mut signers = liquid_prelude::collections::BTreeSet::new();
                    #(signers.extend(liquid_lang::acquire_addrs(#signers));)*
                    signers
                }
            };

            quote! {
                impl liquid_lang::AcquireSigners for #ident {
                    #acquire_signers
                }

                impl liquid_lang::AcquireSigners for #mated_name {
                    #acquire_signers
                }
            }
        })
    }

    fn generate_codecs(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        let contracts = &self.collaboration.contracts;
        contracts.iter().map(|contract| {
            let ident = &contract.ident;
            quote! {
                impl scale::Decode for ContractId<#ident> {
                    fn decode<I: scale::Input>(input: &mut I) -> ::core::result::Result<Self, scale::Error> {
                        let __liquid_id = <u32 as scale::Decode>::decode(input)?;
                        Ok(Self {
                            __liquid_id,
                            __liquid_marker: Default::default(),
                        })
                    }
                }
            }
        })
    }

    fn generate_contract_visitors(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        let contracts = &self.collaboration.contracts;
        contracts.iter().map(|contract| {
            let ident = &contract.ident;
            let mated_name = &contract.mated_name;
            let state_name = &contract.state_name;
            quote! {
                impl liquid_lang::ContractVisitor for ContractId<#ident> {
                    type Contract = #ident;
                    type ContractId = Self;

                    fn fetch(&self) -> #ident {
                        let storage = __liquid_acquire_storage_instance();
                        let contracts = &mut storage.#state_name;

                        if let Some((contract, _)) = contracts.get(&self.__liquid_id) {
                            let encoded = <#mated_name as scale::Encode>::encode(contract);
                            let decoded = <#ident as scale::Decode>::decode(&mut encoded.as_slice()).unwrap();
                            decoded
                        } else {
                            Self::inexistent_error(self.__liquid_id);
                            unreachable!();
                        }
                    }

                    fn sign_new_contract(contract: #ident) -> Self {
                        let storage = __liquid_acquire_storage_instance();
                        let contracts = &mut storage.#state_name;
                        let signers = <#ident as liquid_lang::AcquireSigners>::acquire_signers(&contract);
                        if signers.is_empty() {
                            liquid_lang::env::revert(&String::from(Self::NO_AVAILABLE_SIGNERS_ERROR));
                        }
            
                        if !__liquid_authorization_check(&signers) {
                            liquid_lang::env::revert(&String::from(Self::UNAUTHORIZED_SIGNING_ERROR));
                        }
                        let len = contracts.len();
                        let mated = unsafe {
                            core::mem::transmute::<#ident, #mated_name>(contract)
                        };
                        contracts.insert(&len, (mated, false));
                        Self {
                            __liquid_id: len,
                            __liquid_marker: Default::default(),
                        }
                    }
                }
            }
        })
    }

    fn generate_fns(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        let contracts = &self.collaboration.contracts;
        contracts.iter().map(move |contract| {
            let ident = &contract.ident;
            let mated_name = &contract.mated_name;
            let state_name = &contract.state_name;

            let rights = self
                .collaboration
                .all_item_rights
                .iter()
                .filter(|item_rights| item_rights.ident == *ident)
                .map(|item_rights| item_rights.rights.iter())
                .flatten();

            let fns = rights.map(|right| {
                let sig = &right.sig;
                let fn_name = &sig.ident;
                let inputs = &sig.inputs.iter().skip(1).collect::<Vec<_>>();
                let input_idents = utils::generate_input_idents(&sig.inputs);
                let output = &sig.output;
                let need_abolish = !sig.is_self_ref();
                let execute = if need_abolish {
                    quote! {
                        let encoded = <#mated_name as scale::Encode>::encode(contract);
                        let decoded = <#mated_name as scale::Decode>::decode(&mut encoded.as_slice()).unwrap();
                        decoded.#fn_name(#(#input_idents,)*)
                    }
                } else {
                    quote! {
                        contract.#fn_name(#(#input_idents,)*)
                    }
                };

                quote! {
                    pub fn #fn_name(&self, #(#inputs,)*) #output {
                        let contract = self.__liquid_validity_check(#need_abolish);
                        #execute
                    }
                }
            });

            quote! {
                impl ContractId<#ident> {
                    fn __liquid_validity_check(&self, need_abolish: bool) -> &'static mut #mated_name {
                        let storage = __liquid_acquire_storage_instance();
                        let contracts = &mut storage.#state_name;

                        if let Some((contract, abolished)) = contracts.get_mut(&self.__liquid_id) {
                            if *abolished {
                                <Self as liquid_lang::ContractVisitor>::abolished_error(self.__liquid_id);
                            }
                            if need_abolish {
                                *abolished = true;
                            }
                            contract
                        } else {
                            <Self as liquid_lang::ContractVisitor>::inexistent_error(self.__liquid_id);
                            unreachable!();
                        }
                    }

                    #(#fns)*
                }
            }
        })
    }

    fn generate_constants(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        let contracts = &self.collaboration.contracts;
        contracts.iter().map(|contract| {
            let ident = &contract.ident;
            let ident_str = ident.to_string();
            let unauthorized_signing_error =
                format!("signing of contract `{}` is not permitted", ident_str);
            let no_available_signers_error =
                format!("no available signers to sign this `{}` contract", ident_str);
            quote! {
                impl liquid_lang::You_Should_Use_An_Valid_Contract_Type for #ident {}

                impl liquid_lang::ContractName for #ident {
                    const CONTRACT_NAME: &'static str = #ident_str;
                }

                impl ContractId<#ident> {
                    const UNAUTHORIZED_SIGNING_ERROR: &'static str = #unauthorized_signing_error;
                    const NO_AVAILABLE_SIGNERS_ERROR: &'static str = #no_available_signers_error;
                }
            }
        })
    }
}

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

use crate::{collaboration::ir::*, common, utils::filter_non_liquid_attributes};
use derive_more::From;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[derive(From)]
pub struct Dispatch<'a> {
    collaboration: &'a Collaboration,
}

impl<'a> common::GenerateCode for Dispatch<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let marker = self.generate_right_marker();
        let right_traits = self.generate_right_traits();
        let contract_traits = self.generate_contract_traits();
        let dispatch = self.generate_dispatch();
        let entry_point = self.generate_entry_point();

        quote! {
            #[cfg(not(test))]
            const _: () = {
                #marker
                #right_traits
                #contract_traits
                #dispatch
                #entry_point
            };
        }
    }
}

impl<'a> Dispatch<'a> {
    fn generate_right_marker(&self) -> TokenStream2 {
        quote! {
            pub struct Marker<S> {
                marker: core::marker::PhantomData<fn() -> S>,
            }
        }
    }

    fn generate_right_selector(right: &Right) -> [u8; 4] {
        let sig = &right.sig;
        let right_name = &sig.ident;
        let from = &right.from;
        let hash = liquid_primitives::hash::hash(
            format!("{}({})", from.to_string(), right_name.to_string()).as_bytes(),
        );
        [hash[0], hash[1], hash[2], hash[3]]
    }

    fn generate_contract_selector(contract: &ItemContract, for_fetch: bool) -> [u8; 4] {
        let contract_name = &contract.ident;
        let prefix = if for_fetch { "$" } else { "" };
        let hash = liquid_primitives::hash::hash(
            format!("{}{}", prefix, contract_name.to_string()).as_bytes(),
        );
        [hash[0], hash[1], hash[2], hash[3]]
    }

    fn generate_right_id(right: &Right) -> u32 {
        let selector = Self::generate_right_selector(right);
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&selector[..4]);
        u32::from_be_bytes(buf)
    }

    fn generate_contract_id(contract: &ItemContract) -> u32 {
        let selector = Self::generate_contract_selector(contract, false);
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&selector[..4]);
        u32::from_be_bytes(buf)
    }

    fn generate_right_traits(&self) -> TokenStream2 {
        let all_item_rights = &self.collaboration.all_item_rights;
        let traits = all_item_rights
            .iter()
            .map(|item_rights| {
                let rights = &item_rights.rights;
                rights
                    .iter()
                    .map(move |right| self.generate_right_trait(right))
            })
            .flatten();

        quote! {
            #(#traits)*
        }
    }

    fn generate_contract_traits(&self) -> TokenStream2 {
        let contracts = &self.collaboration.contracts;
        let traits = contracts
            .iter()
            .map(|item_contract| self.generate_contract_trait(item_contract));

        quote! {
            #(#traits)*
        }
    }

    fn generate_right_trait(&self, right: &Right) -> TokenStream2 {
        let right_id = Self::generate_right_id(right);
        let right_marker = quote! { Marker::<[(); #right_id as usize]> };
        let sig = &right.sig;

        let output = &sig.output;
        let (output_ty_checker, output_span) = match output {
            syn::ReturnType::Default => (quote! {()}, output.span()),
            syn::ReturnType::Type(_, ty) => {
                let return_ty = &*ty;
                (
                    quote! {
                        <#return_ty as liquid_lang::You_Should_Use_An_Valid_Output_Type>::T
                    },
                    return_ty.span(),
                )
            }
        };
        let right_output = quote_spanned! { output_span =>
            impl liquid_lang::FnOutput for #right_marker {
                type Output = #output_ty_checker;
            }
        };

        let selector = Self::generate_right_selector(right);
        let right_selector = {
            let input_tys = common::generate_input_tys(sig);
            let input_ty_checker = common::generate_ty_checker(input_tys.as_slice());
            let input_ty_checker_ident = Ident::new(
                &format!("__LIQUID_RIGHT_INPUT_CHECKER_{}", right_id),
                right.span,
            );

            quote! {
                #[allow(non_camel_case_types)]
                struct #input_ty_checker_ident #input_ty_checker;

                impl liquid_lang::FnSelector for #right_marker {
                    const SELECTOR: liquid_primitives::Selector =
                        liquid_primitives::Selector::from_le_bytes([#(#selector,)*]);
                }
            }
        };

        quote! {
            #right_output
            #right_selector
        }
    }

    fn generate_contract_trait(&self, contract: &ItemContract) -> TokenStream2 {
        let contract_id = Self::generate_contract_id(contract);
        let contract_marker = quote! { Marker::<[(); #contract_id as usize]> };

        let selector = Self::generate_contract_selector(contract, false);
        let contract_selector = {
            quote! {
                impl liquid_lang::FnSelector for #contract_marker {
                    const SELECTOR: liquid_primitives::Selector =
                        liquid_primitives::Selector::from_le_bytes([#(#selector,)*]);
                }
            }
        };

        quote! {
            #contract_selector
        }
    }

    fn generate_rights_dispatch_fragment(
        &self,
        item_rights: &ItemRights,
    ) -> TokenStream2 {
        use heck::SnakeCase;

        let rights = &item_rights.rights;
        let ty = &item_rights.ident;
        let fragments = rights.iter().map(|right| {
            let right_id = Self::generate_right_id(right);
            let right_marker = quote! { Marker::<[(); #right_id as usize]> };
            let sig = &right.sig;
            let right_name = &sig.ident;
            let input_idents = common::generate_input_idents(sig);
            let input_tys = common::generate_input_tys(sig);
            let inputs = input_idents.iter().zip(input_tys.iter()).map(|(ident, ty)| {
                quote! {
                    let #ident = <#ty as scale::Decode>::decode(data_ptr).map_err(|_| liquid_lang::DispatchError::InvalidParams)?;
                }
            });

            let flush = if !sig.is_self_ref() || sig.is_mut() {
                quote! {
                    <Storage as liquid_lang::storage::Flush>::flush(storage);
                }
            } else {
                quote! {}
            };

            quote! {
                if selector == <#right_marker as liquid_lang::FnSelector>::SELECTOR {
                    let data_ptr = &mut data.as_slice();
                    #[allow(unused_mut)]
                    let mut contract_id = <ContractId<#ty> as scale::Decode>::decode(data_ptr).map_err(|_| liquid_lang::DispatchError::InvalidParams)?;
                    #(#inputs)*

                    let result = contract_id.#right_name(#(#input_idents,)*);

                    #flush

                    if core::any::TypeId::of::<<#right_marker as liquid_lang::FnOutput>::Output>() != core::any::TypeId::of::<()>() {
                        liquid_lang::env::finish(&result);
                    }

                    return Ok(());
                }
            }
        });

        quote! {
            #(#fragments)*
        }
    }

    fn generate_contract_dispatch_fragment(
        &self,
        item_contract: &ItemContract,
    ) -> TokenStream2 {
        let contract_ident = &item_contract.ident;
        let contract_id = Self::generate_contract_id(item_contract);
        let contract_marker = quote! { Marker::<[(); #contract_id as usize]> };
        let input_idents = item_contract
            .fields
            .named
            .iter()
            .map(|field| &field.ident)
            .collect::<Vec<_>>();
        let inputs = item_contract
            .fields
            .named
            .iter()
            .map(|field| {
                let ident = &field.ident;
                let ty = &field.ty;
                quote! {
                    let #ident = <#ty as scale::Decode>::decode(data_ptr).map_err(|_| liquid_lang::DispatchError::InvalidParams)?;
                }
            })
            .collect::<Vec<_>>();
        let fetch_selector = liquid_primitives::Selector::from_le_bytes(
            Self::generate_contract_selector(item_contract, true),
        );
        quote! {
            if selector == <#contract_marker as liquid_lang::FnSelector>::SELECTOR {
                let data_ptr = &mut data.as_slice();
                #(#inputs)*;
                let contract_id = liquid_macro::sign! (#contract_ident => #(#input_idents,)*);
                <Storage as liquid_lang::storage::Flush>::flush(storage);
                liquid_lang::env::finish(&contract_id);

                return Ok(());
            }

            if selector == #fetch_selector {
                let data_ptr = &mut data.as_slice();
                let contract_id = <ContractId<#contract_ident> as scale::Decode>::decode(data_ptr)
                    .map_err(|_| liquid_lang::DispatchError::InvalidParams)?;

                let contract = <ContractId<#contract_ident> as liquid_lang::ContractVisitor>::fetch(&contract_id);
                liquid_lang::env::finish(&contract);
                return Ok(());
            }
        }
    }

    fn generate_dispatch(&self) -> TokenStream2 {
        let all_item_rights = &self.collaboration.all_item_rights;
        let item_contracts = &self.collaboration.contracts;
        let rights_fragments = all_item_rights
            .iter()
            .map(|item_rights| self.generate_rights_dispatch_fragment(item_rights));
        let contract_fragments = item_contracts
            .iter()
            .map(|item_contract| self.generate_contract_dispatch_fragment(item_contract));

        quote! {
            impl Storage {
                pub fn dispatch() -> liquid_lang::DispatchResult {
                    let storage = __liquid_acquire_storage_instance();
                    let call_data = liquid_lang::env::get_call_data(liquid_lang::env::CallMode::Call)
                        .map_err(|_| liquid_lang::DispatchError::CouldNotReadInput)?;
                    let selector = call_data.selector;
                    let data = call_data.data;

                    #(#rights_fragments)*
                    #(#contract_fragments)*

                    Err(liquid_lang::DispatchError::UnknownSelector)
                }
            }
        }
    }

    #[cfg(feature = "std")]
    fn generate_entry_point(&self) -> TokenStream2 {
        quote!()
    }

    #[cfg(not(feature = "std"))]
    fn generate_entry_point(&self) -> TokenStream2 {
        let contracts = &self.collaboration.contracts;
        let contract_names = contracts
            .iter()
            .map(|contract| {
                let ident_str = contract.ident.to_string();
                quote! { liquid_prelude::string::String::from(#ident_str) }
            })
            .collect::<Vec<_>>();
        let existent_errors = contracts.iter().map(|contract| {
            let ident = &contract.ident;
            let info = format!("contract `{}` already exists", ident);
            quote! { &liquid_prelude::string::String::from(#info) }
        });
        let register_errors = contracts.iter().map(|contract| {
            let ident = &contract.ident;
            let info = format!("fail to register contract `{}`", ident);
            quote! { &liquid_prelude::string::String::from(#info) }
        });
        let version = quote! { liquid_prelude::string::String::from("collaboration") };

        let addr_check =
            contract_names
                .iter()
                .zip(existent_errors)
                .map(|(name, error)| {
                    quote! {
                        let addr = Cns::get_contract_address(#name, #version);
                        if let Some(addr) = addr {
                            if addr != Address::empty() {
                                liquid_lang::env::revert(#error)
                            }
                        }
                    }
                });
        let register = contract_names.iter().zip(register_errors).enumerate().map(
            |(i, (name, error))| {
                quote! {
                    let ret = Cns::insert(#name, #version, self_addr.clone(), abis[#i as usize].clone());
                    let error = #error;
                    match ret {
                        Some(code) => if code == 0.into() {
                            liquid_lang::env::revert(error);
                        }
                        None => liquid_lang::env::revert(error),
                    }
                }
            },
        );
        quote! {
            #[no_mangle]
            fn hash_type() -> u32 {
                if cfg!(feature = "gm") {
                    1
                } else {
                    0
                }
            }

            #[no_mangle]
            fn deploy() {
                use liquid_prelude::string::ToString;
                use liquid_lang::precompiled::Cns;

                let self_addr = liquid_lang::env::get_address().to_string();

                let call_data = liquid_lang::env::get_call_data(liquid_lang::env::CallMode::Deploy);
                if let Ok(call_data) = call_data {
                    let data = call_data.data;
                    let abis = <Vec<String> as scale::Decode>::decode(&mut data.as_slice());
                    if abis.is_err() {
                        let ret_info = liquid_lang::DispatchRetInfo::from(liquid_lang::DispatchError::InvalidParams);
                        liquid_lang::env::revert(&ret_info.get_info_string());
                    }
                    let abis = abis.unwrap();

                    #(#addr_check)*
                    #(#register)*
                } else {
                    let ret_info = liquid_lang::DispatchRetInfo::from(liquid_lang::DispatchError::CouldNotReadInput);
                    liquid_lang::env::revert(&ret_info.get_info_string());
                }
            }

            #[no_mangle]
            fn main() {
                let ret_info = liquid_lang::DispatchRetInfo::from(Storage::dispatch());
                if !ret_info.is_success() {
                    liquid_lang::env::revert(&ret_info.get_info_string());
                }
            }
        }
    }
}

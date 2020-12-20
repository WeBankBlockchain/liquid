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
    collaboration::{codegen::utils, ir::*},
    traits::GenerateCode,
    utils::filter_non_liquid_attributes,
};
use derive_more::From;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[derive(From)]
pub struct Dispatch<'a> {
    collaboration: &'a Collaboration,
}

impl<'a> GenerateCode for Dispatch<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let marker = self.generate_right_marker();
        let traits = self.generate_right_traits();
        let dispatch = self.generate_dispatch();
        let entry_point = self.generate_entry_point();

        quote! {
            #[cfg(not(test))]
            const _: () = {
                #marker
                #traits
                #dispatch
                #entry_point
            };
        }
    }
}

impl<'a> Dispatch<'a> {
    fn generate_right_marker(&self) -> TokenStream2 {
        quote! {
            pub struct RightMarker<S> {
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

    fn generate_right_id(right: &Right) -> u32 {
        let selector = Self::generate_right_selector(right);
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
                rights.iter().map(|right| self.generate_right_trait(right))
            })
            .flatten();

        quote! {
            #(#traits)*
        }
    }

    fn generate_right_trait(&self, right: &Right) -> TokenStream2 {
        let right_id = Self::generate_right_id(right);
        let right_marker = quote! { RightMarker::<[(); #right_id as usize]> };
        let sig = &right.sig;

        let input_tys = utils::generate_input_tys(sig);
        let input_ty_checker = utils::generate_ty_checker(input_tys.as_slice());
        let right_input = quote! {
            impl liquid_lang::FnInput for #right_marker {
                type Input = (u32, (#(#input_tys,)*));
            }
        };

        let output = &sig.output;
        let (output_ty_checker, output_span) = match output {
            syn::ReturnType::Default => (quote! {()}, output.span()),
            syn::ReturnType::Type(_, ty) => {
                let return_ty = &*ty;
                (
                    quote! {
                        <#return_ty as liquid_lang::You_Should_Use_An_Valid_Return_Type>::T
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
            let input_checker = Ident::new(
                &format!("__LIQUID_RIGHT_INPUT_CHECKER_{}", right_id),
                right.span,
            );

            quote! {
                #[allow(non_camel_case_types)]
                struct #input_checker #input_ty_checker;

                impl liquid_lang::FnSelector for #right_marker {
                    const SELECTOR: liquid_primitives::Selector = [#(#selector,)*];
                }
            }
        };

        quote! {
            #right_input
            #right_output
            #right_selector
        }
    }

    fn generate_dispatch_fragment(&self, item_rights: &ItemRights) -> TokenStream2 {
        use heck::SnakeCase;

        let ident = &item_rights.ty;
        let field_name = Ident::new(
            &format!("__liquid_{}", ident.to_string().to_snake_case()),
            Span::call_site(),
        );
        let rights = &item_rights.rights;
        let fragments = rights.iter().map(|right| {
            let right_id = Self::generate_right_id(right);
            let right_marker = quote! { RightMarker::<[(); #right_id as usize]> };
            let sig = &right.sig;
            let right_name = &sig.ident;
            let input_idents = utils::generate_input_idents(&sig.inputs);
            let pat_idents = if input_idents.is_empty() {
                quote! { (id, _) }
            } else {
                quote! { (id, (#(#input_idents,)*)) }
            };

            let (ref_ty, get_ty) = if sig.is_mut() {
                (quote! { mut }, quote! { get_mut })
            } else {
                (quote! { }, quote! { get })
            };

            let execute = if sig.is_self_ref() {
                quote! {
                    let contract = &#ref_ty storage.#field_name.#get_ty(&id).unwrap().0;
                    contract.#right_name(#(#input_idents,)*)
                }
            } else {
                quote! {
                    let (contract, abolished) = storage.#field_name.get_mut(&id).unwrap();
                    *abolished = true;
                    let encoded = <#ident as scale::Encode>::encode(contract);
                    let cloned = <#ident as scale::Decode>::decode(&mut encoded.as_slice()).unwrap();
                    cloned.#right_name(#(#input_idents,)*)
                }
            };

            let flush = if !sig.is_self_ref() || sig.is_mut() {
                quote! {
                    <Storage as liquid_lang::storage::Flush>::flush(&mut storage);
                }
            } else {
                quote! {}
            };

            quote! {
                if selector == <#right_marker as liquid_lang::FnSelector>::SELECTOR {
                    let #pat_idents = <<#right_marker as liquid_lang::FnInput>::Input as scale::Decode>::decode(&mut data.as_slice())
                        .map_err(|_| liquid_lang::DispatchError::InvalidParams)?;

                    if !storage.#field_name.contains_key(&id) {
                        liquid_lang::env::revert(&"the contract is not exist".to_owned());
                    }

                    let abolished = storage.#field_name.get(&id).unwrap().1;
                    if abolished {
                        liquid_lang::env::revert(&"the contract had been abolished".to_owned());
                    }

                    let result = { #execute };

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

    fn generate_dispatch(&self) -> TokenStream2 {
        let all_item_rights = &self.collaboration.all_item_rights;
        let fragments = all_item_rights
            .iter()
            .map(|item_rights| self.generate_dispatch_fragment(item_rights));

        quote! {
            impl Storage {
                pub fn dispatch() -> liquid_lang::DispatchResult {
                    let mut storage = <Storage as liquid_lang::storage::New>::new();
                    let call_data = liquid_lang::env::get_call_data(liquid_lang::env::CallMode::Call)
                        .map_err(|_| liquid_lang::DispatchError::CouldNotReadInput)?;
                    let selector = call_data.selector;
                    let data = call_data.data;

                    #(
                        #fragments
                    )*

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
        let contract_idents = contracts
            .iter()
            .map(|contract| contract.ident.to_string())
            .collect::<Vec<_>>();
        let existent_errors = contract_idents
            .iter()
            .map(|ident| format!("contract `{}` already exists", ident));
        let register_errors = contract_idents
            .iter()
            .map(|ident| format!("fail to register contract `{}`", ident));
        let addr_check = contract_idents.iter().zip(existent_errors).map(|(ident, error)| {
            quote! {
                let addr = cns.get_contract_address(#ident.to_owned(), "0".to_owned());
                if let Some(addr) = addr {
                    if addr != address::empty() {
                        liquid_lang::env::revert(&#error.to_owned())
                    }
                }
            }
        });
        let register = contract_idents.iter().zip(register_errors).map(|(ident, error)| {
            quote! {
                let ret = cns.insert(#ident.to_owned(), "0".to_owned(), self_addr.clone(), "".to_owned());
                let error = #error.to_owned();
                match ret {
                    Some(code) => if code == 0.into() {
                        liquid_lang::env::revert(&error);
                    }
                    None => liquid_lang::env::revert(&error),
                }
            }
        });
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

                let mut storage = <Storage as liquid_lang::storage::New>::new();
                let self_addr = liquid_lang::env::get_address().to_string();
                let cns = liquid_lang::precompiled::CNS::new();

                #(#addr_check)*
                #(#register)*
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

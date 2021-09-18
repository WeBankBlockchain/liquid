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
    common,
    contract::ir::{Contract, Function, FunctionKind},
};

use derive_more::From;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[derive(From)]
pub struct Dispatch<'a> {
    contract: &'a Contract,
}

impl<'a> common::GenerateCode for Dispatch<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let marker = self.generate_external_fn_marker();
        let traits = self.generate_external_fn_traits();
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
    fn generate_external_fn_marker(&self) -> TokenStream2 {
        quote! {
            pub struct FnMarker<S> {
                marker: core::marker::PhantomData<fn() -> S>,
            }
        }
    }

    fn generate_external_fn_traits(&self) -> TokenStream2 {
        let traits = self
            .contract
            .functions
            .iter()
            .filter(|func| matches!(&func.kind, FunctionKind::External(..)))
            .map(|func| self.generate_external_fn_trait(func));

        quote! {
            #(#traits)*
        }
    }

    fn generate_external_fn_trait(&self, func: &Function) -> TokenStream2 {
        let fn_id = match &func.kind {
            FunctionKind::External(fn_id) => fn_id,
            _ => unreachable!(),
        };

        let fn_marker = quote! { FnMarker::<[(); #fn_id]> };
        let sig = &func.sig;

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
        let fn_output = quote_spanned! { output_span =>
            impl liquid_lang::FnOutput for #fn_marker {
                type Output = #output_ty_checker;
            }
        };

        let fn_name = sig.ident.to_string();
        let fn_name_bytes = fn_name.as_bytes();

        let selector = {
            let input_tys = common::generate_input_tys(sig);
            let input_ty_checker = common::generate_ty_checker(input_tys.as_slice());
            let input_ty_checker_ident = Ident::new(
                &format!("__LIQUID_EXTERNAL_INPUT_CHECKER_{}", fn_id),
                func.span(),
            );

            quote! {
                #[allow(non_camel_case_types)]
                struct #input_ty_checker_ident #input_ty_checker;

                impl liquid_lang::FnSelector for #fn_marker {
                    const SELECTOR: liquid_primitives::Selector = {
                        let hash = liquid_primitives::hash::hash(&[#(#fn_name_bytes),*]);
                        liquid_primitives::Selector::from_le_bytes(
                            [hash[0], hash[1], hash[2], hash[3]]
                        )
                    };
                }
            }
        };

        let is_mut = sig.is_mut();
        let mutability = quote! {
            impl liquid_lang::FnMutability for #fn_marker {
                const IS_MUT: bool = #is_mut;
            }
        };

        quote! {
            #fn_output
            #selector
            #mutability
        }
    }

    fn generate_dispatch_fragment(&self, func: &Function) -> TokenStream2 {
        let fn_id = match &func.kind {
            FunctionKind::External(fn_id) => fn_id,
            _ => return quote! {},
        };
        let namespace = quote! { FnMarker<[(); #fn_id]> };

        let sig = &func.sig;
        let fn_name = &sig.ident;
        let input_idents = common::generate_input_idents(sig);
        let input_tys = common::generate_input_tys(sig);
        let inputs = input_idents.iter().zip(input_tys.iter()).map(|(ident, ty)| {
            quote! {
                let #ident = <#ty as scale::Decode>::decode(data_ptr).map_err(|_| liquid_lang::DispatchError::InvalidParams)?;
            }
        });

        quote! {
            if selector == <#namespace as liquid_lang::FnSelector>::SELECTOR {
                let data_ptr = &mut data.as_slice();
                #(#inputs)*

                #[allow(deprecated)]
                let result = storage.#fn_name(#(#input_idents,)*);

                if <#namespace as liquid_lang::FnMutability>::IS_MUT {
                    <Storage as liquid_lang::storage::Flush>::flush(&mut storage);
                }

                if core::any::TypeId::of::<<#namespace as liquid_lang::FnOutput>::Output>() != core::any::TypeId::of::<()>() {
                    liquid_lang::env::finish(&result);
                }

                return Ok(());
            }
        }
    }

    fn generate_dispatch(&self) -> TokenStream2 {
        let fragments = self
            .contract
            .functions
            .iter()
            .map(|func| self.generate_dispatch_fragment(func));

        let constr = &self.contract.constructor;
        let constr_input_tys = common::generate_input_tys(&constr.sig);
        let constr_input_ty_checker =
            common::generate_ty_checker(constr_input_tys.as_slice());

        quote! {
            #[allow(non_camel_case_types)]
            struct __LIQUID_CONSTRUCTOR_INPUT_TY_CHECKER #constr_input_ty_checker;

            impl Storage {
                pub fn __liquid_dispatch() -> liquid_lang::DispatchResult {
                    let mut storage = <Storage as liquid_lang::storage::New>::new();
                    let call_data = liquid_lang::env::get_call_data(liquid_lang::env::CallMode::Call)
                        .map_err(|_| liquid_lang::DispatchError::CouldNotReadInput)?;
                    let selector = call_data.selector;
                    let data = call_data.data;

                    #(#fragments)*

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
        let constr = &self.contract.constructor;
        let constr_sig = &constr.sig;
        let ident = &constr_sig.ident;
        let asset_registers: Vec<TokenStream2> = self
            .contract
            .assets
            .iter()
            .map(|asset| {
                let ident = asset.ident.clone();
                let err_message = format!("register {} failed", ident.to_string());
                quote! {
                    require(#ident::register(),#err_message);
                }
            })
            .collect();

        let constr_input_tys = common::generate_input_tys(&constr_sig);
        let constr_input_idents = common::generate_input_idents(&constr_sig);
        let constr_inputs = constr_input_idents.iter().zip(constr_input_tys.iter()).map(|(ident, ty)| {
            quote! {
                let #ident = <#ty as scale::Decode>::decode(data_ptr).unwrap_or_else(|_| {
                    liquid_lang::env::revert(&String::from("invalid params"));
                    unreachable!();
                });
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
                let mut storage = <Storage as liquid_lang::storage::New>::new();
                let call_data = liquid_lang::env::get_call_data(liquid_lang::env::CallMode::Deploy);
                if let Ok(call_data) = call_data {
                    let data = call_data.data;
                    let data_ptr = &mut data.as_slice();
                    #(#constr_inputs)*
                    storage.#ident(#(#constr_input_idents,)*);
                    <Storage as liquid_lang::storage::Flush>::flush(&mut storage);
                } else {
                    liquid_lang::env::revert(&String::from("could not read input"));
                }
                #(#asset_registers)*
            }

            #[no_mangle]
            fn main() {
                let ret_info = liquid_lang::DispatchRetInfo::from(Storage::__liquid_dispatch());
                if !ret_info.is_success() {
                    liquid_lang::env::revert(&ret_info.get_info_string());
                }
            }
        }
    }
}

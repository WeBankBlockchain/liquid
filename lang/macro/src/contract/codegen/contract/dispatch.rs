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
    contract::{
        codegen::utils,
        ir::{Contract, Function, FunctionKind},
    },
};

use derive_more::From;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[derive(From)]
pub struct Dispatch<'a> {
    contract: &'a Contract,
}

impl<'a> GenerateCode for Dispatch<'a> {
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
            FunctionKind::External(fn_id, _) => fn_id,
            _ => unreachable!(),
        };

        let fn_marker = quote! { FnMarker::<[(); #fn_id]> };
        let sig = &func.sig;

        let input_tys = utils::generate_input_tys(sig);
        let input_ty_checker = utils::generate_ty_checker(input_tys.as_slice());
        let fn_input = quote! {
            impl liquid_lang::FnInput for #fn_marker {
                type Input = (#(#input_tys,)*);
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
        let fn_output = quote_spanned! { output_span =>
            impl liquid_lang::FnOutput for #fn_marker {
                type Output = #output_ty_checker;
            }
        };

        let fn_name = sig.ident.to_string();
        let fn_name_bytes = fn_name.as_bytes();
        let fn_name_len = fn_name.len();

        let selector = if cfg!(feature = "solidity-compatible") {
            quote! {
                impl liquid_lang::FnSelector for #fn_marker {
                    const SELECTOR: liquid_primitives::Selector = {
                        const SIG_LEN: usize =
                            liquid_ty_mapping::len::<#input_ty_checker>()
                            + #fn_name_len
                            + 2;
                        const SIG: [u8; SIG_LEN] = liquid_ty_mapping::composite::<(#(#input_tys,)*), SIG_LEN>(&[#(#fn_name_bytes),*]);
                        let hash = liquid_primitives::hash::hash(&SIG);
                        [hash[0], hash[1], hash[2], hash[3]]
                    };
                }
            }
        } else {
            let input_checker = Ident::new(
                &format!("__LIQUID_EXTERNAL_INPUT_CHECKER_{}", fn_id),
                func.span(),
            );

            quote! {
                #[allow(non_camel_case_types)]
                struct #input_checker #input_ty_checker;

                impl liquid_lang::FnSelector for #fn_marker {
                    const SELECTOR: liquid_primitives::Selector = {
                        let hash = liquid_primitives::hash::hash(&[#(#fn_name_bytes),*]);
                        [hash[0], hash[1], hash[2], hash[3]]
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
            #fn_input
            #fn_output
            #selector
            #mutability
        }
    }

    fn generate_dispatch_fragment(
        &self,
        func: &Function,
        is_getter: bool,
    ) -> TokenStream2 {
        let fn_id = match &func.kind {
            FunctionKind::External(fn_id, _) => fn_id,
            _ => return quote! {},
        };
        let namespace = quote! { FnMarker<[(); #fn_id]> };

        let sig = &func.sig;
        let fn_name = &sig.ident;
        let input_idents = utils::generate_input_idents(&sig.inputs);
        let pat_idents = if input_idents.is_empty() {
            quote! { _ }
        } else {
            quote! { (#(#input_idents,)*) }
        };
        let attr = if is_getter {
            quote! { #[allow(deprecated)] }
        } else {
            quote! {}
        };

        quote! {
            if selector == <#namespace as liquid_lang::FnSelector>::SELECTOR {
                #[cfg(feature = "solidity-compatible")]
                let #pat_idents = <<#namespace as liquid_lang::FnInput>::Input as liquid_abi_codec::Decode>::decode(&mut data.as_slice())
                    .map_err(|_| liquid_lang::DispatchError::InvalidParams)?;
                #[cfg(not(feature = "solidity-compatible"))]
                let #pat_idents = <<#namespace as liquid_lang::FnInput>::Input as scale::Decode>::decode(&mut data.as_slice())
                    .map_err(|_| liquid_lang::DispatchError::InvalidParams)?;

                #attr
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

    fn generate_constr_input_ty_checker(&self) -> TokenStream2 {
        let constr = &self.contract.constructor;
        let sig = &constr.sig;
        let input_tys = utils::generate_input_tys(sig);
        let guards = input_tys.iter().map(|ty| {
            quote_spanned! {ty.span() => <#ty as liquid_lang::You_Should_Use_An_Valid_Input_Type>::T}
        });
        quote! {
            #[allow(non_camel_case_types)]
            struct __LIQUID_CONSTRUCTOR_INPUT_TY_CHECKER(#(#guards,)*);
        }
    }

    fn generate_dispatch(&self) -> TokenStream2 {
        let fragments = self.contract.functions.iter().map(|func| {
            let is_getter = matches!(func.kind, FunctionKind::External(_, true));
            self.generate_dispatch_fragment(func, is_getter)
        });

        let constr_input_ty_checker = self.generate_constr_input_ty_checker();

        quote! {
            #constr_input_ty_checker

            impl Storage {
                pub fn dispatch() -> liquid_lang::DispatchResult {
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
        let sig = &constr.sig;
        let input_tys = utils::generate_input_tys(sig);
        let ident = &sig.ident;
        let input_idents = utils::generate_input_idents(&sig.inputs);
        let asset_idents: Vec<Ident> = self
            .contract
            .assets
            .iter()
            .map(|asset| asset.ident.clone())
            .collect();
        let pat_idents = if input_idents.is_empty() {
            quote! { _ }
        } else {
            quote! { (#(#input_idents,)*) }
        };

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
                let result = liquid_lang::env::get_call_data(liquid_lang::env::CallMode::Deploy);
                if let Ok(call_data) = result {
                    let data = call_data.data;
                    #[cfg(feature = "solidity-compatible")]
                    let result = <(#(#input_tys,)*) as liquid_abi_codec::Decode>::decode(&mut data.as_slice());
                    #[cfg(not(feature = "solidity-compatible"))]
                    let result = <(#(#input_tys,)*) as scale::Decode>::decode(&mut data.as_slice());

                    if let Ok(data) = result {
                        let #pat_idents = data;
                        storage.#ident(#(#input_idents,)*);
                        <Storage as liquid_lang::storage::Flush>::flush(&mut storage);
                    } else {
                        liquid_lang::env::revert(&String::from("invalid params"));
                    }
                } else {
                    liquid_lang::env::revert(&String::from("could not read input"));
                }
                #(#asset_idents::register();)*
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

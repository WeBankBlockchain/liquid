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
    codegen::GenerateCode,
    ir::{Contract, FnArg, Function, FunctionKind, Signature},
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{punctuated::Punctuated, spanned::Spanned, Token};

pub struct Dispatch<'a> {
    contract: &'a Contract,
}

impl<'a> From<&'a Contract> for Dispatch<'a> {
    fn from(contract: &'a Contract) -> Self {
        Self { contract }
    }
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

fn generate_input_tys<'a>(sig: &'a Signature) -> Vec<&'a syn::Type> {
    sig.inputs
        .iter()
        .skip(1)
        .map(|arg| match arg {
            FnArg::Typed(ident_type) => &ident_type.ty,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>()
}

fn generate_input_idents<'a>(
    args: &'a Punctuated<FnArg, Token![,]>,
) -> (Vec<&'a proc_macro2::Ident>, TokenStream2) {
    let input_idents = args
        .iter()
        .skip(1)
        .filter_map(|arg| match arg {
            FnArg::Typed(ident_type) => Some(&ident_type.ident),
            _ => None,
        })
        .collect::<Vec<_>>();

    let pat_idents = if input_idents.is_empty() {
        quote! { _ }
    } else {
        quote! { (#(#input_idents,)*) }
    };

    (input_idents, pat_idents)
}

fn generate_input_ty_checker(tys: &[&syn::Type]) -> TokenStream2 {
    let guards = tys.iter().map(|ty| {
        quote! {
            <#ty as liquid_lang::You_Should_Use_An_Valid_Input_Type>::T
        }
    });

    quote! { (#(#guards,)*) }
}

impl<'a> Dispatch<'a> {
    fn generate_external_fn_marker(&self) -> TokenStream2 {
        quote! {
            pub struct ExternalMarker<S> {
                marker: core::marker::PhantomData<fn() -> S>,
            }

            pub struct DispatchHelper<S, T> {
                marker_s: core::marker::PhantomData<fn() -> S>,
                marker_t: core::marker::PhantomData<fn() -> T>,
            }
        }
    }

    fn generate_external_fn_traits(&self) -> TokenStream2 {
        let (traits, external_markers): (Vec<_>, Vec<_>) = self
            .contract
            .functions
            .iter()
            .filter(|func| matches!(&func.kind, FunctionKind::External(_)))
            .map(|func| self.generate_external_fn_trait(func))
            .unzip();
        let selectors = external_markers.iter().map(|marker| {
            quote!(
                <#marker as liquid_lang::FnSelectors>::KECCAK256_SELECTOR,
                <#marker as liquid_lang::FnSelectors>::SM3_SELECTOR)
        });
        let selector_conflict_detector = quote! {
            const _: () = liquid_lang::selector_conflict_detect::detect(&[#(#selectors,)*]);
        };

        quote! {
            #(#traits)*

            #selector_conflict_detector
        }
    }

    fn generate_external_fn_trait(
        &self,
        func: &Function,
    ) -> (TokenStream2, TokenStream2) {
        let fn_id = match &func.kind {
            FunctionKind::External(fn_id) => fn_id,
            _ => unreachable!(),
        };

        let span = func.span();
        let external_marker = quote! { ExternalMarker<[(); #fn_id]> };
        let sig = &func.sig;

        let input_tys = generate_input_tys(sig);
        let input_ty_checker = generate_input_ty_checker(input_tys.as_slice());
        let fn_input = quote_spanned! { sig.inputs.span() =>
            impl liquid_lang::FnInput for #external_marker  {
                type Input = #input_ty_checker;
            }
        };

        let output = &sig.output;
        let output_ty_checker = match output {
            syn::ReturnType::Default => quote_spanned! { output.span() => ()},
            syn::ReturnType::Type(_, ty) => {
                let return_ty = &*ty;
                quote_spanned! { output.span() =>
                    <#return_ty as liquid_lang::You_Should_Use_An_Valid_Return_Type>::T
                }
            }
        };
        let fn_output = quote_spanned! { output.span() =>
            impl liquid_lang::FnOutput for #external_marker {
                type Output = #output_ty_checker;
            }
        };

        let mut selectors = quote_spanned! { span =>
            impl liquid_ty_mapping::SolTypeName for DispatchHelper<#external_marker, ()> {
                const NAME: &'static [u8] = <() as liquid_ty_mapping::SolTypeName>::NAME;
            }
            impl liquid_ty_mapping::SolTypeNameLen for DispatchHelper<#external_marker, ()> {
                const LEN: usize = <() as liquid_ty_mapping::SolTypeNameLen>::LEN;
            }
        };
        for i in 1..=input_tys.len() {
            let tys = &input_tys[..i];
            let first_tys = &tys[0..i - 1];
            let rest_ty = &tys[i - 1];
            if i > 1 {
                selectors.extend(quote_spanned! { span =>
                    impl liquid_ty_mapping::SolTypeName for DispatchHelper<#external_marker, (#(#tys,)*)> {
                        const NAME: &'static [u8] = {
                            const LEN: usize =
                                <(#(#first_tys,)*) as liquid_ty_mapping::SolTypeNameLen<_>>::LEN
                                + <#rest_ty as liquid_ty_mapping::SolTypeNameLen<_>>::LEN
                                + 1;
                            &liquid_ty_mapping::concat::<DispatchHelper<#external_marker, (#(#first_tys,)*)>, #rest_ty, (), _, LEN>(true)
                        };
                    }
                });
            } else {
                selectors.extend(quote_spanned! { span =>
                    impl liquid_ty_mapping::SolTypeName for DispatchHelper<#external_marker, (#rest_ty,)> {
                        const NAME: &'static [u8] = <#rest_ty as liquid_ty_mapping::SolTypeName<_>>::NAME;
                    }
                });
            }
        }

        let fn_name = sig.ident.to_string();
        let fn_name_bytes = fn_name.as_bytes();
        let fn_name_len = fn_name_bytes.len();
        let composite_sig = quote! {
            const SIG_LEN: usize =
                <(#(#input_tys,)*) as liquid_ty_mapping::SolTypeNameLen<_>>::LEN + #fn_name_len
                + 2;
            const SIG: [u8; SIG_LEN] =
                liquid_ty_mapping::composite::<SIG_LEN>(
                    &[#(#fn_name_bytes),*],
                    <DispatchHelper<#external_marker, (#(#input_tys,)*)> as liquid_ty_mapping::SolTypeName<_>>::NAME);
        };
        selectors.extend(quote_spanned! { span =>
            impl liquid_lang::FnSelectors for #external_marker {
                const KECCAK256_SELECTOR: liquid_primitives::Selector = {
                    #composite_sig
                    liquid_primitives::hash::keccak::keccak256(&SIG)
                };
                const SM3_SELECTOR: liquid_primitives::Selector = {
                    #composite_sig
                    liquid_primitives::hash::sm3::sm3(&SIG)
                };
            }
        });

        let is_mut = sig.is_mut();
        let mutability = quote_spanned! { span =>
            impl liquid_lang::FnMutability for #external_marker {
                const IS_MUT: bool = #is_mut;
            }
        };

        (
            quote_spanned! { span =>
                #fn_input
                #fn_output
                #selectors
                #mutability
                impl liquid_lang::ExternalFn for #external_marker {}
            },
            external_marker,
        )
    }

    fn generate_dispatch_fragment(
        &self,
        func: &Function,
        is_getter: bool,
    ) -> TokenStream2 {
        let fn_id = match &func.kind {
            FunctionKind::External(fn_id) => fn_id,
            _ => return quote! {},
        };
        let namespace = quote! { ExternalMarker<[(); #fn_id]> };

        let sig = &func.sig;
        let fn_name = &sig.ident;
        let (input_idents, pat_idents) = generate_input_idents(&sig.inputs);
        let attr = if is_getter {
            quote! { #[allow(deprecated)] }
        } else {
            quote! {}
        };

        quote! {
            if selector == <#namespace as liquid_lang::FnSelectors>::KECCAK256_SELECTOR ||
                selector == <#namespace as liquid_lang::FnSelectors>::SM3_SELECTOR {
                let #pat_idents = <<#namespace as liquid_lang::FnInput>::Input as liquid_abi_codec::Decode>::decode(&mut data.as_slice())
                    .map_err(|_| liquid_lang::DispatchError::InvalidParams)?;
                #attr
                let result = storage.#fn_name(#(#input_idents,)*);

                if <#namespace as liquid_lang::FnMutability>::IS_MUT {
                    <Storage as liquid_core::storage::Flush>::flush(&mut storage);
                }

                if core::any::TypeId::of::<<#namespace as liquid_lang::FnOutput>::Output>() != core::any::TypeId::of::<()>() {
                    liquid_core::env::finish(&result);
                }

                return Ok(());
            }
        }
    }

    fn generate_constr_input_ty_checker(&self) -> TokenStream2 {
        let constr = &self.contract.constructor;
        let sig = &constr.sig;
        let inputs = &sig.inputs;
        let input_tys = generate_input_tys(sig);
        let marker = quote! { ExternalMarker<[(); 0]> };
        let input_ty_checker = generate_input_ty_checker(input_tys.as_slice());
        quote_spanned! { inputs.span() =>
            impl liquid_lang::FnInput for #marker  {
                type Input = #input_ty_checker;
            }
        }
    }

    fn generate_dispatch(&self) -> TokenStream2 {
        let fragments = self.contract.functions.iter().enumerate().map(|(i, func)| {
            let is_getter = self.contract.functions.len() - i
                <= self.contract.storage.public_fields.len();
            self.generate_dispatch_fragment(func, is_getter)
        });

        let constr_input_ty_checker = self.generate_constr_input_ty_checker();

        quote! {
            #constr_input_ty_checker

            impl Storage {
                pub fn dispatch() -> liquid_lang::DispatchResult {
                    let mut storage = <Storage as liquid_core::storage::New>::new();
                    let call_data = liquid_core::env::get_call_data(liquid_core::env::CallMode::Call)
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
        let constr = &self.contract.constructor;
        let sig = &constr.sig;
        let input_tys = generate_input_tys(sig);
        let ident = &sig.ident;
        let (input_idents, pat_idents) = generate_input_idents(&sig.inputs);

        quote! {
            #[no_mangle]
            fn deploy() {
                let mut storage = <Storage as liquid_core::storage::New>::new();
                let call_data = liquid_core::env::get_call_data(liquid_core::env::CallMode::Deploy)
                    .map_err(|_| liquid_lang::DispatchError::CouldNotReadInput)?;
                let data = call_data.data;
                let #pat_idents = <(#(#input_tys,)*) as liquid_abi_codec::Decode>::decode(&mut data.as_slice())
                    .map_err(|_| liquid_lang::DispatchError::InvalidParams)?;
                storage.#ident(#(#input_idents,)*);
                <Storage as liquid_core::storage::Flush>::flush(&mut storage);
                return Ok(());
            }

            #[no_mangle]
            fn main() {
                let ret_info = liquid_lang::DispatchRetInfo::from(Storage::dispatch());
                if !ret_info.is_success() {
                    liquid_core::env::revert(&ret_info.get_info_string());
                }
            }
        }
    }
}

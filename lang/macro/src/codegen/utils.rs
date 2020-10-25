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

use crate::ir::{FnArg, Signature};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::Type;

pub fn generate_ty_checker(tys: &[&Type]) -> TokenStream2 {
    let guards = tys.iter().map(|ty| {
        quote! {
            <#ty as liquid_lang::You_Should_Use_An_Valid_Input_Type>::T
        }
    });

    quote! { (#(#guards,)*) }
}

pub fn generate_input_tys(sig: &Signature, skip_first: bool) -> Vec<&syn::Type> {
    sig.inputs
        .iter()
        .skip(if skip_first { 1 } else { 0 })
        .map(|arg| match arg {
            FnArg::Typed(ident_type) => &ident_type.ty,
            _ => unreachable!(),
        })
        .collect::<Vec<_>>()
}

pub fn generate_ty_mapping(
    fn_id: usize,
    fn_name: &Ident,
    input_tys: &[&syn::Type],
) -> TokenStream2 {
    let fn_marker = quote! { FnMarker<[(); #fn_id]> };
    let mut impls = quote! {
        impl liquid_ty_mapping::SolTypeName for TyMappingHelper<#fn_marker, ()> {
            const NAME: &'static [u8] = <() as liquid_ty_mapping::SolTypeName>::NAME;
        }

        impl liquid_ty_mapping::SolTypeNameLen for TyMappingHelper<#fn_marker, ()> {
            const LEN: usize = <() as liquid_ty_mapping::SolTypeNameLen>::LEN;
        }
    };

    for i in 1..=input_tys.len() {
        let tys = &input_tys[..i];
        let first_tys = &tys[0..i - 1];
        let rest_ty = &tys[i - 1];
        if i > 1 {
            impls.extend(quote! {
                impl liquid_ty_mapping::SolTypeName for TyMappingHelper<#fn_marker, (#(#tys,)*)> {
                    const NAME: &'static [u8] = {
                        const LEN: usize =
                            <(#(#first_tys,)*) as liquid_ty_mapping::SolTypeNameLen<_>>::LEN
                            + <#rest_ty as liquid_ty_mapping::SolTypeNameLen<_>>::LEN
                            + 1;
                        &liquid_ty_mapping::concat::<TyMappingHelper<#fn_marker, (#(#first_tys,)*)>, #rest_ty, (), _, LEN>(true)
                    };
                }
            });
        } else {
            impls.extend(quote! {
                impl liquid_ty_mapping::SolTypeName for TyMappingHelper<#fn_marker, (#rest_ty,)> {
                    const NAME: &'static [u8] = <#rest_ty as liquid_ty_mapping::SolTypeName<_>>::NAME;
                }
            });
        }
    }

    let fn_name = fn_name.to_string();
    let fn_name_bytes = fn_name.as_bytes();
    let fn_name_len = fn_name.len();
    let composite_sig = quote! {
        const SIG_LEN: usize =
            <(#(#input_tys,)*) as liquid_ty_mapping::SolTypeNameLen<_>>::LEN + #fn_name_len
            + 2;
        const SIG: [u8; SIG_LEN] =
            liquid_ty_mapping::composite::<SIG_LEN>(
                &[#(#fn_name_bytes),*],
                <TyMappingHelper<#fn_marker, (#(#input_tys,)*)> as liquid_ty_mapping::SolTypeName<_>>::NAME);
    };

    impls.extend(quote! {
        impl liquid_lang::FnSelector for #fn_marker {
            const SELECTOR: liquid_primitives::Selector = {
                #composite_sig
                let hash = liquid_primitives::hash::hash(&SIG);
                [hash[0], hash[1], hash[2], hash[3]]
            };
        }
    });
    impls
}

pub fn generate_primitive_types() -> TokenStream2 {
    quote! {
        type Address = liquid_primitives::types::Address;
        type Timestamp = liquid_primitives::types::Timestamp;
        type BlockNumber = liquid_primitives::types::BlockNumber;
        type Hash = liquid_primitives::types::Hash;
        type u256 = liquid_primitives::types::u256;
        type i256 = liquid_primitives::types::i256;

        type Vec<T> = liquid_prelude::vec::Vec<T>;
        type String = liquid_prelude::string::String;
    }
}

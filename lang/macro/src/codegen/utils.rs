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
use proc_macro2::TokenStream as TokenStream2;
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

pub fn generate_primitive_types() -> TokenStream2 {
    quote! {
        pub use liquid_primitives::types::*;
        pub use liquid_prelude::string::String;
        pub type Vec<T> = liquid_prelude::vec::Vec<T>;
    }
}

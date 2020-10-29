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
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
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
    let mut fixed_size_bytes = quote! {};
    for i in 1..=32 {
        let old_ident = Ident::new(&format!("Bytes{}", i), Span::call_site());
        let new_ident = Ident::new(&format!("bytes{}", i), Span::call_site());
        fixed_size_bytes.extend(quote! {
            #[allow(non_camel_case_types)]
            pub type #new_ident = liquid_primitives::types::#old_ident;
        });
    }

    quote! {
        #[allow(non_camel_case_types)]
        pub type address = liquid_primitives::types::Address;
        #[allow(non_camel_case_types)]
        pub type bytes = liquid_primitives::types::Bytes;
        #[allow(non_camel_case_types)]
        pub type byte = liquid_primitives::types::Byte;

        #fixed_size_bytes

        pub use liquid_primitives::types::u256;
        pub use liquid_primitives::types::i256;
        pub use liquid_prelude::string::String;
        pub type Vec<T> = liquid_prelude::vec::Vec<T>;
    }
}

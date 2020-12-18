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

use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;
use std::string::ToString;
use syn::Error;

pub fn check_idents(input: TokenStream2) -> Result<(), Error> {
    for token in input.into_iter() {
        match token {
            TokenTree::Ident(ident) => {
                if ident.to_string().starts_with("__liquid")
                    || ident.to_string().starts_with("__LIQUID")
                {
                    return Err(format_err_span!(
                        ident.span(),
                        "identifiers starting with `__liquid` or `__LIQUID` are \
                         forbidden"
                    ));
                }
            }
            TokenTree::Group(group) => {
                check_idents(group.stream())?;
            }
            _ => (),
        }
    }

    Ok(())
}

pub fn calculate_fn_id<T>(ident: &T) -> usize
where
    T: ToString,
{
    let ident_hash = liquid_primitives::hash::hash(ident.to_string().as_bytes());
    // In declaration of array where the `fn_id` is used, the length of array must be an `usize` integer.
    // **DO NOT** attempt to change the return type of this function.
    u32::from_be_bytes([ident_hash[0], ident_hash[1], ident_hash[2], ident_hash[3]])
        as usize
}

pub fn is_liquid_attribute(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("liquid")
}

pub fn filter_non_liquid_attributes<'a, I>(
    attrs: I,
) -> impl Iterator<Item = &'a syn::Attribute>
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    attrs.into_iter().filter(|attr| !is_liquid_attribute(attr))
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

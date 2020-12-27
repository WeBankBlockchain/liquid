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

use crate::derive::utils;
use liquid_prelude::vec::Vec;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{self, parse::Result, spanned::Spanned, DeriveInput, Type};

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
}

fn generate_encode_shadow_struct(
    ident: &Ident,
    field_names: &[&Ident],
    field_tys: &[&Type],
) -> TokenStream2 {
    let fields = field_names.iter().enumerate().map(|(i, field_name)| {
        let field_ty = field_tys[i];
        quote!(#field_name: &'a #field_ty,)
    });
    let assigns = field_names
        .iter()
        .map(|field_name| quote!(#field_name: &origin.#field_name,));

    let field_checkers = field_tys.iter().enumerate().map(|(i, ty)| {
        let field_checker = Ident::new(
            &format!("__LIQUID_STATE_FIELD_CHECKER_{}", i),
            Span::call_site(),
        );

        quote_spanned! { ty.span() =>
            #[allow(non_camel_case_types)]
            struct #field_checker(<#ty as liquid_lang::You_Should_Use_An_Valid_State_Type>::T);
        }
    });

    quote! {
        #(#field_checkers)*

        #[derive(scale::Encode)]
        struct EncodeShadow<'a> {
            #(#fields)*
        }

        impl<'a> From<&'a #ident> for EncodeShadow<'a> {
            fn from(origin: &'a #ident) -> Self {
                Self {
                    #(#assigns)*
                }
            }
        }

        impl scale::Encode for #ident {
            fn encode(&self) -> __std::Vec<u8> {
                use scale::Encode;

                let encode_shadow: EncodeShadow::<'_> = self.into();
                encode_shadow.encode()
            }
        }
    }
}

fn generate_decode_shadow_struct(
    ident: &Ident,
    field_names: &[&Ident],
    field_tys: &[&Type],
) -> TokenStream2 {
    let fields = field_names.iter().enumerate().map(|(i, field_name)| {
        let field_ty = field_tys[i];
        quote!(#field_name: #field_ty,)
    });
    let assigns = field_names
        .iter()
        .map(|field_name| quote!(#field_name: origin.#field_name,));

    quote! {
        #[derive(scale::Decode)]
        struct DecodeShadow {
            #(#fields)*
        }

        impl scale::Decode for #ident {
            fn decode<I: scale::Input>(value: &mut I) -> ::core::result::Result<Self, scale::Error> {
                use scale::Decode;

                let origin = <DecodeShadow as scale::Decode>::decode(value)?;
                Ok(Self {
                    #(#assigns)*
                })
            }
        }
    }
}

fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
    let ast: DeriveInput = syn::parse2(input)?;
    let (field_names, field_tys, _): (Vec<_>, Vec<_>, Span) =
        utils::struct_syntax_check(&ast)?;
    let ident = &ast.ident;

    let encode_shadow_struct =
        generate_encode_shadow_struct(ident, &field_names, &field_tys);
    let decode_shadow_struct =
        generate_decode_shadow_struct(ident, &field_names, &field_tys);

    Ok(quote! {
        #encode_shadow_struct
        #decode_shadow_struct

        impl liquid_lang::You_Should_Use_An_Valid_State_Type for #ident {}
    })
}

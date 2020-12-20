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
use liquid_prelude::{string::ToString, vec::Vec};
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{self, parse::Result, spanned::Spanned, DeriveInput};

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
}

fn generate_abi_gen(
    field_names: &[&Ident],
    field_tys: &[&syn::Type],
    ident: &Ident,
) -> TokenStream2 {
    debug_assert!(field_names.len() == field_tys.len());

    let field_param_abis = field_names
        .iter()
        .map(|name| name.to_string())
        .zip(field_tys.iter())
        .map(|(field_name, field_ty)| {
            quote! {
                <#field_ty as GenerateParamABI>::generate_param_abi(#field_name.to_owned())
            }
        });

    quote! {
        #[cfg(feature = "liquid-abi-gen")]
        impl liquid_abi_gen::GenerateParamABI for #ident {
            fn generate_ty_name() -> liquid_prelude::string::String {
                String::from("tuple")
            }

            fn generate_param_abi(name: String) -> liquid_abi_gen::ParamABI {
                let mut components = __std::Vec::new();
                #(components.push(#field_param_abis);)*
                liquid_abi_gen::CompositeABI {
                    trivial: liquid_abi_gen::TrivialABI::new(Self::generate_ty_name(), name),
                    components,
                }
            }
        }

        #[cfg(feature = "liquid-abi-gen")]
        impl liquid_abi_gen::GenerateOutputs for #ident {
            fn generate_outputs(builder: &mut liquid_abi_gen::ExternalFnABIBuilder) {
                let param_abi = <Self as GenerateParamABI>::generate_param_abi("".into());
                builder.output(param_abi);
            }
        }
    }
}

fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
    let ast: DeriveInput = syn::parse2(input)?;
    let (field_names, field_tys, fields_span): (Vec<_>, Vec<_>, Span) =
        utils::struct_syntax_check(&ast)?;
    let ident = &ast.ident;
    let fields_count = field_names.len();

    let mut decode_tokens = Vec::new();
    let mut field_checkers = Vec::new();
    for i in 0..fields_count {
        let name = &field_names[i];
        let ty = &field_tys[i];

        decode_tokens.push(quote!{
            #name: {
                let decode_result = <#ty as liquid_abi_codec::MediateDecode>::decode(&tail, new_offset)?;
                new_offset = decode_result.new_offset;
                decode_result.value
            }
        });

        let field_checker = Ident::new(
            &format!("__LIQUID_INOUT_FIELD_CHECKER_{}", i),
            Span::call_site(),
        );
        field_checkers.push(quote_spanned! { ty.span() =>
            #[allow(non_camel_case_types)]
            struct #field_checker(<#ty as liquid_lang::You_Should_Use_An_Valid_Field_Type>::T);
        })
    }

    let abi_gen_helper = generate_abi_gen(&field_names, &field_tys, &ident);

    Ok(quote_spanned! { fields_span =>
        #(#field_checkers)*

        impl liquid_abi_codec::TypeInfo for #ident {
            #[inline(always)]
            fn is_dynamic() -> bool {
                #(<<#field_tys as liquid_lang::You_Should_Use_An_Valid_Field_Type>::T as liquid_abi_codec::TypeInfo>::is_dynamic() ||)* false
            }

            #[inline]
            fn size_hint() -> u32 {
                if Self::is_dynamic() {
                    unreachable!();
                } else {
                    #(<<#field_tys as liquid_lang::You_Should_Use_An_Valid_Field_Type>::T as liquid_abi_codec::TypeInfo>::size_hint() +)* 0
                }
            }
        }

        impl liquid_abi_codec::MediateEncode for #ident {
            fn encode(&self) -> liquid_abi_codec::Mediate {
                let mut mediates = __std::Vec::new();
                #(mediates.push(liquid_abi_codec::MediateEncode::encode(&self.#field_names));)*
                if <Self as liquid_abi_codec::TypeInfo>::is_dynamic() {
                    liquid_abi_codec::Mediate::PrefixedTuple(mediates)
                } else {
                    liquid_abi_codec::Mediate::RawTuple(mediates)
                }
            }
        }

        impl liquid_abi_codec::MediateDecode for #ident {
            fn decode(slices: &[liquid_abi_codec::Word], offset: usize) -> ::core::result::Result::Result<liquid_abi_codec::DecodeResult<Self>, liquid_primitives::Error>{
                let is_dynamic = <Self as liquid_abi_codec::TypeInfo>::is_dynamic();

                // The first element in a dynamic Tuple is an offset to the Tuple's data
                // For a static Tuple the data begins right away
                let (tail, mut new_offset) = if is_dynamic {
                    (&slices[((liquid_abi_codec::as_u32(liquid_abi_codec::peek(slices, offset)?)? as usize) / liquid_abi_codec::WORD_SIZE)..], 0)
                } else {
                    (slices, offset)
                };

                // The returned new_offset depends on whether the Tuple is dynamic
                // dynamic Tuple -> follows the prefixed Tuple data offset element
                // static Tuple  -> follows the last data element
                let result = liquid_abi_codec::DecodeResult {
                    value: Self {
                        #(#decode_tokens,)*
                    },
                    new_offset: if is_dynamic { offset + 1 } else { new_offset },
                };

                Ok(result)
            }
        }

        impl liquid_ty_mapping::MappingToSolidityType for #ident {
            const MAPPED_TYPE_NAME: [u8; liquid_ty_mapping::MAX_LENGTH_OF_MAPPED_TYPE_NAME] = {
                const LEN: usize = liquid_ty_mapping::MAX_LENGTH_OF_MAPPED_TYPE_NAME;
                liquid_ty_mapping::composite::<(#(#field_tys,)*), LEN>(&[])
            };
        }

        liquid_lang::gen_basic_type_notations!(#ident, liquid_lang);

        #abi_gen_helper
    })
}

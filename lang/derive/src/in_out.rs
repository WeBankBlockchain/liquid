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

use crate::utils;
use liquid_prelude::{string::ToString, vec::Vec};
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{self, parse::Result, DeriveInput};

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
}

fn generate_ty_mapping(input_tys: &[&syn::Type]) -> TokenStream2 {
    let mut expanded = quote! {
        #[allow(non_camel_case_types)]
        struct TyMappingHelper<T> {
            marker: core::marker::PhantomData<fn() -> T>,
        }

        impl _ty_mapping::SolTypeName for TyMappingHelper<()> {
            const NAME: &'static [u8] = <() as _ty_mapping::SolTypeName>::NAME;
        }

        impl _ty_mapping::SolTypeNameLen for TyMappingHelper<()> {
            const LEN: usize = <() as _ty_mapping::SolTypeNameLen>::LEN;
        }
    };

    for i in 1..=input_tys.len() {
        let tys = &input_tys[..i];
        let first_tys = &tys[0..i - 1];
        let rest_ty = &tys[i - 1];

        if i > 1 {
            expanded.extend(quote! {
                impl _ty_mapping::SolTypeName for TyMappingHelper<(#(#tys,)*)> {
                    const NAME: &'static [u8] = {
                        const LEN: usize =
                            <(#(#first_tys,)*) as _ty_mapping::SolTypeNameLen<_>>::LEN
                            + <#rest_ty as _ty_mapping::SolTypeNameLen<_>>::LEN
                            + 1;
                        &_ty_mapping::concat::<TyMappingHelper<(#(#first_tys,)*)>, #rest_ty, (), (), LEN>(true)
                    };
                }

                impl _ty_mapping::SolTypeNameLen for TyMappingHelper<(#(#tys,)*)> {
                    const LEN: usize =
                        <TyMappingHelper<(#(#first_tys,)*)> as _ty_mapping::SolTypeNameLen<_>>::LEN
                        + <#rest_ty as _ty_mapping::SolTypeNameLen<_>>::LEN
                        + 1;
                }
            });
        } else {
            expanded.extend(quote! {
                impl _ty_mapping::SolTypeName for TyMappingHelper<(#rest_ty,)> {
                    const NAME: &'static [u8] = <#rest_ty as _ty_mapping::SolTypeName<_>>::NAME;
                }

                impl _ty_mapping::SolTypeNameLen for TyMappingHelper<(#rest_ty,)> {
                    const LEN: usize = <#rest_ty as _ty_mapping::SolTypeNameLen<_>>::LEN;
                }
            });
        }
    }

    expanded
}

fn generate_abi_gen(
    field_names: &[&Ident],
    field_tys: &[&syn::Type],
    ident: &Ident,
) -> TokenStream2 {
    let field_names = field_names.iter().map(|name| name.to_string());

    quote! {
        #[cfg(feature = "liquid-abi-gen")]
        impl liquid_abi_gen::GenerateComponents for #ident {
            fn generate_components() -> __std::Vec<liquid_abi_gen::ParamABI> {
                let mut ret = __std::Vec::new();
                #(ret.push({
                    let mut param_abi = liquid_abi_gen::ParamABI::empty();
                    param_abi.components = <#field_tys as liquid_abi_gen::GenerateComponents>::generate_components();
                    if !param_abi.components.is_empty() {
                        param_abi.ty = String::from("tuple");
                    } else {
                        param_abi.ty = String::from_utf8((<#field_tys as _ty_mapping::SolTypeName>::NAME).to_vec()).expect("the type name of a function argument must an valid utf-8 string");
                    }
                    param_abi
                });)*

                let mut indexes = 0..ret.len();
                #(ret[indexes.next().unwrap()].name = #field_names.to_string();)*

                ret
            }
        }

        #[cfg(feature = "liquid-abi-gen")]
        impl liquid_abi_gen::TyName for #ident {
            fn ty_name() -> liquid_prelude::string::String {
                String::from("tuple")
            }
        }
    }
}

fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
    let ast: DeriveInput = syn::parse2(input)?;
    let (field_names, field_tys): (Vec<_>, Vec<_>) = utils::struct_syntax_check(&ast)?;
    let ident = &ast.ident;
    let fields_count = field_names.len();

    let mut decode_tokens = Vec::new();
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
    }

    let ty_mapping_helper = generate_ty_mapping(&field_tys);
    let abi_gen_helper = generate_abi_gen(&field_names, &field_tys, &ident);

    Ok(quote! {
        impl liquid_abi_codec::IsDynamic for #ident {
            fn is_dynamic() -> bool {
                #(<#field_tys as liquid_abi_codec::IsDynamic>::is_dynamic() ||)* false
            }
        }

        impl liquid_abi_codec::MediateEncode for #ident {
            fn encode(&self) -> liquid_abi_codec::Mediate {
                let mut mediates = __std::Vec::new();
                #(mediates.push(liquid_abi_codec::MediateEncode::encode(&self.#field_names));)*
                if <Self as liquid_abi_codec::IsDynamic>::is_dynamic() {
                    liquid_abi_codec::Mediate::PrefixedTuple(mediates)
                } else {
                    liquid_abi_codec::Mediate::RawTuple(mediates)
                }
            }
        }

        impl liquid_abi_codec::MediateDecode for #ident {
            fn decode(slices: &[liquid_abi_codec::Word], offset: usize) -> Result<liquid_abi_codec::DecodeResult<Self>, liquid_abi_codec::Error>{
                let is_dynamic = <Self as liquid_abi_codec::IsDynamic>::is_dynamic();

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

        #ty_mapping_helper

        impl _ty_mapping::SolTypeNameLen for #ident {
            const LEN: usize =
                <TyMappingHelper<(#(#field_tys,)*)> as _ty_mapping::SolTypeNameLen>::LEN
                + 2;
        }

        impl _ty_mapping::SolTypeName for #ident {
            const NAME: &'static [u8] = {
                const LEN: usize =
                    <TyMappingHelper<(#(#field_tys,)*)> as _ty_mapping::SolTypeNameLen>::LEN
                    + 2;

                &_ty_mapping::composite::<LEN>(
                    b"",
                    <TyMappingHelper<(#(#field_tys,)*)> as _ty_mapping::SolTypeName>::NAME)
            };
        }

        impl liquid_lang::You_Should_Use_An_Valid_Parameter_Type for #ident {}
        impl liquid_lang::You_Should_Use_An_Valid_Return_Type for #ident {}
        impl liquid_lang::You_Should_Use_An_Valid_Input_Type for #ident {}
        impl liquid_lang::You_Should_Use_An_Valid_Event_Data_Type for #ident {}

        impl _ty_mapping::SolTypeNameLen<#ident> for liquid_prelude::vec::Vec<#ident> {
            const LEN: usize = <#ident as _ty_mapping::SolTypeNameLen>::LEN + 2;
        }

        impl _ty_mapping::SolTypeName<#ident> for liquid_prelude::vec::Vec<#ident> {
            const NAME: &'static [u8] = {
                const LEN: usize = <#ident as _ty_mapping::SolTypeNameLen>::LEN + 2;

                &_ty_mapping::concat::<#ident, _ty_mapping::DynamicArraySuffix, (), (), LEN>(false)
            };
        }

        #abi_gen_helper
    })
}

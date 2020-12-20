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

use liquid_prelude::vec::Vec;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{self, parse::Result, spanned::Spanned, Data, DeriveInput, Fields, Type};

pub fn generate(input: TokenStream2) -> TokenStream2 {
    match generate_impl(input) {
        Ok(output) => output,
        Err(err) => err.to_compile_error(),
    }
}

fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
    let ast: DeriveInput = syn::parse2(input)?;

    match &ast.vis {
        syn::Visibility::Public(_) => (),
        _ => bail!(ast, "the visibility of this type should be `pub`"),
    }

    if ast.generics.type_params().count() > 0 {
        bail!(&ast.generics, "generic structs are not supported")
    }

    let ident = &ast.ident;
    let mut shadow = match &ast.data {
        Data::Union(ref union_data) => {
            bail!(&union_data.union_token, "unions are not supported")
        }
        Data::Struct(ref struct_data) => {
            let fields = &struct_data.fields;

            let (field_names, field_tys): (Vec<_>, Vec<_>) = match fields {
                Fields::Named(fields_named) => fields_named
                    .named
                    .iter()
                    .map(|field| (field.ident.as_ref().unwrap().clone(), &field.ty))
                    .unzip(),
                Fields::Unnamed(fields_unnamed) => fields_unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, field)| {
                        (Ident::new(&format!("_{}", i), field.span()), &field.ty)
                    })
                    .unzip(),
                Fields::Unit => (Vec::new(), Vec::new()),
            };

            let encode_shadow_struct =
                generate_encode_shadow_struct(ident, &field_names, &field_tys);
            let decode_shadow_struct =
                generate_decode_shadow_struct(ident, &field_names, &field_tys);

            let mut field_checkers = Vec::new();
            for i in 0..field_names.len() {
                let ty = &field_tys[i];

                let field_checker = Ident::new(
                    &format!("__LIQUID_FIELD_CHECKER_{}", i),
                    Span::call_site(),
                );
                field_checkers.push(quote_spanned! { ty.span() =>
                    #[allow(non_camel_case_types)]
                    struct #field_checker(<#ty as liquid_lang::You_Should_Use_An_Valid_Field_Type>::T);
                })
            }

            quote! {
                #(#field_checkers)*
                #encode_shadow_struct
                #decode_shadow_struct
            }
        }
        Data::Enum(ref enum_data) => {
            if enum_data.variants.is_empty() {
                bail!(ast, "empty enum is not supported")
            }

            for variant in &enum_data.variants {
                if let Some(_) = variant.discriminant {
                    bail!(variant, "custom discriminant is not supported")
                }
            }

            let mut field_checkers = Vec::new();
            let variants = enum_data
                .variants
                .iter()
                .map(|variant| {
                    let ident = &variant.ident;
                    let fields = &variant.fields;
                    let mut unnamed = true;
                    let mut is_unit = false;

                    let (field_names, field_tys): (Vec<_>, Vec<_>) = match fields {
                        Fields::Named(fields_named) => {
                            unnamed = false;

                            fields_named
                                .named
                                .iter()
                                .map(|field| {
                                    let field_checker = Ident::new(
                                        &format!("__LIQUID_FIELD_CHECKER_{}", field_checkers.len()),
                                        Span::call_site(),
                                    );
                                    let ty = &field.ty;
                                    field_checkers.push(quote_spanned! { ty.span() =>
                                        #[allow(non_camel_case_types)]
                                        struct #field_checker(<#ty as liquid_lang::You_Should_Use_An_Valid_Field_Type>::T);
                                    });

                                    (field.ident.as_ref().unwrap().clone(), ty)
                                })
                                .unzip()
                        }
                        Fields::Unnamed(fields_unnamed) => fields_unnamed
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, field)| {
                                let field_checker = Ident::new(
                                    &format!("__LIQUID_FIELD_CHECKER_{}", field_checkers.len()),
                                    Span::call_site(),
                                );
                                let ty = &field.ty;
                                field_checkers.push(quote_spanned! { ty.span() =>
                                    #[allow(non_camel_case_types)]
                                    struct #field_checker(<#ty as liquid_lang::You_Should_Use_An_Valid_Field_Type>::T);
                                });

                                (Ident::new(&format!("_{}", i), field.span()), ty)
                            })
                            .unzip(),
                        Fields::Unit => {
                            is_unit = true;
                            (Vec::new(), Vec::new())
                        },
                    };

                    Variant {
                        ident,
                        unnamed,
                        is_unit,
                        field_names,
                        field_tys,
                    }
                })
                .collect::<Vec<_>>();

            let encode_shadow_enum = generate_encode_shadow_enum(ident, variants.iter());
            let decode_shadow_enum = generate_decode_shadow_enum(ident, variants.iter());

            quote! {
                #(#field_checkers)*
                #encode_shadow_enum
                #decode_shadow_enum
            }
        }
    };

    shadow.extend(quote! {
        liquid_lang::gen_basic_type_notations!(#ident, liquid_lang);
    });
    Ok(shadow)
}

struct Variant<'a> {
    ident: &'a Ident,
    unnamed: bool,
    is_unit: bool,
    field_names: Vec<Ident>,
    field_tys: Vec<&'a Type>,
}

fn generate_encode_shadow_struct(
    ident: &Ident,
    field_names: &[Ident],
    field_tys: &[&Type],
) -> TokenStream2 {
    let fields = field_names.iter().enumerate().map(|(i, field_name)| {
        let field_ty = field_tys[i];
        quote!(#field_name: &'a #field_ty,)
    });
    let assigns = field_names
        .iter()
        .map(|field_name| quote!(#field_name: &origin.#field_name,));

    quote! {
        #[derive(scale::Encode)]
        struct EncodeShadow<'a> {
            #(#fields)*
            #[codec(skip)]
            _marker: core::marker::PhantomData<&'a ()>,
        }

        impl<'a> From<&'a #ident> for EncodeShadow<'a> {
            fn from(origin: &'a #ident) -> Self {
                Self {
                    #(#assigns)*
                    _marker: Default::default(),
                }
            }
        }

        impl scale::Encode for #ident {
            fn encode(&self) -> __std::Vec<u8> {
                let encode_shadow: EncodeShadow::<'_> = self.into();
                encode_shadow.encode()
            }
        }
    }
}

fn generate_encode_shadow_enum<'a>(
    ident: &Ident,
    variants: impl Iterator<Item = &'a Variant<'a>>,
) -> TokenStream2 {
    let (new_variants, arms): (Vec<_>, Vec<_>) = variants
        .map(|variant| {
            let variant_ident = variant.ident;
            let field_names = &variant.field_names;
            let field_tys = &variant.field_tys;
            let fields =
                field_names
                    .iter()
                    .zip(field_tys.iter())
                    .map(|(field_name, field_ty)| {
                        quote! {
                            #field_name: &'a #field_ty
                        }
                    });
            let new_variant = quote! {
                #variant_ident {
                    #(#fields,)*
                },
            };

            let arms = if variant.is_unit {
                quote! {
                    #ident::#variant_ident => Self::#variant_ident{},
                }
            } else {
                let ref_fields = field_names
                    .iter()
                    .map(|field_name| {
                        quote! {
                            ref #field_name
                        }
                    })
                    .collect::<Vec<_>>();

                if variant.unnamed {
                    quote! {
                        #ident::#variant_ident(#(#ref_fields,)*) => Self::#variant_ident {
                            #(#field_names,)*
                        },
                    }
                } else {
                    quote! {
                        #ident::#variant_ident{#(#ref_fields,)*} => Self::#variant_ident {
                            #(#field_names,)*
                        },
                    }
                }
            };
            (new_variant, arms)
        })
        .unzip();

    quote! {
        #[derive(scale::Encode)]
        enum EncodeShadow<'a> {
            #(#new_variants)*
        }

        impl<'a> From<&'a #ident> for EncodeShadow<'a> {
            fn from(origin: &'a #ident) -> Self {
                match origin {
                    #(#arms)*
                }
            }
        }

        impl scale::Encode for #ident {
            fn encode(&self) -> __std::Vec<u8> {
                let encode_shadow: EncodeShadow::<'_> = self.into();
                encode_shadow.encode()
            }
        }
    }
}

fn generate_decode_shadow_struct(
    ident: &Ident,
    field_names: &[Ident],
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
                let origin = <DecodeShadow as scale::Decode>::decode(value)?;
                Ok(Self {
                    #(#assigns)*
                })
            }
        }
    }
}

fn generate_decode_shadow_enum<'a>(
    ident: &Ident,
    variants: impl Iterator<Item = &'a Variant<'a>>,
) -> TokenStream2 {
    let (new_variants, arms): (Vec<_>, Vec<_>) = variants.map(|variant| {
        let variant_ident = variant.ident;
        let field_names = &variant.field_names;
        let field_tys = &variant.field_tys;
        let fields =
            field_names
                .iter()
                .zip(field_tys.iter())
                .map(|(field_name, field_ty)| {
                    quote! {
                        #field_name: #field_ty
                    }
                });
        let new_variants = quote! {
            #variant_ident {
                #(#fields,)*
            },
        };

        let arms = if variant.is_unit {
            debug_assert!(field_names.is_empty());
            quote! {
                DecodeShadow::#variant_ident{} => Ok(#ident::#variant_ident),
            }
        } else {
            if variant.unnamed {
                quote! {
                    DecodeShadow::#variant_ident{#(#field_names,)*} => Ok(#ident::#variant_ident (
                        #(#field_names,)*
                    )),
                }
            } else {
                quote! {
                    DecodeShadow::#variant_ident{#(#field_names,)*} => Ok(#ident::#variant_ident {
                        #(#field_names,)*
                    }),
                }
            }
        };
        (new_variants, arms)
    }).unzip();

    quote! {
        #[derive(scale::Decode)]
        enum DecodeShadow {
            #(#new_variants)*
        }

        impl scale::Decode for #ident {
            fn decode<I: scale::Input>(value: &mut I) -> ::core::result::Result<Self, scale::Error> {
                let origin = <DecodeShadow as scale::Decode>::decode(value)?;
                match origin {
                    #(#arms)*
                }
            }
        }
    }
}

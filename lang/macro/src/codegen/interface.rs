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
    codegen::{utils as codegen_utils, GenerateCode},
    ir::{FnArg, ForeignFn, Interface},
    utils as lang_utils,
};
use either::Either;
use heck::{CamelCase, ShoutySnakeCase};
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{punctuated::Punctuated, Token};

impl GenerateCode for Interface {
    fn generate_code(&self) -> TokenStream2 {
        let ident = &self.ident;
        let imports = &self.imports;
        let span = self.span;
        let types = codegen_utils::generate_primitive_types();

        let foreign_structs = self.generate_foreign_structs();
        let foreign_contract_ident = Ident::new(&ident.to_string().to_camel_case(), span);
        let foreign_contract = self.generate_foreign_contract(&foreign_contract_ident);

        quote! {
            mod #ident {
                #(#imports)*
                #(#foreign_structs)*

                mod __liquid_private {
                    use super::*;

                    #types

                    struct FnMarker<S> {
                        marker: core::marker::PhantomData<fn() -> S>,
                    }

                    struct TyMappingHelper<S, T> {
                        marker_s: core::marker::PhantomData<fn() -> S>,
                        marker_t: core::marker::PhantomData<fn() -> T>,
                    }

                    #foreign_contract
                }

                pub use __liquid_private::#foreign_contract_ident;
            }
        }
    }
}

pub fn generate_input_idents(args: &Punctuated<FnArg, Token![,]>) -> TokenStream2 {
    let input_idents = args
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(ident_type) => Some(&ident_type.ident),
            _ => None,
        })
        .collect::<Vec<_>>();
    quote! { #(#input_idents,)* }
}

pub fn generate_selector_ident(fn_name: &Ident) -> Ident {
    let shouty_name = &fn_name.to_string().to_shouty_snake_case();
    Ident::new(&shouty_name, Span::call_site())
}

pub fn generate_trivial_fn(foreign_fn: &ForeignFn) -> TokenStream2 {
    let attrs = &foreign_fn.attrs;
    let sig = &foreign_fn.sig;
    let span = foreign_fn.span;
    let fn_ident = &sig.ident;
    let fn_id = foreign_fn.fn_id;

    let inputs = &sig.inputs;
    let input_tys = codegen_utils::generate_input_tys(&sig, false);
    let input_ty_checker = codegen_utils::generate_ty_checker(input_tys.as_slice());
    let input_idents = generate_input_idents(inputs);

    let output = &sig.output;
    let output_ty = match output {
        syn::ReturnType::Default => {
            quote! { () }
        }
        syn::ReturnType::Type(_, ty) => quote! {
            #ty
        },
    };

    let ty_mapping = codegen_utils::generate_ty_mapping(fn_id, fn_ident, &input_tys);
    let selector_ident = generate_selector_ident(fn_ident);

    quote_spanned! { span =>
        #(#attrs)*
        #[allow(non_snake_case)]
        pub fn #fn_ident(&self, #inputs) -> Option<#output_ty> {
            const _: () = {
                #ty_mapping
            };

            const #selector_ident: liquid_primitives::Selector = <FnMarker<([(); #fn_id])> as liquid_lang::FnSelector>::SELECTOR;
            type Input = #input_ty_checker;

            let encoded = <Input as liquid_abi_codec::Encode>::encode(&(#input_idents));
            let mut data = #selector_ident.to_vec();
            data.extend(encoded);
            liquid_core::env::call::<#output_ty>(&self.__liquid_address, &data).ok()
        }
    }
}

pub fn generate_overloaded_fn(
    fn_ident: &Ident,
    foreign_fns: &[ForeignFn],
) -> TokenStream2 {
    let impls = foreign_fns.iter().enumerate().map(|(i, foreign_fn)| {
        let sig = &foreign_fn.sig;
        let span = foreign_fn.span;

        let inputs = &sig.inputs;
        let input_tys = codegen_utils::generate_input_tys(&sig, false);
        let input_ty_checker = codegen_utils::generate_ty_checker(input_tys.as_slice());
        let input_idents = generate_input_idents(inputs);

        let output = &sig.output;
        let output_ty = match output {
            syn::ReturnType::Default => {
                quote! { () }
            }
            syn::ReturnType::Type(_, ty) => quote! {
                #ty
            },
        };

        let mut origin_fn_id = foreign_fn.fn_id.to_string();
        origin_fn_id.push_str(&i.to_string());
        let fn_id = lang_utils::calculate_fn_id(&origin_fn_id);
        let origin_fn_ident = &sig.ident;
        let fn_name = format!("{}_{}", origin_fn_ident, i);
        let fn_ident = Ident::new(&fn_name, span);

        let ty_mapping = codegen_utils::generate_ty_mapping(fn_id, &fn_ident, &input_tys);
        let selector_ident = generate_selector_ident(&fn_ident);

        quote_spanned! { span =>
            const _: () = {
                #ty_mapping
            };

            #[allow(non_snake_case)]
            fn #fn_ident(__liquid_address: &liquid_primitives::types::Address, #inputs) -> Option<#output_ty> {
                const #selector_ident: liquid_primitives::Selector = <FnMarker<([(); #fn_id])> as liquid_lang::FnSelector>::SELECTOR;
                type Input = #input_ty_checker;

                let encoded = <Input as liquid_abi_codec::Encode>::encode(&(#input_idents));
                let mut data = #selector_ident.to_vec();
                data.extend(encoded);
                liquid_core::env::call::<#output_ty>(__liquid_address, &data).ok()
            }

            impl Fn<(#(#input_tys,)*)> for #origin_fn_ident {
                extern "rust-call" fn call(&self, (#input_idents): (#(#input_tys,)*)) -> Self::Output {
                    #fn_ident(&self.__liquid_address, #input_idents)
                }
            }

            impl FnOnce<(#(#input_tys,)*)> for #origin_fn_ident {
                type Output = Option<#output_ty>;

                extern "rust-call" fn call_once(self, args: (#(#input_tys,)*)) -> Self::Output {
                    self.call(args)
                }
            }

            impl FnMut<(#(#input_tys,)*)> for #origin_fn_ident {
                extern "rust-call" fn call_mut(&mut self, args: (#(#input_tys,)*)) -> Self::Output {
                    self.call(args)
                }
            }
        }
    });

    quote! {
        #[allow(non_camel_case_types)]
        pub struct #fn_ident {
            __liquid_address: liquid_primitives::types::Address,
        }

        impl From<liquid_primitives::types::Address> for #fn_ident {
            fn from(__liquid_address: liquid_primitives::types::Address) -> Self {
                Self {
                    __liquid_address,
                }
            }
        }

        #(#impls)*
    }
}

impl Interface {
    fn generate_foreign_structs(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.foreign_structs.iter().map(|foreign_struct| {
            let attrs = &foreign_struct.attrs;
            let ident = &foreign_struct.ident;
            let fields = foreign_struct.fields.named.iter().map(|field| {
                let field_attrs = &field.attrs;
                let field_ident = &field.ident;
                let field_ty = &field.ty;

                quote! {
                    #(#field_attrs)*
                    pub #field_ident: #field_ty,
                }
            });

            quote_spanned! { foreign_struct.span =>
                #(#attrs)*
                #[derive(liquid_lang::InOut)]
                pub struct #ident {
                    #(#fields)*
                }
            }
        })
    }

    fn generate_foreign_contract(&self, foreign_contract_ident: &Ident) -> TokenStream2 {
        let span = self.span;

        let (trivial_fns, overloaded_fns): (Vec<_>, Vec<_>) =
            self.foreign_fns.iter().partition_map(|(ident, fns)| {
                if fns.len() == 1 {
                    let trivial_fn = fns.first().unwrap();
                    Either::Left(generate_trivial_fn(trivial_fn))
                } else {
                    Either::Right((ident, generate_overloaded_fn(ident, fns)))
                }
            });
        let (overloaded_idents, overloaded_impls): (Vec<_>, Vec<_>) =
            overloaded_fns.into_iter().unzip();

        quote_spanned! { span =>
            pub struct #foreign_contract_ident {
                __liquid_address: liquid_primitives::types::Address,
                #(
                    pub #overloaded_idents: #overloaded_idents,
                )*
            }

            impl #foreign_contract_ident {
                pub fn at(address: liquid_primitives::types::Address) -> Self {
                    Self {
                        __liquid_address: address,
                        #(
                            #overloaded_idents: address.into(),
                        )*
                    }
                }
            }

            impl From<liquid_primitives::types::Address> for #foreign_contract_ident {
                fn from(address: liquid_primitives::types::Address) -> Self {
                    Self::at(address)
                }
            }

            impl scale::Decode for #foreign_contract_ident {
                fn decode<I: scale::Input>(value: &mut I) -> Result<Self, scale::Error> {
                    let __liquid_address = liquid_primitives::types::Address::decode(value)?;
                    Ok(Self {
                        __liquid_address,
                        #(
                            #overloaded_idents: __liquid_address.into(),
                        )*
                    })
                }
            }

            impl scale::Encode for #foreign_contract_ident {
                fn encode(&self) -> Vec<u8> {
                    self.__liquid_address.encode()
                }
            }

            #(#overloaded_impls)*

            impl Into<liquid_primitives::types::Address> for #foreign_contract_ident {
                fn into(self) -> liquid_primitives::types::Address {
                    self.__liquid_address
                }
            }

            impl liquid_ty_mapping::SolTypeName for #foreign_contract_ident {
                const NAME: &'static [u8] = liquid_ty_mapping::ADDRESS_MAPPED_TYPE.as_bytes();
            }

            impl liquid_ty_mapping::SolTypeNameLen for #foreign_contract_ident {
                const LEN: usize = liquid_ty_mapping::ADDRESS_MAPPED_TYPE.len();
            }

            impl liquid_ty_mapping::SolTypeName<#foreign_contract_ident> for Vec<#foreign_contract_ident> {
                const NAME: &'static [u8] = liquid_ty_mapping::ADDRESS_ARRAY_MAPPED_TYPE.as_bytes();
            }

            impl liquid_ty_mapping::SolTypeNameLen<#foreign_contract_ident> for Vec<#foreign_contract_ident> {
                const LEN: usize = liquid_ty_mapping::ADDRESS_MAPPED_TYPE.len() + 2;
            }

            impl liquid_abi_codec::IsDynamic for #foreign_contract_ident {}

            impl liquid_abi_codec::MediateEncode for #foreign_contract_ident {
                fn encode(&self) -> liquid_abi_codec::Mediate {
                    self.__liquid_address.encode()
                }
            }

            impl liquid_abi_codec::MediateDecode for #foreign_contract_ident {
                fn decode(
                    slices: &[liquid_abi_codec::Word],
                    offset: usize
                ) -> Result<liquid_abi_codec::DecodeResult<Self>, liquid_abi_codec::Error> {
                    let decode_result = <liquid_primitives::types::Address as liquid_abi_codec::MediateDecode>::decode(slices, offset)?;
                    let value = Self {
                        __liquid_address: decode_result.value,
                        #(
                            #overloaded_idents: decode_result.value.into(),
                        )*
                    };
                    Ok(liquid_abi_codec::DecodeResult {
                        value,
                        new_offset: decode_result.new_offset
                    })
                }
            }

            impl liquid_lang::You_Should_Use_An_Valid_Parameter_Type for #foreign_contract_ident {}
            impl liquid_lang::You_Should_Use_An_Valid_Return_Type for #foreign_contract_ident {}
            impl liquid_lang::You_Should_Use_An_Valid_Input_Type for #foreign_contract_ident {}

            impl #foreign_contract_ident{
                #(#trivial_fns)*
            }
        }
    }
}

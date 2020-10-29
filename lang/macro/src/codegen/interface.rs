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
        let foreign_contract_ident = if self.meta_info.interface_name.is_empty() {
            Ident::new(&ident.to_string().to_camel_case(), span)
        } else {
            Ident::new(&self.meta_info.interface_name, span)
        };
        let foreign_contract = self.generate_foreign_contract(&foreign_contract_ident);

        quote! {
            mod #ident {
                #(#imports)*
                #(#foreign_structs)*

                mod __liquid_private {
                    use super::*;

                    #types

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
            <#ty as liquid_lang::You_Should_Use_An_Valid_Return_Type>::T
        },
    };

    let selector_ident = generate_selector_ident(fn_ident);
    let fn_name = fn_ident.to_string();
    let fn_name_bytes = fn_name.as_bytes();
    let fn_name_len = fn_name.len();
    quote_spanned! { span =>
        #(#attrs)*
        #[allow(non_snake_case)]
        pub fn #fn_ident(&self, #inputs) -> Option<#output_ty> {
            type Input = #input_ty_checker;
            const #selector_ident: liquid_primitives::Selector = {
                const SIG_LEN: usize =
                    liquid_ty_mapping::len::<Input>()
                    + #fn_name_len
                    + 2;

                const SIG: [u8; SIG_LEN] =
                    liquid_ty_mapping::composite::<Input, SIG_LEN>(&[#(#fn_name_bytes),*]);

                let hash = liquid_primitives::hash::hash(&SIG);
                [hash[0], hash[1], hash[2], hash[3]]
            };

            let mut encoded = #selector_ident.to_vec();
            encoded.extend(<Input as liquid_abi_codec::Encode>::encode(&(#input_idents)));
            liquid_core::env::call::<#output_ty>(&self.__liquid_address, &encoded).ok()
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
                <#ty as liquid_lang::You_Should_Use_An_Valid_Return_Type>::T
            },
        };

        let origin_fn_ident = &sig.ident;
        let fn_name = format!("{}_{}", origin_fn_ident, i);
        let fn_ident = Ident::new(&fn_name, span);

        let selector_ident = generate_selector_ident(&fn_ident);
        let origin_fn_name = origin_fn_ident.to_string();
        let origin_fn_name_bytes = origin_fn_name.as_bytes();
        let origin_fn_name_len = origin_fn_name.len();
        quote_spanned! { span =>
            #[allow(non_snake_case)]
            fn #fn_ident(__liquid_address: &liquid_primitives::types::address, #inputs) -> Option<#output_ty> {
                type Input = #input_ty_checker;
                const #selector_ident: liquid_primitives::Selector = {
                    const SIG_LEN: usize =
                        liquid_ty_mapping::len::<Input>()
                        + #origin_fn_name_len
                        + 2;

                    const SIG: [u8; SIG_LEN] =
                        liquid_ty_mapping::composite::<Input, SIG_LEN>(&[#(#origin_fn_name_bytes),*]);

                    let hash = liquid_primitives::hash::hash(&SIG);
                    [hash[0], hash[1], hash[2], hash[3]]
                };

                let mut encoded = #selector_ident.to_vec();
                encoded.extend(<Input as liquid_abi_codec::Encode>::encode(&(#input_idents)));
                liquid_core::env::call::<#output_ty>(&__liquid_address, &encoded).ok()
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
            __liquid_address: liquid_primitives::types::address,
        }

        impl From<liquid_primitives::types::address_impl::address> for #fn_ident {
            fn from(__liquid_address: liquid_primitives::types::address) -> Self {
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
            #[allow(non_camel_case_types)]
            pub struct #foreign_contract_ident {
                __liquid_address: liquid_primitives::types::address,
                #(
                    pub #overloaded_idents: #overloaded_idents,
                )*
            }

            impl #foreign_contract_ident {
                pub fn at(addr: liquid_primitives::types::address) -> Self {
                    Self {
                        __liquid_address: addr,
                        #(
                            #overloaded_idents: addr.into(),
                        )*
                    }
                }
            }

            impl From<liquid_primitives::types::address> for #foreign_contract_ident {
                fn from(addr: liquid_primitives::types::address) -> Self {
                    Self::at(addr)
                }
            }

            impl scale::Decode for #foreign_contract_ident {
                fn decode<I: scale::Input>(value: &mut I) -> Result<Self, scale::Error> {
                    let __liquid_address = liquid_primitives::types::address::decode(value)?;
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

            impl Into<liquid_primitives::types::address> for #foreign_contract_ident {
                fn into(self) -> liquid_primitives::types::address {
                    self.__liquid_address
                }
            }

            impl liquid_ty_mapping::MappingToSolidityType for #foreign_contract_ident {
                const MAPPED_TYPE_NAME: [u8; liquid_ty_mapping::MAX_LENGTH_OF_MAPPED_TYPE_NAME] =
                    <liquid_primitives::types::address as liquid_ty_mapping::MappingToSolidityType>::MAPPED_TYPE_NAME;
            }

            impl liquid_abi_codec::TypeInfo for #foreign_contract_ident {}

            impl liquid_abi_codec::MediateEncode for #foreign_contract_ident {
                fn encode(&self) -> liquid_abi_codec::Mediate {
                    self.__liquid_address.encode()
                }
            }

            impl liquid_abi_codec::MediateDecode for #foreign_contract_ident {
                fn decode(
                    slices: &[liquid_abi_codec::Word],
                    offset: usize
                ) -> Result<liquid_abi_codec::DecodeResult<Self>, liquid_primitives::Error> {
                    let decode_result = <liquid_primitives::types::address as liquid_abi_codec::MediateDecode>::decode(slices, offset)?;
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
            impl liquid_lang::You_Should_Use_An_Valid_Field_Data_Type for #foreign_contract_ident {}

            impl #foreign_contract_ident{
                #(#trivial_fns)*
            }
        }
    }
}

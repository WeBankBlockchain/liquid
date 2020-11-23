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
    codegen::{interface_impl::mockable::Mockable, utils as codegen_utils, GenerateCode},
    ir::{utils as ir_utils, ForeignFn, Interface},
};
use either::Either;
use heck::ShoutySnakeCase;
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};

impl GenerateCode for Interface {
    fn generate_code(&self) -> TokenStream2 {
        let ident = &self.ident;
        let imports = &self.imports;
        let interface_ident = &self.interface_ident;
        let types = codegen_utils::generate_primitive_types();
        let mockable = Mockable::from(self);

        let foreign_structs = self.generate_foreign_structs();
        let foreign_contract = self.generate_foreign_contract();
        let mockable_contract = mockable.generate_code();

        quote! {
            mod #ident {
                #(#imports)*
                #(#foreign_structs)*

                mod __liquid_private {
                    use super::*;

                    #types

                    #[cfg(not(test))]
                    #[allow(dead_code)]
                    pub mod interface {
                        use super::*;
                        #foreign_contract
                    }

                    #[cfg(test)]
                    pub mod interface {
                        use super::*;
                        #mockable_contract
                    }
                }

                pub type #interface_ident = __liquid_private::interface::Interface;
            }
        }
    }
}

fn generate_selector_ident(fn_name: &Ident) -> Ident {
    let shouty_name = &fn_name.to_string().to_shouty_snake_case();
    Ident::new(&shouty_name, Span::call_site())
}

fn generate_trivial_fn(foreign_fn: &ForeignFn) -> TokenStream2 {
    let attrs = ir_utils::filter_non_liquid_attributes(foreign_fn.attrs.iter());
    let sig = &foreign_fn.sig;
    let span = foreign_fn.span;
    let fn_ident = &sig.ident;

    let inputs = &sig.inputs;
    let input_tys = codegen_utils::generate_input_tys(&sig);
    let input_ty_checker = codegen_utils::generate_ty_checker(input_tys.as_slice());
    let input_idents = codegen_utils::generate_input_idents(inputs);

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

    let inputs = inputs.iter().skip(1);
    let is_mut = sig.is_mut();

    quote_spanned! { span =>
        #(#attrs)*
        #[allow(non_snake_case)]
        pub fn #fn_ident(&self, #(#inputs,)*) -> Option<#output_ty> {
            #[allow(dead_code)]
            type Input = #input_ty_checker;

            #[allow(dead_code)]
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
            encoded.extend(<Input as liquid_abi_codec::Encode>::encode(&(#(#input_idents,)*)));
            if #is_mut {
                liquid_core::storage::mutable_call_happens();
            }
            liquid_core::env::call::<#output_ty>(&self.__liquid_address, &encoded).ok()
        }
    }
}

fn generate_overloaded_fn(fn_ident: &Ident, foreign_fns: &[ForeignFn]) -> TokenStream2 {
    let impls = foreign_fns.iter().enumerate().map(|(i, foreign_fn)| {
        let attrs = ir_utils::filter_non_liquid_attributes(foreign_fn.attrs.iter());
        let sig = &foreign_fn.sig;
        let span = foreign_fn.span;

        let inputs = &sig.inputs;
        let input_tys = codegen_utils::generate_input_tys(&sig);
        let input_ty_checker = codegen_utils::generate_ty_checker(input_tys.as_slice());
        let input_idents = codegen_utils::generate_input_idents(inputs);

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

        let inputs = inputs.iter().skip(1);
        let is_mut = sig.is_mut();

        quote_spanned! { span =>
            #[allow(non_snake_case)]
            #(#attrs)*
            fn #fn_ident(__liquid_address: &liquid_primitives::types::Address, #(#inputs,)*) -> Option<#output_ty> {
                #[allow(dead_code)]
                type Input = #input_ty_checker;

                #[allow(dead_code)]
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
                encoded.extend(<Input as liquid_abi_codec::Encode>::encode(&(#(#input_idents,)*)));
                if #is_mut {
                    liquid_core::storage::mutable_call_happens();
                }
                liquid_core::env::call::<#output_ty>(&__liquid_address, &encoded).ok()
            }

            impl FnOnce<(#(#input_tys,)*)> for #origin_fn_ident {
                type Output = Option<#output_ty>;

                extern "rust-call" fn call_once(self, (#(#input_idents,)*): (#(#input_tys,)*)) -> Self::Output {
                    #fn_ident(unsafe {
                        &*self.__liquid_address
                    }, #(#input_idents,)*)
                }
            }

            impl FnMut<(#(#input_tys,)*)> for #origin_fn_ident {
                extern "rust-call" fn call_mut(&mut self, (#(#input_idents,)*): (#(#input_tys,)*)) -> Self::Output {
                    #fn_ident(unsafe {
                        &*self.__liquid_address
                    }, #(#input_idents,)*)
                }
            }

            impl Fn<(#(#input_tys,)*)> for #origin_fn_ident {
                extern "rust-call" fn call(&self, (#(#input_idents,)*): (#(#input_tys,)*)) -> Self::Output {
                    #fn_ident(unsafe {
                        &*self.__liquid_address
                    }, #(#input_idents,)*)
                }
            }
        }
    });

    quote! {
        #[allow(non_camel_case_types)]
        pub struct #fn_ident {
            __liquid_address: *const liquid_primitives::types::Address,
        }

        impl #fn_ident {
            pub fn init(&mut self, addr: *const liquid_primitives::types::Address) {
                self.__liquid_address = addr;
            }
        }

        impl Default for #fn_ident {
            fn default() -> Self {
                Self {
                    __liquid_address: core::ptr::null(),
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

    fn generate_foreign_contract(&self) -> TokenStream2 {
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
            pub struct InterfaceImpl {
                __liquid_address: liquid_primitives::types::Address,
                __liquid_marker: core::marker::PhantomPinned,
                #(
                    pub #overloaded_idents: #overloaded_idents,
                )*
            }

            pub struct Interface(core::pin::Pin<liquid_prelude::boxed::Box<InterfaceImpl>>);

            impl Interface {
                pub fn at(addr: liquid_primitives::types::Address) -> Self {
                    let iface = InterfaceImpl {
                        __liquid_address: addr,
                        __liquid_marker: core::marker::PhantomPinned,
                        #(
                            #overloaded_idents: Default::default(),
                        )*
                    };

                    #[allow(unused_mut)]
                    let mut boxed = liquid_prelude::boxed::Box::pin(iface);
                    #[allow(unused_variables)]
                    let addr_ptr: *const liquid_primitives::types::Address = &boxed.as_ref().__liquid_address;
                    #[allow(unused_unsafe)]
                    unsafe {
                        #(
                            boxed.as_mut().get_unchecked_mut().#overloaded_idents.init(addr_ptr);
                        )*
                    }

                    Self(boxed)
                }
            }

            impl From<liquid_primitives::types::Address> for Interface {
                fn from(addr: liquid_primitives::types::Address) -> Self {
                    Self::at(addr)
                }
            }

            impl scale::Decode for Interface {
                fn decode<I: scale::Input>(value: &mut I) -> Result<Self, scale::Error> {
                    let addr = liquid_primitives::types::Address::decode(value)?;
                    Ok(Self::at(addr))
                }
            }

            impl scale::Encode for Interface {
                fn encode(&self) -> Vec<u8> {
                    self.0.__liquid_address.encode()
                }
            }

            #(#overloaded_impls)*

            impl Into<liquid_primitives::types::Address> for Interface {
                fn into(self) -> liquid_primitives::types::Address {
                    self.0.__liquid_address
                }
            }

            impl liquid_ty_mapping::MappingToSolidityType for Interface {
                const MAPPED_TYPE_NAME: [u8; liquid_ty_mapping::MAX_LENGTH_OF_MAPPED_TYPE_NAME] =
                    <liquid_primitives::types::Address as liquid_ty_mapping::MappingToSolidityType>::MAPPED_TYPE_NAME;
            }

            impl liquid_abi_codec::TypeInfo for Interface {}

            impl liquid_abi_codec::MediateEncode for Interface {
                fn encode(&self) -> liquid_abi_codec::Mediate {
                    self.0.__liquid_address.encode()
                }
            }

            impl liquid_abi_codec::MediateDecode for Interface {
                fn decode(
                    slices: &[liquid_abi_codec::Word],
                    offset: usize
                ) -> Result<liquid_abi_codec::DecodeResult<Self>, liquid_primitives::Error> {
                    let decode_result = <liquid_primitives::types::Address as liquid_abi_codec::MediateDecode>::decode(slices, offset)?;
                    let value = Self::at(decode_result.value);
                    Ok(liquid_abi_codec::DecodeResult {
                        value,
                        new_offset: decode_result.new_offset
                    })
                }
            }

            impl liquid_lang::You_Should_Use_An_Valid_Parameter_Type for Interface {}
            impl liquid_lang::You_Should_Use_An_Valid_Return_Type for Interface {}
            impl liquid_lang::You_Should_Use_An_Valid_Input_Type for Interface {}
            impl liquid_lang::You_Should_Use_An_Valid_Field_Data_Type for Interface {}

            impl InterfaceImpl {
                #(#trivial_fns)*
            }

            impl core::ops::Deref for Interface {
                type Target = InterfaceImpl;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }
        }
    }
}

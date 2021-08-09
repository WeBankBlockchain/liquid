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

mod mockable;

use crate::{
    common,
    contract::ir::{ForeignFn, Interface},
    utils,
};
use either::Either;
use heck::ShoutySnakeCase;
use itertools::Itertools;
use mockable::Mockable;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};

impl common::GenerateCode for Interface {
    fn generate_code(&self) -> TokenStream2 {
        let ident = &self.ident;
        let imports = &self.imports;
        let interface_ident = &self.interface_ident;
        let types = utils::generate_primitive_types();
        let mockable = Mockable::from(self);

        let foreign_structs = self.generate_foreign_structs();
        let foreign_contract = self.generate_foreign_contract();
        let mockable_contract = mockable.generate_code();

        quote! {
            mod #ident {
                #(#imports)*
                #(#foreign_structs)*

                mod __liquid_private {
                    #[allow(unused_imports)]
                    use super::*;

                    #types

                    #[cfg(not(test))]
                    #[allow(dead_code)]
                    pub mod __liquid_interface {
                        use super::*;
                        #foreign_contract
                    }

                    #[cfg(test)]
                    pub mod __liquid_interface {
                        use super::*;
                        #mockable_contract
                    }
                }

                pub type #interface_ident = __liquid_private::__liquid_interface::Interface;
            }
        }
    }
}

fn generate_trivial_fn(foreign_fn: &ForeignFn) -> TokenStream2 {
    let attrs = utils::filter_non_liquid_attributes(foreign_fn.attrs.iter());
    let sig = &foreign_fn.sig;
    let span = foreign_fn.span;
    let fn_ident = &sig.ident;

    let inputs = &sig.inputs;
    let input_tys = common::generate_input_tys(sig);
    let input_ty_checker = common::generate_ty_checker(input_tys.as_slice());
    let input_idents = common::generate_input_idents(sig);

    let output = &sig.output;
    let output_ty = match output {
        syn::ReturnType::Default => {
            quote! { () }
        }
        syn::ReturnType::Type(_, ty) => quote! {
            <#ty as liquid_lang::You_Should_Use_An_Valid_Output_Type>::T
        },
    };

    let fn_name = fn_ident.to_string();
    let fn_name_bytes = fn_name.as_bytes();

    let is_mut = sig.is_mut();
    let input_encodes = input_idents
        .iter()
        .zip(input_tys.iter())
        .map(|(ident, ty)| {
            quote! {
                <#ty as scale::Encode>::encode(&#ident)
            }
        });

    // Although it seems redundant here, if I don't do this, the error message
    // will display twice when type of inputs or outputs is not suitable, what
    // is ugly to developer. The key to prevent this phenomenon is the `span`
    // used in `quote_spanned` macro.
    let receiver = if is_mut {
        quote_spanned!(span => &mut self)
    } else {
        quote_spanned!(span => &self)
    };
    let actual_inputs = inputs.iter().skip(1);

    quote_spanned! { span =>
        #(#attrs)*
        #[allow(non_snake_case)]
        #[allow(unused_mut)]
        pub fn #fn_ident(#receiver, #(#actual_inputs,)*) -> Option<#output_ty> {
            #[allow(dead_code)]
            struct __LiquidInputTyChecker #input_ty_checker;

            #[allow(dead_code)]
            const __LIQUID_SELECTOR: liquid_primitives::Selector = {
                let hash = liquid_primitives::hash::hash(&[#(#fn_name_bytes),*]);
                [hash[0], hash[1], hash[2], hash[3]]
            };

            let mut __liquid_encoded = __LIQUID_SELECTOR.to_vec();
            #(
                __liquid_encoded.extend(#input_encodes);
            )*
            liquid_lang::env::call::<#output_ty>(&self.0, &__liquid_encoded).ok()
        }
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

        let trivial_fns = self
            .foreign_fns
            .iter()
            .map(|(_, foreign_fn)| generate_trivial_fn(foreign_fn))
            .collect::<Vec<_>>();

        let impls = quote_spanned! { span =>
            pub struct Interface(liquid_primitives::types::Address);

            impl Interface {
                pub fn at(addr: liquid_primitives::types::Address) -> Self {
                    Self(addr)
                }
            }

            impl From<liquid_primitives::types::Address> for Interface {
                fn from(addr: liquid_primitives::types::Address) -> Self {
                    Self::at(addr)
                }
            }

            impl scale::Decode for Interface {
                fn decode<I: scale::Input>(value: &mut I) -> ::core::result::Result<Self, scale::Error> {
                    let addr = liquid_primitives::types::Address::decode(value)?;
                    Ok(Self::at(addr))
                }
            }

            impl scale::Encode for Interface {
                fn encode(&self) -> Vec<u8> {
                    self.0.encode()
                }
            }

            impl Into<liquid_primitives::types::Address> for Interface {
                fn into(self) -> liquid_primitives::types::Address {
                    self.0
                }
            }

            #[cfg(feature = "liquid-abi-gen")]
            impl liquid_abi_gen::traits::TypeToString for Interface {
                fn type_to_string() -> liquid_prelude::string::String {
                    <liquid_primitives::types::Address as liquid_abi_gen::traits::TypeToString>::type_to_string()
                }
            }

            impl liquid_lang::You_Should_Use_An_Valid_Input_Type for Interface {}
            impl liquid_lang::You_Should_Use_An_Valid_Output_Type for Interface {}
            impl liquid_lang::You_Should_Use_An_Valid_Topic_Type for Interface {}
            impl liquid_lang::You_Should_Use_An_Valid_State_Type for Interface {}

            impl Interface {
                #(#trivial_fns)*
            }
        };

        impls
    }
}

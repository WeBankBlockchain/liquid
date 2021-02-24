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
    common::GenerateCode,
    contract::ir::{Contract, FnArg, Signature},
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[derive(From)]
pub struct AbiGen<'a> {
    contract: &'a Contract,
}

impl<'a> GenerateCode for AbiGen<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let constructor_abi = self.generate_constructor_abi();
        let external_fn_abis = self.generate_external_fn_abis();
        let event_abis = self.generate_event_abis();

        quote! {
            #[cfg(feature = "liquid-abi-gen")]
            #[allow(non_camel_case_types)]
            pub struct __LIQUID_ABI_GEN;

            #[cfg(feature = "liquid-abi-gen")]
            const _: () = {
                impl liquid_lang::GenerateAbi for __LIQUID_ABI_GEN {
                    fn generate_abi() -> liquid_abi_gen::ContractAbi {
                        let constructor_abi = #constructor_abi;
                        let external_fn_abis = #external_fn_abis;
                        let event_abis = #event_abis;

                        liquid_abi_gen::ContractAbi {
                            constructor_abi,
                            external_fn_abis,
                            event_abis,
                        }
                    }
                }
            };
        }
    }
}

fn generate_fn_inputs(sig: &Signature) -> impl Iterator<Item = TokenStream2> + '_ {
    sig.inputs.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ident_type) => {
            let ident = &ident_type.ident.to_string();
            let ty = &ident_type.ty;

            quote! {
                <#ty as liquid_abi_gen::traits::GenerateParamAbi>::generate_param_abi(#ident.to_owned())
            }
        }
        _ => unreachable!(),
    })
}

impl<'a> AbiGen<'a> {
    fn generate_constructor_abi(&self) -> TokenStream2 {
        let constructor = &self.contract.constructor;
        let input_args = generate_fn_inputs(&constructor.sig);

        quote! {
            liquid_abi_gen::ConstructorAbi::new_builder()
                #(.input(#input_args))*
                .done()
        }
    }

    fn generate_external_fn_abis(&self) -> TokenStream2 {
        let external_fns = &self.contract.functions;
        let fn_abis = external_fns.iter().filter(|func| func.is_external_fn() && !func.is_internal_fn()).map(|external_fn| {
            let ident = external_fn.sig.ident.to_string();
            let input_args = generate_fn_inputs(&external_fn.sig);
            let output = &external_fn.sig.output;
            let output_args = match output {
                syn::ReturnType::Default => quote! {},
                syn::ReturnType::Type(_, ty) => {
                    quote! {
                        <#ty as liquid_abi_gen::traits::GenerateOutputs>::generate_outputs(&mut builder);
                    }
                }
            };

            let constant = !external_fn.sig.is_mut();
            let build_args = if cfg!(feature = "solidity-compatible") {
                let state_mutability = if constant { "view" } else { "nonpayable" };
                quote! {
                    String::from(#ident), String::from(#state_mutability), #constant
                }
            } else {
                quote! {
                    String::from(#ident), #constant
                }
            };

            quote! {
                {
                    let mut builder = liquid_abi_gen::ExternalFnAbi::new_builder(#build_args);
                    #(builder.input(#input_args);)*
                    #output_args
                    builder.done()
                }
            }
        });

        quote! {
            {
                let mut external_fn_abis = Vec::new();
                #(external_fn_abis.push(#fn_abis);)*
                external_fn_abis
            }
        }
    }

    fn generate_event_abis(&self) -> TokenStream2 {
        let events = &self.contract.events;
        let abis = events.iter().map(|event| {
            let event_name = event.ident.to_string();
            let inputs = event.fields.iter().enumerate().map(|(i, field)|{
                let name = match &field.ident {
                    Some(ident) => ident.to_string(),
                    _ => String::new(),
                };
                let field_ty = &field.ty;
                let is_indexed = event.indexed_fields.iter().any(|index| *index == i);

                quote!{
                    <#field_ty as liquid_abi_gen::traits::GenerateParamAbi>::generate_param_abi(#name.to_owned()), #is_indexed
                }});

            quote! {
                {
                    let mut builder = liquid_abi_gen::EventAbi::new_builder(String::from(#event_name));
                    #(builder.input(#inputs);)*
                    builder.done()
                }
            }
        });

        quote! {
            {
                let mut event_abis = Vec::new();
                #(event_abis.push(#abis);)*
                event_abis
            }
        }
    }
}

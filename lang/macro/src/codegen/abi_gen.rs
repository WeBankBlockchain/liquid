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
    codegen::GenerateCode,
    ir::{Contract, FnArg, Signature},
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[derive(From)]
pub struct ABIGen<'a> {
    contract: &'a Contract,
}

impl<'a> GenerateCode for ABIGen<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let storage_ident = &self.contract.storage.ident;
        let constructor_abi = self.generate_constructor_abi();
        let external_fn_abis = self.generate_external_fn_abis();
        let event_abis = self.generate_event_abis();

        quote! {
            #[cfg(feature = "liquid-abi-gen")]
            const _: () = {
                impl liquid_lang::GenerateABI for #storage_ident {
                    fn generate_abi() -> liquid_abi_gen::ContractABI {
                        let constructor_abi = #constructor_abi;
                        let external_fn_abis = #external_fn_abis;
                        let event_abis = #event_abis;

                        liquid_abi_gen::ContractABI {
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

fn generate_components(ty: &syn::Type) -> TokenStream2 {
    quote! {
        <#ty as liquid_abi_gen::GenerateComponents>::generate_components()
    }
}

fn generate_ty_name(ty: &syn::Type) -> TokenStream2 {
    quote! {
        <#ty as liquid_abi_gen::TyName>::ty_name()
    }
}

fn generate_fn_inputs<'a>(sig: &'a Signature) -> impl Iterator<Item = TokenStream2> + 'a {
    sig.inputs.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ident_type) => {
            let ident = &ident_type.ident.to_string();
            let ty = &ident_type.ty;

            let ty_name = generate_ty_name(ty);
            let components = generate_components(ty);

            quote! {
                #components, String::from(#ident), #ty_name
            }
        }
        _ => unreachable!(),
    })
}

impl<'a> ABIGen<'a> {
    fn generate_constructor_abi(&self) -> TokenStream2 {
        let constructor = &self.contract.constructor;
        let input_args = generate_fn_inputs(&constructor.sig);

        quote! {
            liquid_abi_gen::ConstructorABI::new_builder()
            #(.input(#input_args))*
            .done()
        }
    }

    fn generate_external_fn_abis(&self) -> TokenStream2 {
        let external_fns = &self.contract.functions;
        let fn_abis = external_fns.iter().filter(|func| func.is_external_fn()).map(|external_fn| {
            let ident = external_fn.sig.ident.to_string();
            let input_args = generate_fn_inputs(&external_fn.sig);
            let output = &external_fn.sig.output;
            let output_args = match output {
                syn::ReturnType::Default => quote! {},
                syn::ReturnType::Type(_, ty) => {
                    quote! {
                        <#ty as liquid_abi_gen::GenerateOutputs>::generate_outputs(&mut builder);
                    }
                }
            };

            let constant = !external_fn.sig.is_mut();
            let state_mutability = if constant { "view" } else { "nonpayable" };

            quote! {
                {
                    let mut builder = liquid_abi_gen::ExternalFnABI::new_builder(String::from(#ident), String::from(#state_mutability), #constant);
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
                let field_name = field.ident.as_ref().unwrap().to_string();
                let field_ty = &field.ty;
                let is_indexed = event.indexed_fields.iter().any(|index| *index == i);

                let ty_name = generate_ty_name(field_ty);
                let components = generate_components(field_ty);

                quote!{
                    #components, String::from(#field_name), #ty_name, #is_indexed
                }});
            quote! {
                {
                    let mut builder = liquid_abi_gen::EventABI::new_builder(String::from(#event_name));
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

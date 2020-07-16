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

        quote! {
            #[cfg(feature = "liquid-abi-gen")]
            const _: () = {
                impl liquid_lang::GenerateABI for #storage_ident {
                    fn generate_abi() -> liquid_abi_gen::ContractABI {
                        let constructor_abi = #constructor_abi;
                        let external_fn_abis = #external_fn_abis;

                        liquid_abi_gen::ContractABI {
                            constructor_abi,
                            external_fn_abis
                        }
                    }
                }
            };
        }
    }
}

fn generate_fn_inputs<'a>(sig: &'a Signature) -> impl Iterator<Item = TokenStream2> + 'a {
    sig.inputs.iter().skip(1).map(|arg| match arg {
        FnArg::Typed(ident_type) => {
            let ident = &ident_type.ident.to_string();
            let ty = &ident_type.ty;

            quote! {
                String::from(#ident), String::from_utf8((<#ty as liquid_lang::ty_mapping::SolTypeName>::NAME).to_vec()).expect("the type name of a function argument must an valid utf-8 string")
            }
        },
        _ => unreachable!(),
    })
}

fn generate_fn_outputs(sig: &Signature) -> Vec<TokenStream2> {
    let output = &sig.output;

    match output {
        syn::ReturnType::Default => Vec::new(),
        syn::ReturnType::Type(_, ty) => {
            if let syn::Type::Tuple(tuple_ty) = &(**ty) {
                tuple_ty.elems.iter().map(|elem| {
                        quote! {
                            String::from_utf8((<#elem as liquid_lang::ty_mapping::SolTypeName>::NAME).to_vec()).expect("the type name of a function argument must an valid utf-8 string")
                        }
                    }).collect::<Vec<_>>()
            } else {
                vec![quote! {
                    String::from_utf8((<#ty as liquid_lang::ty_mapping::SolTypeName>::NAME).to_vec()).expect("the type name of a function argument must an valid utf-8 string")
                }]
            }
        }
    }
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
        let fn_abis = external_fns.iter().map(|external_fn| {
            let ident = external_fn.sig.ident.to_string();
            let input_args = generate_fn_inputs(&external_fn.sig);
            let output_args = generate_fn_outputs(&external_fn.sig);
            let constant = !external_fn.sig.is_mut();
            let state_mutability = if constant {
                "view"
            } else {
                "nonpayable"
            };

            quote! {
                liquid_abi_gen::ExternalFnABI::new_builder(String::from(#ident), String::from(#state_mutability), #constant)
                #(.input(#input_args))*
                #(.output(#output_args))*
                .done()
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
}

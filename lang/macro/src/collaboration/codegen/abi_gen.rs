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
    collaboration::ir::{Collaboration, FnArg, Right, Signature},
    common::GenerateCode,
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[derive(From)]
pub struct AbiGen<'a> {
    collaboration: &'a Collaboration,
}

impl<'a> GenerateCode for AbiGen<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let contract_abis = self.generate_contract_abis();

        quote! {
            #[cfg(feature = "liquid-abi-gen")]
            #[allow(non_camel_case_types)]
            pub struct __LIQUID_ABI_GEN;

            #[cfg(feature = "liquid-abi-gen")]
            impl liquid_lang::GenerateAbi for __LIQUID_ABI_GEN {
                fn generate_abi() -> liquid_abi_gen::CollaborationAbi {
                    let mut contract_abis = Vec::new();
                    #(contract_abis.push(#contract_abis);)*

                    liquid_abi_gen::CollaborationAbi {
                        contract_abis,
                    }
                }
            }
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

fn generate_right_abis(rights: &[Right]) -> impl Iterator<Item = TokenStream2> + '_ {
    rights.iter().filter(|right| !right.is_internal_fn()).map(|right| {
        let sig = &right.sig;
        let ident = sig.ident.to_string();
        let input_args = generate_fn_inputs(sig);
        let output = &sig.output;
        let output_args = match output {
            syn::ReturnType::Default => quote! {},
            syn::ReturnType::Type(_, ty) => {
                quote! {
                    <#ty as liquid_abi_gen::traits::GenerateOutputs>::generate_outputs(&mut builder);
                }
            }
        };

        let constant = sig.is_self_ref() && !sig.is_mut() ;
        quote! {
            {
                let mut builder = liquid_abi_gen::RightAbi::new_builder(String::from(#ident), #constant);
                #(builder.input(#input_args);)*
                #output_args
                builder.done()
            }
        }
    })
}

impl<'a> AbiGen<'a> {
    fn generate_contract_abis(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        let contracts = &self.collaboration.contracts;
        contracts.iter().map(move |contract| {
            let contract_ident = &contract.ident.to_string();
            let fields = &contract.fields;
            let data_fields = fields.named.iter().map(|field| {
                let field_ident = field.ident.as_ref().unwrap().to_string();
                let field_ty = &field.ty;
                quote! {
                    <#field_ty as liquid_abi_gen::traits::GenerateParamAbi>::generate_param_abi(String::from(#field_ident))
                }
            });
            let data = quote! {
                {
                    let mut data = Vec::new();
                    #(data.push(#data_fields);)*
                    data
                }
            };
            let right_abis = self.collaboration.all_item_rights.iter().filter(|item_rights| {
                item_rights.ident == contract_ident
            }).map(|item_rights| {
                let rights = &item_rights.rights;
                generate_right_abis(rights.as_slice())
            }).flatten();
            quote! {
                liquid_abi_gen::ContractAbi {
                    name: String::from(#contract_ident),
                    data: #data,
                    rights: {
                        let mut rights = Vec::new();
                        #(rights.push(#right_abis);)*
                        rights
                    },
                }
            }
        })
    }
}

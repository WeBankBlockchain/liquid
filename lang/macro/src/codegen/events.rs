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
    ir::{utils, Contract},
};
use derive_more::From;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};

#[derive(From)]
pub struct Events<'a> {
    contract: &'a Contract,
}

impl<'a> GenerateCode for Events<'a> {
    fn generate_code(&self) -> TokenStream2 {
        if self.contract.events.is_empty() {
            return quote! {};
        }

        let event_enum = self.generate_event_enum();
        let topics_impls = self.generate_topics_impls();
        let emit_trait = self.generate_emit_trait();

        quote! {
            mod __liquid_event {
                #[allow(unused_imports)]
                use super::*;

                pub struct EventSigHelper<S, T> {
                    marker_s: core::marker::PhantomData<fn() -> S>,
                    marker_t: core::marker::PhantomData<fn() -> T>,
                }

                #(#topics_impls)*
                #event_enum
                #emit_trait
            }

            pub use __liquid_event::{Event, Emit};
        }
    }
}

impl<'a> Events<'a> {
    fn generate_emit_trait(&self) -> TokenStream2 {
        quote! {
            pub trait Emit {
                type Event;

                fn emit<E>(self, event: E)
                where
                    E: Into<Self::Event>;
            }

            impl Emit for liquid_lang::EnvAccess {
                type Event = Event;

                fn emit<E>(self, event: E)
                where
                    E: Into<Self::Event>
                {
                    liquid_core::env::emit(event.into())
                }
            }
        }
    }

    fn generate_event_enum(&self) -> TokenStream2 {
        let event_idents = self
            .contract
            .events
            .iter()
            .map(|item_event| &item_event.ident)
            .collect::<Vec<_>>();

        quote! {
            pub enum Event {
                #(#event_idents(#event_idents),)*
            }

            #(
                impl From<#event_idents> for Event {
                    fn from(event: #event_idents) -> Self {
                        Event::#event_idents(event)
                    }
                }
            )*

            impl liquid_primitives::Topics for Event {
                fn topics(&self) -> liquid_prelude::vec::Vec<liquid_primitives::types::hash> {
                    match self {
                        #(
                            Event::#event_idents(event) => event.topics(),
                        )*
                    }
                }
            }

            impl liquid_abi_codec::Encode for Event {
                fn encode(&self) -> Vec<u8> {
                    match self {
                        #(
                            Event::#event_idents(event) => event.encode(),
                        )*
                    }
                }
            }
        }
    }

    fn generate_topics_impls(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract.events.iter().map(move |item_event| {
            let span = item_event.span;
            let event_ident = &item_event.ident;
            let event_fields = &item_event.fields;
            let event_field_tys = event_fields.iter().enumerate().map(|(i, field)| {
                let ty = &field.ty;
                if !item_event.indexed_fields.iter().any(|index| *index == i) {
                    quote_spanned! { span =>
                        <#ty as liquid_lang::You_Should_Use_An_Valid_Event_Data_Type>::T
                    }
                } else {
                    quote_spanned! { span =>
                        <#ty as liquid_lang::You_Should_Use_An_Valid_Event_Topic_Type>::T
                    }
                }
            }).collect::<Vec<_>>();

            let event_name = event_ident.to_string();
            let event_name_bytes = event_name.as_bytes();
            let event_name_len = event_name_bytes.len();
            let sig_hash = quote_spanned! { span =>
                {
                    const SIG_LEN: usize =
                        liquid_ty_mapping::len::<(#(#event_field_tys,)*)>()
                        + #event_name_len
                        + 2;

                    const SIG: [u8; SIG_LEN] =
                        liquid_ty_mapping::composite::<(#(#event_field_tys,)*), SIG_LEN>(&[#(#event_name_bytes),*]);

                    liquid_primitives::hash::hash(&SIG)
                }
            };

            let topic_hash = {
                let calculate_topics = item_event.indexed_fields.iter().map(|index| {
                    let ident = &event_fields[*index].ident;
                    let ty = &event_fields[*index].ty;
                    quote_spanned! { span =>
                        <#ty as liquid_lang::You_Should_Use_An_Valid_Event_Topic_Type>::topic(&self.#ident)
                    }
                });

                quote! {
                    #(
                        #calculate_topics,
                    )*
                }
            };

            let mediate_encode = item_event.unindexed_fields.iter().map(|index| {
                let ident = &event_fields[*index].ident;
                let ty = &event_fields[*index].ty;
                quote! {
                    <#ty as liquid_abi_codec::MediateEncode>::encode(&self.#ident)
                }
            });

            quote_spanned! { span =>
                impl liquid_primitives::Topics for #event_ident {
                    fn topics(&self) -> liquid_prelude::vec::Vec<liquid_primitives::types::hash> {
                        [#sig_hash.into(), #topic_hash].to_vec()
                    }
                }

                impl liquid_abi_codec::Encode for #event_ident {
                    fn encode(&self) -> liquid_prelude::vec::Vec<u8> {
                        #[allow(unused_mut)]
                        let mut mediates = Vec::<liquid_abi_codec::Mediate>::new();
                        #(
                            mediates.push(#mediate_encode);
                        )*
                        liquid_abi_codec::encode_head_tail(&mediates).iter().flat_map(|word| word.to_vec()).collect()
                    }
                }
            }
        })
    }
}

#[derive(From)]
pub struct EventStructs<'a> {
    contract: &'a Contract,
}

impl<'a> GenerateCode for EventStructs<'a> {
    fn generate_code(&self) -> TokenStream2 {
        if self.contract.events.is_empty() {
            return quote! {};
        }

        let event_struts = self.generate_event_structs();

        quote! {
            #(#event_struts)*
        }
    }
}

impl<'a> EventStructs<'a> {
    fn generate_event_structs(&'a self) -> impl Iterator<Item = TokenStream2> + 'a {
        self.contract.events.iter().map(move |item_event| {
            let span = item_event.span;
            let ident = &item_event.ident;
            let attrs = utils::filter_non_liquid_attributes(&item_event.attrs);
            let mut fields = item_event.fields.clone();
            fields.iter_mut().for_each(|field| {
                field.vis = syn::Visibility::Public(syn::VisPublic {
                    pub_token: Default::default(),
                });
                field.attrs.retain(|attr| !utils::is_liquid_attribute(attr));
            });

            quote_spanned!(span =>
                #(#attrs)*
                pub struct #ident {
                    #(#fields,)*
                }
            )
        })
    }
}

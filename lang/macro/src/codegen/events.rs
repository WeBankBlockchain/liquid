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
    ir::{utils, Contract, HashType},
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

            impl liquid_core::env::types::Topics for Event {
                fn topics(&self) -> liquid_prelude::vec::Vec<liquid_core::env::types::Hash> {
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

            let mut sig_helper = quote_spanned! { span =>
                impl liquid_ty_mapping::SolTypeName for EventSigHelper<#event_ident, ()> {
                    const NAME: &'static [u8] = <() as liquid_ty_mapping::SolTypeName>::NAME;
                }
                impl liquid_ty_mapping::SolTypeNameLen for EventSigHelper<#event_ident, ()> {
                    const LEN: usize = <() as liquid_ty_mapping::SolTypeNameLen>::LEN;
                }
            };

            let event_field_tys = event_fields.iter().map(|field| &field.ty).collect::<Vec<_>>();
            for i in 1..=event_field_tys.len() {
                let tys = &event_field_tys[..i];
                let first_tys = &tys[0..i - 1];
                let rest_ty = &tys[i - 1];
                if i > 1 {
                    sig_helper.extend(quote_spanned! { span =>
                        impl liquid_ty_mapping::SolTypeName for EventSigHelper<#event_ident, (#(#tys,)*)> {
                            const NAME: &'static [u8] = {
                                const LEN: usize =
                                    <(#(#first_tys,)*) as liquid_ty_mapping::SolTypeNameLen<_>>::LEN
                                    + <#rest_ty as liquid_ty_mapping::SolTypeNameLen<_>>::LEN
                                    + 1;
                                &liquid_ty_mapping::concat::<EventSigHelper<#event_ident, (#(#first_tys,)*)>, #rest_ty, (), _, LEN>(true)
                            };
                        }
                    });
                } else {
                    sig_helper.extend(quote_spanned! { span =>
                        impl liquid_ty_mapping::SolTypeName for EventSigHelper<#event_ident, (#rest_ty,)> {
                            const NAME: &'static [u8] = <#rest_ty as liquid_ty_mapping::SolTypeName<_>>::NAME;
                        }
                    });
                }
            }

            let event_name = event_ident.to_string();
            let event_name_bytes = event_name.as_bytes();
            let event_name_len = event_name_bytes.len();
            let composite_sig = quote! {
                const SIG_LEN: usize =
                    <(#(#event_field_tys,)*) as liquid_ty_mapping::SolTypeNameLen<_>>::LEN + #event_name_len
                    + 2;
                const SIG: [u8; SIG_LEN] =
                    liquid_ty_mapping::composite::<SIG_LEN>(
                        &[#(#event_name_bytes),*],
                        <EventSigHelper<#event_ident, (#(#event_field_tys,)*)> as liquid_ty_mapping::SolTypeName<_>>::NAME);
            };

            let indexed_field_idents = item_event.indexed_fields.iter().map(|index| &event_fields[*index].ident);

            macro_rules! hash_by {
                ($t:ident) => {
                    (
                        quote! {
                            {
                                #composite_sig
                                liquid_primitives::hash::$t(&SIG)
                            }
                        },
                        quote! {
                            #(
                                {
                                    let encoded = liquid_abi_codec::Encode::encode(&self.#indexed_field_idents);
                                    if encoded.len() != 32 {
                                        liquid_primitives::hash::$t(&encoded)
                                    }
                                    else {
                                        let mut topic_hash: liquid_core::env::types::Hash = Default::default();
                                        for i in 0..topic_hash.len() {
                                            topic_hash[i] = encoded[i];
                                        }
                                        topic_hash
                                    }
                                },
                            )*
                        }
                    )
                };
            }

            let (sig_hash, topic_hash) = match self.contract.meta_info.hash_type {
                HashType::Keccak256 => {
                    hash_by!(keccak256)
                }
                HashType::SM3 => {
                    hash_by!(sm3)
                }
            };

            let mediate_encode = item_event.unindexed_fields.iter().map(|index| {
                let ident = &event_fields[*index].ident;
                let ty = &event_fields[*index].ty;
                quote! {
                    <#ty as liquid_abi_codec::MediateEncode>::encode(&self.#ident)
                }
            });

            quote! {
                #sig_helper

                impl liquid_core::env::types::Topics for #event_ident {
                    fn topics(&self) -> liquid_prelude::vec::Vec<liquid_core::env::types::Hash> {
                        [#sig_hash, #topic_hash].to_vec()
                    }
                }

                impl liquid_abi_codec::Encode for #event_ident {
                    fn encode(&self) -> liquid_prelude::vec::Vec<u8> {
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

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
    common,
    contract::ir::{FnArg, ForeignFn, Interface},
    utils as lang_utils,
};
use derive_more::From;
use either::Either;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};

#[derive(From)]
pub struct Mockable<'a> {
    interface: &'a Interface,
}

impl<'a> common::GenerateCode for Mockable<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let types = lang_utils::generate_primitive_types();
        let interface_ident = &self.interface.interface_ident;
        let mockable = self.generate_foreign_contract_mock(interface_ident);

        quote! {
            mod __liquid_mockable {
                use super::*;
                use core::cell::RefCell;
                use predicates::Predicate;
                use liquid_lang::mock::{DefaultReturner, ReturnDefault};

                #types
                #mockable
            }

            pub use __liquid_mockable::Interface;
        }
    }
}

fn generate_mock_common(foreign_fn: &ForeignFn) -> TokenStream2 {
    let sig = &foreign_fn.sig;
    let span = foreign_fn.span;

    let input_tys = common::generate_input_tys(&sig);
    let input_idents = common::generate_input_idents(&sig);
    let inputs = &sig.inputs;
    let ref_inputs = inputs
        .iter()
        .skip(1)
        .map(|arg| match arg {
            FnArg::Typed(ident_type) => {
                let ident = &ident_type.ident;
                let ty = &ident_type.ty;
                quote! { #ident: &#ty }
            }
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();

    let ref_input_tys = input_tys.iter().map(|ty| {
        quote! { &#ty }
    });

    let matcher_evals = input_idents.iter().enumerate().map(|(i, ident)| {
        let idx = syn::Index::from(i);
        quote! {
            preds.#idx.eval(#ident)
        }
    });

    let output = &sig.output;
    let output_ty = match output {
        syn::ReturnType::Default => {
            quote! { () }
        }
        syn::ReturnType::Type(_, ty) => {
            quote! { #ty }
        }
    };

    let when_generics = input_tys
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let generic_param = format!("T{}", i);
            Ident::new(&generic_param, span)
        })
        .collect::<Vec<_>>();
    let when_where_clauses =
        when_generics
            .iter()
            .zip(input_tys.iter())
            .map(|(generic_param, ty)| {
                quote! {
                    #generic_param: Predicate<#ty> + 'static
                }
            });
    let when_inputs =
        when_generics
            .iter()
            .zip(input_idents.iter())
            .map(|(generic_param, ident)| {
                quote! {
                    #ident: #generic_param
                }
            });

    let useless_params = input_idents.iter().map(|_| quote! {_});
    let inputs = inputs.iter().skip(1);

    quote! {
        pub enum Matcher {
            Always,
            Pred(Box<(#(Box<dyn Predicate<#input_tys>>,)*)>),
            Func(Box<dyn Fn(#(&#input_tys,)*) -> bool + 'static>),
        }

        impl Matcher {
            pub fn matches(&self, #(#ref_inputs,)*) -> bool {
                match self {
                    Matcher::Always => true,
                    Matcher::Pred(preds) => {
                        [#(#matcher_evals,)*].iter().all(|eval_result| *eval_result)
                    }
                    Matcher::Func(f) => f(#(#input_idents,)*),
                }
            }
        }

        pub enum Returner {
            Default,
            Func(Box<dyn FnMut(#(#input_tys,)*) -> #output_ty + 'static>),
            Exception,
        }

        pub struct Expectation {
            matcher: Matcher,
            return_fn: Returner,
        }

        impl Default for Expectation {
            fn default() -> Self {
                Self {
                    matcher: Matcher::Always,
                    return_fn: Returner::Default,
                }
            }
        }

        impl Expectation {
            pub fn call(&mut self, #(#inputs,)*) -> Option<#output_ty> {
                match self.return_fn {
                    Returner::Default => {
                        let default_value = DefaultReturner::<#output_ty>::return_default();
                        if let Some(default_value) = default_value {
                            Some(default_value)
                        } else {
                            panic!("can only return default values for types that impl `std::Default`");
                        }
                    }
                    Returner::Func(ref mut f) => Some(f(#(#input_idents,)*)),
                    Returner::Exception => None,
                }
            }

            pub fn matches(&self, #(#ref_inputs,)*) -> bool {
                self.matcher.matches(#(#input_idents,)*)
            }

            pub fn when<#(#when_generics,)*>(&mut self,#(#when_inputs,)*) -> &mut Self
            where
                #(#when_where_clauses,)*
            {
                self.matcher = Matcher::Pred(Box::new((#(Box::new(#input_idents),)*)));
                self
            }

            pub fn when_fn<F>(&mut self, f: F) -> &mut Self
            where
                F: Fn(#(#ref_input_tys,)*) -> bool + 'static,
            {
                self.matcher = Matcher::Func(Box::new(f));
                self
            }

            pub fn returns<T>(&mut self, return_value: T)
            where
                T: Clone + Into<#output_ty> + 'static,
            {
                self.return_fn =
                    Returner::Func(Box::new(move |#(#useless_params,)*| return_value.clone().into()));
            }

            pub fn returns_fn<F>(&mut self, f: F)
            where
                F: FnMut(#(#input_tys,)*) -> #output_ty + 'static,
            {
                self.return_fn = Returner::Func(Box::new(f))
            }

            pub fn throws(&mut self) {
                self.return_fn = Returner::Exception;
            }
        }
    }
}

fn generate_trivial_fn(foreign_fn: &ForeignFn, interface_ident: &Ident) -> TokenStream2 {
    let attrs = lang_utils::filter_non_liquid_attributes(foreign_fn.attrs.iter());
    let sig = &foreign_fn.sig;
    let fn_ident = &sig.ident;
    let span = foreign_fn.span;

    let mock_context_getter = match &foreign_fn.mock_context_getter {
        Some(getter) => getter.clone(),
        None => Ident::new(&format!("{}_context", fn_ident.to_string()), span),
    };

    let common = generate_mock_common(foreign_fn);

    let inputs = &sig.inputs;
    let input_idents = common::generate_input_idents(&sig);
    let no_self_inputs = inputs.iter().skip(1);

    let ref_input_idents = input_idents.iter().map(|ident| quote! {&#ident});
    let is_mut = sig.is_mut();

    let output = &sig.output;
    let output_ty = match output {
        syn::ReturnType::Default => {
            quote! { () }
        }
        syn::ReturnType::Type(_, ty) => {
            quote! { #ty }
        }
    };

    quote! {
        const _: () =  {
            #common

            thread_local!(
                static EXPECTATIONS: RefCell<Vec<Expectation>> = RefCell::new(Vec::new());
            );

            pub struct Context;

            impl Context {
                pub fn expect(&self) -> &mut Expectation {
                    EXPECTATIONS.with(|expectations| unsafe {
                        expectations.borrow_mut().push(Default::default());
                        (*expectations.as_ptr()).last_mut().unwrap()
                    })
                }
            }

            impl Drop for Context {
                fn drop(&mut self) {
                    EXPECTATIONS.with(|expectations| {
                        expectations.borrow_mut().clear();
                    });
                }
            }

            impl Interface {
                #[allow(non_snake_case)]
                pub fn #mock_context_getter() -> Context {
                    Context {}
                }
            }

            impl Interface {
                #(#attrs)*
                #[allow(non_snake_case)]
                pub fn #fn_ident(&self, #(#no_self_inputs,)*) -> Option<#output_ty> {
                    EXPECTATIONS.with(|expectations| {
                        for expectation in expectations.borrow_mut().iter_mut() {
                            if expectation.matches(#(#ref_input_idents,)*) {
                                if #is_mut {
                                    liquid_lang::storage::mutable_call_happens();
                                }
                                return expectation.call(#(#input_idents,)*);
                            }
                        }

                        panic!(
                            "no matched expectation is found for `{}({})` in `{}`",
                            stringify!(#fn_ident),
                            stringify!(#inputs)
                                .replace(" : ", ": ")
                                .replace("& self", "&self")
                                .replace("& mut", "&mut"),
                            stringify!(#interface_ident),
                        );
                    })
                }
            }
        };
    }
}

impl<'a> Mockable<'a> {
    fn generate_foreign_contract_mock(&self, interface_ident: &Ident) -> TokenStream2 {
        let interface = self.interface;
        let span = interface.span;

        let trivial_mocks = interface
            .foreign_fns
            .iter()
            .map(|(_, foreign_fn)| generate_trivial_fn(foreign_fn, interface_ident))
            .collect::<Vec<_>>();

        quote_spanned! { span =>
            #[derive(Debug, Clone)]
            pub struct Interface;

            impl Interface {
                pub fn at(_: liquid_primitives::types::Address) -> Self {
                    Self {}
                }
            }

            impl From<liquid_primitives::types::Address> for Interface {
                fn from(addr: liquid_primitives::types::Address) -> Interface {
                    Self::at(addr)
                }
            }

            impl scale::Decode for Interface {
                fn decode<I: scale::Input>(value: &mut I) -> ::core::result::Result<Self, scale::Error> {
                    let _ = <() as scale::Decode>::decode(value)?;
                    Ok(Self {})
                }
            }

            impl scale::Encode for Interface {
                fn encode(&self) -> Vec<u8> {
                    ().encode()
                }
            }

            impl liquid_lang::You_Should_Use_An_Valid_State_Type for Interface {}

            #(#trivial_mocks)*
        }
    }
}

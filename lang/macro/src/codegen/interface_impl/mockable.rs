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
    ir::{utils as ir_utils, FnArg, ForeignFn, Interface},
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

impl<'a> GenerateCode for Mockable<'a> {
    fn generate_code(&self) -> TokenStream2 {
        let types = codegen_utils::generate_primitive_types();
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

fn generate_mock_common(foreign_fn: &ForeignFn, suffix: usize) -> TokenStream2 {
    let sig = &foreign_fn.sig;
    let span = foreign_fn.span;

    let inputs = &sig.inputs;
    let input_tys = codegen_utils::generate_input_tys(&sig);
    let input_idents = codegen_utils::generate_input_idents(inputs);
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

    let matcher = Ident::new(&format!("Matcher{}", suffix), span);
    let returner = Ident::new(&format!("Returner{}", suffix), span);
    let expectation = Ident::new(&format!("Expectation{}", suffix), span);

    let inputs = inputs.iter().skip(1);

    quote! {
        pub enum #matcher {
            Always,
            Pred(Box<(#(Box<dyn Predicate<#input_tys>>,)*)>),
            Func(Box<dyn Fn(#(&#input_tys,)*) -> bool + 'static>),
        }

        impl #matcher {
            pub fn matches(&self, #(#ref_inputs,)*) -> bool {
                match self {
                    #matcher::Always => true,
                    #matcher::Pred(preds) => {
                        [#(#matcher_evals,)*].iter().all(|eval_result| *eval_result)
                    }
                    #matcher::Func(f) => f(#(#input_idents,)*),
                }
            }
        }

        pub enum #returner {
            Default,
            Func(Box<dyn FnMut(#(#input_tys,)*) -> #output_ty + 'static>),
            Exception,
        }

        pub struct #expectation {
            matcher: #matcher,
            return_fn: #returner,
        }

        impl Default for #expectation {
            fn default() -> Self {
                Self {
                    matcher: #matcher::Always,
                    return_fn: #returner::Default,
                }
            }
        }

        impl #expectation {
            pub fn call(&mut self, #(#inputs,)*) -> Option<#output_ty> {
                match self.return_fn {
                    #returner::Default => {
                        let default_value = DefaultReturner::<#output_ty>::return_default();
                        if let Some(default_value) = default_value {
                            Some(default_value)
                        } else {
                            panic!("can only return default values for types that impl `std::Default`");
                        }
                    }
                    #returner::Func(ref mut f) => Some(f(#(#input_idents,)*)),
                    #returner::Exception => None,
                }
            }

            pub fn matches(&self, #(#ref_inputs,)*) -> bool {
                self.matcher.matches(#(#input_idents,)*)
            }

            pub fn when<#(#when_generics,)*>(&mut self,#(#when_inputs,)*) -> &mut Self
            where
                #(#when_where_clauses,)*
            {
                self.matcher = #matcher::Pred(Box::new((#(Box::new(#input_idents),)*)));
                self
            }

            pub fn when_fn<F>(&mut self, f: F) -> &mut Self
            where
                F: Fn(#(#ref_input_tys,)*) -> bool + 'static,
            {
                self.matcher = #matcher::Func(Box::new(f));
                self
            }

            pub fn returns<T>(&mut self, return_value: T)
            where
                T: Clone + Into<#output_ty> + 'static,
            {
                self.return_fn =
                    #returner::Func(Box::new(move |#(#useless_params,)*| return_value.clone().into()));
            }

            pub fn returns_fn<F>(&mut self, f: F)
            where
                F: FnMut(#(#input_tys,)*) -> #output_ty + 'static,
            {
                self.return_fn = #returner::Func(Box::new(f))
            }

            pub fn throws(&mut self) {
                self.return_fn = #returner::Exception;
            }
        }
    }
}

fn generate_trivial_fn(foreign_fn: &ForeignFn, interface_ident: &Ident) -> TokenStream2 {
    let attrs = ir_utils::filter_non_liquid_attributes(foreign_fn.attrs.iter());
    let sig = &foreign_fn.sig;
    let fn_ident = &sig.ident;
    let span = foreign_fn.span;

    let mock_context_getter = match &foreign_fn.mock_context_getter {
        Some(getter) => getter.clone(),
        None => Ident::new(&format!("{}_context", fn_ident.to_string()), span),
    };

    let common = generate_mock_common(foreign_fn, 0);

    let inputs = &sig.inputs;
    let input_idents = codegen_utils::generate_input_idents(inputs);
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
                static EXPECTATIONS: RefCell<Vec<Expectation0>> = RefCell::new(Vec::new());
            );

            pub struct Context;

            impl Context {
                pub fn expect(&self) -> &mut Expectation0 {
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

            impl InterfaceImpl {
                #(#attrs)*
                #[allow(non_snake_case)]
                pub fn #fn_ident(&self, #(#no_self_inputs,)*) -> Option<#output_ty> {
                    EXPECTATIONS.with(|expectations| {
                        for expectation in expectations.borrow_mut().iter_mut() {
                            if expectation.matches(#(#ref_input_idents,)*) {
                                if #is_mut {
                                    liquid_core::storage::mutable_call_happens();
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

fn generate_overloaded_fn(
    fn_ident: &Ident,
    foreign_fns: &[ForeignFn],
    interface_ident: &Ident,
) -> TokenStream2 {
    let mock_context_getter = match &foreign_fns[0].mock_context_getter {
        Some(getter) => getter.clone(),
        None => Ident::new(
            &format!("{}_context", fn_ident.to_string()),
            foreign_fns[0].span,
        ),
    };

    let all_expectations_drop = foreign_fns.iter().enumerate().map(|(i, foreign_fn)| {
        let expectations = Ident::new(&format!("EXPECTATIONS{}", i), foreign_fn.span);

        quote! {
            #expectations.with(|expectations| {
                expectations.borrow_mut().clear();
            });
        }
    });

    let overloaded_mocks = foreign_fns.iter().enumerate().map(|(i, foreign_fn)| {
        let sig = &foreign_fn.sig;
        let span = foreign_fn.span;

        let inputs = &sig.inputs;
        let input_tys = codegen_utils::generate_input_tys(&sig);
        let input_idents = codegen_utils::generate_input_idents(inputs);

        let ref_input_idents = input_idents.iter().map(|ident| quote! {&#ident});
        let is_mut = sig.is_mut();

        let output = &sig.output;
        let output_ty = match output {
            syn::ReturnType::Default => {
                quote! { () }
            }
            syn::ReturnType::Type(_, ty) => {
                quote! { #ty }
            },
        };

        let common = generate_mock_common(foreign_fn, i);
        let call_expectation = Ident::new(&format!("call_expectation{}", i), span);
        let expectation = Ident::new(&format!("Expectation{}", i), span);
        let expectations = Ident::new(&format!("EXPECTATIONS{}", i), span);

        quote! {
            #common

            thread_local! (
                static #expectations: RefCell<Vec<#expectation>> = RefCell::new(Vec::new());
            );

            impl ExpectationTarget for (#(#input_tys,)*) {
                type E = #expectation;

                fn return_expectation() -> &'static mut Self::E {
                    #expectations.with(|expectations| unsafe {
                        expectations.borrow_mut().push(Default::default());
                        (*expectations.as_ptr()).last_mut().unwrap()
                    })
                }
            }

            impl #fn_ident {
                fn #call_expectation((#(#input_idents,)*): (#(#input_tys,)*)) -> Option<#output_ty> {
                    #expectations.with(|expectations| {
                        for expectation in expectations.borrow_mut().iter_mut() {
                            if expectation.matches(#(#ref_input_idents,)*) {
                                if #is_mut {
                                    liquid_core::storage::mutable_call_happens();
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
                            stringify!(#interface_ident)
                        );
                    })
                }
            }

            impl FnOnce<(#(#input_tys,)*)> for #fn_ident {
                type Output = Option<#output_ty>;
                extern "rust-call" fn call_once(self, args: (#(#input_tys,)*)) -> Self::Output {
                    Self::#call_expectation(args)
                }
            }

            impl FnMut<(#(#input_tys,)*)> for #fn_ident {
                extern "rust-call" fn call_mut(&mut self, args: (#(#input_tys,)*)) -> Self::Output {
                    Self::#call_expectation(args)
                }
            }

            impl Fn<(#(#input_tys,)*)> for #fn_ident {
                extern "rust-call" fn call(&self, args: (#(#input_tys,)*)) -> Self::Output {
                    Self::#call_expectation(args)
                }
            }
        }
    });

    quote! {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone)]
        pub struct #fn_ident;

        const _: () = {
            #(#overloaded_mocks)*

            pub trait ExpectationTarget {
                type E;

                fn return_expectation() -> &'static mut Self::E;
            }

            pub struct Context;

            impl Context {
                pub fn expect<T: ExpectationTarget>(&self) -> &'static mut T::E {
                    T::return_expectation()
                }
            }

            impl Drop for Context {
                fn drop(&mut self) {
                    #(
                        #all_expectations_drop
                    )*
                }
            }

            impl Interface {
                #[allow(non_snake_case)]
                pub fn #mock_context_getter() -> Context {
                    Context {}
                }
            }
        };
    }
}

impl<'a> Mockable<'a> {
    fn generate_foreign_contract_mock(&self, interface_ident: &Ident) -> TokenStream2 {
        let interface = self.interface;
        let span = interface.span;

        let (trivial_mocks, overloaded_fns): (Vec<_>, Vec<_>) =
            interface.foreign_fns.iter().partition_map(|(ident, fns)| {
                if fns.len() == 1 {
                    let trivial_fn = fns.first().unwrap();
                    Either::Left(generate_trivial_fn(trivial_fn, interface_ident))
                } else {
                    Either::Right((
                        ident,
                        generate_overloaded_fn(ident, fns, interface_ident),
                    ))
                }
            });
        let (overloaded_idents, overloaded_mocks): (Vec<_>, Vec<_>) =
            overloaded_fns.into_iter().unzip();

        quote_spanned! { span =>
            #[derive(Debug, Clone)]
            pub struct InterfaceImpl {
                #(
                    pub #overloaded_idents: #overloaded_idents,
                )*
            }

            #[derive(Debug, Clone)]
            pub struct Interface(InterfaceImpl);

            impl Interface {
                pub fn at(_: liquid_primitives::types::Address) -> Self {
                    Self(InterfaceImpl {
                        #(
                            #overloaded_idents: #overloaded_idents {},
                        )*
                    })
                }
            }

            impl From<liquid_primitives::types::Address> for Interface {
                fn from(addr: liquid_primitives::types::Address) -> Interface {
                    Self::at(addr)
                }
            }

            impl scale::Decode for Interface {
                fn decode<I: scale::Input>(value: &mut I) -> Result<Self, scale::Error> {
                    let _ = <() as scale::Decode>::decode(value)?;
                    Ok(Self(InterfaceImpl {
                        #(
                            #overloaded_idents: #overloaded_idents {},
                        )*
                    }))
                }
            }

            impl scale::Encode for Interface {
                fn encode(&self) -> Vec<u8> {
                    ().encode()
                }
            }

            impl std::ops::Deref for Interface {
                type Target = InterfaceImpl;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
            }

            #(#trivial_mocks)*

            #(#overloaded_mocks)*
        }
    }
}

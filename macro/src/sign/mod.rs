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

#![allow(dead_code)]

use crate::utils::*;
use proc_macro2::{
    token_stream::IntoIter as TokenIter, Ident, Spacing, Span,
    TokenStream as TokenStream2, TokenTree,
};
use quote::quote;

pub fn sign_impl(input: TokenStream2) -> Result<TokenStream2> {
    let mut iter = input.into_iter();
    let ident = expect_ident(&mut iter)?;
    expect_right_arrow(&mut iter)?;

    let mut expr_construct = quote! { #(#iter)* };
    let expr_struct = syn::parse2::<syn::ExprStruct>({
        let expr = expr_construct.clone();
        quote! {
            #ident {
                #expr
            }
        }
    })
    .map_err(|err| {
        let message = err.to_string();
        let span = err.span();
        SyntaxError { message, span }
    })?;
    if expr_struct.dot2_token.is_some() {
        if let Some(rest) = &expr_struct.rest {
            let mut expr = rest.as_ref();
            while let syn::Expr::Paren(expr_paren) = expr {
                expr = expr_paren.expr.as_ref();
            }
            if let syn::Expr::Path(expr_path) = expr {
                if let Some(path) = expr_path.path.get_ident() {
                    if path == "self" {
                        let fields = &expr_struct.fields;
                        expr_construct = quote! {
                            ..{
                                let cloned = Self {
                                    #fields
                                    ..self
                                };
                                unsafe {
                                    ::core::mem::transmute::<_, #ident>(cloned)
                                }
                            }
                        };
                    }
                }
            }
        }
    };

    Ok(quote! {
        {
            type T = <#ident as liquid_lang::ContractType>::T;
            let contract = T {
                #expr_construct
            };
            <ContractId<T> as liquid_lang::ContractVisitor>::sign_new_contract(contract)
        }
    })
}

pub fn expect_ident(iter: &mut TokenIter) -> Result<Ident> {
    match next_token(iter)? {
        TokenTree::Ident(ident) => Ok(ident),
        other => Err(SyntaxError::new("expected ident".to_owned(), &other)),
    }
}

pub fn expect_right_arrow(iter: &mut TokenIter) -> Result<()> {
    match next_token(iter)? {
        TokenTree::Punct(punct)
            if punct.as_char() != '=' || punct.spacing() == Spacing::Joint =>
        {
            match next_token(iter)? {
                TokenTree::Punct(punct)
                    if punct.as_char() != '>' || punct.spacing() == Spacing::Alone =>
                {
                    Ok(())
                }
                other => Err(SyntaxError::new("expected ident".to_owned(), &other)),
            }
        }
        other => Err(SyntaxError::new("expected ident".to_owned(), &other)),
    }
}

fn next_token(iter: &mut TokenIter) -> Result<TokenTree> {
    iter.next().ok_or_else(|| SyntaxError {
        message: "unexpected end of input".to_owned(),
        span: Span::call_site(),
    })
}

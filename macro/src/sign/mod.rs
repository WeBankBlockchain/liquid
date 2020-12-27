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

    Ok(quote! {
        {
            let collection = <ContractId::<#ident> as liquid_lang::FetchContract<#ident>>::fetch_collection();
            let contract = #ident {
                #(#iter)*
            };

            let signers = <#ident as liquid_lang::AcquireSigners>::acquire_signers(&contract);
            if signers.is_empty() {
                liquid_lang::env::revert(&String::from(ContractId::<#ident>::NO_AVAILABLE_SIGNERS_ERROR));
            }

            if !__liquid_authorization_check(&signers) {
                liquid_lang::env::revert(&String::from(ContractId::<#ident>::UNAUTHORIZED_SIGNING_ERROR));
                unreachable!();
            }

            let len = collection.len();
            collection.insert(&len, (contract, false));
            let (__liquid_contract, _) = collection.get_mut(&len).unwrap();
            let ptrs = <ContractId::<#ident> as FetchContract<#ident>>::fetch_ptrs();
            ptrs.push(__liquid_contract as *const #ident);

            ContractId::<#ident> {
                __liquid_id: len,
                __liquid_marker: Default::default(),
            }
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

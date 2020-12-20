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

use crate::utils::*;
use core::iter::FromIterator;
use liquid_prelude::vec;
use proc_macro2::{Ident, Spacing, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;

pub fn create_impl(input: TokenStream2) -> Result<TokenStream2> {
    let tokens = Vec::from_iter(input);
    let mut has_struct_update_syntax = false;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(punct) => {
                if punct.as_char() == '.' && punct.spacing() == Spacing::Joint {
                    if i + 1 < tokens.len() {
                        match &tokens[i + 1] {
                            TokenTree::Punct(punct) => {
                                if punct.as_char() == '.'
                                    && punct.spacing() == Spacing::Alone
                                {
                                    has_struct_update_syntax = true;
                                    break;
                                }
                            }
                            _ => {
                                i += 1;
                                continue;
                            }
                        }
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                    continue;
                }
            }
            _ => {
                i += 1;
                continue;
            }
        }
    }
    let mut iter = tokens.into_iter();

    let ident = expect_ident(&mut iter)?;
    expect_right_arrow(&mut iter)?;
    let implicit_field_assign = if has_struct_update_syntax {
        quote! {}
    } else {
        quote! { __liquid_forbids_constructing_contract: Default::default() }
    };
    Ok(quote! {
        {
            let contract_collection = <#ident as liquid_lang::This_Contract_Type_Is_Not_Exist>::fetch();
            let contract = #ident {
                #(#iter)*
                #implicit_field_assign
            };

            let storage = __liquid_acquire_storage_instance();
            let signers = <#ident as liquid_lang::AcquireSigners>::acquire_signers(&contract);
            if signers.is_empty() {
                liquid_lang::env::revert(&#ident::__LIQUID_NO_AVAILABLE_SIGNERS_ERROR.to_owned())
            }
            let authorizers = &storage.__liquid_authorizers;
            for signer in signers {
                if !authorizers.contains(&signer) {
                    liquid_lang::env::revert(&#ident::__LIQUID_UNAUTHORIZED_CREATE_ERROR.to_owned());
                }
            }

            let len = contract_collection.len();
            contract_collection.insert(&len, (contract, false));

            liquid_lang::ContractId::<#ident> {
                __liquid_index: len,
                __liquid_marker: Default::default(),
            }
        }
    })
}

pub fn expect_ident(iter: &mut vec::IntoIter<TokenTree>) -> Result<Ident> {
    match next_token(iter)? {
        TokenTree::Ident(ident) => Ok(ident),
        other => Err(SyntaxError::new("expected ident".to_owned(), &other)),
    }
}

pub fn expect_right_arrow(iter: &mut vec::IntoIter<TokenTree>) -> Result<()> {
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

fn next_token(iter: &mut vec::IntoIter<TokenTree>) -> Result<TokenTree> {
    iter.next().ok_or_else(|| SyntaxError {
        message: "unexpected end of input".to_owned(),
        span: Span::call_site(),
    })
}

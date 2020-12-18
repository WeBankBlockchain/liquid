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
use proc_macro2::{
    token_stream::IntoIter as TokenIter, Ident, Span, TokenStream as TokenStream2,
    TokenTree,
};
use quote::quote;

pub fn create_impl(input: TokenStream2) -> Result<TokenStream2> {
    let mut iter = input.into_iter();
    let ident = expect_ident(&mut iter)?;
    Ok(quote! {
        liquid_lang::ContractId::<#ident> {
            __liquid_index: 0,
            __liquid_marker: Default::default(),
        }
    })
}

pub fn expect_ident(iter: &mut TokenIter) -> Result<Ident> {
    match next_token(iter)? {
        TokenTree::Ident(ident) => Ok(ident),
        other => Err(SyntaxError::new("expected ident".to_owned(), &other)),
    }
}

fn next_token(iter: &mut TokenIter) -> Result<TokenTree> {
    iter.next().ok_or_else(|| SyntaxError {
        message: "unexpected end of input".to_owned(),
        span: Span::call_site(),
    })
}

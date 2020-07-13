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

extern crate proc_macro;

mod parse;

use core::iter::{self, FromIterator};
use parse::*;
use proc_macro::TokenStream;
use proc_macro2::{
    Delimiter, Group, Ident, Literal, TokenStream as TokenStream2, TokenTree,
};

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    match seq_impl(input.into()) {
        Ok(expanded) => expanded.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

#[derive(Copy, Clone)]
struct Range {
    begin: u64,
    end: u64,
    inclusive: bool,
}

impl IntoIterator for Range {
    type Item = u64;
    type IntoIter = Box<dyn Iterator<Item = u64>>;

    fn into_iter(self) -> Self::IntoIter {
        if self.inclusive {
            Box::new(self.begin..=self.end)
        } else {
            Box::new(self.begin..self.end)
        }
    }
}

fn seq_impl(input: TokenStream2) -> Result<TokenStream2> {
    let iter = &mut input.into_iter();
    let var = expect_ident(iter)?;
    expect_keyword(iter, "in")?;
    let begin = expect_integer(iter)?;
    expect_punct(iter, '.')?;
    expect_punct(iter, '.')?;
    let inclusive = expect_optional_punct(iter, '=')?;
    let end = expect_integer(iter)?;
    let body = expect_body(iter)?;
    expect_end(iter)?;

    let range = Range {
        begin,
        end,
        inclusive,
    };

    let mut has_repetition = false;
    let expanded = expand_repetitions(&var, range, body.clone(), &mut has_repetition);
    if has_repetition {
        Ok(expanded)
    } else {
        Ok(repeat(&var, range, body))
    }
}

fn repeat(var: &Ident, range: Range, body: TokenStream2) -> TokenStream2 {
    let mut expanded = TokenStream2::new();
    for number in range {
        expanded.extend(substitute_number(var, number, body.clone()))
    }
    expanded
}

fn capture_repetition(tokens: &[TokenTree]) -> Option<TokenStream2> {
    assert_eq!(tokens.len(), 3);
    match &tokens[0] {
        TokenTree::Punct(punct) if punct.as_char() == '#' => {}
        _ => return None,
    }

    match &tokens[2] {
        TokenTree::Punct(punct) if punct.as_char() == '*' => {}
        _ => return None,
    }

    match &tokens[1] {
        TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
            Some(group.stream())
        }
        _ => None,
    }
}

fn substitute_number(var: &Ident, number: u64, body: TokenStream2) -> TokenStream2 {
    let mut tokens = Vec::from_iter(body);

    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Ident(ident) if ident == var => {
                let original_span = tokens[i].span();
                let mut literal = Literal::u64_suffixed(number);
                literal.set_span(original_span);
                tokens[i] = TokenTree::Literal(literal);
                i += 1;
                continue;
            }
            _ => (),
        };

        if i + 3 <= tokens.len() {
            let prefix = match &tokens[i..i + 3] {
                [TokenTree::Ident(prefix), TokenTree::Punct(pound), TokenTree::Ident(ident)]
                    if pound.as_char() == '#' && ident == var =>
                {
                    Some(prefix)
                }
                _ => None,
            };
            if let Some(prefix) = prefix {
                let concat = format!("{}{}", prefix, number);
                let ident = Ident::new(&concat, prefix.span());
                tokens.splice(i..i + 3, iter::once(TokenTree::Ident(ident)));
                i += 1;
                continue;
            }
        }

        if let TokenTree::Group(group) = &mut tokens[i] {
            let content = substitute_number(var, number, group.stream());
            let original_span = group.span();
            *group = Group::new(group.delimiter(), content);
            group.set_span(original_span);
        }

        i += 1;
    }

    TokenStream2::from_iter(tokens)
}

fn expand_repetitions(
    var: &Ident,
    range: Range,
    body: TokenStream2,
    has_repetition: &mut bool,
) -> TokenStream2 {
    let mut tokens = Vec::from_iter(body);

    let mut i = 0;
    while i < tokens.len() {
        if let TokenTree::Group(group) = &mut tokens[i] {
            let content = expand_repetitions(var, range, group.stream(), has_repetition);
            let original_span = group.span();
            *group = Group::new(group.delimiter(), content);
            group.set_span(original_span);
            i += 1;
            continue;
        }

        if i + 3 > tokens.len() {
            i += 1;
            continue;
        }

        let template = match capture_repetition(&tokens[i..i + 3]) {
            Some(template) => template,
            None => {
                i += 1;
                continue;
            }
        };

        *has_repetition = true;
        let mut repeated = Vec::new();
        for number in range {
            repeated.extend(substitute_number(var, number, template.clone()));
        }

        let repeated_len = repeated.len();
        tokens.splice(i..i + 3, repeated);
        i += repeated_len;
    }

    TokenStream2::from_iter(tokens)
}

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
    collaboration::{
        ir::{
            ItemContract, ItemRights, LiquidItem, Marker, SelectFrom, SelectWith,
            Selector,
        },
        obj_path,
    },
    utils as lang_utils,
};
use core::{
    cmp::{max, min},
    convert::TryFrom,
};
use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use syn::Result;

pub fn filter_map_liquid_attributes<'a, I>(attrs: I) -> Result<Vec<Marker>>
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    let mut markers = Vec::new();
    for attr in attrs {
        if lang_utils::is_liquid_attribute(attr) {
            let marker = Marker::try_from(attr.clone());
            if let Ok(marker) = marker {
                markers.push(marker);
            } else {
                return Err(marker.unwrap_err());
            }
        }
    }

    Ok(markers)
}

pub type CollaborationItems = (Vec<ItemContract>, Vec<ItemRights>);

pub fn split_items(items: Vec<LiquidItem>, span: Span) -> Result<CollaborationItems> {
    use either::Either;
    use itertools::Itertools;

    let (contracts, impl_blocks): (Vec<_>, Vec<_>) =
        items.into_iter().partition_map(|item| match item {
            LiquidItem::Contract(contract) => Either::Left(contract),
            LiquidItem::Rights(rights) => Either::Right(rights),
        });

    if contracts.is_empty() {
        return Err(format_err_span!(
            span,
            "no `#[liquid(contract)]` struct found in this collaboration"
        ));
    }

    for impl_block in &impl_blocks {
        if !contracts
            .iter()
            .any(|contract| contract.ident == impl_block.ty)
        {
            bail!(
                impl_block.ty,
                "the rights need to be associated with an existed `#[liquid(contract)]` \
                 struct"
            )
        }
    }

    Ok((contracts, impl_blocks))
}

fn check_non_ascii(input: &str, span: Span) -> Result<()> {
    if !input.is_ascii() {
        bail_span! {
            span,
            "the path contains non-ASCII character(s)",
        }
    }
    Ok(())
}

pub fn parse_select_path(path: &str, span: Span) -> Result<SelectWith> {
    fn respan_token_stream(stream: TokenStream, span: Span) -> TokenStream {
        stream
            .into_iter()
            .map(|token| respan_token_tree(token, span))
            .collect()
    }

    fn respan_token_tree(mut token: TokenTree, span: Span) -> TokenTree {
        if let TokenTree::Group(group) = &mut token {
            *group =
                Group::new(group.delimiter(), respan_token_stream(group.stream(), span));
        }

        token.set_span(span);
        token
    }

    if path.starts_with("::") {
        let stream = syn::parse_str(&path)?;
        let tokens = respan_token_stream(stream, span);
        let expr_path = syn::parse2::<syn::ExprPath>(tokens)?;
        Ok(SelectWith::Func(expr_path))
    } else {
        check_non_ascii(path, span)?;

        let parser = obj_path::Parser::new(&path);
        match parser.parse() {
            Ok(ast) => Ok(SelectWith::Obj(ast)),
            Err(err) => {
                let err_span = err.span;
                let path_len = path.len();
                let start = max(0, err_span.start - 3);
                let end = min(path_len, err_span.end + 3);

                let snippet = format!(
                    "{}{}{}",
                    if start == 0 { "" } else { ".." },
                    &path[start..end],
                    if end == path_len { "" } else { ".." },
                );
                bail_span! {
                    span,
                    "around \"{}\", {}", snippet, err.msg
                }
            }
        }
    }
}

pub struct OwnersParser<'a> {
    input: &'a [u8],
    cur_pos: usize,
    allow_select_from_arg: bool,
    span: Span,
}

pub type Selectors = Vec<Selector>;

impl<'a> OwnersParser<'a> {
    pub fn new(input: &'a str, span: Span, allow_select_from_arg: bool) -> Result<Self> {
        check_non_ascii(input, span)?;

        Ok(Self {
            input: input.as_bytes(),
            allow_select_from_arg,
            cur_pos: 0,
            span,
        })
    }

    pub fn parse(mut self) -> Result<Selectors> {
        self.eat_whitespace();
        if self.is_end() {
            return Ok(vec![]);
        }
        self.parse_owners()
    }

    fn parse_owners(&mut self) -> Result<Selectors> {
        self.eat_whitespace();
        let owner = self.parse_owner()?;
        self.parse_other_owners(vec![owner])
    }

    fn parse_owner(&mut self) -> Result<Selector> {
        self.eat_whitespace();
        let from = self.parse_from()?;
        self.eat_whitespace();
        if self.is_end() || self.peek_char() == ',' {
            return Ok(Selector { from, with: None });
        }

        let with = match self.next_char() {
            '{' => {
                let end = self
                    .input
                    .iter()
                    .skip(self.cur_pos)
                    .position(|ch| (*ch as char) == '}');
                if end.is_none() {
                    bail_span!(
                        self.span,
                        "no matched right bracket(`}}`) found for left bracket(`{{`) at \
                         position {}",
                        self.cur_pos - 1
                    )
                }

                let end = end.unwrap();
                let select_path =
                    String::from_utf8(self.input[self.cur_pos..end].to_vec()).unwrap();
                let with = parse_select_path(&select_path, self.span)?;
                self.cur_pos = end + 1;
                Some(with)
            }
            ch => bail_span!(
                self.span,
                "unexpected character `{}` found at position `{}`, expected `{{`",
                ch,
                self.cur_pos - 1
            ),
        };

        Ok(Selector { from, with })
    }

    fn parse_other_owners(&mut self, mut prev: Vec<Selector>) -> Result<Selectors> {
        self.eat_whitespace();
        if self.is_end() {
            return Ok(prev);
        }

        match self.next_char() {
            ',' => {
                let owner = self.parse_owner()?;
                prev.push(owner);
                self.parse_other_owners(prev)
            }
            ch => bail_span!(
                self.span,
                "unexpected character `{}` found at position `{}`, expected `,`",
                ch,
                self.cur_pos - 1
            ),
        }
    }

    fn parse_from(&mut self) -> Result<SelectFrom> {
        self.eat_whitespace();
        match self.peek_char() {
            '^' if self.allow_select_from_arg => {
                self.next_char();
                let ident = self.parse_ident()?;
                Ok(SelectFrom::Argument(Ident::new(&ident, Span::call_site())))
            }
            ch if ch.is_ascii_alphabetic() || ch == '_' => {
                let ident = self.parse_ident()?;
                Ok(SelectFrom::This(Ident::new(&ident, Span::call_site())))
            }
            ch => bail_span!(
                self.span,
                "unexpected character `{}` found at position `{}`, expect `^`, `_` or \
                 alphabet",
                ch,
                self.cur_pos - 1
            ),
        }
    }

    fn parse_ident(&mut self) -> Result<String> {
        self.eat_whitespace();
        let mut ident = String::new();
        let ch = self.next_char();
        if ch.is_ascii_alphabetic() || ch == '_' {
            ident.push(ch);
        } else {
            bail_span!(
                self.span,
                "illegal start of identifier `{}` at position {}, expect `_` or alphabet",
                self.cur_pos - 1,
                ch
            )
        }

        loop {
            if self.is_end() {
                return Ok(ident);
            }
            let ch = self.peek_char();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.eat_char();
                ident.push(ch);
            } else {
                return Ok(ident);
            }
        }
    }

    fn is_end(&self) -> bool {
        self.cur_pos >= self.input.len()
    }

    fn eat_whitespace(&mut self) {
        loop {
            if !self.is_end() && (self.input[self.cur_pos] as char).is_ascii_whitespace()
            {
                self.cur_pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> char {
        assert!(!self.is_end());
        self.input[self.cur_pos] as char
    }

    fn next_char(&mut self) -> char {
        let ch = self.peek_char();
        self.cur_pos += 1;
        ch
    }

    fn eat_char(&mut self) {
        let _ = self.next_char();
    }
}

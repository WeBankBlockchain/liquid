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

use core::iter::FromIterator;
use proc_macro2::{
    Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream as TokenStream2,
    TokenTree,
};

pub struct SyntaxError {
    pub message: String,
    pub span: Span,
}

impl SyntaxError {
    pub fn new(message: String, token: &TokenTree) -> Self {
        Self {
            message,
            span: token.span(),
        }
    }

    pub fn into_compile_error(self) -> TokenStream2 {
        let token_stream = [
            TokenTree::Ident(Ident::new("compile_error", self.span)),
            TokenTree::Punct({
                let mut punct = Punct::new('!', Spacing::Alone);
                punct.set_span(self.span);
                punct
            }),
            TokenTree::Group({
                let mut group = Group::new(
                    Delimiter::Brace,
                    TokenStream2::from_iter(
                        [TokenTree::Literal({
                            let mut string = Literal::string(&self.message);
                            string.set_span(self.span);
                            string
                        })]
                        .to_vec(),
                    ),
                );
                group.set_span(self.span);
                group
            }),
        ];
        TokenStream2::from_iter(token_stream.to_vec())
    }
}

pub type Result<T> = core::result::Result<T, SyntaxError>;

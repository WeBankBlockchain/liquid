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

mod into;
mod syn_def;
mod utils;

pub use syn_def::{
    Collaboration, FnArg, IdentType, Item, ItemContract, ItemRights, LiquidItem, Marker,
    Right, RustItem, SelectFrom, SelectWith, Selector, Signature,
};

use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream, Result},
    spanned::Spanned,
};

#[derive(Debug)]
pub enum AttrValue {
    LitStr(syn::LitStr),
    Ident(syn::Ident),
    None,
}

impl Spanned for AttrValue {
    fn span(&self) -> Span {
        match self {
            Self::LitStr(lit_str) => lit_str.span(),
            Self::Ident(ident) => ident.span(),
            Self::None => Span::call_site(),
        }
    }
}

impl Parse for AttrValue {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::Ident) {
            let ident = input.parse::<syn::Ident>()?;
            return Ok(Self::Ident(ident));
        }

        if input.peek(syn::LitStr) {
            let lit_str = input.parse::<syn::LitStr>()?;
            return Ok(Self::LitStr(lit_str));
        }

        Err(input.error(
            "invalid value of an liquid attribute, identifier or a literal string \
             required",
        ))
    }
}

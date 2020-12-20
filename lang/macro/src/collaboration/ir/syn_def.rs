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

use crate::collaboration::obj_path;
use derive_more::From;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::ToTokens;
use std::collections::BTreeMap;
use syn::{punctuated::Punctuated, spanned::Spanned, Token};

pub type RustItem = syn::Item;

/// Collaboration item.
#[derive(From)]
pub enum Item {
    Liquid(Box<LiquidItem>),
    Rust(Box<RustItem>),
}

#[derive(From)]
pub enum LiquidItem {
    Contract(ItemContract),
    Rights(ItemRights),
}

#[derive(Clone)]
pub enum SelectFrom {
    This(Ident),
    Argument(Ident),
}

impl Spanned for SelectFrom {
    /// Returns the span of the original `struct` definition.
    fn span(&self) -> Span {
        match self {
            Self::This(ident) => ident.span(),
            Self::Argument(ident) => ident.span(),
        }
    }
}

#[derive(Clone)]
pub enum SelectWith {
    Func(syn::ExprPath),
    Obj(obj_path::Ast),
}

#[derive(Clone)]
pub struct Selector {
    pub from: SelectFrom,
    pub with: Option<SelectWith>,
}

/// The description of contract.
pub struct ItemContract {
    /// Outer attributes of the contract.
    pub attrs: Vec<syn::Attribute>,
    /// The `struct` token.
    pub struct_token: Token![struct],
    /// The name of the the contract.
    pub ident: Ident,
    /// Fields of the the contract.
    pub fields: syn::FieldsNamed,
    /// Signers of the contract.
    pub field_signers: Vec<Selector>,
    /// Span of the contract.
    pub span: Span,
}

impl Spanned for ItemContract {
    /// Returns the span of the original `struct` definition.
    fn span(&self) -> Span {
        self.span
    }
}

/// The right of a contract.
pub struct ItemRights {
    /// Inner attributes.
    pub attrs: Vec<syn::Attribute>,
    /// The `impl` token.
    pub impl_token: Token![impl],
    /// The implementer type.
    pub ty: Ident,
    /// The `{` and `}` tokens.
    pub brace_token: syn::token::Brace,
    /// The rights.
    pub rights: Vec<Right>,
    /// Constants defined for the contract.
    pub constants: Vec<syn::ImplItemConst>,
}

pub struct Right {
    /// The attributes of the right.
    pub attrs: Vec<syn::Attribute>,
    /// The owners of the right.
    pub owners: Vec<Selector>,
    /// The signature of the right.
    pub sig: Signature,
    /// The body of the right.
    pub body: syn::Block,
    /// In which contract the right is declared.
    pub from: Ident,
    /// The span of the function.
    pub span: Span,
}

impl Spanned for Right {
    fn span(&self) -> Span {
        self.span
    }
}

pub struct Signature {
    /// The `fn` token.
    pub fn_token: Token![fn],
    /// The name of the function.
    pub ident: Ident,
    /// The parentheses `(` and `)`.
    pub paren_token: syn::token::Paren,
    /// The inputs of the function.
    pub inputs: Punctuated<FnArg, Token![,]>,
    /// The return type of the function.
    pub output: syn::ReturnType,
}

impl Spanned for Signature {
    fn span(&self) -> Span {
        self.fn_token
            .span()
            .join(self.output.span())
            .expect("fn token and fn output are in the same file")
    }
}

impl Signature {
    pub fn is_mut(&self) -> bool {
        self.self_arg().mutability.is_some()
    }

    pub fn is_self_ref(&self) -> bool {
        self.self_arg().reference.is_some()
    }

    pub fn self_arg(&self) -> &syn::Receiver {
        match &self.inputs[0] {
            FnArg::Receiver(receiver) => receiver,
            _ => unreachable!("first argument of a right must be an receiver"),
        }
    }
}

#[derive(From)]
pub enum FnArg {
    Receiver(Box<syn::Receiver>),
    Typed(Box<IdentType>),
}

impl ToTokens for FnArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            FnArg::Receiver(receiver) => receiver.to_tokens(tokens),
            FnArg::Typed(ident_type) => ident_type.to_tokens(tokens),
        }
    }
}

pub struct IdentType {
    /// The attributes of the argument
    pub attrs: Vec<syn::Attribute>,
    /// The mutability of the argument
    pub mutability: Option<Token![mut]>,
    /// The name of the argument.
    pub ident: Ident,
    /// The `:` token.
    pub colon_token: Token![:],
    /// The type of the argument.
    pub ty: syn::Type,
    /// The span of the argument
    pub span: Span,
}

impl ToTokens for IdentType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.attrs.iter().for_each(|attr| attr.to_tokens(tokens));
        self.mutability.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

/// markers use to indicate certain liquid specific properties.
///
/// # Note
///
/// Generally these are the subset of Rust attributes that have `liquid` as identifier.
///
/// # Examples
///
/// `#[liquid(contract)]` on a `struct` indicates that the `struct` represents a contract.
///
/// ```no_compile
/// #[liquid(contract)]
/// struct Proposal { ... }
/// ```
#[derive(Debug)]
pub struct Marker {
    /// The parentheses around the single identifier.
    pub paren_token: syn::token::Paren,
    /// The single identifier.
    pub ident: Ident,
    /// The optional attribute value assigned to the identifier.
    pub value: Option<(String, Span)>,
}

impl Spanned for Marker {
    fn span(&self) -> Span {
        self.paren_token.span
    }
}

/// The collaboration with all required information.
pub struct Collaboration {
    /// The `mod` token.
    pub mod_token: Token![mod],
    /// The modules snake case identifier.
    pub ident: Ident,
    /// All contracts.
    pub contracts: Vec<ItemContract>,
    /// All rights for each contract.
    pub all_item_rights: Vec<ItemRights>,
    /// The non-liquid items.
    pub rust_items: Vec<RustItem>,
}

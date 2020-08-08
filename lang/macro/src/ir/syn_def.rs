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

use derive_more::From;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::{punctuated::Punctuated, spanned::Spanned, Token};

/// The major, minor and patch version of the version parameter.
#[derive(Clone)]
pub struct MetaVersion {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

/// The meta info for a contract.
pub struct MetaInfo {
    pub liquid_version: MetaVersion,
}

/// Contract item.
#[derive(From)]
pub enum Item {
    Liquid(Box<LiquidItem>),
    Rust(Box<RustItem>),
}

#[derive(From)]
pub enum LiquidItem {
    Storage(ItemStorage),
    Impl(ItemImpl),
}

#[derive(From)]
pub struct RustItem {
    item: syn::Item,
}

impl ToTokens for RustItem {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.item.to_tokens(tokens);
    }
}

/// The state struct of the contract.
pub struct ItemStorage {
    /// Outer attributes of the storage struct.
    pub attrs: Vec<syn::Attribute>,
    /// The `struct` token.
    pub struct_token: Token![struct],
    /// The name of the storage struct.
    pub ident: Ident,
    /// Fields of the storage struct.
    pub fields: syn::FieldsNamed,
    /// Public fields that need to generate a corresponding getter.
    pub public_fields: Vec<usize>,
    /// Span of the storage struct.
    pub span: Span,
}

impl Spanned for ItemStorage {
    /// Returns the span of the original `struct` definition.
    fn span(&self) -> Span {
        self.span
    }
}

/// The implementation of the storage struct
pub struct ItemImpl {
    /// Inner attributes.
    pub attrs: Vec<syn::Attribute>,
    /// The `impl` token.
    pub impl_token: Token![impl],
    /// The implementer type.
    pub ty: Ident,
    /// The `{` and `}` tokens.
    pub brace_token: syn::token::Brace,
    /// Constructor and external functions.
    pub functions: Vec<Function>,
}

pub struct Function {
    /// The attributes of the function
    pub attrs: Vec<syn::Attribute>,
    /// The kind of the function
    pub kind: FunctionKind,
    /// The signature of the function
    pub sig: Signature,
    /// The body of the function
    pub body: syn::Block,
    /// The span of the function
    pub span: Span,
}

impl Spanned for Function {
    fn span(&self) -> Span {
        self.span
    }
}

pub enum FunctionKind {
    Constructor,
    Normal,
    External(usize),
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

    pub fn self_arg(&self) -> &syn::Receiver {
        match &self.inputs[0] {
            FnArg::Receiver(receiver) => receiver,
            _ => unreachable!("First argument of liquid function must be an receiver"),
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
/// `#[liquid(storage)]` on a `struct` indicates that the `struct` represents the contract's storage.
///
/// ```no_compile
/// #[liquid(storage)]
/// struct MyStorage { ... }
/// ```
pub struct Marker {
    /// The parentheses around the single identifier.
    pub paren_token: syn::token::Paren,
    /// The single identifier.
    pub ident: Ident,
}

impl Spanned for Marker {
    fn span(&self) -> Span {
        self.paren_token.span
    }
}

/// The contract with all required information.
pub struct Contract {
    /// The `mod` token.
    pub mod_token: Token![mod],
    /// The modules snake case identifier.
    pub ident: Ident,
    /// Special liquid meta attributes.
    pub meta_info: MetaInfo,
    /// The contract storage.
    pub storage: ItemStorage,
    /// Constructor function.
    pub constructor: Function,
    /// External and normal functions of the contract.
    pub functions: Vec<Function>,
    /// The non-liquid items.
    pub rust_items: Vec<RustItem>,
}

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

use crate::common::AttrValue;
use derive_more::From;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::ToTokens;
use std::collections::BTreeMap;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Result, Token,
};

/// The major, minor and patch version of the version parameter.
#[derive(Clone)]
pub struct MetaVersion {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

/// The meta info for a contract.
pub struct ContractMetaInfo {
    pub liquid_version: MetaVersion,
}

/// The meta info for an interface.
pub struct InterfaceMetaInfo {
    pub interface_name: String,
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
    Event(ItemEvent),
    Asset(ItemAsset),
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

mod kw {
    syn::custom_keyword!(issuer);
    syn::custom_keyword!(total);
    // syn::custom_keyword!(destroyable);
    syn::custom_keyword!(fungible);
    syn::custom_keyword!(description);
}

#[derive(Debug, Clone)]
pub enum AssetAttribute {
    Issuer {
        issuer_token: kw::issuer,
        eq_token: Token![=],
        value: syn::LitStr,
    },
    TotalSupply {
        total_token: kw::total,
        eq_token: Token![=],
        value: syn::LitInt,
    },
    // Destroyable {
    //     destroyable_token: kw::destroyable,
    //     eq_token: Token![=],
    //     value: syn::LitBool,
    // },
    Fungible {
        fungible_token: kw::fungible,
        eq_token: Token![=],
        value: syn::LitBool,
    },
    Description {
        description_token: kw::description,
        eq_token: Token![=],
        value: syn::LitStr,
    },
}

impl Parse for AssetAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::issuer) {
            Ok(AssetAttribute::Issuer {
                issuer_token: input.parse::<kw::issuer>()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::total) {
            Ok(AssetAttribute::TotalSupply {
                total_token: input.parse::<kw::total>()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        // } else if lookahead.peek(kw::destroyable) {
        //     Ok(AssetAttribute::Destroyable {
        //         destroyable_token: input.parse::<kw::destroyable>()?,
        //         eq_token: input.parse()?,
        //         value: input.parse()?,
        //     })
        } else if lookahead.peek(kw::description) {
            Ok(AssetAttribute::Description {
                description_token: input.parse::<kw::description>()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::fungible) {
            Ok(AssetAttribute::Fungible {
                fungible_token: input.parse::<kw::fungible>()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

/// The state struct of the contract.
pub struct ItemAsset {
    /// Outer attributes of the storage struct.
    pub attrs: Vec<syn::Attribute>,
    /// The `struct` token.
    pub struct_token: Token![struct],
    /// The name of the storage struct.
    pub ident: Ident,
    /// Span of the storage struct.
    pub span: Span,
    /// total supply
    pub total_supply: u64,
    pub issuer: String,
    // pub destroyable: bool,
    pub fungible: bool,
    pub description: String,
}

impl Spanned for ItemAsset {
    /// Returns the span of the original `struct` definition.
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug)]
pub struct AssetMetaInfo{
    pub total_supply: u64,
    pub issuer: String,
    // pub destroyable: bool,
    pub fungible: bool,
    pub description: String,
}

impl AssetMetaInfo{
    pub fn default() -> Self {
        AssetMetaInfo{
            total_supply : u64::MAX,
            issuer: String::new(),
            // destroyable: true,
            fungible: true,
            description: String::new(),
        }
    }
}

/// An event struct.
pub struct ItemEvent {
    /// Outer attributes of the event.
    pub attrs: Vec<syn::Attribute>,
    /// The `struct` token.
    pub struct_token: Token![struct],
    /// The name of the event.
    pub ident: Ident,
    /// fields of the event.
    pub fields: Vec<syn::Field>,
    /// indexed fields of the event.
    pub indexed_fields: Vec<usize>,
    /// unindexed fields of the event.
    pub unindexed_fields: Vec<usize>,
    /// Span of the event.
    pub span: Span,
}

impl Spanned for ItemEvent {
    /// Returns the span of the original `struct` definition.
    fn span(&self) -> Span {
        self.span
    }
}

/// The implementation of the storage struct.
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
    /// Constants defined for the contract.
    pub constants: Vec<syn::ImplItemConst>,
}

pub struct Function {
    /// The attributes of the function.
    pub attrs: Vec<syn::Attribute>,
    /// The kind of the function.
    pub kind: FunctionKind,
    /// The signature of the function.
    pub sig: Signature,
    /// The body of the function.
    pub body: syn::Block,
    /// The span of the function.
    pub span: Span,
}

impl Function {
    pub fn is_external_fn(&self) -> bool {
        matches!(self.kind, FunctionKind::External(_))
    }
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
            _ => unreachable!("first argument of liquid function must be an receiver"),
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

#[derive(Debug)]
pub enum AttrValue {
    LitStr(syn::LitStr),
    Ident(syn::Ident),
    Fields(Vec<AssetAttribute>),
    None,
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
#[derive(Debug)]
pub struct Marker {
    /// The parentheses around the single identifier.
    pub paren_token: syn::token::Paren,
    /// The single identifier.
    pub ident: Ident,
    /// The optional attribute value assigned to the identifier.
    pub value: AttrValue,
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
    /// Special contract meta attributes.
    pub meta_info: ContractMetaInfo,
    /// The contract storage.
    pub storage: ItemStorage,
    /// The contract events.
    pub events: Vec<ItemEvent>,
    /// The contract assets.
    pub assets: Vec<ItemAsset>,
    /// Constructor function.
    pub constructor: Function,
    /// External and normal functions of the contract.
    pub functions: Vec<Function>,
    /// Constants defined for the contract.
    pub constants: Vec<syn::ImplItemConst>,
    /// The non-liquid items.
    pub rust_items: Vec<RustItem>,
}

/// The user-defined data structure declared in an interface.
pub struct ForeignStruct {
    pub attrs: Vec<syn::Attribute>,
    /// The `struct` token.
    pub struct_token: Token![struct],
    pub ident: Ident,
    /// The named fields of the foreign struct.
    pub fields: syn::FieldsNamed,
    /// The span of the foreign struct.
    pub span: Span,
}

impl Spanned for ForeignStruct {
    fn span(&self) -> Span {
        self.span
    }
}

/// The method declared in an interface.
pub struct ForeignFn {
    pub attrs: Vec<syn::Attribute>,
    /// The signature of the foreign method.
    pub sig: Signature,
    /// The semicolon token.
    pub semi_token: Token![;],
    /// The span of the foreign method.
    pub span: Span,
    /// The name of the mock context getter.
    pub mock_context_getter: Option<Ident>,
}

impl Spanned for ForeignFn {
    fn span(&self) -> Span {
        self.span
    }
}

pub enum LangType {
    Solidity,
    Liquid,
}

/// The interface with all required information.
pub struct Interface {
    /// The `mod` token.
    pub mod_token: Token![mod],
    /// The modules snake case identifier.
    pub ident: Ident,
    /// Special interface meta attributes.
    pub meta_info: InterfaceMetaInfo,
    /// The user-defined data structures.
    pub foreign_structs: Vec<ForeignStruct>,
    /// The declarations of methods.
    pub foreign_fns: BTreeMap<Ident, Vec<ForeignFn>>,
    /// The use declarations to import other symbols.
    pub imports: Vec<syn::ItemUse>,
    /// The name of auto-generated interface struct.
    pub interface_ident: Ident,
    /// The programming language who implements this interface in remote.
    pub lang_type: LangType,
    /// The span of the interface.
    pub span: Span,
}

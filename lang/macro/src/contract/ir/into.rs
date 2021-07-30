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
    contract::{
        ir::{self, utils as ir_utils},
        SUPPORTS_ASSET_NAME, SUPPORTS_ASSET_SIGNATURE,
    },
    utils as lang_utils,
};
use core::convert::TryFrom;
use either::Either;
use heck::CamelCase;
use itertools::Itertools;
use proc_macro2::{Ident, Span};
use quote::quote;
use regex::Regex;
use std::collections::{BTreeMap, HashSet};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Result, Token,
};

const MAX_ASSET_NAME_LENGTH: usize = 32;

impl TryFrom<&String> for ir::MetaVersion {
    type Error = regex::Error;

    fn try_from(content: &String) -> core::result::Result<Self, Self::Error> {
        let re = Regex::new(
            r"(?x)
            ^(?P<major>0|[1-9]\d*) # major version
            \.
            (?P<minor>0|[1-9]\d*)  # minor version
            \.
            (?P<patch>0|[1-9]\d*)  # patch version

            (?:-
                (?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)
                (?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))
            *))?

            (?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
        ",
        )
        .unwrap();
        let captures = re
            .captures(content.as_str())
            .ok_or_else(|| regex::Error::Syntax("invalid semantic version".to_owned()))?;
        let major = captures["major"]
            .parse::<usize>()
            .expect("major version parsing cannot fail");
        let minor = captures["minor"]
            .parse::<usize>()
            .expect("minor version parsing cannot fail");
        let patch = captures["patch"]
            .parse::<usize>()
            .expect("patch version parsing cannot fail");

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl Parse for ir::Marker {
    fn parse(input: ParseStream) -> Result<Self> {
        const SINGLE_MARKER: [&str; 4] = ["indexed", "storage", "event", "methods"];

        let content;
        let paren_token = syn::parenthesized!(content in input);
        let ident = content.parse::<Ident>()?;
        if content.is_empty() {
            Ok(ir::Marker {
                paren_token,
                ident,
                value: ir::AttrValue::None,
            })
        } else if ident == "asset" {
            let attributes_content;
            syn::parenthesized!(attributes_content in content);
            let fields = attributes_content
                .parse_terminated::<ir::AssetAttribute, Token![,]>(
                    ir::AssetAttribute::parse,
                )?;
            Ok(ir::Marker {
                paren_token,
                ident,
                value: ir::AttrValue::Fields(fields.iter().cloned().collect::<Vec<_>>()),
            })
        } else {
            let ident_str = ident.to_string();
            if SINGLE_MARKER
                .iter()
                .any(|&single_marker| single_marker == ident_str)
            {
                bail_span!(
                    ident.span(),
                    "`{}` should be used without any parameters",
                    ident_str,
                )
            }
            let _ = content.parse::<Token![=]>()?;
            let value = content.parse::<ir::AttrValue>()?;
            Ok(ir::Marker {
                paren_token,
                ident,
                value,
            })
        }
    }
}

impl TryFrom<syn::Attribute> for ir::Marker {
    type Error = Error;

    fn try_from(attr: syn::Attribute) -> Result<Self> {
        syn::parse2::<Self>(attr.tokens)
    }
}

impl TryFrom<(ir::ContractParams, syn::ItemMod)> for ir::Contract {
    type Error = Error;

    fn try_from((params, item_mod): (ir::ContractParams, syn::ItemMod)) -> Result<Self> {
        if item_mod.vis != syn::Visibility::Inherited {
            bail!(
                item_mod.vis,
                "contract module must have no visibility modifier",
            )
        }

        let items = match &item_mod.content {
            None => bail!(
                item_mod,
                "contract module must be inline, e.g. `mod m {{ ... }}`",
            ),
            Some((_, items)) => items.clone(),
        };

        let (liquid_items, rust_items): (Vec<_>, Vec<_>) = items
            .into_iter()
            .map(ir::Item::try_from)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .partition_map(|item| match item {
                ir::Item::Liquid(liquid_item) => Either::Left(*liquid_item),
                ir::Item::Rust(rust_item) => Either::Right(*rust_item),
            });

        let span = item_mod.span();
        let (storage, events, assets, mut functions, mut constants) =
            ir_utils::split_items(liquid_items, span)?;

        storage.public_fields.iter().for_each(|index| {
            let field = &storage.fields.named[*index];
            let ident = &field.ident.as_ref().unwrap();
            let ty = &field.ty;

            let getter = syn::parse2::<syn::ItemFn>(quote! {
                #[deprecated(note = "Please visit the storage field directly instead of using its getter function")]
                pub fn #ident(&self, index: <#ty as liquid_lang::storage::Getter>::Index) -> <#ty as liquid_lang::storage::Getter>::Output {
                    <#ty as liquid_lang::storage::Getter>::getter_impl(&self.#ident, index)
                }
            }).unwrap();

            functions.push(ir::Function{
                attrs: getter.attrs,
                kind: ir::FunctionKind::External(lang_utils::calculate_fn_id(ident)),
                sig: ir::Signature::try_from(&getter.sig).unwrap(),
                body: *getter.block,
                span: field.span(),
            });
        });

        let assets_names = assets
            .iter()
            .map(|asset| asset.ident.to_string())
            .collect::<Vec<String>>()
            .join(",");
        let supports_asset_constant = syn::parse2::<syn::ImplItemConst>(quote! {
        const SUPPORTS_ASSET : &'static str = #assets_names;
        })
        .unwrap();
        constants.push(supports_asset_constant);
        // Don't use `span` here please, otherwise it will make intelli sense working
        // improperly!
        let supports_asset_name = Ident::new(SUPPORTS_ASSET_NAME, Span::call_site());
        let supports_asset_fn = syn::parse2::<syn::ItemFn>(quote! {
            pub fn #supports_asset_name(&self, asset: String) -> bool {
                Self::SUPPORTS_ASSET.contains(&asset)
            }
        })
        .unwrap();
        functions.push(ir::Function {
            attrs: supports_asset_fn.attrs,
            kind: ir::FunctionKind::External(lang_utils::calculate_fn_id(
                &SUPPORTS_ASSET_SIGNATURE,
            )),
            sig: ir::Signature::try_from(&supports_asset_fn.sig).unwrap(),
            body: *supports_asset_fn.block,
            span,
        });

        let (mut constructor, mut external_func_count) = (None, 0);
        for (pos, func) in functions.iter().enumerate() {
            match func.kind {
                ir::FunctionKind::Constructor => {
                    if constructor.is_some() {
                        bail_span!(
                            func.span(),
                            "duplicate constructor definition found here"
                        )
                    }
                    constructor = Some(pos);
                }
                ir::FunctionKind::External(..) => {
                    if !func.is_internal_fn() {
                        external_func_count += 1;
                    }
                }
                _ => (),
            }
        }

        if constructor.is_none() {
            bail!(item_mod, "no constructor found for this contract")
        }

        if external_func_count < 1 {
            bail!(item_mod, "contract needs at least one external function")
        }

        let constructor = functions.remove(constructor.unwrap());
        let meta_info = ir::ContractMetaInfo::try_from(params)?;
        Ok(Self {
            mod_token: item_mod.mod_token,
            ident: item_mod.ident,
            meta_info,
            storage,
            events,
            assets,
            constructor,
            functions,
            constants,
            rust_items,
        })
    }
}

impl TryFrom<ir::ContractParams> for ir::ContractMetaInfo {
    type Error = Error;

    fn try_from(params: ir::ContractParams) -> Result<Self> {
        let mut unique_param_names = HashSet::new();
        let mut liquid_version = None;
        for param in params.params.iter() {
            let name = param.ident().to_string();
            if !unique_param_names.insert(name.clone()) {
                bail_span!(param.span(), "duplicate parameter encountered: {}", name)
            }

            match param {
                ir::ContractMetaParam::Version(param) => {
                    liquid_version = Some(param.version.clone())
                }
            }
        }

        let liquid_version = match liquid_version {
            None => ir::MetaVersion::try_from(&"1.0.0-rc2".to_owned()).unwrap(),
            Some(liquid_version) => liquid_version,
        };

        Ok(Self { liquid_version })
    }
}

impl TryFrom<ir::InterfaceParams> for ir::InterfaceMetaInfo {
    type Error = Error;

    fn try_from(params: ir::InterfaceParams) -> Result<Self> {
        let mut unique_param_names = HashSet::new();
        let mut interface_name = None;
        for param in params.params.iter() {
            let name = param.ident().to_string();
            if !unique_param_names.insert(name.clone()) {
                bail_span!(param.span(), "duplicate parameter encountered: {}", name)
            }

            match param {
                ir::InterfaceMetaParam::Name(param_name) => {
                    if let ir::NameValue::Name(name) = &param_name.value {
                        // The parsing of interface meta info ensures that
                        // if `name` is not specified as `auto` then it must
                        // be a non-empty string.
                        interface_name = Some(name.clone());
                    } else {
                        // Represent that interface name is specified as `auto`.
                        interface_name = Some(String::new());
                    }
                }
            }
        }

        let interface_name = match interface_name {
            None => bail_span!(
                params.span(),
                "expected `name` parameter in `#[liquid::interface]` attribute",
            ),
            Some(interface_name) => interface_name,
        };

        Ok(Self { interface_name })
    }
}

impl TryFrom<syn::FnArg> for ir::FnArg {
    type Error = Error;

    fn try_from(arg: syn::FnArg) -> Result<Self> {
        let span = arg.span();
        match arg {
            syn::FnArg::Receiver(receiver) => Ok(Box::new(receiver).into()),
            syn::FnArg::Typed(pat_type) => match *(pat_type.pat) {
                syn::Pat::Ident(pat_ident) => {
                    if pat_ident.by_ref.is_some() {
                        bail!(
                            pat_ident.by_ref,
                            "`ref` modifier is unsupported for liquid function arguments",
                        )
                    }

                    Ok(Box::new(ir::IdentType {
                        attrs: pat_ident.attrs,
                        mutability: pat_ident.mutability,
                        ident: pat_ident.ident,
                        colon_token: pat_type.colon_token,
                        ty: *pat_type.ty,
                        span,
                    })
                    .into())
                }
                unsupported => bail!(
                    unsupported,
                    "encountered unsupported function argument syntax for liquid \
                     function",
                ),
            },
        }
    }
}

impl TryFrom<&syn::Signature> for ir::Signature {
    type Error = Error;

    fn try_from(sig: &syn::Signature) -> Result<Self> {
        if sig.constness.is_some() {
            bail!(
                sig.constness,
                "`const` is not supported for methods in contract",
            )
        }

        if sig.asyncness.is_some() {
            bail!(
                sig.asyncness,
                "`async` is not supported for methods in contract",
            )
        }

        if sig.unsafety.is_some() {
            bail!(
                sig.unsafety,
                "`unsafe` is not supported for methods in contract",
            )
        }

        if sig.abi.is_some() {
            bail! {
                sig.abi,
                "specifying ABI is not supported for methods in contract",
            }
        }

        if !(sig.generics.params.is_empty() && sig.generics.where_clause.is_none()) {
            bail! {
                sig.generics,
                "generic is not supported for methods in contract",
            }
        }

        if sig.variadic.is_some() {
            bail! {
                sig.variadic,
                "variadic is not supported for methods in contract",
            }
        }

        if sig.inputs.is_empty() {
            bail!(
                sig,
                "`&self` or `&mut self` is mandatory first parameter for all contract \
                 methods",
            )
        }

        let inputs = sig
            .inputs
            .iter()
            .cloned()
            .map(ir::FnArg::try_from)
            .collect::<Result<Punctuated<ir::FnArg, Token![,]>>>()?;
        let output = &sig.output;

        match &inputs[0] {
            ir::FnArg::Typed(ident_type) => bail_span!(
                ident_type.span(),
                "`&self` or `&mut self` is mandatory first parameter for all contract \
                 methods",
            ),
            ir::FnArg::Receiver(receiver) if receiver.reference.is_none() => bail_span!(
                receiver.span(),
                "`&self` or `&mut self` is mandatory first parameter for all contract \
                 methods",
            ),
            _ => (),
        }

        for arg in inputs.iter().skip(1) {
            if let ir::FnArg::Receiver(receiver) = arg {
                bail_span!(receiver.span(), "unexpected `self` argument",)
            }
        }

        let output_args_count = match output {
            syn::ReturnType::Default => 0,
            syn::ReturnType::Type(_, ty) => match &(**ty) {
                syn::Type::Tuple(tuple_ty) => tuple_ty.elems.len(),
                _ => 1,
            },
        };
        if output_args_count > 16 {
            bail_span!(
                output.span(),
                "the number of output arguments should not exceed 16"
            )
        }

        Ok(ir::Signature {
            fn_token: sig.fn_token,
            ident: sig.ident.clone(),
            paren_token: sig.paren_token,
            inputs,
            output: output.clone(),
        })
    }
}

impl TryFrom<syn::ImplItemMethod> for ir::Function {
    type Error = Error;

    fn try_from(method: syn::ImplItemMethod) -> Result<Self> {
        if method.defaultness.is_some() {
            bail!(
                method.defaultness,
                "`default` modifiers are not allowed for methods in contract",
            )
        }

        match method.vis {
            syn::Visibility::Crate(_) | syn::Visibility::Restricted(_) => bail!(
                method.vis,
                "crate-level visibility or visibility level restricted to some path is \
                 not supported for methods in contract",
            ),
            _ => (),
        }

        let span = method.span();
        let sig = ir::Signature::try_from(&method.sig)?;
        let ident = &sig.ident;

        let kind = if ident == "new" {
            match method.vis {
                syn::Visibility::Public(_) => {
                    // The process of parsing signature ensures that the first parameter must be a reference
                    // to `self`, so here we just test wether it's a mutable reference.
                    if !sig.is_mut() {
                        bail_span!(
                            sig.inputs[0].span(),
                            "`&mut self` is mandatory first parameter for constructor \
                             of contract"
                        )
                    }
                    if let syn::ReturnType::Type(t, ty) = sig.output {
                        bail_span!(
                            t.span().join(ty.span()).expect(
                                "right arrow token and return type are in the same file"
                            ),
                            "contract constructor should not have return value"
                        )
                    }

                    ir::FunctionKind::Constructor
                }
                _ => bail!(
                    ident,
                    "the visibility for contract constructor should be `pub`",
                ),
            }
        } else if let syn::Visibility::Public(_) = method.vis {
            let fn_id = crate::utils::calculate_fn_id(ident);
            ir::FunctionKind::External(fn_id)
        } else {
            ir::FunctionKind::Normal
        };

        Ok(Self {
            attrs: method.attrs,
            kind,
            sig,
            body: method.block,
            span,
        })
    }
}

impl TryFrom<syn::ItemImpl> for ir::ItemImpl {
    type Error = Error;

    fn try_from(item_impl: syn::ItemImpl) -> Result<Self> {
        if item_impl.defaultness.is_some() {
            bail!(
                item_impl.defaultness,
                "default implementation blocks not supported in liquid",
            )
        }

        if item_impl.unsafety.is_some() {
            bail!(
                item_impl.unsafety,
                "unsafe implementation blocks are not supported in liquid",
            )
        }

        if !(item_impl.generics.params.is_empty()
            && item_impl.generics.where_clause.is_none())
        {
            bail!(
                item_impl.generics,
                "generic implementation blocks are not supported in liquid",
            )
        }

        if item_impl.trait_.is_some() {
            bail!(
                item_impl,
                "trait implementations are not supported in liquid",
            )
        }

        let type_path = match &*item_impl.self_ty {
            syn::Type::Path(type_path) => type_path,
            _ => bail!(
                item_impl.self_ty,
                "encountered invalid liquid implementer type ascription",
            ),
        };

        if let Some(qself) = &type_path.qself {
            let span = qself
                .lt_token
                .span()
                .join(qself.gt_token.span())
                .expect("all spans are in the same file");
            bail_span!(
                span,
                "implementation blocks for self qualified paths are not supported in \
                 liquid",
            )
        }

        let ident = match type_path.path.get_ident() {
            Some(ident) => ident.clone(),
            None => bail!(
                type_path.path,
                "don't use type path for liquid storage implementer",
            ),
        };

        let mut functions = Vec::new();
        let mut constants = Vec::new();
        for item in item_impl.items.into_iter() {
            match item {
                syn::ImplItem::Method(method) => {
                    functions.push(ir::Function::try_from(method)?);
                }
                syn::ImplItem::Const(constant) => {
                    constants.push(constant);
                }
                unsupported => bail!(
                    unsupported,
                    "only methods and constants are supported inside impl blocks in \
                     liquid",
                ),
            }
        }

        Ok(Self {
            attrs: item_impl.attrs,
            impl_token: item_impl.impl_token,
            ty: ident,
            brace_token: item_impl.brace_token,
            functions,
            constants,
        })
    }
}

impl TryFrom<syn::ItemStruct> for ir::ItemStorage {
    type Error = Error;
    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        if item_struct.vis != syn::Visibility::Inherited {
            bail!(
                item_struct.vis,
                "visibility modifiers are not allowed for `#[liquid(storage)]` struct",
            )
        }

        if item_struct.generics.type_params().count() > 0 {
            bail!(
                item_struct.generics,
                "generics are not allowed for `#[liquid(storage)]` struct"
            )
        }

        let mut public_fields = Vec::new();
        let span = item_struct.span();
        let fields = match item_struct.fields {
            syn::Fields::Named(named_fields) => {
                let fields = &named_fields.named;
                for (i, field) in fields.iter().enumerate() {
                    let visibility = &field.vis;
                    match visibility {
                        syn::Visibility::Public(_) => {
                            public_fields.push(i);
                        }
                        syn::Visibility::Inherited => (),
                        _ => bail!(
                            field,
                            "crate-level visibility or visibility level restricted to \
                             some path is not allowed for fields in
                             `#[liquid(storage)]` struct"
                        ),
                    }
                }
                named_fields
            }
            syn::Fields::Unnamed(_) => bail!(
                item_struct,
                "tuple-struct is not allowed for `#[liquid(storage)]`"
            ),
            syn::Fields::Unit => bail!(
                item_struct,
                "unit-struct is not allowed for `#[liquid(storage)]`"
            ),
        };

        Ok(ir::ItemStorage {
            attrs: item_struct.attrs,
            struct_token: item_struct.struct_token,
            ident: item_struct.ident,
            fields,
            public_fields,
            span,
        })
    }
}

impl TryFrom<syn::ItemStruct> for ir::ItemAsset {
    type Error = Error;
    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        if item_struct.vis != syn::Visibility::Inherited {
            bail!(
                item_struct.vis,
                "visibility modifiers are not allowed for `#[liquid(asset)]` struct",
            )
        }

        if item_struct.generics.type_params().count() > 0 {
            bail!(
                item_struct.generics,
                "generics are not allowed for `#[liquid(asset)]` struct"
            )
        }
        let span = item_struct.span();
        match item_struct.fields {
            syn::Fields::Named(_) => bail!(
                item_struct,
                "user defined fields not allowed for `#[liquid(asset)]`"
            ),
            syn::Fields::Unnamed(_) => bail!(
                item_struct,
                "tuple-struct is not allowed for `#[liquid(asset)]`"
            ),
            syn::Fields::Unit => (),
        };
        let markers = ir_utils::filter_map_liquid_attributes(&item_struct.attrs)?;

        if markers.len() > 1 {
            bail!(
                item_struct,
                "`#[liquid(asset)]` is mutually exclusive with other attributes"
            )
        }
        let mut asset_meta = ir::AssetMetaInfo::default();
        if let ir::AttrValue::Fields(fields) = &markers[0].value {
            for field in fields {
                match field {
                    ir::AssetAttribute::Issuer {
                        issuer_token: _,
                        eq_token: _,
                        value,
                    } => asset_meta.issuer = value.value(),
                    ir::AssetAttribute::TotalSupply {
                        total_token: _,
                        eq_token: _,
                        value,
                    } => asset_meta.total_supply = value.base10_parse()?,
                    // ir::AssetAttribute::Destroyable {
                    //     destroyable_token:_,
                    //     eq_token:_,
                    //     value,
                    // } => asset_meta.destroyable = value.value,
                    ir::AssetAttribute::Fungible {
                        fungible_token: _,
                        eq_token: _,
                        value,
                    } => asset_meta.fungible = value.value,
                    ir::AssetAttribute::Description {
                        description_token: _,
                        eq_token: _,
                        value,
                    } => asset_meta.description = value.value(),
                }
            }
        } else {
            bail!(item_struct, "`#[liquid(asset)]` with mismatch attributes")
        }
        if item_struct.ident.to_string().len() > MAX_ASSET_NAME_LENGTH {
            bail!(item_struct, "`#[liquid(asset)]` ")
        }
        Ok(ir::ItemAsset {
            attrs: item_struct.attrs,
            struct_token: item_struct.struct_token,
            ident: item_struct.ident,
            span,
            total_supply: asset_meta.total_supply,
            issuer: asset_meta.issuer,
            // destroyable: asset_meta.destroyable,
            fungible: asset_meta.fungible,
            description: asset_meta.description,
        })
    }
}

impl TryFrom<syn::ItemStruct> for ir::ItemEvent {
    type Error = Error;
    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        if item_struct.vis != syn::Visibility::Inherited {
            bail!(
                item_struct.vis,
                "visibility modifiers are not allowed for `#[liquid(event)]` struct",
            )
        }

        if item_struct.generics.type_params().count() > 0 {
            bail!(
                item_struct.generics,
                "generics are not allowed for `#[liquid(event)]` struct"
            )
        }

        let span = item_struct.span();
        let mut topic_count = 0;
        let (fields, indexed_fields, unindexed_fields) = match item_struct.fields {
            syn::Fields::Named(named_fields) => {
                let mut fields = Vec::new();
                let mut indexed_fields = Vec::new();
                let mut unindexed_fields = Vec::new();

                for field in &named_fields.named {
                    let visibility = &field.vis;
                    match visibility {
                        syn::Visibility::Inherited => (),
                        _ => bail!(
                            field,
                            "visibility modifiers are not allowed for field in \
                             `liquid(event)` struct"
                        ),
                    }

                    fields.push(field.clone());
                    let index = fields.len() - 1;

                    let is_topic =
                        ir_utils::filter_map_liquid_attributes(field.attrs.iter())?
                            .iter()
                            .any(|marker| marker.ident == "indexed");
                    if is_topic {
                        topic_count += 1;
                        if topic_count > 3 {
                            bail!(
                                field,
                                "the number of topics should not exceed 3 in \
                                 `liquid(event)` struct"
                            )
                        }

                        indexed_fields.push(index);
                    } else {
                        unindexed_fields.push(index);
                    }
                }
                (fields, indexed_fields, unindexed_fields)
            }
            syn::Fields::Unnamed(_) => bail!(
                item_struct,
                "tuple-struct is not allowed for `#[liquid(event)]`"
            ),
            syn::Fields::Unit => bail!(
                item_struct,
                "unit-struct is not allowed for `#[liquid(event)]`"
            ),
        };

        Ok(ir::ItemEvent {
            attrs: item_struct.attrs,
            struct_token: item_struct.struct_token,
            ident: item_struct.ident,
            fields,
            indexed_fields,
            unindexed_fields,
            span,
        })
    }
}

impl TryFrom<syn::Item> for ir::Item {
    type Error = Error;

    fn try_from(item: syn::Item) -> Result<Self> {
        match item.clone() {
            syn::Item::Struct(item_struct) => {
                let markers = ir_utils::filter_map_liquid_attributes(&item_struct.attrs)?;
                if markers.is_empty() {
                    return Ok(ir::Item::Rust(Box::new(item.into())));
                }
                if markers.len() > 1 {
                    bail!(
                        item_struct,
                        "a struct can be marked by only one of the followings: \
                         `liquid(storage)`, `liquid(event)` or `liquid(asset)` at the \
                         same time"
                    )
                }

                let marker = markers[0].ident.to_string();
                match marker.as_str() {
                    "storage" => ir::ItemStorage::try_from(item_struct)
                        .map(Into::into)
                        .map(Box::new)
                        .map(ir::Item::Liquid),
                    "event" => ir::ItemEvent::try_from(item_struct)
                        .map(Into::into)
                        .map(Box::new)
                        .map(ir::Item::Liquid),
                    "asset" => ir::ItemAsset::try_from(item_struct)
                        .map(Into::into)
                        .map(Box::new)
                        .map(ir::Item::Liquid),
                    _ => Ok(ir::Item::Rust(Box::new(item.into()))),
                }
            }
            syn::Item::Impl(item_impl) => {
                let is_contract_impl;
                {
                    let markers =
                        ir_utils::filter_map_liquid_attributes(&item_impl.attrs)?;
                    is_contract_impl =
                        markers.iter().any(|marker| marker.ident == "methods");
                }

                if is_contract_impl {
                    ir::ItemImpl::try_from(item_impl)
                        .map(Into::into)
                        .map(Box::new)
                        .map(ir::Item::Liquid)
                } else {
                    bail!(
                        item_impl,
                        "`impl` blocks in contract should be tagged with \
                         `#[liquid(methods)]`"
                    )
                }
            }
            _ => Ok(ir::Item::Rust(Box::new(item.into()))),
        }
    }
}

impl TryFrom<syn::ItemStruct> for ir::ForeignStruct {
    type Error = Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        if item_struct.vis != syn::Visibility::Inherited {
            bail!(
                item_struct.vis,
                "`struct` in interface must have no visibility modifier",
            )
        }

        if item_struct.generics.type_params().count() > 0 {
            bail!(
                item_struct.generics,
                "generics are not allowed for `struct` in interface"
            )
        }

        let span = item_struct.span();
        let fields = match item_struct.fields {
            syn::Fields::Named(named_fields) => {
                for field in &named_fields.named {
                    match &field.vis {
                        syn::Visibility::Inherited => (),
                        _ => bail!(
                            field,
                            "visibility modifier is not allowed for fields of
                             `struct` in interface"
                        ),
                    }
                }
                named_fields
            }
            syn::Fields::Unnamed(_) => bail!(
                item_struct,
                "tuple-struct is not allowed for `struct` in interface"
            ),
            syn::Fields::Unit => bail!(
                item_struct,
                "unit-struct is not allowed for `struct` in interface`"
            ),
        };

        Ok(Self {
            attrs: item_struct.attrs,
            struct_token: item_struct.struct_token,
            ident: item_struct.ident,
            fields,
            span,
        })
    }
}

impl TryFrom<&syn::ForeignItem> for ir::ForeignFn {
    type Error = Error;

    fn try_from(foreign_item: &syn::ForeignItem) -> Result<Self> {
        match foreign_item {
            syn::ForeignItem::Fn(foreign_fn) => {
                match &foreign_fn.vis {
                    syn::Visibility::Inherited => (),
                    _ => bail!(
                        &foreign_fn.vis,
                        "visibility modifier is not allowed for method declarations in \
                         interface"
                    ),
                }

                let sig = ir::Signature::try_from(&foreign_fn.sig)?;
                let span = foreign_fn.span();

                let markers = ir_utils::filter_map_liquid_attributes(&foreign_fn.attrs)?;

                let mock_context_getter = if let Some(marker) = markers
                    .iter()
                    .find(|marker| marker.ident == "mock_context_getter")
                {
                    let value = match &marker.value {
                        ir::AttrValue::LitStr(value) => value,
                        _ => bail_span!(
                            marker.span(),
                            "the attribute `mock_context_getter` should be assigned \
                             with a literal string"
                        ),
                    };

                    let getter = syn::parse_str::<syn::Ident>(&value.value());
                    if getter.is_err() {
                        bail_span!(
                            value.span(),
                            "invalid identifier for mock context getter"
                        )
                    }
                    Some(getter.unwrap())
                } else {
                    None
                };

                Ok(Self {
                    attrs: foreign_fn.attrs.clone(),
                    sig,
                    semi_token: foreign_fn.semi_token,
                    mock_context_getter,
                    span,
                })
            }
            unsupported => bail!(
                unsupported,
                "only function declaration is allowed for the `extern` block"
            ),
        }
    }
}

impl TryFrom<(ir::InterfaceParams, syn::ItemMod)> for ir::Interface {
    type Error = Error;

    fn try_from((params, item_mod): (ir::InterfaceParams, syn::ItemMod)) -> Result<Self> {
        if item_mod.vis != syn::Visibility::Inherited {
            bail!(
                item_mod.vis,
                "interface module must have no visibility modifier",
            )
        }

        let items = match &item_mod.content {
            None => bail!(
                item_mod,
                "interface module must be inline, e.g. `mod m {{ ... }}`",
            ),
            Some((_, items)) => items.clone(),
        };

        let mut foreign_structs = Vec::new();
        let mut foreign_fns = BTreeMap::<_, ir::ForeignFn>::new();
        let mut imports = Vec::new();
        let span = item_mod.span();

        let meta_info = ir::InterfaceMetaInfo::try_from(params)?;
        let interface_ident = if meta_info.interface_name.is_empty() {
            Ident::new(&item_mod.ident.to_string().to_camel_case(), span)
        } else {
            Ident::new(&meta_info.interface_name, span)
        };

        for item in items {
            match item {
                syn::Item::Struct(item_struct) => {
                    foreign_structs.push(ir::ForeignStruct::try_from(item_struct)?);
                }
                syn::Item::Use(item_use) => {
                    imports.push(item_use);
                }
                syn::Item::ForeignMod(item_foreign_mod) => {
                    if !foreign_fns.is_empty() {
                        bail!(
                            item_foreign_mod,
                            "interface module must have exactly one `extern` block"
                        )
                    }

                    let abi = &item_foreign_mod.abi;
                    if let Some(ref name) = abi.name {
                        let name = name.value();
                        match name.as_str() {
                            "liquid" => (),
                            _ => bail!(
                                abi,
                                "ABI should be specified  as `\"liquid\"` or empty"
                            ),
                        }
                    }

                    for foreign_item in item_foreign_mod.items.iter() {
                        let foreign_fn = ir::ForeignFn::try_from(foreign_item)?;
                        let ident = foreign_fn.sig.ident.clone();
                        if let Some(foreign_fn) = foreign_fns.get_mut(&ident) {
                            bail_span!(
                                foreign_fn.span,
                                "overriding methods is not supported in liquid"
                            )
                        } else {
                            foreign_fns.insert(ident, foreign_fn);
                        }
                    }

                    if foreign_fns.is_empty() {
                        bail!(
                            item_foreign_mod,
                            "at least one declaration of method should be provided in \
                             `extern` block"
                        )
                    }
                }
                other => bail!(
                    other,
                    "only `struct`, `extern` blocks or `use` statements are allowed in \
                     interface"
                ),
            }
        }

        Ok(Self {
            mod_token: item_mod.mod_token,
            ident: item_mod.ident,
            meta_info,
            foreign_structs,
            foreign_fns,
            imports,
            interface_ident,
            span,
        })
    }
}

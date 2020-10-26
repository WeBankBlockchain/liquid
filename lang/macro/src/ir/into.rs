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
    ir::{self, utils as ir_utils},
    utils as lang_utils,
};
use core::convert::TryFrom;
use either::Either;
use itertools::Itertools;
use proc_macro2::Ident;
use quote::quote;
use regex::Regex;
use std::collections::{BTreeMap, HashSet};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Result, Token,
};

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
        let content;
        let paren_token = syn::parenthesized!(content in input);
        let ident = content.parse::<Ident>()?;
        if content.is_empty() {
            return Ok(ir::Marker { paren_token, ident });
        }
        bail_span!(
            paren_token.span,
            "invalid liquid attribute in the given context",
        )
    }
}

impl TryFrom<syn::Attribute> for ir::Marker {
    type Error = Error;

    fn try_from(attr: syn::Attribute) -> Result<Self> {
        if !attr.path.is_ident("liquid") {
            bail!(attr, "encountered non-liquid attribute")
        }
        syn::parse2::<Self>(attr.tokens)
    }
}

impl TryFrom<(ir::Params, syn::ItemMod)> for ir::Contract {
    type Error = Error;

    fn try_from((params, item_mod): (ir::Params, syn::ItemMod)) -> Result<Self> {
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

        let (storage, events, mut functions, constants) =
            ir_utils::split_items(liquid_items)?;
        storage.public_fields.iter().for_each(|index| {
            let field = &storage.fields.named[*index];
            let ident = &field.ident.as_ref().unwrap();
            let ty = &field.ty;

            let getter = syn::parse2::<syn::ItemFn>(quote! {
                #[deprecated(note = "Please visit the storage field directly instead of using its getter function")]
                pub fn #ident(&self, index: <#ty as liquid_core::storage::Getter>::Index) -> <#ty as liquid_core::storage::Getter>::Output {
                    <#ty as liquid_core::storage::Getter>::getter_impl(&self.#ident, index)
                }
            }).unwrap();

            functions.push(ir::Function{
                attrs: getter.attrs,
                kind: ir::FunctionKind::External(lang_utils::calculate_fn_id(ident)),
                sig: ir::Signature::try_from((&getter.sig, false)).unwrap(),
                body: *getter.block,
                span: field.span(),
            });
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
                ir::FunctionKind::External(_) => {
                    external_func_count += 1;
                }
                _ => (),
            }
        }

        if constructor.is_none() {
            bail!(item_mod, "no constructor found for this contract")
        }

        if external_func_count < 1 {
            bail!(item_mod, "contract needs at least 1 external function")
        }

        let constructor = functions.remove(constructor.unwrap());
        let meta_info = ir::MetaInfo::try_from(params)?;
        Ok(Self {
            mod_token: item_mod.mod_token,
            ident: item_mod.ident,
            meta_info,
            storage,
            events,
            constructor,
            functions,
            constants,
            rust_items,
        })
    }
}

impl TryFrom<ir::Params> for ir::MetaInfo {
    type Error = Error;

    fn try_from(params: ir::Params) -> Result<Self> {
        let mut unique_param_names = HashSet::new();
        let mut liquid_version = None;
        for param in params.params.iter() {
            let name = param.ident().to_string();
            if !unique_param_names.insert(name.clone()) {
                bail_span!(param.span(), "duplicate parameter encountered: {}", name)
            }

            match param {
                ir::MetaParam::Version(param) => {
                    liquid_version = Some(param.version.clone())
                }
            }
        }

        let liquid_version = match liquid_version {
            None => bail_span!(
                params.span(),
                "expected `version` parameter at `#[liquid::contract(..)]`",
            ),
            Some(liquid_version) => liquid_version,
        };

        Ok(Self { liquid_version })
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

impl TryFrom<(&syn::Signature, bool)> for ir::Signature {
    type Error = Error;

    fn try_from((sig, for_interface): (&syn::Signature, bool)) -> Result<Self> {
        if sig.constness.is_some() {
            bail!(
                sig.constness,
                "`const` is not supported for functions in contract",
            )
        }

        if sig.asyncness.is_some() {
            bail!(
                sig.asyncness,
                "`async` is not supported for functions in contract",
            )
        }

        if sig.unsafety.is_some() {
            bail!(
                sig.unsafety,
                "`unsafe` is not supported for functions in contract",
            )
        }

        if sig.abi.is_some() {
            bail! {
                sig.abi,
                "specifying ABI is not supported for functions in contract",
            }
        }

        if !(sig.generics.params.is_empty() && sig.generics.where_clause.is_none()) {
            bail! {
                sig.generics,
                "generic is not supported for functions in contract",
            }
        }

        if sig.variadic.is_some() {
            bail! {
                sig.variadic,
                "variadic functions are not allowed in liquid functions",
            }
        }

        if !for_interface && sig.inputs.is_empty() {
            bail!(
                sig,
                "`&self` or `&mut self` is mandatory for liquid functions",
            )
        }

        let inputs = sig
            .inputs
            .iter()
            .cloned()
            .map(ir::FnArg::try_from)
            .collect::<Result<Punctuated<ir::FnArg, Token![,]>>>()?;
        let output = &sig.output;

        if !for_interface {
            if let ir::FnArg::Typed(ident_type) = &inputs[0] {
                bail_span!(
                    ident_type.span(),
                    "first argument of liquid functions must be `&self` or `&mut self`",
                )
            }
        }

        for arg in inputs.iter().skip(if for_interface { 0 } else { 1 }) {
            if let ir::FnArg::Receiver(receiver) = arg {
                bail_span!(receiver.span(), "unexpected `self` argument",)
            }
        }

        let mut input_args_count = inputs.len();
        if !for_interface {
            input_args_count -= 1;
        }
        if input_args_count > 16 {
            bail_span!(
                inputs[1]
                    .span()
                    .join(inputs.last().span())
                    .expect("first argument and last argument are in the same file"),
                "the number of input arguments should not be more than 16"
            )
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
                "the number of output arguments should not be more than 16"
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
                "`default` modifiers are not allowed for liquid functions",
            )
        }

        match method.vis {
            syn::Visibility::Crate(_) | syn::Visibility::Restricted(_) => bail!(
                method.vis,
                "crate-level visibility or visibility level restricted to some path is \
                 not supported for liquid functions",
            ),
            _ => (),
        }

        let span = method.span();
        let sig = ir::Signature::try_from((&method.sig, false))?;
        let ident = &sig.ident;

        let kind = if ident == "new" {
            match method.vis {
                syn::Visibility::Public(_) => {
                    if !sig.is_mut() {
                        bail_span!(
                            sig.inputs[0].span(),
                            "`&mut self` is mandatory for constructor of contract"
                        )
                    }
                    if let syn::ReturnType::Type(t, ty) = sig.output {
                        bail_span!(
                            t.span().join(ty.span()).expect(
                                "right arrow token and return type are in the same file"
                            ),
                            "constructor of contract should not have return value"
                        )
                    }

                    ir::FunctionKind::Constructor
                }
                _ => bail!(
                    ident,
                    "the visibility for constructor of contract should be `pub`",
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

                    let is_topic = field
                        .attrs
                        .iter()
                        .filter_map(|attr| ir::Marker::try_from(attr.clone()).ok())
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
                let is_contract_storage;
                let is_event;
                {
                    let markers =
                        ir_utils::filter_map_liquid_attributes(&item_struct.attrs)
                            .collect::<Vec<_>>();
                    is_contract_storage =
                        markers.iter().any(|marker| marker.ident == "storage");
                    is_event = markers.iter().any(|marker| marker.ident == "event");
                }

                match (is_contract_storage, is_event) {
                    (true, true) => bail!(
                        item_struct,
                        "a struct can not be marked as `liquid(event)` and \
                         `liquid(storage)` simultaneously"
                    ),
                    (true, false) => ir::ItemStorage::try_from(item_struct)
                        .map(Into::into)
                        .map(Box::new)
                        .map(ir::Item::Liquid),
                    (false, true) => ir::ItemEvent::try_from(item_struct)
                        .map(Into::into)
                        .map(Box::new)
                        .map(ir::Item::Liquid),
                    _ => Ok(ir::Item::Rust(Box::new(item.into()))),
                }
            }
            syn::Item::Impl(item_impl) => {
                let is_contract_impl;
                {
                    let mut markers =
                        ir_utils::filter_map_liquid_attributes(&item_impl.attrs);
                    is_contract_impl = markers.any(|marker| marker.ident == "methods");
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
                let sig = ir::Signature::try_from((&foreign_fn.sig, true))?;
                let fn_id = lang_utils::calculate_fn_id(&sig.ident);
                let span = foreign_fn.span();

                Ok(Self {
                    attrs: foreign_fn.attrs.clone(),
                    sig,
                    semi_token: foreign_fn.semi_token,
                    fn_id,
                    span,
                })
            }
            unsupported => bail!(
                unsupported,
                "only function declaration is allowed for `extern` block in interface"
            ),
        }
    }
}

impl TryFrom<syn::ItemMod> for ir::Interface {
    type Error = Error;

    fn try_from(item_mod: syn::ItemMod) -> Result<Self> {
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
        let mut foreign_fns = BTreeMap::<_, Vec<ir::ForeignFn>>::new();
        let mut imports = Vec::new();
        let span = item_mod.span();

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
                            "the total number of `extern` block in interface module \
                             should not be more than 1"
                        )
                    }

                    let abi = &item_foreign_mod.abi;
                    match abi.name {
                        Some(ref name) if name.value() != "liquid" => bail!(
                            abi,
                            "ABI should be specified  as `\"liquid\"` or nothing"
                        ),
                        _ => (),
                    }

                    for foreign_item in item_foreign_mod.items.iter() {
                        let foreign_fn = ir::ForeignFn::try_from(foreign_item)?;
                        let ident = foreign_fn.sig.ident.clone();
                        if let Some(fns) = foreign_fns.get_mut(&ident) {
                            fns.push(foreign_fn);
                        } else {
                            foreign_fns.insert(ident, vec![foreign_fn]);
                        }
                    }

                    if foreign_fns.is_empty() {
                        bail!(
                            item_foreign_mod,
                            "at least 1 declaration of method should be provided in \
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
            foreign_structs,
            foreign_fns,
            imports,
            span,
        })
    }
}

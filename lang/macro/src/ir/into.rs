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

use crate::ir::{self, utils};
use core::convert::TryFrom;
use either::Either;
use itertools::Itertools;
use proc_macro2::Ident;
use regex::Regex;
use std::collections::HashSet;
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

        let (storage, mut functions) = utils::split_items(liquid_items)?;
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
            bail!(
                item_mod,
                "liquid contract needs at least 1 external function"
            )
        }

        let constructor = functions.remove(constructor.unwrap());
        let meta_info = ir::MetaInfo::try_from(params)?;
        Ok(Self {
            mod_token: item_mod.mod_token,
            ident: item_mod.ident,
            meta_info,
            storage,
            constructor,
            functions,
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
                "`const` is not supported for functions in liquid contract",
            )
        }

        if sig.asyncness.is_some() {
            bail!(
                sig.asyncness,
                "`async` is not supported for functions in liquid contract",
            )
        }

        if sig.unsafety.is_some() {
            bail!(
                sig.unsafety,
                "`unsafe` is not supported for functions in liquid contract",
            )
        }

        if sig.abi.is_some() {
            bail! {
                sig.abi,
                "specifying ABI is not supported for functions in liquid contract",
            }
        }

        if !(sig.generics.params.is_empty() && sig.generics.where_clause.is_none()) {
            bail! {
                sig.generics,
                "generic is not supported for functions in liquid contract",
            }
        }

        if sig.variadic.is_some() {
            bail! {
                sig.variadic,
                "variadic functions are not allowed in liquid functions",
            }
        }

        if sig.inputs.is_empty() {
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

        if let ir::FnArg::Typed(ident_type) = &inputs[0] {
            bail_span!(
                ident_type.span(),
                "first argument of liquid functions must be `&self` or `&mut self`",
            )
        }

        for arg in inputs.iter().skip(1) {
            if let ir::FnArg::Receiver(receiver) = arg {
                bail_span!(
                    receiver.span(),
                    "unexpected `self` argument found for liquid function",
                )
            }
        }

        Ok(ir::Signature {
            fn_token: sig.fn_token,
            ident: sig.ident.clone(),
            paren_token: sig.paren_token,
            inputs,
            output: sig.output.clone(),
        })
    }
}

impl TryFrom<syn::ImplItemMethod> for ir::Function {
    type Error = Error;

    fn try_from(method: syn::ImplItemMethod) -> Result<Self> {
        if method.vis != syn::Visibility::Inherited {
            bail!(
                method.vis,
                "visibility modifiers are not allowed for liquid functions",
            )
        }

        if method.defaultness.is_some() {
            bail!(
                method.defaultness,
                "`default` modifiers are not allowed for liquid functions",
            )
        }

        let span = method.span();
        let sig = ir::Signature::try_from(&method.sig)?;

        let (liquid_markers, non_liquid_markers): (Vec<_>, Vec<_>) = method
            .attrs
            .into_iter()
            .partition_map(|attr| match ir::Marker::try_from(attr.clone()) {
                Ok(marker) => Either::Left(marker),
                Err(_) => Either::Right(attr),
            });

        let (mut has_constructor_marker, mut has_external_marker) = (false, false);
        for marker in liquid_markers {
            match marker.ident.to_string().as_str() {
                "constructor" => has_constructor_marker = true,
                "external" => has_external_marker = true,
                _ => bail_span!(marker.span(), "unknown marker for the liquid function"),
            }
        }

        let kind = match (has_constructor_marker, has_external_marker) {
            (true, true) => bail_span!(
                span,
                "liquid functions can't be marked as `constructor` and `external` \
                 simultaneously"
            ),
            (true, false) => {
                if !sig.is_mut() {
                    bail_span!(
                        sig.inputs[0].span(),
                        "`&mut self` is mandatory for constructor of liquid contract"
                    )
                }

                if let syn::ReturnType::Type(t, ty) = sig.output {
                    bail_span!(
                        t.span().join(ty.span()).expect(
                            "right arrow token and return type are in the same file"
                        ),
                        "constructor of liquid contract should not have return value"
                    )
                }

                ir::FunctionKind::Constructor
            }
            (false, true) => {
                let fn_name = method.sig.ident;
                let fn_hash = liquid_primitives::hash::keccak::keccak256(
                    fn_name.to_string().as_bytes(),
                );
                let fn_id = u32::from_be_bytes(fn_hash) as usize;
                ir::FunctionKind::External(fn_id)
            }
            _ => ir::FunctionKind::Normal,
        };

        Ok(Self {
            attrs: non_liquid_markers,
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
            bail!(item_impl.defaultness, "`default` not supported in liquid",)
        }

        if item_impl.unsafety.is_some() {
            bail!(
                item_impl.unsafety,
                "`unsafe` implementation blocks are not supported in liquid",
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
                "encountered invalid liquid implementer type path",
            ),
        };

        let mut functions = Vec::new();
        for item in item_impl.items.into_iter() {
            match item {
                syn::ImplItem::Method(method) => {
                    functions.push(ir::Function::try_from(method)?);
                }
                unsupported => bail!(
                    unsupported,
                    "only methods are supported inside impl blocks in liquid",
                ),
            }
        }

        Ok(Self {
            attrs: item_impl.attrs,
            impl_token: item_impl.impl_token,
            ty: ident,
            brace_token: item_impl.brace_token,
            functions,
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

        let span = item_struct.span();
        let fields = match item_struct.fields {
            syn::Fields::Named(named_fields) => named_fields,
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
            span,
        })
    }
}

impl TryFrom<syn::Item> for ir::Item {
    type Error = Error;

    fn try_from(item: syn::Item) -> Result<Self> {
        match item.clone() {
            syn::Item::Struct(item_struct) => {
                let markers = utils::filter_map_liquid_attributes(&item_struct.attrs)
                    .collect::<Vec<_>>();
                let storage_marker_pos =
                    markers.iter().position(|marker| marker.ident == "storage");

                match storage_marker_pos {
                    Some(_) => ir::ItemStorage::try_from(item_struct)
                        .map(Into::into)
                        .map(Box::new)
                        .map(ir::Item::Liquid),
                    _ => Ok(ir::Item::Rust(Box::new(item.into()))),
                }
            }
            syn::Item::Impl(item_impl) => {
                let has_liquid_functions = item_impl
                    .items
                    .iter()
                    .filter_map(|item| match item {
                        syn::ImplItem::Method(method) => Some(method),
                        _ => None,
                    })
                    .filter(|method| {
                        utils::has_liquid_attribute(method.attrs.iter().cloned())
                    })
                    .count()
                    > 0;

                if has_liquid_functions {
                    ir::ItemImpl::try_from(item_impl)
                        .map(Into::into)
                        .map(Box::new)
                        .map(ir::Item::Liquid)
                } else {
                    Ok(ir::Item::Rust(Box::new(item.into())))
                }
            }
            _ => Ok(ir::Item::Rust(Box::new(item.into()))),
        }
    }
}

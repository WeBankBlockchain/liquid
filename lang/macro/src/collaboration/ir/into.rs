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

use crate::collaboration::{
    ir,
    ir::{utils::*, AttrValue},
};
use core::convert::TryFrom;
use either::Either;
use itertools::Itertools;
use proc_macro2::{Ident, Span};
use std::collections::BTreeMap;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Error, Result, Token,
};

impl Parse for ir::Marker {
    fn parse(input: ParseStream) -> Result<Self> {
        const SINGLE_MARKER: [&str; 2] = ["contract", "rights"];
        const VALUED_MARKER: [&str; 2] = ["belongs_to", "rights_belong_to"];

        let content;
        let paren_token = syn::parenthesized!(content in input);
        let ident = content.parse::<Ident>()?;
        let ident_str = ident.to_string();
        if content.is_empty() {
            if VALUED_MARKER
                .iter()
                .any(|&valued_marker| ident_str == valued_marker)
            {
                bail_span!(
                    ident.span(),
                    "`{}` should be used with providing a parameter",
                    ident_str,
                )
            }

            Ok(ir::Marker {
                paren_token,
                ident,
                value: (AttrValue::None, Span::call_site()),
            })
        } else {
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
            let value = content.parse::<AttrValue>()?;
            let span = value.span();
            Ok(ir::Marker {
                paren_token,
                ident,
                value: (value, span),
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

impl TryFrom<syn::ItemMod> for ir::Collaboration {
    type Error = Error;

    fn try_from(item_mod: syn::ItemMod) -> Result<Self> {
        if item_mod.vis != syn::Visibility::Inherited {
            bail!(
                item_mod.vis,
                "collaboration module must have no visibility modifier",
            )
        }

        let items = match &item_mod.content {
            None => bail!(
                item_mod,
                "collaboration module must be inline, e.g. `mod m {{ ... }}`",
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
        let (contracts, impl_blocks) = split_items(liquid_items, span)?;
        let mod_ident = item_mod.ident;

        Ok(Self {
            mod_token: item_mod.mod_token,
            mod_ident,
            contracts,
            rust_items,
            all_item_rights: impl_blocks,
        })
    }
}

impl TryFrom<syn::ItemStruct> for ir::ItemContract {
    type Error = Error;

    fn try_from(item_struct: syn::ItemStruct) -> Result<Self> {
        match item_struct.vis {
            syn::Visibility::Public(_) => (),
            _ => bail!(
                item_struct,
                "visibility of `#[liquid(contract)]` struct should be `pub`",
            ),
        }

        if item_struct.generics.type_params().count() > 0 {
            bail!(
                item_struct.generics,
                "generics are not allowed for `#[liquid(contract)]` struct"
            )
        }

        let span = item_struct.span();
        let fields = match item_struct.fields {
            syn::Fields::Named(ref named_fields) => {
                let fields = &named_fields.named;
                for field in fields {
                    let visibility = &field.vis;
                    match visibility {
                        syn::Visibility::Inherited => (),
                        _ => bail!(
                            field,
                            "visibility declaration is not allowed for fields in
                             `#[liquid(contract)]` struct"
                        ),
                    }
                }
                named_fields
            }
            syn::Fields::Unnamed(_) => bail!(
                item_struct,
                "tuple-struct is not allowed for `#[liquid(contract)]` struct"
            ),
            syn::Fields::Unit => bail!(
                item_struct,
                "unit-struct is not allowed for `#[liquid(contract)]` struct"
            ),
        };

        let mut field_signers = Vec::new();
        for field in &fields.named {
            let markers = filter_map_liquid_attributes(&field.attrs)?;
            let signers = markers
                .iter()
                .filter(|marker| marker.ident == "signers")
                .collect::<Vec<_>>();
            if signers.is_empty() {
                continue;
            }

            let name = field.ident.as_ref().unwrap();
            if signers.len() > 1 {
                bail!(
                    field,
                    "duplicated `#[liquid(signers)]` attributes defined for this field"
                )
            }

            field_signers.push(ir::Selector {
                from: ir::SelectFrom::This(name.clone()),
                with: match &signers[0].value {
                    (AttrValue::None, _) => None,
                    (AttrValue::LitStr(path), span) => {
                        let select_path = parse_select_path(&path.value(), *span)?;
                        Some(select_path)
                    }
                    (AttrValue::Ident(ident), span) => {
                        if ident == "inherited" {
                            Some(ir::SelectWith::Inherited(field.ty.clone()))
                        } else {
                            bail_span!(
                                *span,
                                "invalid indicators of signers: `{}`",
                                ident
                            )
                        }
                    }
                },
            });
        }

        if field_signers.is_empty() {
            bail!(item_struct, "this contract has no signers")
        }

        let ident = item_struct.ident;
        let state_name = generate_state_name(&ident);
        let mated_name = generate_mated_name(&ident);

        Ok(ir::ItemContract {
            attrs: item_struct.attrs,
            struct_token: item_struct.struct_token,
            ident,
            fields: fields.clone(),
            field_signers,
            state_name,
            mated_name,
            span,
        })
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
                    "encountered unsupported argument syntax for the right",
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
                "`self`, `mut self`, `&self` or `&mut self` is mandatory for contract \
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

        if let ir::FnArg::Typed(ident_type) = &inputs[0] {
            bail_span!(
                ident_type.span(),
                "first argument of a right must be `self`, `mut self`, `&self` or `&mut \
                 self`",
            )
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
        if output_args_count > 18 {
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

impl TryFrom<(syn::ImplItemMethod, Selectors, Ident)> for ir::Right {
    type Error = Error;

    fn try_from(
        (method, outer_owners, from): (syn::ImplItemMethod, Selectors, Ident),
    ) -> Result<Self> {
        if method.defaultness.is_some() {
            bail!(
                method.defaultness,
                "`default` modifiers are not allowed for right",
            )
        }

        match method.vis {
            syn::Visibility::Public(_) => (),
            _ => bail!(method.vis, "the visibility of a right must be `pub`",),
        }

        let markers = filter_map_liquid_attributes(&method.attrs)?;
        let owners = markers
            .iter()
            .filter(|marker| marker.ident == "belongs_to")
            .collect::<Vec<_>>();

        let owners = if !outer_owners.is_empty() {
            if !owners.is_empty() {
                bail! {
                    method,
                    "`#[liquid(belongs_to)]` is not allowed to be used in impl block which tagged with `#[liquid(rights_belong_to)]` attribute"
                }
            }
            outer_owners
        } else {
            if owners.len() > 1 {
                bail! {
                    method,
                    "duplicated `#[liquid(belongs_to)]` attributes defined for this right"
                }
            }

            if owners.is_empty() {
                bail! {
                    method,
                    "no `#[liquid(belongs_to)]` attribute defined for this right"
                }
            }

            let owners = &owners[0];
            match &owners.value {
                (AttrValue::None, _) => bail!(&owners.ident, "no right owner specified"),
                (AttrValue::Ident(ident), _) => {
                    bail!(ident, "invalid indicator of right owner: `{}`", ident)
                }
                (AttrValue::LitStr(path), span) => {
                    let path = &path.value();
                    let owner_parser = OwnersParser::new(&path, *span, true)?;
                    owner_parser.parse()?
                }
            }
        };

        let span = method.span();
        let sig = ir::Signature::try_from(&method.sig)?;

        Ok(Self {
            attrs: method.attrs,
            owners,
            sig,
            body: method.block,
            from,
            span,
        })
    }
}

impl TryFrom<(syn::ItemImpl, Selectors)> for ir::ItemRights {
    type Error = Error;

    fn try_from((item_impl, outer_owners): (syn::ItemImpl, Selectors)) -> Result<Self> {
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
            None => bail!(type_path.path, "don't use type path for rights",),
        };

        let mut functions = Vec::new();
        let mut constants = Vec::new();
        for item in item_impl.items.into_iter() {
            match item {
                syn::ImplItem::Method(method) => {
                    functions.push(ir::Right::try_from((
                        method,
                        outer_owners.clone(),
                        ident.clone(),
                    ))?);
                }
                syn::ImplItem::Const(constant) => {
                    constants.push(constant);
                }
                unsupported => bail!(
                    unsupported,
                    "only methods and constants are supported inside impl blocks of \
                     rights",
                ),
            }
        }

        let state_name = generate_state_name(&ident);
        let mated_name = generate_mated_name(&ident);

        Ok(Self {
            attrs: item_impl.attrs,
            impl_token: item_impl.impl_token,
            ident,
            state_name,
            mated_name,
            brace_token: item_impl.brace_token,
            rights: functions,
            constants,
        })
    }
}

impl TryFrom<syn::Item> for ir::Item {
    type Error = Error;

    fn try_from(item: syn::Item) -> Result<Self> {
        match item.clone() {
            syn::Item::Struct(item_struct) => {
                let markers = filter_map_liquid_attributes(&item_struct.attrs)?;
                let is_contract = markers.iter().any(|marker| marker.ident == "contract");
                if is_contract {
                    ir::ItemContract::try_from(item_struct)
                        .map(Into::into)
                        .map(Box::new)
                        .map(ir::Item::Liquid)
                } else {
                    Ok(ir::Item::Rust(Box::new(item)))
                }
            }
            syn::Item::Impl(item_impl) => {
                let markers = filter_map_liquid_attributes(&item_impl.attrs)?;
                let rights_marker = markers
                    .iter()
                    .filter(|marker| {
                        marker.ident == "rights" || marker.ident == "rights_belong_to"
                    })
                    .collect::<Vec<_>>();

                if rights_marker.len() > 1 {
                    bail!(
                        item_impl,
                        "duplicated `#[liquid(rights)]` or #[liquid(rights_belong_to)] \
                         attributes defined for this impl block"
                    )
                }

                if rights_marker.is_empty() {
                    bail!(
                        item_impl,
                        "`impl` block in collaboration must be tagged with \
                         `#[liquid(rights)]` or `#[liquid(rights_belong_to)]` attribute"
                    )
                } else {
                    let rights_marker = &rights_marker[0];
                    let ident = rights_marker.ident.to_string();

                    let outer_owners = if ident == "rights_belong_to" {
                        match &rights_marker.value {
                            (AttrValue::None, _) => {
                                bail!(&rights_marker.ident, "no right owner specified")
                            }
                            (AttrValue::Ident(ident), _) => bail!(
                                ident,
                                "invalid indicator of right owner: `{}`",
                                ident
                            ),
                            (AttrValue::LitStr(path), span) => {
                                let path = path.value();
                                let owner_parser = OwnersParser::new(&path, *span, true)?;
                                owner_parser.parse()?
                            }
                        }
                    } else {
                        vec![]
                    };

                    ir::ItemRights::try_from((item_impl, outer_owners))
                        .map(Into::into)
                        .map(Box::new)
                        .map(ir::Item::Liquid)
                }
            }
            _ => Ok(ir::Item::Rust(Box::new(item))),
        }
    }
}

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

use super::{Function, ItemEvent, ItemStorage, LiquidItem, Marker};
use proc_macro2::Span;
use syn::{spanned::Spanned, Result};

pub fn is_liquid_attribute(attr: &syn::Attribute) -> bool {
    attr.path.is_ident("liquid")
}

pub fn filter_non_liquid_attributes<'a, I>(
    attrs: I,
) -> impl Iterator<Item = &'a syn::Attribute>
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    attrs.into_iter().filter(|attr| !is_liquid_attribute(attr))
}

pub fn filter_map_liquid_attributes<'a, I>(attrs: I) -> impl Iterator<Item = Marker>
where
    I: IntoIterator<Item = &'a syn::Attribute>,
{
    use core::convert::TryFrom;
    attrs
        .into_iter()
        .cloned()
        .filter_map(|attr| Marker::try_from(attr).ok())
}

pub fn split_items(
    items: Vec<LiquidItem>,
) -> Result<(ItemStorage, Vec<ItemEvent>, Vec<Function>)> {
    use either::Either;
    use itertools::Itertools;

    let (mut storages, others): (Vec<_>, Vec<_>) =
        items.into_iter().partition_map(|item| match item {
            LiquidItem::Storage(storage) => Either::Left(storage),
            other => Either::Right(other),
        });
    let storage = match storages.len() {
        0 => {
            return Err(format_err_span!(
                Span::call_site(),
                "no #[liquid(storage)] struct found in this contract"
            ))
        }
        1 => storages.pop().unwrap(),
        _ => {
            return Err(format_err_span!(
                storages[1].span(),
                "duplicate #[liquid(storage)] struct definition found here"
            ))
        }
    };

    let (events, impl_blocks): (Vec<_>, Vec<_>) =
        others.into_iter().partition_map(|item| match item {
            LiquidItem::Event(event) => Either::Left(event),
            LiquidItem::Impl(impl_block) => Either::Right(impl_block),
            _ => unreachable!(),
        });

    for item_impl in &impl_blocks {
        if item_impl.ty != storage.ident {
            bail!(
                item_impl.ty,
                "liquid impl blocks need to be implemented for the #[liquid(storage)] \
                 struct"
            )
        }
    }

    let functions = impl_blocks
        .into_iter()
        .map(|block| block.functions)
        .flatten()
        .collect::<Vec<_>>();

    Ok((storage, events, functions))
}

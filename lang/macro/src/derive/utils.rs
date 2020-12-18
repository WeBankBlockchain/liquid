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

use liquid_prelude::vec::Vec;
use proc_macro2::{Ident, Span};
use syn::{self, parse::Result, Data, DeriveInput, Fields, Type};

#[allow(dead_code)]
pub fn struct_syntax_check(ast: &DeriveInput) -> Result<(Vec<&Ident>, Vec<&Type>, Span)> {
    let struct_data = match &ast.data {
        Data::Struct(ref struct_data) => struct_data,
        Data::Enum(ref enum_data) => {
            bail!(&enum_data.enum_token, "enums are not supported")
        }
        Data::Union(ref union_data) => {
            bail!(&union_data.union_token, "unions are not supported")
        }
    };

    match &ast.vis {
        syn::Visibility::Public(_) => (),
        _ => bail!(ast, "the visibility of this type should be `pub`"),
    }

    if ast.generics.type_params().count() > 0 {
        bail!(&ast.generics, "generic structs are not supported")
    }

    let fields = &struct_data.fields;
    let (field_names, field_tys): (Vec<_>, Vec<_>) = match fields {
        Fields::Named(fields_named) => fields_named
            .named
            .iter()
            .map(|field| (field.ident.as_ref().unwrap(), &field.ty))
            .unzip(),
        Fields::Unnamed(fields_unnamed) => {
            bail!(fields_unnamed, "unnamed structs are not supported")
        }
        Fields::Unit => bail!(fields, "unit structs are not supported"),
    };

    if field_names.is_empty() {
        bail!(fields, "empty structs are not supported")
    }

    use syn::spanned::Spanned;
    Ok((field_names, field_tys, fields.span()))
}

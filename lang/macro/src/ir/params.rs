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

use crate::ir::MetaVersion;
use core::convert::TryFrom;
use derive_more::From;
use proc_macro2::{Ident, Span};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    LitStr, Result, Token,
};

macro_rules! params {
    ($t:ident) => {
        paste::item! {
            pub struct [<$t Params>] {
                pub params: Punctuated< [<$t MetaParam>], Token![,]>,
            }

            impl Parse for [<$t Params>] {
                fn parse(input: ParseStream) -> Result<Self> {
                    let params = Punctuated::parse_terminated(input)?;
                    Ok(Self { params })
                }
            }
            impl Spanned for [<$t Params>] {
                fn span(&self) -> Span {
                    if self.params.is_empty() {
                        Span::call_site()
                    } else {
                        self.params
                            .first()
                            .unwrap()
                            .span()
                            .join(self.params.last().unwrap().span())
                            .expect("params in `self` must be within the same file")
                    }
                }
            }
        }
    };
}

// Parameters given to liquid's `#[contract(..)]` attribute.
//
// # Example
//
// ```no_compile
// #[liquid::contract(version = "0.1.0")]
// ```
params!(Contract);

#[derive(From)]
pub enum ContractMetaParam {
    Version(ParamVersion),
}

impl Parse for ContractMetaParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.fork().parse::<Ident>()?;
        match ident.to_string().as_str() {
            "version" => input.parse::<ParamVersion>().map(Into::into),
            unknown => Err(format_err_span!(
                ident.span(),
                "unknown parameter: {}",
                unknown
            )),
        }
    }
}

impl Spanned for ContractMetaParam {
    fn span(&self) -> Span {
        match self {
            ContractMetaParam::Version(param) => param.span(),
        }
    }
}

impl ContractMetaParam {
    pub fn ident(&self) -> &Ident {
        match &self {
            ContractMetaParam::Version(param) => &param.ident,
        }
    }
}

pub struct ParamVersion {
    /// The `version` identifier
    pub ident: Ident,
    /// The `=` token
    pub eq_token: Token![=],
    /// The input version string
    pub value: LitStr,
    /// The decoded semantic version
    pub version: MetaVersion,
}

impl Parse for ParamVersion {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        if ident != "version" {
            bail!(ident, "invalid identifier for meta version info");
        }
        let eq_token = input.parse()?;
        let value: LitStr = input.parse()?;
        let content = value.value();
        let version = MetaVersion::try_from(&content).map_err(|_| {
            format_err_span!(
                value.span(),
                "couldn't match provided version as a semantic version string: {}",
                content,
            )
        })?;
        Ok(Self {
            ident,
            eq_token,
            value,
            version,
        })
    }
}

impl Spanned for ParamVersion {
    fn span(&self) -> Span {
        self.ident
            .span()
            .join(self.value.span())
            .expect("both spans are in the same file AND we are using nightly Rust")
    }
}

// Parameters given to liquid's `#[interface(..)]` attribute.
//
// # Example
//
// ```no_compile
// #[liquid::interface(name = auto)]
// ```
params!(Interface);

#[derive(From)]
pub enum InterfaceMetaParam {
    Name(ParamName),
}

impl Parse for InterfaceMetaParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.fork().parse::<Ident>()?;
        match ident.to_string().as_str() {
            "name" => input.parse::<ParamName>().map(Into::into),
            unknown => Err(format_err_span!(
                ident.span(),
                "unknown parameter: {}",
                unknown
            )),
        }
    }
}

impl Spanned for InterfaceMetaParam {
    fn span(&self) -> Span {
        match self {
            InterfaceMetaParam::Name(param) => param.span(),
        }
    }
}

impl InterfaceMetaParam {
    pub fn ident(&self) -> &Ident {
        match &self {
            InterfaceMetaParam::Name(param) => &param.ident,
        }
    }
}

pub enum NameValue {
    Auto,
    Name(String),
}

pub struct ParamName {
    /// The `name` identifier
    pub ident: Ident,
    /// The `=` token
    pub eq_token: Token![=],
    /// The name specified via user, maybe `auto` or a literal string.
    pub value: NameValue,
    /// The span of `name` parameter.
    pub span: Span,
}

impl Parse for ParamName {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        if ident != "name" {
            bail!(ident, "invalid identifier for meta name info");
        }
        let span = ident.span();
        let eq_token = input.parse::<Token![=]>()?;

        let value = input.parse::<LitStr>();
        let value = if let Ok(lit_str) = value {
            let name = lit_str.value();

            if name.starts_with("r#") {
                return Err(format_err_span!(
                    lit_str.span(),
                    "invalid interface name: `{}`",
                    name,
                ));
            }

            if syn::parse_str::<Ident>(&name).is_err() {
                return Err(format_err_span!(
                    lit_str.span(),
                    "invalid interface name: `{}`",
                    name,
                ));
            }

            span.join(lit_str.span())
                .expect("both spans are in the same file AND we are using nightly Rust");
            NameValue::Name(name)
        } else {
            let value: Ident = input.parse()?;
            if value != "auto" {
                bail!(
                    ident,
                    "invalid interface name, please specified meta name info as `auto` \
                     or a literal string`"
                );
            }
            span.join(ident.span())
                .expect("both spans are in the same file AND we are using nightly Rust");
            NameValue::Auto
        };

        Ok(Self {
            ident,
            eq_token,
            value,
            span,
        })
    }
}

impl Spanned for ParamName {
    fn span(&self) -> Span {
        self.span
    }
}

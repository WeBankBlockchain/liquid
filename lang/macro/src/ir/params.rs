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
use liquid_primitives::HashType;
use proc_macro2::{Ident, Span};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    LitStr, Result, Token,
};

/// Parameters given to liquid's `#[contract(..)]` attribute.
///
/// # Example
///
/// ```no_compile
/// #[liquid::contract(version = "0.1.0")]
/// ```
pub struct Params {
    pub params: Punctuated<MetaParam, Token![,]>,
}

impl Parse for Params {
    fn parse(input: ParseStream) -> Result<Self> {
        let params = Punctuated::parse_terminated(input)?;
        Ok(Self { params })
    }
}

impl Spanned for Params {
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

#[derive(From)]
pub enum MetaParam {
    Version(ParamVersion),
    HashType(ParamHashType),
}

impl Parse for MetaParam {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.fork().parse::<Ident>()?;
        match ident.to_string().as_str() {
            "version" => input.parse::<ParamVersion>().map(Into::into),
            "hash_type" => input.parse::<ParamHashType>().map(Into::into),
            unknown => Err(format_err_span!(
                ident.span(),
                "unknown parameter: {}",
                unknown
            )),
        }
    }
}

impl Spanned for MetaParam {
    fn span(&self) -> Span {
        match self {
            MetaParam::Version(param) => param.span(),
            MetaParam::HashType(param) => param.span(),
        }
    }
}

impl MetaParam {
    pub fn ident(&self) -> &Ident {
        match &self {
            MetaParam::Version(param) => &param.ident,
            MetaParam::HashType(param) => &param.ident,
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

pub struct ParamHashType {
    /// The `hash_type` identifier
    pub ident: Ident,
    /// The `=` token
    pub eq_token: Token![=],
    /// The input hash type string
    pub value: LitStr,
    /// The decoded hash type
    pub hash_type: HashType,
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

impl Parse for ParamHashType {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        if ident != "hash_type" {
            bail!(ident, "invalid identifier for hash type info");
        }
        let eq_token = input.parse()?;
        let value: LitStr = input.parse()?;
        let content = value.value();
        let hash_type = HashType::try_from(&content).map_err(|_| {
            format_err_span!(value.span(), "invalid hash type: {}", content,)
        })?;
        Ok(Self {
            ident,
            eq_token,
            value,
            hash_type,
        })
    }
}

impl Spanned for ParamHashType {
    fn span(&self) -> Span {
        self.ident
            .span()
            .join(self.value.span())
            .expect("both spans are in the same file AND we are using nightly Rust")
    }
}

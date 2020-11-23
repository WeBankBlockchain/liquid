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

use crate::{codegen::GenerateCode, ir, utils::check_idents};
use core::convert::TryFrom;
use proc_macro2::TokenStream as TokenStream2;
use syn::Result;

pub fn generate(attr: TokenStream2, input: TokenStream2) -> TokenStream2 {
    match generate_impl(attr, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn generate_impl(attr: TokenStream2, input: TokenStream2) -> Result<TokenStream2> {
    check_idents(input.clone())?;

    let params = syn::parse2::<ir::ContractParams>(attr)?;
    let item_mod = syn::parse2::<syn::ItemMod>(input)?;
    let liquid_ir = ir::Contract::try_from((params, item_mod))?;
    Ok(liquid_ir.generate_code())
}

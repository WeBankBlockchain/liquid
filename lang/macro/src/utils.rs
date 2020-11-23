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

use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use std::string::ToString;
use syn::Error;

pub fn check_idents(input: TokenStream2) -> Result<(), Error> {
    for token in input.into_iter() {
        match token {
            TokenTree::Ident(ident) => {
                if ident.to_string().starts_with("__liquid")
                    || ident.to_string().starts_with("__LIQUID")
                {
                    return Err(format_err_span!(
                        ident.span(),
                        "identifiers starting with `__liquid` or `__LIQUID` are \
                         forbidden in contract"
                    ));
                }
            }
            TokenTree::Group(group) => {
                check_idents(group.stream())?;
            }
            _ => (),
        }
    }

    Ok(())
}

pub fn calculate_fn_id<T>(ident: &T) -> usize
where
    T: ToString,
{
    let ident_hash = liquid_primitives::hash::hash(ident.to_string().as_bytes());
    // In declaration of array where the `fn_id` is used, the length of array must be an `usize` integer.
    // **DO NOT** attempt to change the return type of this function.
    u32::from_be_bytes([ident_hash[0], ident_hash[1], ident_hash[2], ident_hash[3]])
        as usize
}

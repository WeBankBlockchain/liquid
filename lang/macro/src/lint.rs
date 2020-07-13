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

use proc_macro2::{Ident, TokenStream as TokenStream2, TokenTree};
use syn::Error;

pub fn check_idents<P, E>(input: TokenStream2, pred: P, or_err: E) -> Result<(), Error>
where
    P: Copy + Fn(&Ident) -> bool,
    E: Copy + Fn(&Ident) -> Error,
{
    for token in input.into_iter() {
        match token {
            TokenTree::Ident(ident) => {
                if !pred(&ident) {
                    return Err(or_err(&ident));
                }
            }
            TokenTree::Group(group) => {
                check_idents(group.stream(), pred, or_err)?;
            }
            _ => (),
        }
    }

    Ok(())
}

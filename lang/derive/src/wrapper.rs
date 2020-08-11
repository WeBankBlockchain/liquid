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

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn generate_wrapper(impls: TokenStream2) -> TokenStream2 {
    quote! {
        const _: () = {
            use liquid_ty_mapping as _ty_mapping;

            #[cfg(feature = "std")]
            mod __std {
                pub use ::std::vec::Vec;
            }

            #[cfg(not(feature = "std"))]
            mod __std {
                extern crate alloc;
                pub use alloc::vec::Vec;
            }

            #impls
        };
    }
}

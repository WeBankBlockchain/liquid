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

mod abi_gen;
mod assets;
mod dispatch;
mod events;
mod storage;
mod testable;

use crate::{common::GenerateCode, contract::ir, utils};
use abi_gen::AbiGen;
use assets::Assets;
use dispatch::Dispatch;
use events::{EventStructs, Events};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use storage::Storage;
use testable::Testable;

impl GenerateCode for ir::Contract {
    fn generate_code(&self) -> TokenStream2 {
        let ident = &self.ident;
        let storage_ident = &self.storage.ident;
        let types = utils::generate_primitive_types();
        let storage = Storage::from(self).generate_code();
        let events = Events::from(self).generate_code();
        let assets = Assets::from(self).generate_code();
        // let asset_idents = self.assets.iter().map(|asset| let asset_ident = asset.ident;
        //     quote! {
        //         #[cfg(test)]
        //         #[allow(non_snake_case)]
        //         pub type #asset_ident = __liquid_private::#asset_ident;
        //     }
        // );
        let event_struct = EventStructs::from(self).generate_code();
        let dispatch = Dispatch::from(self).generate_code();
        let testable = Testable::from(self).generate_code();
        let abi = AbiGen::from(self).generate_code();
        let rust_items = &self.rust_items;

        quote! {
            mod #ident {
                #[allow(unused_imports)]
                use liquid_lang::intrinsics::*;
                #[allow(unused_imports)]
                use liquid_lang::Env;
                #types

                mod __liquid_private {
                    #[allow(unused_imports)]
                    use super::*;

                    #storage
                    #assets
                    #events
                    #dispatch
                    #testable
                    #abi
                }

                #[cfg(test)]
                #[allow(non_snake_case)]
                pub type #storage_ident = __liquid_private::TestableStorage;


                // #(#asset_idents)*

                #[cfg(not(test))]
                #[allow(non_snake_case)]
                pub type #storage_ident = __liquid_private::Storage;

                #[cfg(feature = "liquid-abi-gen")]
                pub use __liquid_private::__LIQUID_ABI_GEN;

                #event_struct

                #(#rust_items)*
            }

            #[cfg(feature = "liquid-abi-gen")]
            pub use crate::#ident::__LIQUID_ABI_GEN;
        }
    }
}

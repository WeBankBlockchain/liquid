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

use liquid_macro::seq;
pub use crate::types::fixed_size_bytes::*;

#[macro_export]
// macro_rules! fixed_point {
//     (
//         $USymbol:ty, 
//         $UINT: ty,
//         $UIntBytes: expr,
//         $UFrac: ty,
//         $UFracBytes: expr
//     ) => {
//         pub struct {
//             wb_symbol: $USymbol,
//             wb_int: $UINT,
//             wb_frac: $UFrac,
//         };
//         impl scale::Encode for s1{
//             fn encode(&Self) -> Vec<u8> {
//                 wb_symbol.encode();
//                 wb_int.encode();
//                 wb_frac.encode()
//             }
//         }
//     };
// }

seq!(M in 1..3{
    seq!(N in 1..=14{
        // fixed_point!{
        //     types1,
        //     types#N,
        //     stringify!(#N),
        //     types#M,
        //     stringify!(#M)
        // }

    });
});


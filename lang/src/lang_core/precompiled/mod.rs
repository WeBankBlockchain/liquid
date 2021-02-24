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

mod cns;

use cfg_if::cfg_if;
pub use cns::Cns;
use liquid_prelude::vec::{self, Vec};
use liquid_primitives::types::Address;

pub const CNS_ADDRESS: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x10, 0x04,
]);

struct ReturnDataWrapper {
    pub data: Vec<u8>,
}

cfg_if! {
    if #[cfg(feature = "solidity-compatible")] {
        use liquid_abi_codec::{Decode, Input, TypeInfo};
        use liquid_primitives::Error;

        impl TypeInfo for ReturnDataWrapper {
            fn is_dynamic() -> bool {
                true
            }
        }

        impl Decode for ReturnDataWrapper {
            fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
                let remaining_len = value.remaining_len();
                let mut buffer = vec::from_elem(0, remaining_len);
                value.read_bytes(&mut buffer)?;
                Ok(Self { data: buffer })
            }
        }
    } else {
        use scale::{Decode, Error, Input};

        impl Decode for ReturnDataWrapper {
            fn decode<I: Input>(value: &mut I) -> Result<Self, Error> {
                let remaining_len = match value.remaining_len()? {
                    Some(len) => len,
                    _ => 0,
                };
                let mut buffer = vec::from_elem(0, remaining_len);
                value.read(&mut buffer)?;
                Ok(Self { data: buffer })
            }
        }
    }
}

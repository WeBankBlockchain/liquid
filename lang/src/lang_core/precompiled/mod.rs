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

pub use cns::Cns;
use lazy_static::lazy_static;
use liquid_prelude::vec::{self, Vec};
use liquid_primitives::types::Address;

lazy_static! {
    pub static ref CNS_ADDRESS: Address =
        "0x0000000000000000000000000000000000001004".into();
}

struct ReturnDataWrapper {
    pub data: Vec<u8>,
}

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

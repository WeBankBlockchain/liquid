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

use liquid_abi_coder::{Decode, Encode, Error, Input};
use liquid_prelude::vec::{from_elem, Vec};
use liquid_primitives::Selector;

pub struct CallData {
    pub selector: Selector,
    pub data: Vec<u8>,
}

impl Decode for CallData {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let remaining_len = input.remaining_len();
        if remaining_len < 4 {
            return Err(Error::from("require at least 4 bytes for input data"));
        }
        let mut selector: Selector = Default::default();
        input.read_bytes(&mut selector)?;
        let mut data = from_elem(0, input.remaining_len());
        input.read_bytes(&mut data)?;
        Ok(Self { selector, data })
    }
}

impl Encode for CallData {
    fn encode(&self) -> Vec<u8> {
        let mut buf = self.selector.to_vec();
        buf.extend(self.data.as_slice());
        buf
    }
}

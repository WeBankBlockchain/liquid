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

use scale::{Decode, Encode, Error, Input};

use liquid_prelude::vec::{from_elem, Vec};
use liquid_primitives::Selector;

pub struct CallData {
    pub selector: Selector,
    pub data: Vec<u8>,
}

impl Decode for CallData {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let remaining_len = input.remaining_len()?;
        match remaining_len {
            None => return Err(Error::from("empty input data")),
            Some(len) if len < 4 => {
                return Err(Error::from("require at least 4 bytes for input data"))
            }
            _ => (),
        }
        let mut selector = [0u8; 4];
        input.read(&mut selector)?;
        let remaining_len = if let Some(len) = input.remaining_len()? {
            len
        } else {
            return Err(Error::from("unable to read remaining input data"));
        };
        let mut data = from_elem(0, remaining_len);
        input.read(&mut data)?;

        let selector = u32::from_le_bytes(selector);
        Ok(Self { selector, data })
    }
}

impl Encode for CallData {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.selector.to_le_bytes());
        buf.extend(self.data.as_slice());
        buf
    }
}

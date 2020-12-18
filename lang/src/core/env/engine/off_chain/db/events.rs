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

use cfg_if::cfg_if;
use liquid_primitives::{types::Hash, Topics};

cfg_if! {
    if #[cfg(feature = "solidity-compatible")] {
        use liquid_abi_codec::{Decode, Encode};
    } else {
        use scale::{Decode, Encode};
    }
}

#[derive(Clone)]
pub struct Event {
    pub data: Vec<u8>,
    pub topics: Vec<Hash>,
}

impl Event {
    pub fn new<E>(event: E) -> Self
    where
        E: Topics + Encode,
    {
        Self {
            data: event.encode(),
            topics: event.topics(),
        }
    }

    pub fn decode_data<R>(&self) -> R
    where
        R: Decode,
    {
        <R as Decode>::decode(&mut self.data.as_slice()).unwrap()
    }
}

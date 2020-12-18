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

use crate::core::env::error::{EnvError, Result};
use liquid_prelude::{collections::BTreeMap, vec::Vec};
use scale::{Decode, Encode};

pub struct ContractStorage {
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl ContractStorage {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn get_storage<R>(&self, key: &[u8]) -> Result<R>
    where
        R: Decode,
    {
        match self.entries.get(key) {
            Some(encoded) => <R as Decode>::decode(&mut &encoded[..]).map_err(Into::into),
            None => Err(EnvError::UnableToReadFromStorage),
        }
    }

    pub fn set_storage<V>(&mut self, key: &[u8], value: &V)
    where
        V: Encode,
    {
        self.entries.insert(key.to_vec(), value.encode());
    }

    pub fn remove_storage(&mut self, key: &[u8]) {
        self.entries.remove(key);
    }
}

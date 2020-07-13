use crate::{EnvError, Result};
use scale::{Decode, Encode};
use liquid_prelude::{collections::BTreeMap, vec::Vec};
use liquid_primitives::Key;

pub struct ContractStorage {
    entries: BTreeMap<Key, Vec<u8>>,
}

impl ContractStorage {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn get_storage<R>(&self, key: Key) -> Result<R>
    where
        R: Decode,
    {
        match self.entries.get(key) {
            Some(encoded) => <R as Decode>::decode(&mut &encoded[..]).map_err(Into::into),
            None => Err(EnvError::UnableToReadFromStorage),
        }
    }

    pub fn set_storage<V>(&mut self, key: Key, value: &V)
    where
        V: Encode,
    {
        self.entries.insert(key, value.encode());
    }
}

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

use super::{ArbitraryKey, U32Key};
use crate::env as liquid_env;
use core::{cell::RefCell, marker::PhantomData};
use liquid_prelude::vec::Vec;
use liquid_primitives::Key;
use scale::{Decode, Encode};

const SEP: u8 = 0x24; // '$'

#[derive(Debug)]
pub struct TypedChunk<T, M> {
    key_buf: RefCell<Vec<u8>>,
    prefix_len: usize,
    marker: PhantomData<fn() -> (T, M)>,
}

impl<T> TypedChunk<T, U32Key> {
    const KEY_SUFFIX: [u8; 5] = [SEP, 0x00, 0x00, 0x00, 0x00];

    pub fn new(key: Key) -> Self {
        Self {
            key_buf: RefCell::new({
                let mut vec = Vec::with_capacity(key.len() + Self::KEY_SUFFIX.len());
                vec.extend(key.as_bytes());
                vec.extend(&Self::KEY_SUFFIX);
                vec
            }),
            prefix_len: key.len() + 1,
            marker: Default::default(),
        }
    }
}

impl<T> TypedChunk<T, ArbitraryKey> {
    pub fn new(key: Key) -> Self {
        Self {
            key_buf: RefCell::new({
                let mut vec = Vec::with_capacity(key.len() + 1);
                vec.extend(key.as_bytes());
                vec.push(SEP);
                vec
            }),
            prefix_len: key.len() + 1,
            marker: Default::default(),
        }
    }
}

impl<T> TypedChunk<T, U32Key> {
    fn prepare_inner_key(&self, index: u32) {
        self.key_buf.borrow_mut()[self.prefix_len..]
            .copy_from_slice(&index.to_le_bytes());
    }
}

impl<T> TypedChunk<T, ArbitraryKey> {
    fn prepare_inner_key(&self, index: &[u8]) {
        self.key_buf.borrow_mut().extend(index);
    }
}

impl<T> TypedChunk<T, U32Key>
where
    T: Decode,
{
    pub fn load(&self, index: u32) -> Option<T> {
        self.prepare_inner_key(index);
        liquid_env::get_storage(self.key_buf.borrow().as_slice()).ok()
    }
}

impl<T> TypedChunk<T, ArbitraryKey>
where
    T: Decode,
{
    pub fn load(&self, index: &[u8]) -> Option<T> {
        self.prepare_inner_key(index);
        let ret = liquid_env::get_storage(self.key_buf.borrow().as_slice()).ok();
        self.key_buf.borrow_mut().truncate(self.prefix_len);
        ret
    }
}

impl<T> TypedChunk<T, U32Key>
where
    T: Encode,
{
    pub fn store(&mut self, index: u32, new_value: &T) {
        self.prepare_inner_key(index);
        liquid_env::set_storage(self.key_buf.borrow().as_slice(), new_value);
    }
}

impl<T> TypedChunk<T, ArbitraryKey>
where
    T: Encode,
{
    pub fn store(&mut self, index: &[u8], new_value: &T) {
        self.prepare_inner_key(index);
        liquid_env::set_storage(self.key_buf.borrow().as_slice(), new_value);
        self.key_buf.borrow_mut().truncate(self.prefix_len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<T> TypedChunk<T, U32Key> {
        pub(crate) fn get_inner_key(&self, index: U32Key) -> Vec<u8> {
            self.prepare_inner_key(index);
            self.key_buf.borrow().clone()
        }
    }

    impl<T> TypedChunk<T, ArbitraryKey> {
        pub(crate) fn get_inner_key(&self, index: ArbitraryKey) -> Vec<u8> {
            self.prepare_inner_key(index);
            let ret = self.key_buf.borrow().clone();
            self.key_buf.borrow_mut().truncate(self.prefix_len);
            ret
        }
    }

    fn dummy_chunk_u32_key() -> TypedChunk<u32, U32Key> {
        TypedChunk::<u32, U32Key>::new("var")
    }

    fn dummy_chunk_arbitrary_key() -> TypedChunk<u32, ArbitraryKey> {
        TypedChunk::<u32, ArbitraryKey>::new("var")
    }

    #[test]
    fn u32_key() {
        const TEST_LEN: u32 = 5;
        let mut chunk = dummy_chunk_u32_key();

        for i in 0..TEST_LEN {
            let mut expected_key = vec![];
            expected_key.extend_from_slice("var$".as_bytes());
            expected_key.extend_from_slice(&i.to_le_bytes());

            assert_eq!(chunk.load(i), None);
            assert_eq!(chunk.get_inner_key(i), expected_key);
        }

        for i in 0..TEST_LEN {
            chunk.store(i, &i);
        }

        for i in 0..TEST_LEN {
            assert_eq!(chunk.load(i), Some(i));
        }
    }

    #[test]
    fn arbitrary_key() {
        const TEST_KEYS: [&'static str; 3] = ["Alice", "Bob", "Charlie"];
        let mut chunk = dummy_chunk_arbitrary_key();

        for i in 0..3 {
            let mut expected_key = vec![];
            expected_key.extend_from_slice("var$".as_bytes());
            expected_key.extend_from_slice(TEST_KEYS[i].as_bytes());

            assert_eq!(chunk.load(TEST_KEYS[i].as_bytes()), None);
            assert_eq!(chunk.get_inner_key(TEST_KEYS[i].as_bytes()), expected_key);
        }

        for i in 0..3 {
            chunk.store(TEST_KEYS[i].as_bytes(), &(i as u32));
        }

        for i in 0..3 {
            assert_eq!(chunk.load(TEST_KEYS[i].as_bytes()), Some(i as u32));
        }
    }
}

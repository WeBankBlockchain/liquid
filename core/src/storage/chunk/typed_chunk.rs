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

use crate::env as liquid_env;
use core::{cell::RefCell, marker::PhantomData};
use liquid_prelude::vec::Vec;
use scale::{Decode, Encode};

const SEP: u8 = 0x24; // '$'

#[cfg_attr(feature = "std", derive(Debug))]
pub struct TypedChunk<T> {
    key_buf: RefCell<Vec<u8>>,
    prefix_len: usize,
    marker: PhantomData<fn() -> T>,
}

impl<T> TypedChunk<T> {
    pub fn new(key: &[u8]) -> Self {
        Self {
            key_buf: RefCell::new({
                let mut vec = Vec::with_capacity(key.len() + 1);
                vec.extend_from_slice(key);
                vec.push(SEP);
                vec
            }),
            prefix_len: key.len() + 1,
            marker: Default::default(),
        }
    }
}

impl<T> TypedChunk<T> {
    fn prepare_inner_key<Q: AsRef<[u8]>>(&self, index: Q) {
        self.key_buf.borrow_mut().extend_from_slice(index.as_ref());
    }
}

impl<T> TypedChunk<T> {
    pub fn remove<Q: AsRef<[u8]>>(&mut self, index: Q) {
        self.prepare_inner_key(index);
        liquid_env::remove_storage(self.key_buf.borrow().as_slice());
    }
}

impl<T> TypedChunk<T>
where
    T: Decode,
{
    pub fn load<Q: AsRef<[u8]>>(&self, index: Q) -> Option<T> {
        self.prepare_inner_key(index);
        let ret = liquid_env::get_storage(self.key_buf.borrow().as_slice()).ok();
        self.key_buf.borrow_mut().truncate(self.prefix_len);
        ret
    }
}

impl<T> TypedChunk<T>
where
    T: Encode,
{
    pub fn store<Q: AsRef<[u8]>>(&mut self, index: Q, new_value: &T) {
        self.prepare_inner_key(index);
        liquid_env::set_storage(self.key_buf.borrow().as_slice(), new_value);
        self.key_buf.borrow_mut().truncate(self.prefix_len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<T> TypedChunk<T> {
        pub(crate) fn get_inner_key<Q: AsRef<[u8]>>(&self, index: Q) -> Vec<u8> {
            self.prepare_inner_key(index);
            let ret = self.key_buf.borrow().clone();
            self.key_buf.borrow_mut().truncate(self.prefix_len);
            ret
        }
    }

    fn dummy_chunk() -> TypedChunk<u32> {
        TypedChunk::<u32>::new(b"var")
    }

    #[test]
    fn arbitrary_key() {
        const TEST_KEYS: [&[u8]; 3] = [b"Alice", b"Bob", b"Charlie"];
        let mut chunk = dummy_chunk();

        for i in 0..3 {
            let mut expected_key = vec![];
            expected_key.extend_from_slice(b"var$");
            expected_key.extend_from_slice(TEST_KEYS[i]);

            assert_eq!(chunk.load(TEST_KEYS[i]), None);
            assert_eq!(chunk.get_inner_key(TEST_KEYS[i]), expected_key);
        }

        for i in 0..3 {
            chunk.store(TEST_KEYS[i], &(i as u32));
        }

        for i in 0..3 {
            assert_eq!(chunk.load(TEST_KEYS[i]), Some(i as u32));
        }
    }
}

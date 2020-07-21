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
use crate::storage::{CacheEntry, Flush, TypedChunk};
use core::cell::RefCell;
use liquid_prelude::collections::BTreeMap;
use liquid_primitives::Key;
use scale::{Codec, Decode, Encode};

#[derive(Debug)]
pub struct CachedChunk<T, M> {
    chunk: TypedChunk<T, M>,
    cache: RefCell<BTreeMap<M, CacheEntry<T>>>,
}

macro_rules! impl_chunk_for {
    ($name:tt) => {
        impl<T> CachedChunk<T, $name> {
            pub fn new(key: Key) -> Self {
                Self {
                    chunk: TypedChunk::<T, $name>::new(key),
                    cache: RefCell::new(Default::default()),
                }
            }
        }

        impl<T> CachedChunk<T, $name>
        where
            T: Decode,
        {
            fn get_cache_entry(&self, index: $name) -> Option<&CacheEntry<T>> {
                unsafe { (*self.cache.as_ptr()).get(&index) }
            }

            fn get_cache_entry_mut(&self, index: $name) -> Option<&mut CacheEntry<T>> {
                unsafe { (*self.cache.as_ptr()).get_mut(&index) }
            }

            fn sync_from_storage(&self, index: $name) {
                let loaded = self.chunk.load(index);
                self.cache
                    .borrow_mut()
                    .insert(index, CacheEntry::<T>::new(loaded));
            }

            pub fn get(&self, index: $name) -> Option<&T> {
                let cache_entry = self.get_cache_entry(index);
                if let Some(entry) = cache_entry {
                    entry.get()
                } else {
                    self.sync_from_storage(index);
                    self.get_cache_entry(index).and_then(|entry| entry.get())
                }
            }

            pub fn get_mut(&mut self, index: $name) -> Option<&mut T> {
                let cache_entry = self.get_cache_entry_mut(index);
                if let Some(entry) = cache_entry {
                    entry.get_mut()
                } else {
                    self.sync_from_storage(index);
                    self.get_cache_entry_mut(index)
                        .and_then(|entry| entry.get_mut())
                }
            }

            pub fn take(&mut self, index: $name) -> Option<T> {
                let cache_entry = self.get_cache_entry_mut(index);
                if let Some(entry) = cache_entry {
                    entry.take()
                } else {
                    self.sync_from_storage(index);
                    self.get_cache_entry_mut(index)
                        .and_then(|entry| entry.take())
                }
            }
        }

        impl<T> CachedChunk<T, $name>
        where
            T: Codec,
        {
            pub fn set(&mut self, index: $name, new_val: T) {
                let cache_entry = self.get_cache_entry_mut(index);
                if let Some(entry) = cache_entry {
                    entry.update(Some(new_val));
                    entry.mark_dirty();
                } else {
                    let mut entry = CacheEntry::<T>::new(Some(new_val));
                    entry.mark_dirty();
                    self.cache.borrow_mut().insert(index, entry);
                }
            }

            pub fn mutate_with<F>(&mut self, index: $name, f: F) -> Option<&T>
            where
                F: FnOnce(&mut T),
            {
                if let Some(val) = self.get_mut(index) {
                    f(val);
                    return Some(&*val);
                }
                None
            }

            pub fn put(&mut self, index: $name, new_val: T) -> Option<T> {
                let cache_entry = self.get_cache_entry_mut(index);
                if let Some(entry) = cache_entry {
                    entry.put(Some(new_val))
                } else {
                    self.sync_from_storage(index);
                    self.get_cache_entry_mut(index)
                        .and_then(|entry| entry.put(Some(new_val)))
                }
            }
        }

        impl<T> Flush for CachedChunk<T, $name>
        where
            T: Encode,
        {
            fn flush(&mut self) {
                for (index, entry) in &*self.cache.borrow() {
                    if entry.is_dirty() {
                        if let Some(new_val) = entry.get() {
                            self.chunk.store(*index, new_val);
                        }
                    }
                }
            }
        }
    };
}

impl_chunk_for!(U32Key);
impl_chunk_for!(ArbitraryKey);

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_chunk_u32_key() -> CachedChunk<u32, U32Key> {
        CachedChunk::<u32, U32Key>::new("var")
    }

    fn dummy_chunk_arbitrary_key() -> CachedChunk<u32, ArbitraryKey> {
        CachedChunk::<u32, ArbitraryKey>::new("var")
    }

    #[test]
    fn u32_key() {
        let mut chunk = dummy_chunk_u32_key();
        assert_eq!(chunk.get(0), None);
        chunk.set(0, 5);
        assert_eq!(chunk.get(0), Some(&5));
        chunk.mutate_with(0, |val| *val += 10);
        assert_eq!(chunk.get(0), Some(&15));
    }

    #[test]
    fn arbitrary_key() {
        let mut chunk = dummy_chunk_arbitrary_key();
        assert_eq!(chunk.get("Alice".as_bytes()), None);
        chunk.set("Alice".as_bytes(), 5);
        assert_eq!(chunk.get("Alice".as_bytes()), Some(&5));
        chunk.mutate_with("Alice".as_bytes(), |val| *val += 10);
        assert_eq!(chunk.get("Alice".as_bytes()), Some(&15));
    }

    #[test]
    fn multi_session() {
        let mut chunk_1 = dummy_chunk_u32_key();
        assert_eq!(chunk_1.get(0), None);
        chunk_1.set(0, 5);
        assert_eq!(chunk_1.get(0), Some(&5));
        // Using same key as `chunk_1`, overlapping access but different caches,
        // but cache has not yet been synced.
        assert_eq!(dummy_chunk_u32_key().get(0), None);
        // Sync cache now
        chunk_1.flush();
        // Cache has been flushed before
        assert_eq!(dummy_chunk_u32_key().get(0), Some(&5));

        let mut chunk_2 = dummy_chunk_arbitrary_key();
        assert_eq!(chunk_2.get("Alice".as_bytes()), None);
        chunk_2.set("Alice".as_bytes(), 5);
        assert_eq!(chunk_2.get("Alice".as_bytes()), Some(&5));
        // Using same key as `chunk_2`, overlapping access but different caches,
        // but cache has not yet been synced.
        assert_eq!(dummy_chunk_arbitrary_key().get("Alice".as_bytes()), None);
        // Sync cache now
        chunk_2.flush();
        // Cache has been flushed before
        assert_eq!(
            dummy_chunk_arbitrary_key().get("Alice".as_bytes()),
            Some(&5)
        );
    }
}

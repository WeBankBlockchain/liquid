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

use crate::core::storage::{CacheEntry, Flush, TypedChunk};
use core::{borrow::Borrow, cell::RefCell};
use liquid_prelude::{collections::BTreeMap, vec::Vec};
use scale::{Codec, Decode, Encode};

#[cfg_attr(feature = "std", derive(Debug))]
pub struct CachedChunk<T> {
    chunk: TypedChunk<T>,
    cache: RefCell<BTreeMap<Vec<u8>, CacheEntry<T>>>,
}

impl<T> CachedChunk<T> {
    pub fn insert_cache(&self, key: &[u8], val: Option<T>, is_dirty: bool) {
        let mut entry = CacheEntry::<T>::new(val);
        if is_dirty {
            entry.mark_dirty();
        }
        self.cache.borrow_mut().insert(key.to_vec(), entry);
    }
}

impl<T> CachedChunk<T> {
    pub fn new(key: &[u8]) -> Self {
        Self {
            chunk: TypedChunk::<T>::new(key),
            cache: Default::default(),
        }
    }
    pub fn remove(&mut self, index: &[u8]) {
        self.chunk.remove(index);
        self.cache.borrow_mut().remove(index.borrow());
    }
}

impl<T> CachedChunk<T>
where
    T: Decode,
{
    fn get_cache_entry(&self, index: &[u8]) -> Option<&CacheEntry<T>> {
        unsafe { (*self.cache.as_ptr()).get(index.borrow()) }
    }

    fn get_cache_entry_mut(&self, index: &[u8]) -> Option<&mut CacheEntry<T>> {
        unsafe { (*self.cache.as_ptr()).get_mut(index.borrow()) }
    }

    fn sync_from_storage(&self, index: &[u8]) {
        let loaded = self.chunk.load(index);
        self.insert_cache(index, loaded, false);
    }

    pub fn get(&self, index: &[u8]) -> Option<&T> {
        let cache_entry = self.get_cache_entry(index);
        if let Some(entry) = cache_entry {
            entry.get()
        } else {
            self.sync_from_storage(index);
            self.get_cache_entry(index).and_then(|entry| entry.get())
        }
    }

    pub fn get_mut(&mut self, index: &[u8]) -> Option<&mut T> {
        let cache_entry = self.get_cache_entry_mut(index);
        if let Some(entry) = cache_entry {
            entry.get_mut()
        } else {
            self.sync_from_storage(index);
            self.get_cache_entry_mut(index)
                .and_then(|entry| entry.get_mut())
        }
    }

    pub fn take(&mut self, index: &[u8]) -> Option<T> {
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

impl<T> CachedChunk<T>
where
    T: Codec,
{
    pub fn set(&mut self, index: &[u8], new_val: T) {
        let cache_entry = self.get_cache_entry_mut(index);
        if let Some(entry) = cache_entry {
            entry.update(Some(new_val));
            entry.mark_dirty();
        } else {
            self.insert_cache(index, Some(new_val), true);
        }
    }

    pub fn mutate_with<F>(&mut self, index: &[u8], f: F) -> Option<&T>
    where
        F: FnOnce(&mut T),
    {
        if let Some(val) = self.get_mut(index) {
            f(val);
            return Some(&*val);
        }
        None
    }

    pub fn put(&mut self, index: &[u8], new_val: T) -> Option<T> {
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

impl<T> Flush for CachedChunk<T>
where
    T: Encode,
{
    fn flush(&mut self) {
        for (index, entry) in &*self.cache.borrow() {
            if entry.is_dirty() {
                if let Some(new_val) = entry.get() {
                    self.chunk.store(index, new_val);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_chunk() -> CachedChunk<u32> {
        CachedChunk::<u32>::new(b"var")
    }

    #[test]
    fn arbitrary_key() {
        let mut chunk = dummy_chunk();
        assert_eq!(chunk.get(b"Alice"), None);
        chunk.set(b"Alice", 5);
        assert_eq!(chunk.get(b"Alice"), Some(&5));
        chunk.mutate_with(b"Alice", |val| *val += 10);
        assert_eq!(chunk.get(b"Alice"), Some(&15));
    }

    #[test]
    fn multi_session() {
        let mut chunk_1 = dummy_chunk();
        assert_eq!(chunk_1.get(b"Alice"), None);
        chunk_1.set(b"Alice", 5);
        assert_eq!(chunk_1.get(b"Alice"), Some(&5));
        // Using same key as `chunk_1`, overlapping access but different caches,
        // but cache has not yet been synced.
        assert_eq!(dummy_chunk().get(b"Alice"), None);
        // Sync cache now
        chunk_1.flush();
        // Cache has been flushed before
        assert_eq!(dummy_chunk().get(b"Alice"), Some(&5));
    }
}

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

use crate::{Flush, TypedCell};
use core::cell::RefCell;
use liquid_prelude::boxed::Box;
use liquid_primitives::Key;

#[derive(Debug)]
struct CacheEntry<T> {
    /// If the entry needs to be written back upon a flush.
    dirty: bool,
    /// The value of the cell.
    cell_val: Box<Option<T>>,
}

impl<T> CacheEntry<T> {
    pub fn new(val: Option<T>) -> Self {
        Self {
            dirty: false,
            cell_val: Box::new(val),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Returns an immutable reference to the synchronized cached value.
    pub fn get(&self) -> Option<&T> {
        (&*self.cell_val).as_ref()
    }

    /// Returns a mutable reference to the synchronized cached value.
    ///
    /// This also marks the cache entry as being dirty since
    /// the callee could potentially mutate the value.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.mark_dirty();
        (&mut *self.cell_val).as_mut()
    }

    pub fn update(&mut self, new_val: Option<T>) {
        *self.cell_val = new_val;
    }
}

#[derive(Debug)]
enum Cache<T> {
    /// The cache is desynchronized with the contract storage.
    Desync,
    /// The cache is in sync with the contract storage.
    Sync(CacheEntry<T>),
}

impl<T> Default for Cache<T> {
    fn default() -> Self {
        Cache::Desync
    }
}

impl<T> Cache<T> {
    pub fn update(&mut self, new_val: Option<T>) {
        match self {
            Cache::Desync => *self = Cache::Sync(CacheEntry::new(new_val)),
            Cache::Sync(entry) => entry.update(new_val),
        }
    }

    pub fn is_synced(&self) -> bool {
        match self {
            Cache::Sync(_) => true,
            _ => false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        match self {
            Cache::Desync => false,
            Cache::Sync(entry) => entry.is_dirty(),
        }
    }

    pub fn mark_dirty(&mut self) {
        match self {
            Cache::Desync => (),
            Cache::Sync(entry) => entry.mark_dirty(),
        };
    }

    pub fn mark_clean(&mut self) {
        match self {
            Cache::Desync => (),
            Cache::Sync(entry) => entry.mark_clean(),
        }
    }

    pub fn get(&self) -> Option<&T> {
        match self {
            Cache::Desync => panic!("Error: tried to get the value from a desync cache"),
            Cache::Sync(entry) => entry.get(),
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match self {
            Cache::Desync => panic!("Error: tried to get the value from a desync cache"),
            Cache::Sync(sync_entry) => sync_entry.get_mut(),
        }
    }
}

#[derive(Debug)]
pub struct CachedCell<T> {
    cell: TypedCell<T>,
    cache: RefCell<Cache<T>>,
}

impl<T> CachedCell<T> {
    pub fn new(key: Key) -> Self {
        Self {
            cell: TypedCell::new(key),
            cache: Default::default(),
        }
    }
}

impl<T> Flush for CachedCell<T>
where
    T: scale::Encode,
{
    fn flush(&mut self) {
        if self.cache.borrow().is_dirty() {
            if let Some(val) = self.cache.borrow_mut().get() {
                self.cell.store(val);
            }
            self.cache.borrow_mut().mark_clean();
        }
    }
}

impl<T> CachedCell<T>
where
    T: scale::Decode,
{
    pub fn get(&self) -> Option<&T> {
        self.load();
        unsafe { (*self.cache.as_ptr()).get() }
    }

    fn load(&self) {
        if !self.cache.borrow().is_synced() {
            let loaded = self.cell.load();
            self.cache.borrow_mut().update(loaded);
        }
    }
}

impl<T> CachedCell<T>
where
    T: scale::Encode,
{
    pub fn set(&mut self, new_val: T) {
        self.cache.borrow_mut().update(Some(new_val));
        self.cache.borrow_mut().mark_dirty();
    }
}

impl<T> CachedCell<T>
where
    T: scale::Codec,
{
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.load();
        unsafe { (*self.cache.as_ptr()).get_mut() }
    }

    pub fn mutate_with<F>(&mut self, f: F) -> Option<&T>
    where
        F: FnOnce(&mut T),
    {
        if let Some(val) = self.get_mut() {
            f(val);
            return Some(&*val);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_cell<T>() -> CachedCell<T> {
        CachedCell::new("var")
    }

    #[test]
    fn simple_integer() {
        let mut cell = dummy_cell::<i32>();
        assert_eq!(cell.get(), None);
        cell.set(5);
        assert_eq!(cell.get(), Some(&5));
        cell.mutate_with(|val| *val += 10);
        assert_eq!(cell.get(), Some(&15));
    }

    #[test]
    fn multi_session() {
        let mut cell_1 = dummy_cell::<i32>();
        assert_eq!(cell_1.get(), None);
        cell_1.set(5);
        assert_eq!(cell_1.get(), Some(&5));
        // Using same key as `cell_1`, overlapping access but different caches,
        // but cache has not yet been synced.
        assert_eq!(dummy_cell::<i32>().get(), None);
        // Sync cache now
        cell_1.flush();
        // Cache has been flushed before
        assert_eq!(dummy_cell::<i32>().get(), Some(&5));
    }
}

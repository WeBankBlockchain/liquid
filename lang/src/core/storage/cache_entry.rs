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

use liquid_prelude::boxed::Box;

#[cfg_attr(feature = "std", derive(Debug))]
pub struct CacheEntry<T> {
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

    /// Updates the value of the cached cell.
    pub fn update(&mut self, new_val: Option<T>) {
        *self.cell_val = new_val;
    }

    /// Replaces the cell value from the cache with the new value.
    ///
    /// # Note
    ///
    /// Marks the cache value as dirty.
    pub fn put(&mut self, new_val: Option<T>) -> Option<T> {
        let old_value = core::mem::replace(&mut *self.cell_val, new_val);
        self.mark_dirty();
        old_value
    }

    /// Takes the value in the cache.
    pub fn take(&mut self) -> Option<T> {
        self.put(None)
    }
}

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

use crate::storage::{Bind, CachedCell, CachedChunk, Flush};
use scale::{Codec, Encode};

#[derive(Debug)]
pub struct Vec<T> {
    len: CachedCell<u32>,
    chunk: CachedChunk<T>,
}

pub struct Iter<'a, T> {
    vec: &'a Vec<T>,
    begin: u32,
    end: u32,
}

impl<'a, T> Iter<'a, T> {
    pub(crate) fn new(vec: &'a Vec<T>) -> Self {
        Self {
            vec,
            begin: 0,
            end: vec.len(),
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Codec,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);

        if self.begin == self.end {
            return None;
        }

        let ret = self.vec.get(self.begin);
        self.begin += 1;
        ret
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.end - self.begin) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> where T: Codec {}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: Codec,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);

        if self.begin == self.end {
            return None;
        }

        debug_assert_ne!(self.end, 0);
        self.end -= 1;
        self.vec.get(self.end)
    }
}

impl<T> Bind for Vec<T> {
    fn bind_with(key: &[u8]) -> Self {
        Self {
            len: CachedCell::<u32>::new(key),
            chunk: CachedChunk::<T>::new(key),
        }
    }
}

impl<T> Flush for Vec<T>
where
    T: Encode,
{
    fn flush(&mut self) {
        self.len.flush();
        self.chunk.flush();
    }
}

impl<T> Vec<T> {
    pub fn initialize(&mut self) {
        if self.len.get().is_none() {
            self.len.set(0);
        }
    }

    pub fn len(&self) -> u32 {
        *self.len.get().expect(
            "[liquid_core::Vec::len] Error: expected `len` field to be existed in \
             storage",
        )
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter::<T>::new(self)
    }
}

impl<T> Vec<T>
where
    T: Codec,
{
    fn within_bounds(&self, n: u32) -> Option<u32> {
        if n < self.len() {
            return Some(n);
        }
        None
    }

    /// Returns a reference to the `n`-th element of the vector.
    ///
    /// Returns `None` if `n` is out of bounds.
    pub fn get(&self, n: u32) -> Option<&T> {
        self.within_bounds(n)
            .and_then(|n| self.chunk.get(&n.to_le_bytes()))
    }

    /// Returns a mutable reference to the `n`-th element of the vector.
    ///
    /// Returns `None` if `n` is out of bounds.
    pub fn get_mut(&mut self, n: u32) -> Option<&mut T> {
        self.within_bounds(n)
            .and_then(move |n| self.chunk.get_mut(&n.to_le_bytes()))
    }

    /// Mutates the `n`-th element of the vector.
    ///
    /// Returns a reference to the mutated element.
    /// Returns `None` and won't mutate if `n` out of bounds.
    pub fn mutate_with<F>(&mut self, n: u32, f: F) -> Option<&T>
    where
        F: FnOnce(&mut T),
    {
        self.within_bounds(n)
            .and_then(move |n| self.chunk.mutate_with(&n.to_le_bytes(), f))
    }

    /// Appends an element to the back of the vector.
    pub fn push(&mut self, val: T) {
        if self.len() == u32::MAX {
            panic!(
                "[liquid_core::Vec::push] Error: cannot push more elements than \
                 `u32::MAX`"
            );
        }

        let len = self.len.get_mut().expect(
            "[liquid_core::Vec::push] Error: expected `len` field to be existed in \
             storage",
        );
        self.chunk.set(&len.to_le_bytes(), val);
        *len += 1;
    }

    /// Replaces the `n`-th element of the vector and returns its replaced value.
    ///
    /// Returns `None` if `n` is out of bounds.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let len = self.len.get_mut().expect(
            "[liquid_core::Vec::push] Error: expected `len` field to be existed in \
             storage",
        );
        *len -= 1;
        let ret = self.chunk.take(&len.to_le_bytes());
        self.chunk.remove(&len.to_le_bytes());
        ret
    }

    /// Swaps the `a`-th and the `b`-th elements.
    ///
    /// # Panics
    ///
    /// If one or both indices are out of bounds.
    pub fn swap(&mut self, a: u32, b: u32) {
        if a == b {
            return;
        }

        let a_index = a.to_le_bytes();
        self.within_bounds(a)
            .expect("[liquid_core::Vec::swap] Error: expected `a` to be within bounds");
        self.within_bounds(b)
            .expect("[liquid_core::Vec::swap] Error: expected `b` to be within bounds");
        let item_a = self.chunk.take(&a_index).expect(
            "[liquid_core::Vec::swap] Error: expected `Some` value since vector is not \
             empty",
        );
        let item_b = self.chunk.put(&b.to_le_bytes(), item_a).expect(
            "[liquid_core::Vec::swap] Error: expected `Some` value since vector is not \
             empty",
        );
        self.chunk.set(&a_index, item_b);
    }

    /// Removes the `n`-th element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    /// Returns `None` and does not remove if `n` is out of bounds.
    ///
    /// # Note
    ///
    /// This does not preserve ordering, but is O(1).
    pub fn swap_remove(&mut self, n: u32) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.within_bounds(n)?;
        if n == self.len() - 1 {
            self.pop()
        } else {
            let last_elem = self.pop().expect(
                "[liquid_core::Vec::swap_remove] Error: expected `Some` value since \
                 vector is not empty",
            );

            self.chunk.put(&n.to_le_bytes(), last_elem)
        }
    }
}

impl<T> Extend<T> for Vec<T>
where
    T: Codec,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for i in iter {
            self.push(i);
        }
    }
}

impl<'a, T> Extend<&'a T> for Vec<T>
where
    T: Codec + Copy + 'a,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        self.extend(iter.into_iter().copied())
    }
}

impl<T> core::ops::Index<u32> for Vec<T>
where
    T: Codec,
{
    type Output = T;

    fn index(&self, index: u32) -> &Self::Output {
        self.get(index).expect(
            "[liquid_core::Vec::index] Error: expected `index` to be within bounds",
        )
    }
}

impl<T> core::ops::IndexMut<u32> for Vec<T>
where
    T: Codec,
{
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.get_mut(index).expect(
            "[liquid_core::Vec::index_mut] Error: expected `index` to be within bounds",
        )
    }
}

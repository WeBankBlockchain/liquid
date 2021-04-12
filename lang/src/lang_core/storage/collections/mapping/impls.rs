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

use crate::lang_core::storage::{
    Bind, CachedCell, CachedChunk, Flush,
    You_Should_Use_A_Container_To_Wrap_Your_State_Field_In_Storage,
};
use cfg_if::cfg_if;
use core::{borrow::Borrow, marker::PhantomData};
use scale::{Codec, Encode};

#[cfg_attr(feature = "std", derive(Debug))]
pub struct Mapping<K, V> {
    len: CachedCell<u32>,
    chunk: CachedChunk<V>,
    marker: PhantomData<fn() -> K>,
}

impl<K, V> Bind for Mapping<K, V> {
    fn bind_with(key: &[u8]) -> Self {
        Self {
            len: CachedCell::<u32>::new(key),
            chunk: CachedChunk::<V>::new(key),
            marker: Default::default(),
        }
    }
}

impl<K, V> Flush for Mapping<K, V>
where
    K: Encode,
    V: Encode,
{
    fn flush(&mut self) {
        self.len.flush();
        self.chunk.flush();
    }
}

cfg_if! {
    if #[cfg(feature = "contract")] {
        use crate::lang_core::storage::Getter;

        macro_rules! getter_impl {
            () => {
                type Index = K;
                type Output = V;

                fn getter_impl(&self, index: Self::Index) -> Self::Output {
                    self.get(&index)
                        .expect(
                            "[liquid_lang::Mapping::getter] Error: expected `index` to be existed",
                        )
                        .clone()
                }
            };
        }

        impl<K, V> Getter for Mapping<K, V>
        where
            K: Codec,
            V: Codec + Clone,
        {
            getter_impl!();
        }
    }
}

impl<K, V> Mapping<K, V> {
    pub fn initialize(&mut self) {
        if self.len.get().is_none() {
            self.len.set(0);
        }
    }

    pub fn len(&self) -> u32 {
        *self.len.get().expect(
            "[liquid_lang::Mapping::len] Error: expected `len` field to be existed in \
             storage",
        )
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<K, V> Mapping<K, V>
where
    K: Codec,
    V: Codec,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated,
    /// and the old value is returned.
    pub fn insert<Q>(&mut self, key: &Q, val: V) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Encode,
    {
        if self.len() == u32::MAX {
            panic!(
                "[liquid_lang::Mapping::insert] Error: cannot insert more elements than \
                 `u32::MAX`"
            );
        }

        let encoded_key = key.encode();
        let ret = self.chunk.take(&encoded_key);
        self.chunk.set(&encoded_key, val);

        if ret.is_none() {
            let len = self.len.get_mut().expect(
                "[liquid_lang::Mapping::insert] Error: expected `len` field to be \
                 existed in storage",
            );
            *len += 1;
        }
        ret
    }

    /// Mutates the value associated with the key if any.
    ///
    /// Returns a reference to the mutated element or
    /// Returns `None` and won't mutate if there is no value
    /// associated to the key.
    pub fn mutate_with<Q, F>(&mut self, key: &Q, f: F) -> Option<&V>
    where
        K: Borrow<Q>,
        F: FnOnce(&mut V),
        Q: Encode,
    {
        let encoded_key = key.encode();
        self.chunk.mutate_with(&encoded_key, f)
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Encode,
    {
        let encoded_key = key.encode();
        let ret = self.chunk.take(&encoded_key);
        self.chunk.remove(&encoded_key);

        if ret.is_some() {
            let len = self.len.get_mut().expect(
                "[liquid_lang::Mapping::remove] Error: expected `len` field to be \
                 existed in storage",
            );
            *len -= 1;
        }
        ret
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Encode,
    {
        let encoded_key = key.encode();
        self.chunk.get(&encoded_key)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Encode,
    {
        let encoded_key = key.encode();
        self.chunk.get_mut(&encoded_key)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Encode,
    {
        let encoded_key = key.encode();
        matches!(self.chunk.get(&encoded_key), Some(_))
    }
}

impl<'a, K, Q, V> core::ops::Index<&'a Q> for Mapping<K, V>
where
    K: Codec + Borrow<Q>,
    V: Codec,
    Q: Encode,
{
    type Output = V;

    fn index(&self, index: &'a Q) -> &Self::Output {
        self.get(index)
            .expect("[liquid_lang::Mapping::index] Error: expected `index` to be existed")
    }
}

impl<'a, K, Q, V> core::ops::IndexMut<&'a Q> for Mapping<K, V>
where
    K: Codec + Borrow<Q>,
    V: Codec,
    Q: Encode,
{
    fn index_mut(&mut self, index: &'a Q) -> &mut Self::Output {
        self.get_mut(index).expect(
            "[liquid_lang::Mapping::index_mut] Error: expected `index` to be existed",
        )
    }
}

impl<K, V> Extend<(K, V)> for Mapping<K, V>
where
    K: Codec,
    V: Codec,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        for (k, v) in iter {
            self.insert(&k, v);
        }
    }
}

impl<'a, K, V> Extend<(&'a K, &'a V)> for Mapping<K, V>
where
    K: Codec + Copy,
    V: Codec + Copy,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (&'a K, &'a V)>,
    {
        self.extend(iter.into_iter().map(|(k, v)| (*k, *v)))
    }
}

impl<K, V> You_Should_Use_A_Container_To_Wrap_Your_State_Field_In_Storage
    for Mapping<K, V>
{
    type Wrapped1 = K;
    type Wrapped2 = V;
}

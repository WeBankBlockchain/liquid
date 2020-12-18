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

use crate::core::storage::{
    Bind, Flush, Mapping, Vec,
    You_Should_Use_A_Container_To_Wrap_Your_State_Field_In_Storage,
};
use cfg_if::cfg_if;
use core::borrow::Borrow;
use scale::{Codec, Decode, Encode};

#[derive(Decode, Encode)]
#[cfg_attr(feature = "std", derive(Debug))]
struct KeyEntry<K: Codec> {
    key: K,
    deleted: bool,
}

#[derive(Decode, Encode)]
#[cfg_attr(feature = "std", derive(Debug))]
struct ValueEntry<V: Codec> {
    key_index: u32,
    val: V,
}

#[cfg_attr(feature = "std", derive(Debug))]
pub struct IterableMapping<K: Codec, V: Codec> {
    keys: Vec<KeyEntry<K>>,
    mapping: Mapping<K, ValueEntry<V>>,
}

#[cfg_attr(feature = "std", derive(Debug))]
pub struct Iter<'a, K: Codec, V: Codec> {
    iterable_mapping: &'a IterableMapping<K, V>,
    begin: u32,
    end: u32,
}

impl<'a, K, V> Iter<'a, K, V>
where
    K: Codec,
    V: Codec,
{
    pub(crate) fn new(iterable_mapping: &'a IterableMapping<K, V>) -> Self {
        Self {
            iterable_mapping,
            begin: 0,
            end: iterable_mapping.keys.len(),
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
where
    K: Codec,
    V: Codec,
{
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);

        if self.begin == self.end {
            return None;
        }

        loop {
            let key_entry = &self.iterable_mapping.keys[self.begin];
            if !key_entry.deleted {
                let key = &key_entry.key;
                let val = &self.iterable_mapping.mapping[key].val;
                self.begin += 1;
                return Some((key, val));
            }
            self.begin += 1;

            if self.begin == self.end {
                return None;
            }
        }
    }
}

impl<K, V> Bind for IterableMapping<K, V>
where
    K: Codec,
    V: Codec,
{
    fn bind_with(key: &[u8]) -> Self {
        let mut keys_bind_key = key.to_vec();
        keys_bind_key.extend_from_slice(b"$keys");
        let mut mapping_bind_key = key.to_vec();
        mapping_bind_key.extend_from_slice(b"$mapping");

        Self {
            keys: Vec::<KeyEntry<K>>::bind_with(&keys_bind_key),
            mapping: Mapping::<K, ValueEntry<V>>::bind_with(&mapping_bind_key),
        }
    }
}

impl<K, V> Flush for IterableMapping<K, V>
where
    K: Codec,
    V: Codec,
{
    fn flush(&mut self) {
        self.keys.flush();
        self.mapping.flush();
    }
}

cfg_if! {
    if #[cfg(feature = "contract")] {
        use crate::core::storage::Getter;

        macro_rules! getter_impl {
            () => {
                type Index = K;
                type Output = V;
                fn getter_impl(&self, index: Self::Index) -> Self::Output {
                    self.get(&index)
                        .expect(
                            "liquid_lang::IterableMapping::getter] Error: expected `index` to be \
                             existed",
                        )
                        .clone()
                }
            };
        }

        #[cfg(feature = "solidity-compatible")]
        impl<K, V> Getter for IterableMapping<K, V>
        where
            K: Codec + liquid_abi_codec::Decode,
            V: Codec + liquid_abi_codec::Encode + Clone,
        {
            getter_impl!();
        }

        #[cfg(not(feature = "solidity-compatible"))]
        impl<K, V> Getter for IterableMapping<K, V>
        where
            K: Codec,
            V: Codec + Clone,
        {
            getter_impl!();
        }
    }
}

impl<K, V> IterableMapping<K, V>
where
    K: Codec,
    V: Codec,
{
    pub fn initialize(&mut self) {
        self.keys.initialize();
        self.mapping.initialize();
    }

    pub fn len(&self) -> u32 {
        self.mapping.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        let entry = self.mapping.get(&key);
        if let Some(value_entry) = entry {
            let old_key_index: u32 = value_entry.key_index;
            self.mapping
                .insert(
                    &key,
                    ValueEntry {
                        key_index: old_key_index,
                        val,
                    },
                )
                .map(|value_entry| value_entry.val)
        } else {
            self.mapping.insert(
                &key,
                ValueEntry {
                    key_index: self.keys.len(),
                    val,
                },
            );

            self.keys.push(KeyEntry {
                key,
                deleted: false,
            });

            None
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Encode,
    {
        let ret = self.mapping.remove(key);

        if let Some(ret) = ret {
            let key_index = ret.key_index;
            self.keys[key_index].deleted = true;
            return Some(ret.val);
        }

        None
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Encode,
    {
        self.mapping.contains_key(&key)
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::<'_, K, V>::new(self)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Encode,
    {
        self.mapping.get(key).map(|value_entry| &value_entry.val)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Encode,
    {
        self.mapping
            .get_mut(key)
            .map(|value_entry| &mut value_entry.val)
    }

    pub fn mutate_with<Q, F>(&mut self, key: &Q, f: F) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Encode,
        F: FnOnce(&mut V),
    {
        self.mapping
            .mutate_with(key, |value_entry| f(&mut value_entry.val))
            .map(|value_entry| &value_entry.val)
    }
}

impl<'a, K, V, Q> core::ops::Index<&'a Q> for IterableMapping<K, V>
where
    K: Borrow<Q> + Codec,
    V: Codec,
    Q: Encode,
{
    type Output = V;

    fn index(&self, index: &'a Q) -> &Self::Output {
        self.get(index).expect(
            "[liquid_lang::IterableMapping::index] Error: expected `index` to be existed",
        )
    }
}

impl<'a, K, V, Q> core::ops::IndexMut<&'a Q> for IterableMapping<K, V>
where
    K: Borrow<Q> + Codec,
    V: Codec,
    Q: Encode,
{
    fn index_mut(&mut self, index: &'a Q) -> &mut Self::Output {
        self.get_mut(index).expect(
            "[liquid_lang::IterableMapping::index_mut] Error: expected `index` to be \
             existed",
        )
    }
}

impl<K, V> Extend<(K, V)> for IterableMapping<K, V>
where
    K: Codec,
    V: Codec,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<'a, K, V> Extend<(&'a K, &'a V)> for IterableMapping<K, V>
where
    K: Codec + Copy,
    V: Codec + Copy,
{
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        self.extend(iter.into_iter().map(|(k, v)| (*k, *v)))
    }
}

impl<K: Codec, V: Codec> You_Should_Use_A_Container_To_Wrap_Your_State_Field_In_Storage
    for IterableMapping<K, V>
{
}

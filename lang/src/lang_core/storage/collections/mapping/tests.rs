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

use super::Mapping;
use crate::lang_core::storage::traits::Bind;

fn new_empty<K, V>() -> Mapping<K, V> {
    let mut map = Mapping::<K, V>::bind_with(b"var");
    map.initialize();
    map
}

#[test]
fn empty() {
    let map = new_empty::<String, u8>();
    assert_eq!(map.len(), 0);
    assert_eq!(map.is_empty(), true);
}

#[test]
fn insert_works() {
    let mut map = new_empty::<String, u8>();
    let name = "Alice".to_string();
    assert_eq!(map.insert(&name, 0), None);
    assert_eq!(map.len(), 1);
    assert_eq!(map.is_empty(), false);

    assert_eq!(map.insert(&name, 0), Some(0));
    assert_eq!(map.len(), 1);
    assert_eq!(map.is_empty(), false);
}

#[test]
fn get_works() {
    let mut map = new_empty::<String, u8>();
    assert_eq!(map.insert(&"Alice".to_string(), 0), None);
    assert_eq!(map.insert(&"Bob".to_string(), 1), None);
    assert_eq!(map.get(&"Alice".to_string()), Some(&0));
    assert_eq!(map.get(&"Bob".to_string()), Some(&1));
    assert_eq!(map.get(&"Charlie".to_string()), None);
}

#[test]
fn index_works() {
    let mut map = new_empty::<String, u8>();
    assert_eq!(map.insert(&"Alice".to_string(), 0), None);
    assert_eq!(map.insert(&"Bob".to_string(), 1), None);
    assert_eq!(map[&"Alice".to_string()], 0);
    assert_eq!(map[&"Bob".to_string()], 1);
}

#[test]
fn index_repeat() {
    let mut map = new_empty::<String, u8>();
    let name = "Alice".to_string();
    assert_eq!(map.insert(&name, 0), None);
    assert_eq!(map[&name], 0);
    assert_eq!(map[&name], 0);
}

#[test]
#[should_panic]
fn index_failed() {
    let map = new_empty::<String, u8>();
    let _ = map[&"Alice".to_string()];
}

#[test]
fn contains_works() {
    let mut map = new_empty::<String, u8>();
    let name = "string".to_string();
    assert_eq!(map.contains_key(&name), false);
    assert_eq!(map.insert(&name, 0), None);
    assert_eq!(map.contains_key(&name), true);
}

#[test]
fn remove_works() {
    let mut map = new_empty::<String, u8>();
    let name = "Alice".to_string();
    assert_eq!(map.insert(&name, 0), None);
    assert_eq!(map.len(), 1);
    assert_eq!(map.remove(&name), Some(0));
    assert_eq!(map.len(), 0);
    assert_eq!(map.contains_key(&name), false);
    assert_eq!(map.remove(&name), None);
    assert_eq!(map.len(), 0);
    assert_eq!(map.contains_key(&name), false);
}

#[test]
fn mutate_with_works() {
    let mut map = new_empty::<String, String>();
    // Inserts some elements
    assert_eq!(map.insert(&"Dog Breed".to_string(), "Akita".into()), None);
    assert_eq!(map.insert(&"Cat Breed".to_string(), "Bengal".into()), None);
    assert_eq!(map[&"Dog Breed".to_string()], "Akita");
    assert_eq!(map[&"Cat Breed".to_string()], "Bengal");
    // Change the breeds
    assert_eq!(
        map.mutate_with(&"Dog Breed".to_string(), |breed| *breed =
            "Shiba Inu".into()),
        Some(&"Shiba Inu".into())
    );
    assert_eq!(
        map.mutate_with(&"Cat Breed".to_string(), |breed| breed
            .push_str(" Shorthair")),
        Some(&"Bengal Shorthair".into())
    );
    // Verify the mutated breeds
    assert_eq!(map[&"Dog Breed".to_string()], "Shiba Inu");
    assert_eq!(map[&"Cat Breed".to_string()], "Bengal Shorthair");
    // Mutate for non-existing key
    assert_eq!(
        map.mutate_with(&"Bird Breed".to_string(), |breed| *breed = "Parrot".into()),
        None
    );
}

#[test]
fn extend_works() {
    // given
    let keys = (0..5).collect::<Vec<u32>>();
    let vals = keys.iter().map(|i| i * i).collect::<Vec<u32>>();
    let mut map = new_empty::<u32, u32>();

    // when
    map.extend(keys.iter().zip(vals.iter()));

    // then
    assert_eq!(map.len() as usize, 5);
    for i in 0..5 {
        assert_eq!(map[&keys[i]], vals[i]);
    }
}

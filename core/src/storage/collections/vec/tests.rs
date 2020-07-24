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

use super::Vec;
use crate::storage::traits::Bind;

fn new_empty_vec() -> Vec<u8> {
    let mut vec = Vec::<u8>::bind_with(b"vec");
    vec.initialize();
    vec
}

fn new_filled_vec() -> Vec<u8> {
    let mut vec = Vec::<u8>::bind_with(b"vec");
    vec.initialize();
    vec.push(0x56);
    vec.push(0x49);
    vec.push(0x54);
    vec.push(0x41);
    assert_eq!(vec.len(), 4);
    vec
}

#[test]
fn empty() {
    let vec = new_empty_vec();
    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
    assert_eq!(vec.iter().next(), None);
}

#[test]
fn simple() {
    let mut vec = new_empty_vec();
    assert_eq!(vec.len(), 0);
    vec.push(0);
    assert_eq!(vec.is_empty(), false);
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.get(0), Some(&0));
    {
        let mut iter = vec.iter();
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), None);
    }
    assert_eq!(vec.pop(), Some(0));
    assert_eq!(vec.pop(), None);
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.iter().next(), None);
}

#[test]
fn pop_empty() {
    let mut vec = new_empty_vec();
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.pop(), None);
    assert_eq!(vec.len(), 0);
}

#[test]
fn iter() {
    let vec = new_filled_vec();
    let mut iter = vec.iter();
    assert_eq!(iter.next(), Some(&0x56));
    assert_eq!(iter.next(), Some(&0x49));
    assert_eq!(iter.next(), Some(&0x54));
    assert_eq!(iter.next(), Some(&0x41));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter_back() {
    let vec = new_filled_vec();
    let mut iter = vec.iter();
    assert_eq!(iter.next_back(), Some(&0x41));
    assert_eq!(iter.next_back(), Some(&0x54));
    assert_eq!(iter.next_back(), Some(&0x49));
    assert_eq!(iter.next_back(), Some(&0x56));
    assert_eq!(iter.next_back(), None);
}

#[test]
fn get() {
    let vec = new_filled_vec();
    assert_eq!(vec.get(0), Some(&0x56));
    assert_eq!(vec.get(1), Some(&0x49));
    assert_eq!(vec.get(2), Some(&0x54));
    assert_eq!(vec.get(3), Some(&0x41));
    assert_eq!(vec.get(4), None);
    assert_eq!(vec.get(u32::max_value()), None);
}

#[test]
fn index() {
    let vec = new_filled_vec();
    assert_eq!(vec[0], 0x56);
    assert_eq!(vec[1], 0x49);
    assert_eq!(vec[2], 0x54);
    assert_eq!(vec[3], 0x41);
}

#[test]
fn index_mut() {
    let mut vec = new_filled_vec();
    for i in 0..4 {
        vec[i] = i as u8;
    }
    for i in 0..4 {
        assert_eq!(vec[i], i as u8);
    }
}

#[test]
#[should_panic]
fn out_of_bounds_0() {
    let vec = new_empty_vec();
    let _ = vec[1];
}

#[test]
#[should_panic]
fn out_of_bounds_1() {
    let mut vec = new_filled_vec();
    vec[5] = 5;
}

#[test]
fn mutate_with() {
    let mut vec = new_filled_vec();
    assert_eq!(vec.mutate_with(0, |elem| *elem = 42), Some(&42));
    assert_eq!(vec[0], 42);
}

#[test]
fn swap() {
    let mut vec = new_filled_vec();
    vec.swap(0, 3);
    vec.swap(1, 2);
    assert_eq!(vec[0], 0x41);
    assert_eq!(vec[1], 0x54);
    assert_eq!(vec[2], 0x49);
    assert_eq!(vec[3], 0x56);
}

#[test]
fn swap_same() {
    let mut vec = new_filled_vec();
    vec.swap(0, 0);
    assert_eq!(vec[0], 0x56);
}

#[test]
#[should_panic]
fn swap_out_of_bounds_0() {
    let mut vec = new_filled_vec();
    vec.swap(0, u32::MAX);
}

#[test]
#[should_panic]
fn swap_out_of_bounds_1() {
    let mut vec = new_filled_vec();
    vec.swap(u32::MAX, 0);
}

#[test]
fn swap_remove() {
    let mut vec = new_filled_vec();
    assert_eq!(vec.swap_remove(0), Some(0x56));
    assert_eq!(vec[0], 0x41);
    assert_eq!(vec[1], 0x49);
    assert_eq!(vec[2], 0x54);
}

#[test]
fn swap_remove_empty() {
    let mut vec = new_empty_vec();
    assert_eq!(vec.swap_remove(0), None);
}

#[test]
fn swap_remove_out_of_bounds() {
    let mut vec = new_filled_vec();
    assert_eq!(vec.swap_remove(u32::MAX), None);
}

#[test]
fn swap_remove_last() {
    let mut vec = new_empty_vec();
    vec.push(5);
    assert_eq!(vec.len(), 1);
    assert_eq!(vec.swap_remove(0), Some(5));
    assert_eq!(vec.len(), 0);
}

#[test]
fn extend() {
    let mut vec = new_empty_vec();
    let arr = [0u8, 1, 2, 3];
    vec.extend(&arr);
    assert_eq!(vec.len(), 4);
    for i in 0..4 {
        assert_eq!(vec[i], i as u8);
    }
}

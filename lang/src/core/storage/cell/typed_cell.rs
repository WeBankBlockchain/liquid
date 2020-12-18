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

use crate::core::env;
use core::marker::PhantomData;
use liquid_prelude::vec::Vec;

/// A typed cell.
///
/// Provides interpreted access to the associated contract storage slot.
#[cfg_attr(feature = "std", derive(Debug))]
pub struct TypedCell<T> {
    key: Vec<u8>,
    marker: PhantomData<fn() -> T>,
}

impl<T> TypedCell<T> {
    pub fn new(key: &[u8]) -> Self {
        Self {
            key: key.to_vec(),
            marker: Default::default(),
        }
    }
}

impl<T> TypedCell<T>
where
    T: scale::Decode,
{
    pub fn load(&self) -> Option<T> {
        env::api::get_storage::<T>(&self.key).ok()
    }
}

impl<T> TypedCell<T>
where
    T: scale::Encode,
{
    pub fn store(&mut self, new_value: &T) {
        env::api::set_storage(&self.key, new_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use liquid_prelude::string::String;

    fn dummy_cell<T>() -> TypedCell<T> {
        TypedCell::new(b"var")
    }

    #[test]
    fn simple_integer() {
        let mut cell = dummy_cell::<i32>();
        assert_eq!(cell.load(), None);
        cell.store(&5);
        assert_eq!(cell.load(), Some(5));
    }

    #[test]
    fn simple_string() {
        let mut cell = dummy_cell::<String>();
        assert_eq!(cell.load(), None);
        let s = "cat".to_owned();
        cell.store(&s);
        assert_eq!(cell.load(), Some(s));
    }
}

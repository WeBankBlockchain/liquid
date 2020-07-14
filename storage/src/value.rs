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

use crate::{Bind, CachedCell, Flush};
use liquid_primitives::Key;
use scale::Encode;

#[derive(Debug)]
pub struct Value<T> {
    cell: CachedCell<T>,
}

impl<T> Bind for Value<T> {
    fn bind_with(key: Key) -> Self {
        Self {
            cell: CachedCell::new(key),
        }
    }
}

impl<T> Flush for Value<T>
where
    T: Encode,
{
    fn flush(&mut self) {
        self.cell.flush();
    }
}

impl<T> Value<T>
where
    T: scale::Decode,
{
    pub fn get(&self) -> &T {
        self.cell.get().unwrap()
    }
}

impl<T> Value<T>
where
    T: scale::Encode,
{
    pub fn set(&mut self, new_val: T) {
        self.cell.set(new_val);
    }
}

impl<T> Value<T>
where
    T: scale::Codec,
{
    pub fn get_mut(&mut self) -> &mut T {
        self.cell.get_mut().unwrap()
    }

    pub fn mutate_with<F>(&mut self, f: F) -> &T
    where
        F: FnOnce(&mut T),
    {
        self.cell.mutate_with(f).unwrap()
    }
}

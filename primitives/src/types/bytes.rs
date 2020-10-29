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

use liquid_prelude::vec::Vec;

#[derive(PartialEq, Eq, scale::Decode, scale::Encode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Bytes(Vec<u8>);

impl Default for Bytes {
    fn default() -> Self {
        Bytes(Vec::new())
    }
}

impl Bytes {
    pub fn new() -> Self {
        Default::default()
    }
}

impl core::ops::Deref for Bytes {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for Bytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&[u8]> for Bytes {
    fn from(origin: &[u8]) -> Self {
        Self(origin.to_vec())
    }
}

impl<const N: usize> From<[u8; N]> for Bytes {
    fn from(origin: [u8; N]) -> Self {
        Self(origin.to_vec())
    }
}

impl<const N: usize> From<&[u8; N]> for Bytes {
    fn from(origin: &[u8; N]) -> Self {
        Self(origin.to_vec())
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(origin: Vec<u8>) -> Self {
        Self(origin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_test() {
        let mut b1 = Bytes::new();
        b1.push(1);
        assert_eq!(b1.len(), 1);
        assert_eq!(b1[0], 1);

        let mut b2: Bytes = [0, 1, 2].into();
        assert_eq!(b2.len(), 3);
        b2.pop();
        assert_eq!(b2.len(), 2);
        assert_eq!(b2[0], 0);
        assert_eq!(b2[1], 1);
    }
}

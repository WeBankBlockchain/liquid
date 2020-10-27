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

pub struct StaticBuffer {
    buffer: [u8; Self::CAPACITY],
    len: usize,
}

impl StaticBuffer {
    /// The capacity of the static buffer
    pub const CAPACITY: usize = 1 << 14; // 16KB

    pub const fn new() -> Self {
        Self {
            buffer: [0; Self::CAPACITY],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        return self.len;
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        if self.len + bytes.len() > Self::CAPACITY {
            panic!("static buffer overflowed");
        }

        let start = self.len;
        let bytes_len = bytes.len();
        self.buffer[start..(start + bytes_len)].copy_from_slice(bytes);
        self.len += bytes_len;
    }

    pub fn resize(&mut self, new_len: usize) {
        if new_len > Self::CAPACITY {
            panic!("static buffer overflowed");
        }
        self.len = new_len;
    }
}

impl scale::Output for StaticBuffer {
    fn write(&mut self, bytes: &[u8]) {
        self.write_bytes(bytes);
    }

    fn push_byte(&mut self, byte: u8) {
        self.write_bytes(&[byte]);
    }
}

impl liquid_abi_codec::Output for StaticBuffer {
    fn write(&mut self, bytes: &[u8]) {
        self.write_bytes(bytes);
    }
}

impl<I: core::slice::SliceIndex<[u8]>> core::ops::Index<I> for StaticBuffer {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        core::ops::Index::index(&self.buffer[..self.len], index)
    }
}

impl<I: core::slice::SliceIndex<[u8]>> core::ops::IndexMut<I> for StaticBuffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        core::ops::IndexMut::index_mut(&mut self.buffer[..self.len], index)
    }
}

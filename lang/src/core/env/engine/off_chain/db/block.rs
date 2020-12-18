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

pub struct Block {
    /// The current block number.
    block_number: u64,
    /// The timestamp of the block.
    timestamp: u64,
}

impl Block {
    pub fn new(block_number: u64) -> Self {
        use chrono::prelude::Utc;

        let date_time = Utc::now();
        let timestamp = date_time.timestamp() as u64;
        Self {
            block_number,
            timestamp,
        }
    }

    /// Returns the block number.
    pub fn block_number(&self) -> u64 {
        self.block_number
    }

    /// Returns the timestamp of the block.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

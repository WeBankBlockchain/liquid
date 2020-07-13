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

pub enum DispatchError {
    UnknownSelector,
    InvalidParams,
    CouldNotReadInput,
}

pub struct DispatchRetCode(u32);

impl From<DispatchError> for DispatchRetCode {
    fn from(err: DispatchError) -> Self {
        match err {
            DispatchError::UnknownSelector => Self(0x01),
            DispatchError::InvalidParams => Self(0x02),
            DispatchError::CouldNotReadInput => Self(0x03),
        }
    }
}

impl DispatchRetCode {
    pub fn to_u32(&self) -> u32 {
        self.0
    }
}

pub type DispatchResult = core::result::Result<(), DispatchError>;

impl From<DispatchResult> for DispatchRetCode {
    fn from(result: DispatchResult) -> Self {
        match result {
            Ok(_) => Self(0x00),
            Err(error) => Self::from(error),
        }
    }
}

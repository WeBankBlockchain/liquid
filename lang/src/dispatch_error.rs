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

use crate::bytes_to_hex;
use liquid_prelude::{string::String, vec::Vec};

#[cfg_attr(feature = "std", derive(Debug))]
pub enum DispatchError {
    UnknownSelector(Vec<u8>),
    InvalidParams(String, Vec<u8>),
    CouldNotReadInput,
}

pub struct DispatchRetInfo(bool, String);

impl From<DispatchError> for DispatchRetInfo {
    fn from(err: DispatchError) -> Self {
        match err {
            DispatchError::UnknownSelector(selector) => Self(false, {
                let mut error_info = String::from("unknown selector: ");
                error_info.push_str(&bytes_to_hex(selector.as_slice()));
                error_info
            }),
            DispatchError::InvalidParams(name, data) => Self(false, {
                let mut error_info = String::from("invalid params for `");
                error_info.push_str(&name);
                error_info.push_str("`: ");
                error_info.push_str(&bytes_to_hex(&data));
                error_info
            }),
            DispatchError::CouldNotReadInput => {
                Self(false, String::from("could not read input"))
            }
        }
    }
}

impl DispatchRetInfo {
    pub fn get_info_string(&self) -> String {
        self.1.clone()
    }

    pub fn is_success(&self) -> bool {
        self.0
    }
}

pub type DispatchResult = core::result::Result<(), DispatchError>;

impl From<DispatchResult> for DispatchRetInfo {
    fn from(result: DispatchResult) -> Self {
        match result {
            Ok(_) => Self(true, String::new()),
            Err(error) => Self::from(error),
        }
    }
}

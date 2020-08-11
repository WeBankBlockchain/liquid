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

use super::{EnvInstance, ExecContext};
use crate::env::{
    engine::OnInstance,
    types::{Address, ADDRESS_LENGTH},
};

/// Pushes a contract execution context.
///
/// This is the data behind a single instance of a contract call.
///
/// # Note
///
/// Together with [`pop_execution_context`] this can be used to emulated
/// nested calls.
pub fn push_execution_context(caller: Address) {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.exec_contexts.push(ExecContext::new(caller))
    });
}

/// Pops the top contract execution context.
///
/// # Note
///
/// Together with [`push_execution_context`] this can be used to emulated
/// nested calls.
pub fn pop_execution_context() {
    <EnvInstance as OnInstance>::on_instance(|instance| {
        instance.exec_contexts.pop();
    })
}

/// The default accounts.
pub struct DefaultAccounts {
    pub alice: Address,
    pub bob: Address,
    pub charlie: Address,
    pub david: Address,
    pub eva: Address,
    pub frank: Address,
}

/// Returns the default accounts for testing purposes:
/// Alice, Bob, Charlie, Django, Eve and Frank
pub fn default_accounts() -> DefaultAccounts {
    DefaultAccounts {
        alice: Address::from_bytes(&[0x00; ADDRESS_LENGTH]),
        bob: Address::from_bytes(&[0x01; ADDRESS_LENGTH]),
        charlie: Address::from_bytes(&[0x02; ADDRESS_LENGTH]),
        david: Address::from_bytes(&[0x03; ADDRESS_LENGTH]),
        eva: Address::from_bytes(&[0x04; ADDRESS_LENGTH]),
        frank: Address::from_bytes(&[0x05; ADDRESS_LENGTH]),
    }
}

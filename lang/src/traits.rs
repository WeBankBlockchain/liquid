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

use liquid_abi_coder::{Decode, Encode};
use liquid_primitives::Selector;

pub trait FnInput {
    type Input: Decode + 'static;
}

pub trait FnOutput {
    type Output: Encode + 'static;
}

pub trait FnSelectors {
    const KECCAK256_SELECTOR: Selector;
    const SM3_SELECTOR: Selector;
}

pub trait FnMutability {
    const IS_MUT: bool;
}

pub trait ExternalFn: FnInput + FnOutput + FnSelectors + FnMutability {}

#[allow(non_camel_case_types)]
pub trait The_Type_You_Used_Here_Must_Be_An_Valid_Liquid_Data_Type: Sized {
    type T = Self;
}

#[cfg(feature = "liquid-abi-gen")]
pub trait GenerateABI {
    fn generate_abi() -> liquid_abi_gen::ContractABI;
}

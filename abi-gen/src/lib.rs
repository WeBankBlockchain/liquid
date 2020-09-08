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

mod traits;
pub use traits::*;

use serde::Serialize;

pub struct ContractABI {
    pub constructor_abi: ConstructorABI,
    pub external_fn_abis: Vec<ExternalFnABI>,
    pub event_abis: Vec<EventABI>,
}

#[derive(Serialize)]
pub struct ParamABI {
    #[serde(skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub components: Vec<ParamABI>,
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

impl ParamABI {
    pub fn new(components: Vec<ParamABI>, name: String, ty: String) -> Self {
        Self {
            components,
            name,
            ty,
        }
    }

    pub fn empty() -> Self {
        Self {
            components: Default::default(),
            name: Default::default(),
            ty: Default::default(),
        }
    }
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct ConstructorABI {
    inputs: Vec<ParamABI>,
    payable: bool,
    stateMutability: String,
    #[serde(rename = "type")]
    ty: String,
}

impl ConstructorABI {
    pub fn new_builder() -> ConstructorABIBuilder {
        ConstructorABIBuilder {
            abi: Self {
                inputs: Vec::new(),
                payable: false,
                stateMutability: "nonpayable".to_owned(),
                ty: "constructor".to_owned(),
            },
        }
    }
}

pub struct ConstructorABIBuilder {
    abi: ConstructorABI,
}

impl ConstructorABIBuilder {
    pub fn input(mut self, components: Vec<ParamABI>, name: String, ty: String) -> Self {
        self.abi.inputs.push(ParamABI::new(components, name, ty));
        self
    }

    pub fn done(self) -> ConstructorABI {
        self.abi
    }
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct ExternalFnABI {
    constant: bool,
    inputs: Vec<ParamABI>,
    name: String,
    outputs: Vec<ParamABI>,
    payable: bool,
    stateMutability: String,
    #[serde(rename = "type")]
    ty: String,
}

impl ExternalFnABI {
    pub fn new_builder(
        name: String,
        state_mutability: String,
        constant: bool,
    ) -> ExternalFnABIBuilder {
        ExternalFnABIBuilder {
            abi: Self {
                constant,
                inputs: Vec::new(),
                name,
                outputs: Vec::new(),
                payable: false,
                stateMutability: state_mutability,
                ty: "function".to_owned(),
            },
        }
    }
}

pub struct ExternalFnABIBuilder {
    abi: ExternalFnABI,
}

impl ExternalFnABIBuilder {
    pub fn input(&mut self, components: Vec<ParamABI>, name: String, ty: String) {
        // If type of the input is `()`, just skip it.
        if !ty.is_empty() {
            self.abi.inputs.push(ParamABI::new(components, name, ty));
        }
    }

    pub fn output(&mut self, components: Vec<ParamABI>, ty: String) {
        // If type of the input is `()`, just skip it.
        if !ty.is_empty() {
            self.abi
                .outputs
                .push(ParamABI::new(components, "".to_owned(), ty));
        }
    }

    pub fn done(self) -> ExternalFnABI {
        self.abi
    }
}

#[derive(Serialize)]
pub struct EventParamABI {
    pub indexed: bool,
    #[serde(skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub components: Vec<ParamABI>,
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct EventABI {
    anonymous: bool,
    inputs: Vec<EventParamABI>,
    name: String,
    #[serde(rename = "type")]
    ty: String,
}

pub struct EventABIBuilder {
    abi: EventABI,
}

impl EventABI {
    pub fn new_builder(name: String) -> EventABIBuilder {
        EventABIBuilder {
            abi: Self {
                anonymous: false,
                inputs: Vec::new(),
                name,
                ty: "event".to_owned(),
            },
        }
    }
}

impl EventABIBuilder {
    pub fn input(
        &mut self,
        components: Vec<ParamABI>,
        name: String,
        ty: String,
        indexed: bool,
    ) {
        self.abi.inputs.push(EventParamABI {
            indexed,
            components,
            name,
            ty,
        });
    }

    pub fn done(self) -> EventABI {
        self.abi
    }
}

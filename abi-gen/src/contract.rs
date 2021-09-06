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

use crate::{traits::*, ParamAbi};
use serde::Serialize;
use std::collections::HashMap;

pub struct ContractAbi {
    pub constructor_abi: ConstructorAbi,
    pub fn_abis: Vec<FnAbi>,
    pub event_abis: Vec<EventAbi>,
    pub iface_abis: HashMap<String, Vec<FnAbi>>,
}

#[derive(Serialize)]
pub struct ConstructorAbi {
    inputs: Vec<ParamAbi>,
    #[serde(rename = "type")]
    ty: String,
}

impl ConstructorAbi {
    pub fn new_builder() -> ConstructorAbiBuilder {
        ConstructorAbiBuilder {
            abi: Self {
                inputs: Vec::new(),
                ty: "constructor".to_owned(),
            },
        }
    }
}

#[derive(Serialize, Clone)]
pub struct FnAbi {
    constant: bool,
    inputs: Vec<ParamAbi>,
    name: String,
    outputs: Vec<ParamAbi>,
    #[serde(rename = "type")]
    ty: String,
}

impl FnAbi {
    pub fn new_builder(name: String, constant: bool) -> FnAbiBuilder {
        FnAbiBuilder {
            abi: Self {
                constant,
                inputs: Vec::new(),
                name,
                outputs: Vec::new(),
                ty: "function".to_owned(),
            },
        }
    }
}

pub struct ConstructorAbiBuilder {
    abi: ConstructorAbi,
}

impl ConstructorAbiBuilder {
    pub fn input(mut self, param_abi: ParamAbi) -> Self {
        match param_abi {
            ParamAbi::None => (),
            other => {
                self.abi.inputs.push(other);
            }
        }
        self
    }

    pub fn done(self) -> ConstructorAbi {
        self.abi
    }
}

pub struct FnAbiBuilder {
    abi: FnAbi,
}

impl FnOutputBuilder for FnAbiBuilder {
    fn output(&mut self, param_abi: ParamAbi) {
        self.abi.outputs.push(param_abi);
    }
}

impl FnAbiBuilder {
    pub fn input(&mut self, param_abi: ParamAbi) {
        match param_abi {
            ParamAbi::None => (),
            other => {
                self.abi.inputs.push(other);
            }
        }
    }

    pub fn done(self) -> FnAbi {
        self.abi
    }
}

#[derive(Serialize, Clone)]
pub struct EventParamAbi {
    pub indexed: bool,
    #[serde(flatten)]
    pub param_abi: ParamAbi,
}

#[derive(Serialize, Clone)]
pub struct EventAbi {
    anonymous: bool,
    inputs: Vec<EventParamAbi>,
    name: String,
    #[serde(rename = "type")]
    ty: String,
}

pub struct EventAbiBuilder {
    abi: EventAbi,
}

impl EventAbi {
    pub fn new_builder(name: String) -> EventAbiBuilder {
        EventAbiBuilder {
            abi: Self {
                anonymous: false,
                inputs: Vec::new(),
                name,
                ty: "event".to_owned(),
            },
        }
    }
}

impl EventAbiBuilder {
    pub fn input(&mut self, param_abi: ParamAbi, indexed: bool) {
        self.abi.inputs.push(EventParamAbi { indexed, param_abi });
    }

    pub fn done(self) -> EventAbi {
        self.abi
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum AbiKind {
    Constructor(ConstructorAbi),
    ExternalFn(FnAbi),
    Event(EventAbi),
}

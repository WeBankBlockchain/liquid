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

use crate::traits::*;

use derive_more::From;
use serde::Serialize;

pub struct ContractAbi {
    pub constructor_abi: ConstructorAbi,
    pub external_fn_abis: Vec<ExternalFnAbi>,
    pub event_abis: Vec<EventAbi>,
}

#[derive(Serialize)]
pub struct TrivialAbi {
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(skip_serializing_if = "::std::string::String::is_empty")]
    pub name: String,
}

#[derive(Serialize)]
pub struct OptionAbi {
    #[serde(flatten)]
    pub trivial: TrivialAbi,
    pub some: Box<ParamAbi>,
}

#[derive(Serialize)]
pub struct ResultAbi {
    #[serde(flatten)]
    pub trivial: TrivialAbi,
    pub ok: Box<ParamAbi>,
    pub err: Box<ParamAbi>,
}

#[derive(Serialize, From)]
#[serde(untagged)]
pub enum ParamAbi {
    Opt(OptionAbi),
    Res(ResultAbi),
    Composite(CompositeAbi),
    Trivial(TrivialAbi),
    None,
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

#[derive(Serialize)]
pub struct ExternalFnAbi {
    constant: bool,
    inputs: Vec<ParamAbi>,
    name: String,
    outputs: Vec<ParamAbi>,
    #[serde(rename = "type")]
    ty: String,
}

impl ExternalFnAbi {
    pub fn new_builder(name: String, constant: bool) -> ExternalFnAbiBuilder {
        ExternalFnAbiBuilder {
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

impl TrivialAbi {
    pub fn new(ty: String, name: String) -> Self {
        TrivialAbi { ty, name }
    }
}

#[derive(Serialize)]
pub struct CompositeAbi {
    #[serde(flatten)]
    pub trivial: TrivialAbi,
    #[serde(skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub components: Vec<ParamAbi>,
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

pub struct ExternalFnAbiBuilder {
    abi: ExternalFnAbi,
}

impl FnOutputBuilder for ExternalFnAbiBuilder {
    fn output(&mut self, param_abi: ParamAbi) {
        self.abi.outputs.push(param_abi);
    }
}

impl ExternalFnAbiBuilder {
    pub fn input(&mut self, param_abi: ParamAbi) {
        match param_abi {
            ParamAbi::None => (),
            other => {
                self.abi.inputs.push(other);
            }
        }
    }

    pub fn done(self) -> ExternalFnAbi {
        self.abi
    }
}

#[derive(Serialize)]
pub struct EventParamAbi {
    pub indexed: bool,
    #[serde(flatten)]
    pub param_abi: ParamAbi,
}

#[derive(Serialize)]
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

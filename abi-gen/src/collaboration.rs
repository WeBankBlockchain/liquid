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

pub struct CollaborationAbi {
    pub contract_abis: Vec<ContractAbi>,
}

#[derive(Serialize)]
pub struct ContractAbi {
    pub name: String,
    pub data: Vec<ParamAbi>,
    pub rights: Vec<RightAbi>,
}

#[derive(Serialize)]
pub struct TrivialAbi {
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(skip_serializing_if = "::std::string::String::is_empty")]
    pub name: String,
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

#[derive(Serialize)]
#[serde(untagged)]
#[derive(From)]
pub enum ParamAbi {
    Opt(OptionAbi),
    Res(ResultAbi),
    Composite(CompositeAbi),
    Trivial(TrivialAbi),
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct RightAbi {
    pub constant: bool,
    pub inputs: Vec<ParamAbi>,
    pub name: String,
    pub outputs: Vec<ParamAbi>,
}

impl RightAbi {
    pub fn new_builder(name: String, constant: bool) -> RightAbiBuilder {
        RightAbiBuilder {
            abi: Self {
                constant,
                inputs: Vec::new(),
                name,
                outputs: Vec::new(),
            },
        }
    }
}

pub struct RightAbiBuilder {
    abi: RightAbi,
}

impl RightAbiBuilder {
    pub fn input(&mut self, param_abi: ParamAbi) {
        self.abi.inputs.push(param_abi);
    }

    pub fn output(&mut self, param_abi: ParamAbi) {
        self.abi.outputs.push(param_abi);
    }

    pub fn done(self) -> RightAbi {
        self.abi
    }
}

impl FnOutputBuilder for RightAbiBuilder {
    fn output(&mut self, param_abi: ParamAbi) {
        self.abi.outputs.push(param_abi);
    }
}

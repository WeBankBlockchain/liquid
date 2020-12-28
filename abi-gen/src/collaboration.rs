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

pub struct CollaborationABI {
    pub contract_abis: Vec<ContractABI>,
}

#[derive(Serialize)]
pub struct ContractABI {
    pub name: String,
    pub data: Vec<ParamABI>,
    pub rights: Vec<RightABI>,
}

#[derive(Serialize)]
pub struct TrivialABI {
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(skip_serializing_if = "::std::string::String::is_empty")]
    pub name: String,
}

impl TrivialABI {
    pub fn new(ty: String, name: String) -> Self {
        TrivialABI { ty, name }
    }
}

#[derive(Serialize)]
pub struct CompositeABI {
    #[serde(flatten)]
    pub trivial: TrivialABI,
    #[serde(skip_serializing_if = "::std::vec::Vec::is_empty")]
    pub components: Vec<ParamABI>,
}

#[derive(Serialize)]
pub struct OptionABI {
    #[serde(flatten)]
    pub trivial: TrivialABI,
    pub some: Box<ParamABI>,
}

#[derive(Serialize)]
pub struct ResultABI {
    #[serde(flatten)]
    pub trivial: TrivialABI,
    pub ok: Box<ParamABI>,
    pub err: Box<ParamABI>,
}

#[derive(Serialize)]
#[serde(untagged)]
#[derive(From)]
pub enum ParamABI {
    Opt(OptionABI),
    Res(ResultABI),
    Composite(CompositeABI),
    Trivial(TrivialABI),
}

#[derive(Serialize)]
#[allow(non_snake_case)]
pub struct RightABI {
    pub constant: bool,
    pub inputs: Vec<ParamABI>,
    pub name: String,
    pub outputs: Vec<ParamABI>,
}

impl RightABI {
    pub fn new_builder(name: String, constant: bool) -> RightABIBuilder {
        RightABIBuilder {
            abi: Self {
                constant,
                inputs: Vec::new(),
                name,
                outputs: Vec::new(),
            },
        }
    }
}

pub struct RightABIBuilder {
    abi: RightABI,
}

impl RightABIBuilder {
    pub fn input(&mut self, param_abi: ParamABI) {
        self.abi.inputs.push(param_abi);
    }

    pub fn output(&mut self, param_abi: ParamABI) {
        self.abi.outputs.push(param_abi);
    }

    pub fn done(self) -> RightABI {
        self.abi
    }
}

impl FnOutputBuilder for RightABIBuilder {
    fn output(&mut self, param_abi: ParamABI) {
        self.abi.outputs.push(param_abi);
    }
}

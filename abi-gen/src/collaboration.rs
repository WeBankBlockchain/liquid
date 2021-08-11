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

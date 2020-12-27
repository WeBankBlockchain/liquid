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

use cfg_if::cfg_if;
use derive_more::From;
use serde::Serialize;

pub struct ContractABI {
    pub constructor_abi: ConstructorABI,
    pub external_fn_abis: Vec<ExternalFnABI>,
    pub event_abis: Vec<EventABI>,
}

cfg_if! {
    if #[cfg(feature = "solidity-compatible")] {
        #[derive(Serialize)]
        pub struct TrivialABI {
            #[serde(rename = "type")]
            pub ty: String,
            pub name: String,
        }

        #[derive(Serialize, From)]
        #[serde(untagged)]
        pub enum ParamABI {
            Composite(CompositeABI),
            Trivial(TrivialABI),
            None,
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
    } else {
        #[derive(Serialize)]
        pub struct TrivialABI {
            #[serde(rename = "type")]
            pub ty: String,
            #[serde(skip_serializing_if = "::std::string::String::is_empty")]
            pub name: String,
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

        #[derive(Serialize, From)]
        #[serde(untagged)]
        pub enum ParamABI {
            Opt(OptionABI),
            Res(ResultABI),
            Composite(CompositeABI),
            Trivial(TrivialABI),
            None,
        }

        #[derive(Serialize)]
        pub struct ConstructorABI {
            inputs: Vec<ParamABI>,
            #[serde(rename = "type")]
            ty: String,
        }

        impl ConstructorABI {
            pub fn new_builder() -> ConstructorABIBuilder {
                ConstructorABIBuilder {
                    abi: Self {
                        inputs: Vec::new(),
                        ty: "constructor".to_owned(),
                    },
                }
            }
        }

        #[derive(Serialize)]
        pub struct ExternalFnABI {
            constant: bool,
            inputs: Vec<ParamABI>,
            name: String,
            outputs: Vec<ParamABI>,
            #[serde(rename = "type")]
            ty: String,
        }

        impl ExternalFnABI {
            pub fn new_builder(
                name: String,
                constant: bool,
            ) -> ExternalFnABIBuilder {
                ExternalFnABIBuilder {
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
    }
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

pub struct ConstructorABIBuilder {
    abi: ConstructorABI,
}

impl ConstructorABIBuilder {
    pub fn input(mut self, param_abi: ParamABI) -> Self {
        match param_abi {
            ParamABI::None => (),
            other => {
                self.abi.inputs.push(other);
            }
        }
        self
    }

    pub fn done(self) -> ConstructorABI {
        self.abi
    }
}

pub struct ExternalFnABIBuilder {
    abi: ExternalFnABI,
}

impl FnOutputBuilder for ExternalFnABIBuilder {
    fn output(&mut self, param_abi: ParamABI) {
        self.abi.outputs.push(param_abi);
    }
}

impl ExternalFnABIBuilder {
    pub fn input(&mut self, param_abi: ParamABI) {
        match param_abi {
            ParamABI::None => (),
            other => {
                self.abi.inputs.push(other);
            }
        }
    }

    pub fn done(self) -> ExternalFnABI {
        self.abi
    }
}

#[derive(Serialize)]
pub struct EventParamABI {
    pub indexed: bool,
    #[serde(flatten)]
    pub param_abi: ParamABI,
}

#[derive(Serialize)]
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
    pub fn input(&mut self, param_abi: ParamABI, indexed: bool) {
        self.abi.inputs.push(EventParamABI { param_abi, indexed });
    }

    pub fn done(self) -> EventABI {
        self.abi
    }
}

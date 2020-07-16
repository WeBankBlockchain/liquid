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

use crate::{dispatcher::*, traits::*, DispatchError};

pub type Constructor<Storage, Input> = fn(&mut Storage, Input);

/// The contract definition.
pub struct Contract<Storage, ConstrInput, Externals> {
    pub storage: Storage,
    pub constructor: Constructor<Storage, ConstrInput>,
    pub external_fns: Externals,
}

pub struct ContractBuilder<Storage, ConstrInput, Externals> {
    storage: Storage,
    constructor: Constructor<Storage, ConstrInput>,
    external_fns: Externals,
}

impl Contract<(), (), ()> {
    pub fn new_builder<Storage, ConstrInput>(
        constructor: Constructor<Storage, ConstrInput>,
    ) -> ContractBuilder<Storage, ConstrInput, EmptyList>
    where
        Storage: liquid_core::storage::New,
        ConstrInput: liquid_abi_coder::Decode,
    {
        ContractBuilder {
            storage: Storage::new(),
            constructor,
            external_fns: DispatcherList::empty(),
        }
    }
}

impl<Storage, ConstrInput, Externals> ContractBuilder<Storage, ConstrInput, Externals>
where
    Externals: PushIntoDispatchList,
{
    pub fn on_external<External>(
        self,
        f: DispatchableFn<Storage, External>,
    ) -> ContractBuilder<
        Storage,
        ConstrInput,
        DispatcherList<Dispatcher<Storage, External>, Externals>,
    >
    where
        External: ExternalFn,
    {
        ContractBuilder {
            storage: self.storage,
            constructor: self.constructor,
            external_fns: self.external_fns.push(Dispatcher::new(f)),
        }
    }

    pub fn on_external_mut<External>(
        self,
        f: DispatchableFnMut<Storage, External>,
    ) -> ContractBuilder<
        Storage,
        ConstrInput,
        DispatcherList<DispatcherMut<Storage, External>, Externals>,
    >
    where
        External: ExternalFn,
    {
        ContractBuilder {
            storage: self.storage,
            constructor: self.constructor,
            external_fns: self.external_fns.push(DispatcherMut::new(f)),
        }
    }

    pub fn done(self) -> Contract<Storage, ConstrInput, Externals> {
        Contract {
            storage: self.storage,
            constructor: self.constructor,
            external_fns: self.external_fns,
        }
    }
}

pub enum CallMode {
    // Mode for deploying a contract
    Deploy,
    // Mode for calling an external function
    Call,
}

impl<Storage, ConstrInput, Externals> Contract<Storage, ConstrInput, Externals>
where
    Externals: Dispatch<Storage>,
    ConstrInput: liquid_abi_coder::Decode,
{
    pub fn dispatch(mut self, mode: CallMode) -> Result<()> {
        let call_data = liquid_core::env::get_call_data()
            .map_err(|_| DispatchError::CouldNotReadInput)?;
        match mode {
            CallMode::Deploy => {
                let data = call_data.data;
                let args = <ConstrInput as liquid_abi_coder::Decode>::decode(
                    &mut data.as_slice(),
                )
                .map_err(|_| DispatchError::InvalidParams)?;
                (self.constructor)(&mut self.storage, args);
                Ok(())
            }
            CallMode::Call => self.external_fns.dispatch(&mut self.storage, &call_data),
        }
    }
}

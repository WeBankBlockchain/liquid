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

use crate::{traits::*, DispatchError};
use core::any::TypeId;
use liquid_core::env::{finish, CallData};

pub type Result<T> = core::result::Result<T, DispatchError>;

pub trait Dispatch<Storage> {
    fn dispatch(&self, storage: &mut Storage, input: &CallData) -> Result<()>;
}

pub struct DispatcherList<Current, Rest> {
    current: Current,
    rest: Rest,
}

pub struct EmptyList {}

impl<Storage> Dispatch<Storage> for EmptyList {
    fn dispatch(&self, _: &mut Storage, _: &CallData) -> Result<()> {
        Err(DispatchError::UnknownSelector)
    }
}

impl DispatcherList<(), ()> {
    /// Creates an empty dispatch list.
    pub fn empty() -> EmptyList {
        EmptyList {}
    }
}

pub trait PushIntoDispatchList: Sized {
    fn push<D>(self, dispatcher: D) -> DispatcherList<D, Self>;
}

impl PushIntoDispatchList for EmptyList {
    fn push<D>(self, dispatcher: D) -> DispatcherList<D, Self> {
        DispatcherList {
            current: dispatcher,
            rest: self,
        }
    }
}

impl<Current, Rest> PushIntoDispatchList for DispatcherList<Current, Rest> {
    fn push<D>(self, dispatcher: D) -> DispatcherList<D, Self> {
        DispatcherList {
            current: dispatcher,
            rest: self,
        }
    }
}

impl<Storage, Current, Rest> Dispatch<Storage> for DispatcherList<Current, Rest>
where
    Current: Dispatch<Storage> + FnSelectors,
    Rest: Dispatch<Storage>,
{
    fn dispatch(&self, storage: &mut Storage, data: &CallData) -> Result<()> {
        if <Current as FnSelectors>::KECCAK256_SELECTOR == data.selector
            || <Current as FnSelectors>::SM3_SELECTOR == data.selector
        {
            self.current.dispatch(storage, data)
        } else {
            self.rest.dispatch(storage, data)
        }
    }
}

pub type DispatchableFn<Storage, External> =
    fn(&Storage, <External as FnInput>::Input) -> <External as FnOutput>::Output;
pub type DispatchableFnMut<Storage, External> =
    fn(&mut Storage, <External as FnInput>::Input) -> <External as FnOutput>::Output;

macro_rules! impl_dispatcher_for {
    ($name:ident($dispatchable_fn:ident)$(;$t:tt)?) => {
        pub struct $name<Storage, External>
        where
            External: FnInput + FnOutput,
        {
            dispatchable: $dispatchable_fn<Storage, External>,
        }

        impl<Storage, External> $name<Storage, External>
        where
            External: FnInput + FnOutput,
            <External as FnInput>::Input: liquid_abi_codec::Decode,
            <External as FnOutput>::Output: liquid_abi_codec::Encode,
        {
            pub fn new(dispatchable: $dispatchable_fn<Storage, External>) -> Self {
                Self { dispatchable }
            }

            pub fn eval(&self, storage: & $($t)* Storage, inputs: <External as FnInput>::Input) -> <External as FnOutput>::Output {
                (self.dispatchable)(storage, inputs)
            }
        }

        impl<Storage, External> FnSelectors for $name<Storage, External>
        where
            External: FnInput + FnOutput + FnSelectors,
        {
            const KECCAK256_SELECTOR: liquid_primitives::Selector = <External as FnSelectors>::KECCAK256_SELECTOR;
            const SM3_SELECTOR: liquid_primitives::Selector = <External as FnSelectors>::SM3_SELECTOR;
        }

        impl<Storage, External> Dispatch<Storage> for $name<Storage, External>
        where
            External: ExternalFn,
            <External as FnInput>::Input: liquid_abi_codec::Decode,
            <External as FnOutput>::Output: liquid_abi_codec::Encode,
            Storage: liquid_core::storage::Flush,
        {
            fn dispatch(&self, storage: &mut Storage, input: &CallData) -> Result<()> {
                let args = <<External as FnInput>::Input as liquid_abi_codec::Decode>::decode(&mut input.data.as_slice())
                    .map_err(|_| DispatchError::InvalidParams)?;
                let result = self.eval(storage, args);

                if <External as FnMutability>::IS_MUT {
                    storage.flush();
                }

                if TypeId::of::<<External as FnOutput>::Output>() != TypeId::of::<()>() {
                    finish(&result);
                }
                Ok(())
            }
        }
    };
}

impl_dispatcher_for! { Dispatcher(DispatchableFn) }
impl_dispatcher_for! { DispatcherMut(DispatchableFnMut); mut }

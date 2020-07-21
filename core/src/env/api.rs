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

use crate::env::{
    engine::{EnvInstance, OnInstance},
    CallData, Env, Result,
};

pub fn set_storage<V>(key: &[u8], value: &V)
where
    V: scale::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        Env::set_storage::<V>(instance, key, value);
    })
}

pub fn get_storage<R>(key: &[u8]) -> Result<R>
where
    R: scale::Decode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        Env::get_storage::<R>(instance, key)
    })
}

pub fn get_call_data() -> Result<CallData> {
    <EnvInstance as OnInstance>::on_instance(|instance| Env::get_call_data(instance))
}

pub fn finish<V>(return_value: &V)
where
    V: liquid_abi_coder::Encode,
{
    <EnvInstance as OnInstance>::on_instance(|instance| {
        Env::finish(instance, return_value);
    })
}

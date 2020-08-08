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

use crate::ParamABI;
use liquid_prelude::{string::String, vec::Vec};

pub trait HasComponents<T = ()> {
    const HAS_COMPONENTS: bool = false;
}

pub trait IsDynamicArray<T = ()> {
    const IS_DYNAMIC_ARRAY: bool = false;
}

pub trait GenerateComponents<T = ()> {
    fn generate_components() -> Vec<ParamABI> {
        Vec::new()
    }
}

macro_rules! impl_primitive_tys {
    ($( $t:ty ),*) => {
        $(
            impl GenerateComponents for $t {}

            impl HasComponents for $t {}

            impl IsDynamicArray for $t {}
        )*
    };
}
impl_primitive_tys!(
    bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    String,
    ()
);

impl<T> HasComponents for Vec<T>
where
    T: HasComponents,
{
    const HAS_COMPONENTS: bool = <T as HasComponents>::HAS_COMPONENTS;
}

impl<T> GenerateComponents for Vec<T>
where
    T: GenerateComponents,
{
    fn generate_components() -> Vec<ParamABI> {
        <T as GenerateComponents>::generate_components()
    }
}

impl<T> IsDynamicArray for Vec<T> {
    const IS_DYNAMIC_ARRAY: bool = true;
}

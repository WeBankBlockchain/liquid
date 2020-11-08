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

use std::marker::PhantomData;

pub trait ReturnDefault<T> {
    fn return_default() -> Option<T>;
}

pub struct DefaultReturner<T>(PhantomData<T>);

impl<T> ReturnDefault<T> for DefaultReturner<T> {
    default fn return_default() -> Option<T> {
        None
    }
}

impl<T> ReturnDefault<T> for DefaultReturner<T>
where
    T: Default,
{
    fn return_default() -> Option<T> {
        Some(T::default())
    }
}

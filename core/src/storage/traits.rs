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

pub trait Flush {
    fn flush(&mut self) {}
}

pub trait New {
    fn new() -> Self;
}

pub trait Bind {
    fn bind_with(key: &[u8]) -> Self;
}

#[allow(non_camel_case_types)]
pub trait You_Should_Use_A_Container_To_Wrap_Your_State_Field_In_Storage: Sized {
    type T = Self;
}

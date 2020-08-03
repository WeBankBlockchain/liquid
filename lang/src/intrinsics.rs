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

use liquid_core::env;
use liquid_prelude::string::String;

pub fn require<Q>(expr: bool, msg: Q)
where
    Q: AsRef<str>,
{
    if !expr {
        let err_info = String::from(msg.as_ref());
        // `revert` will terminate the execution of contract immediately.
        env::revert(&err_info);
    }
}

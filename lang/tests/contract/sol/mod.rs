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

use serial_test::serial;

#[test]
#[serial]
fn compile_tests() {
    let t = trybuild::TestCases::new();

    t.compile_fail("tests/contract/sol/ui/fail/01-vec-tuple-return.rs");
    t.compile_fail("tests/contract/sol/ui/fail/02-vec-unit-return.rs");
    t.compile_fail("tests/contract/sol/ui/fail/03-tuple-unit-return.rs");
}

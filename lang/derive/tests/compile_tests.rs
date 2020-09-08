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

#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/01-non-empty-struct.rs");
    t.pass("tests/ui/pass/02-user-defined-inputs.rs");
    t.pass("tests/ui/pass/03-user-defined-output.rs");
    t.pass("tests/ui/pass/04-user-defined-state.rs");
    t.pass("tests/ui/pass/05-dynamic-array-inout.rs");
    t.compile_fail("tests/ui/fail/01-empty-struct.rs");
    t.compile_fail("tests/ui/fail/02-enum.rs");
    t.compile_fail("tests/ui/fail/03-union.rs");
    t.compile_fail("tests/ui/fail/04-not-public.rs");
}

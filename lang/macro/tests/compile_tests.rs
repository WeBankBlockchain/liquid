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
    t.pass("tests/ui/pass/01-noop-contract.rs");
    t.pass("tests/ui/pass/02-hello-world-contract.rs");
    t.pass("tests/ui/pass/03-incrementer-contract.rs");
    t.pass("tests/ui/pass/04-type-alias.rs");
    t.pass("tests/ui/pass/05-different-functions-same-inputs.rs");
    t.pass("tests/ui/pass/06-multiple-returns.rs");
    t.compile_fail("tests/ui/fail/01-constructor-returns.rs");
    t.compile_fail("tests/ui/fail/02-missing-constructor.rs");
    t.compile_fail("tests/ui/fail/03-multiple-constructors.rs");
    t.compile_fail("tests/ui/fail/04-missing-external.rs");
    t.compile_fail("tests/ui/fail/05-forbidden-ident.rs");
    t.compile_fail("tests/ui/fail/06-constructor-no-mut-ref.rs");
    t.compile_fail("tests/ui/fail/07-missing-storage-struct.rs");
    t.compile_fail("tests/ui/fail/08-multiple-storage-struct.rs");
    t.compile_fail("tests/ui/fail/09-invalid-visibility.rs");
    t.compile_fail("tests/ui/fail/10-private-constructor.rs");
    t.compile_fail("tests/ui/fail/11-unsafe-function.rs");
    t.compile_fail("tests/ui/fail/12-const-function.rs");
    t.compile_fail("tests/ui/fail/13-async-function.rs");
    t.compile_fail("tests/ui/fail/14-abi-function.rs");
    t.compile_fail("tests/ui/fail/15-generic-function.rs");
    t.compile_fail("tests/ui/fail/16-invalid-parameter-type.rs");
    t.compile_fail("tests/ui/fail/17-invalid-return-type.rs");
    t.compile_fail("tests/ui/fail/18-too-many-inputs.rs");
    t.compile_fail("tests/ui/fail/19-too-many-outputs.rs");
    t.compile_fail("tests/ui/fail/20-invalid-constructor-parameter.rs");
    t.compile_fail("tests/ui/fail/21-generic-storage.rs");
    t.compile_fail("tests/ui/fail/22-missing-liquid-methods-tag.rs");
}

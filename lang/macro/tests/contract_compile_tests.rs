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
    t.pass("tests/ui/contract/pass/01-noop-contract.rs");
    t.pass("tests/ui/contract/pass/02-hello-world.rs");
    t.pass("tests/ui/contract/pass/03-incrementer.rs");
    t.pass("tests/ui/contract/pass/04-type-alias.rs");
    t.pass("tests/ui/contract/pass/05-different-functions-same-inputs.rs");
    t.pass("tests/ui/contract/pass/06-multiple-returns.rs");
    t.pass("tests/ui/contract/pass/07-getter.rs");
    t.pass("tests/ui/contract/pass/08-unit-return.rs");
    t.pass("tests/ui/contract/pass/09-event.rs");
    t.pass("tests/ui/contract/pass/10-empty-event.rs");
    t.pass("tests/ui/contract/pass/11-u256-i256.rs");
    t.pass("tests/ui/contract/pass/12-multiple-impls.rs");
    t.pass("tests/ui/contract/pass/13-interface.rs");
    t.pass("tests/ui/contract/pass/14-interface-name-alias.rs");
    t.pass("tests/ui/contract/pass/15-fixed-size-bytes.rs");
    t.pass("tests/ui/contract/pass/16-array.rs");
    t.pass("tests/ui/contract/pass/17-bytes.rs");
    t.compile_fail("tests/ui/contract/fail/01-constructor-returns.rs");
    t.compile_fail("tests/ui/contract/fail/02-missing-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/03-multiple-constructors.rs");
    t.compile_fail("tests/ui/contract/fail/04-missing-external.rs");
    t.compile_fail("tests/ui/contract/fail/05-forbidden-ident.rs");
    t.compile_fail("tests/ui/contract/fail/06-constructor-no-mut-ref.rs");
    t.compile_fail("tests/ui/contract/fail/07-missing-storage-struct.rs");
    t.compile_fail("tests/ui/contract/fail/08-multiple-storage-struct.rs");
    t.compile_fail("tests/ui/contract/fail/09-invalid-visibility.rs");
    t.compile_fail("tests/ui/contract/fail/10-private-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/11-unsafe-function.rs");
    t.compile_fail("tests/ui/contract/fail/12-const-function.rs");
    t.compile_fail("tests/ui/contract/fail/13-async-function.rs");
    t.compile_fail("tests/ui/contract/fail/14-abi-function.rs");
    t.compile_fail("tests/ui/contract/fail/15-generic-function.rs");
    t.compile_fail("tests/ui/contract/fail/16-invalid-parameter-type.rs");
    t.compile_fail("tests/ui/contract/fail/17-invalid-return-type.rs");
    t.compile_fail("tests/ui/contract/fail/18-too-many-inputs.rs");
    t.compile_fail("tests/ui/contract/fail/19-too-many-outputs.rs");
    t.compile_fail("tests/ui/contract/fail/20-invalid-constructor-parameter.rs");
    t.compile_fail("tests/ui/contract/fail/21-generic-storage.rs");
    t.compile_fail("tests/ui/contract/fail/22-missing-liquid-methods-tag.rs");
    t.compile_fail("tests/ui/contract/fail/23-not-use-container.rs");
    t.compile_fail("tests/ui/contract/fail/24-vec-tuple-return.rs");
    t.compile_fail("tests/ui/contract/fail/25-vec-unit-return.rs");
    t.compile_fail("tests/ui/contract/fail/26-tuple-unit-return.rs");
    t.compile_fail("tests/ui/contract/fail/27-too-many-topics.rs");
    t.compile_fail("tests/ui/contract/fail/28-invalid-event-data-type.rs");
    t.compile_fail("tests/ui/contract/fail/29-invalid-event-topic-type.rs");
    t.compile_fail("tests/ui/contract/fail/30-invalid-meta-info-key-1.rs");
    t.compile_fail("tests/ui/contract/fail/31-invalid-meta-info-key-2.rs");
    t.compile_fail("tests/ui/contract/fail/32-no-interface-name.rs");
    t.compile_fail("tests/ui/contract/fail/33-empty-interface-name.rs");
    t.compile_fail("tests/ui/contract/fail/34-invalid-interface-name-1.rs");
    t.compile_fail("tests/ui/contract/fail/35-invalid-interface-name-2.rs");
    t.compile_fail("tests/ui/contract/fail/36-invalid-interface-name-3.rs");
    t.compile_fail("tests/ui/contract/fail/37-invalid-item-in-interface.rs");
    t.compile_fail("tests/ui/contract/fail/38-invalid-item-in-extern.rs");
    t.compile_fail("tests/ui/contract/fail/39-too-many-extern-in-interface.rs");
    t.compile_fail("tests/ui/contract/fail/40-invalid-ABI-specification.rs");
    t.compile_fail("tests/ui/contract/fail/41-invalid-method-param-type-in-interface.rs");
    t.compile_fail(
        "tests/ui/contract/fail/42-invalid-method-return-type-in-interface.rs",
    );
    t.compile_fail("tests/ui/contract/fail/43-specify-method-visibility-in-interface.rs");
    t.compile_fail("tests/ui/contract/fail/44-specify-struct-visibility-in-interface.rs");
    t.compile_fail("tests/ui/contract/fail/45-no-receiver-in-interface.rs");
}

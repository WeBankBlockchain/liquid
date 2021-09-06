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

    t.pass("tests/contract/ui/pass/01-noop.rs");
    t.pass("tests/contract/ui/pass/02-hello-world.rs");
    t.pass("tests/contract/ui/pass/03-incrementer.rs");
    t.pass("tests/contract/ui/pass/04-type-alias.rs");
    t.pass("tests/contract/ui/pass/05-different-functions-same-inputs.rs");
    t.pass("tests/contract/ui/pass/06-multiple-returns.rs");
    t.pass("tests/contract/ui/pass/07-getter.rs");
    t.pass("tests/contract/ui/pass/08-unit-return.rs");
    t.pass("tests/contract/ui/pass/09-empty-event.rs");
    t.pass("tests/contract/ui/pass/10-u256-i256.rs");
    t.pass("tests/contract/ui/pass/11-multiple-impls.rs");
    t.pass("tests/contract/ui/pass/12-interface.rs");
    t.pass("tests/contract/ui/pass/13-interface-name-alias.rs");
    t.pass("tests/contract/ui/pass/14-fixed-size-bytes.rs");
    t.pass("tests/contract/ui/pass/15-bytes.rs");
    t.pass("tests/contract/ui/pass/16-mock-context-getter.rs");
    t.pass("tests/contract/ui/pass/17-event.rs");
    t.pass("tests/contract/ui/pass/18-array.rs");
    t.pass("tests/contract/ui/pass/19-vec-tuple-return.rs");
    t.pass("tests/contract/ui/pass/20-vec-unit-return.rs");
    t.pass("tests/contract/ui/pass/21-tuple-unit-return.rs");
    t.pass("tests/contract/ui/pass/22-many-inputs.rs");
    t.compile_fail("tests/contract/ui/fail/01-constructor-returns.rs");
    t.compile_fail("tests/contract/ui/fail/02-missing-constructor.rs");
    t.compile_fail("tests/contract/ui/fail/03-multiple-constructors.rs");
    t.compile_fail("tests/contract/ui/fail/04-missing-external.rs");
    t.compile_fail("tests/contract/ui/fail/05-forbidden-ident.rs");
    t.compile_fail("tests/contract/ui/fail/06-constructor-no-mut-ref.rs");
    t.compile_fail("tests/contract/ui/fail/07-missing-storage-struct.rs");
    t.compile_fail("tests/contract/ui/fail/08-multiple-storage-struct.rs");
    t.compile_fail("tests/contract/ui/fail/09-invalid-visibility.rs");
    t.compile_fail("tests/contract/ui/fail/10-private-constructor.rs");
    t.compile_fail("tests/contract/ui/fail/11-unsafe-function.rs");
    t.compile_fail("tests/contract/ui/fail/12-const-function.rs");
    t.compile_fail("tests/contract/ui/fail/13-async-function.rs");
    t.compile_fail("tests/contract/ui/fail/14-abi-function.rs");
    t.compile_fail("tests/contract/ui/fail/15-generic-function.rs");
    t.compile_fail("tests/contract/ui/fail/16-invalid-parameter-type.rs");
    t.compile_fail("tests/contract/ui/fail/17-invalid-return-type.rs");
    t.compile_fail("tests/contract/ui/fail/18-many-elements-in-tuple.rs");
    t.compile_fail("tests/contract/ui/fail/19-too-many-outputs.rs");
    t.compile_fail("tests/contract/ui/fail/20-invalid-constructor-parameter.rs");
    t.compile_fail("tests/contract/ui/fail/21-generic-storage.rs");
    t.compile_fail("tests/contract/ui/fail/22-missing-liquid-methods-tag.rs");
    t.compile_fail("tests/contract/ui/fail/23-not-use-container.rs");
    t.compile_fail("tests/contract/ui/fail/24-invalid-receiver-1.rs");
    t.compile_fail("tests/contract/ui/fail/25-invalid-receiver-2.rs");
    t.compile_fail("tests/contract/ui/fail/26-event-storage-simultaneously.rs");
    t.compile_fail("tests/contract/ui/fail/27-too-many-topics.rs");
    t.compile_fail("tests/contract/ui/fail/28-invalid-event-data-type.rs");
    t.compile_fail("tests/contract/ui/fail/29-invalid-event-topic-type.rs");
    t.compile_fail("tests/contract/ui/fail/30-invalid-meta-info-key-1.rs");
    t.compile_fail("tests/contract/ui/fail/31-invalid-meta-info-key-2.rs");
    t.compile_fail("tests/contract/ui/fail/32-no-interface-name.rs");
    t.compile_fail("tests/contract/ui/fail/33-empty-interface-name.rs");
    t.compile_fail("tests/contract/ui/fail/34-invalid-interface-name-1.rs");
    t.compile_fail("tests/contract/ui/fail/35-invalid-interface-name-2.rs");
    t.compile_fail("tests/contract/ui/fail/36-invalid-interface-name-3.rs");
    t.compile_fail("tests/contract/ui/fail/37-invalid-item-in-interface.rs");
    t.compile_fail("tests/contract/ui/fail/38-invalid-item-in-extern.rs");
    t.compile_fail("tests/contract/ui/fail/39-too-many-extern-in-interface.rs");
    t.compile_fail("tests/contract/ui/fail/40-invalid-ABI-specification.rs");
    t.compile_fail("tests/contract/ui/fail/41-invalid-param-type-in-interface.rs");
    t.compile_fail("tests/contract/ui/fail/42-invalid-return-type-in-interface.rs");
    t.compile_fail("tests/contract/ui/fail/43-specify-method-visibility-in-interface.rs");
    t.compile_fail("tests/contract/ui/fail/44-specify-struct-visibility-in-interface.rs");
    t.compile_fail("tests/contract/ui/fail/45-no-receiver-in-interface.rs");
    t.compile_fail("tests/contract/ui/fail/46-invalid-mock-context-getter-1.rs");
    t.compile_fail("tests/contract/ui/fail/47-invalid-mock-context-getter-2.rs");
    t.compile_fail("tests/contract/ui/fail/48-overriding-interface.rs");
    t.compile_fail("tests/contract/ui/fail/49-invalid-state-type.rs");
    t.compile_fail("tests/contract/ui/fail/50-contract-redefined.rs");
    t.compile_fail("tests/contract/ui/fail/51-define-interface-in-contract.rs")
}

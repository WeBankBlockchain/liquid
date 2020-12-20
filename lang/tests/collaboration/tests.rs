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
    t.pass("tests/collaboration/ui/pass/01-voting.rs");
    t.pass("tests/collaboration/ui/pass/02-tic-tac-toe.rs");
    t.pass("tests/collaboration/ui/pass/03-auction.rs");
    t.pass("tests/collaboration/ui/pass/04-role.rs");
    t.pass("tests/collaboration/ui/pass/05-shop.rs");
    t.compile_fail("tests/collaboration/ui/fail/01-no-signers.rs");
    t.compile_fail("tests/collaboration/ui/fail/02-no-contract.rs")
}

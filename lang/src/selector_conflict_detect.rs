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

#[allow(dead_code)]
pub const fn detect(selectors: &[[u8; 4]]) {
    let mut i = 0;
    while i < selectors.len() {
        let mut j = i + 1;
        while j < selectors.len() {
            let mut k = 0;
            let mut is_eq = true;
            while k < 4 {
                if selectors[i][k] != selectors[j][k] {
                    is_eq = false;
                    break;
                }
                k += 1;
            }
            if is_eq {
                panic!("selector conflict detected, refuse to compile");
            }
            j += 1;
        }
        i += 1;
    }
}

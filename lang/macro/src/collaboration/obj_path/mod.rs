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

// # Syntax
//
// OBJECT_SELECTOR ::= "$" SELECTORS
// SELECTORS ::= SELECTOR SELECTORS
//             | ε
// SELECTOR ::= FIELD_SELECTOR | ARRAY_SELECTOR | PREDICATE_SELECTOR
// FIELD_SELECTOR ::= "." FIELD
// FIELD ::= IDENTIFIER | NUMBER
// ARRAY_SELECTOR ::= "[" ELEMENTS "]"
// PREDICATE_SELECTOR ::= "(?" EXPRESSIONS ")"
// EXPRESSIONS ::= EXPRESSION OTHER_EXPRESSIONS
// OTHER_EXPRESSIONS ::= LOGIC_OP EXPRESSION OTHER_EXPRESSIONS
//                     | ε
// EXPRESSION ::= UNARY OTHER_EXPRESSION
// OTHER_EXPRESSION ::= REL_OP UNARY OTHER_EXPRESSION
//                    | ε
// UNARY ::= "!" TERM
//         | TERM
// TERM ::= INTEGER
//        | BOOL
//        | "@" SELECTORS
//        | "(" EXPRESSIONS ")"
//        | "!" TERM
// LOGIC_OP ::= "&&" | "||"
// REL_OP ::= "<=" | "<" | "==" | "!=" | ">" | ">="
// ELEMENTS ::= INTEGER OTHER_ELEMENTS
//            | ".." RANGE_TAIL
// OTHER_ELEMENTS ::= ".." RANGE_TAIL
//                  | "," INDEXES
// RANGE_TAIL ::= RANGE_TO STEP
// RANGE_TO := INTEGER
//           | ε
// STEP ::= ";" NUMBER
//        | ε
// UNION_TAIL ::= "," INTEGER UNION_TAIL
//              | ε
// BOOL ::= "true" | "false"
// INTEGER ::= SIGN NUMBER
// SIGN ::= "-" | ε

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, len: usize) -> Self {
        Self {
            start,
            end: start + len,
        }
    }
}

mod parser;
mod tokenizer;

pub use parser::{
    Ast, AstNode, AstNodeArena, AstNodeId, AstNodeType, AstVisitor, Field, LogicOp,
    Parser, RelOp,
};

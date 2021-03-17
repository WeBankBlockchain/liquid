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

use crate::collaboration::obj_path::*;
use proc_macro2::{
    Delimiter, Group, Ident, Literal, Punct, Spacing, Span as TokenSpan,
    TokenStream as TokenStream2, TokenTree,
};
use quote::quote;

pub struct PathVisitor<'a> {
    stmts: Vec<TokenStream2>,
    from: Option<TokenStream2>,
    arena: &'a AstNodeArena,

    ident_count: u32,
    is_array_now: bool,
    current_left_expr: Vec<Ident>,
    predicate_start_point: usize,
}

impl<'a> PathVisitor<'a> {
    pub fn new(from: Option<TokenStream2>, arena: &'a AstNodeArena) -> Self {
        Self {
            stmts: Vec::new(),
            from,
            arena,

            ident_count: 1,
            is_array_now: false,
            current_left_expr: Vec::new(),
            predicate_start_point: 0,
        }
    }

    pub fn eval(&mut self, root: AstNodeId) -> TokenStream2 {
        use std::iter::FromIterator;

        self.visit(self.arena, root);
        let last_var = self.last_var();
        let final_stmts = &self.stmts;

        debug_assert!(self.current_left_expr.is_empty());
        if self.from.is_some() {
            quote! {
                {
                    #(#final_stmts)*
                    #last_var
                }
            }
        } else {
            // For predicate selector.
            quote! {
                #(#final_stmts)*
                *#last_var
            }
        }
    }

    fn last_var(&self) -> Ident {
        Ident::new(
            &format!("_{}", self.ident_count - 1),
            TokenSpan::call_site(),
        )
    }

    fn current_var(&self) -> Ident {
        Ident::new(&format!("_{}", self.ident_count), TokenSpan::call_site())
    }
}

impl<'a> AstVisitor for PathVisitor<'a> {
    fn visit_token(&mut self, node: &AstNode) {
        let ty = &node.ty;
        match ty {
            AstNodeType::Root => {
                // Only in the top of call level, AST node in this type will be visited,
                // so that we can call `unwrap` safely.
                let from = self.from.as_ref().unwrap();
                let current_var = self.current_var();
                self.ident_count += 1;

                self.stmts.push(quote! {
                    let #current_var = #from;
                });
            }
            AstNodeType::Field(field) => {
                let last_var = self.last_var();
                let current_var = self.current_var();
                self.ident_count += 1;

                match field {
                    Field::Identifier(ident) => {
                        let ident = Ident::new(&ident, TokenSpan::call_site());
                        self.stmts.push(
                            if self.is_array_now {
                                quote! {
                                    let #current_var = #last_var.iter().map(|val| &val.#ident).collect::<Vec<_>>();
                                }
                            } else {
                                quote! {
                                    let #current_var = &#last_var.#ident;
                                }
                            },
                        );
                    }
                    Field::Number(num) => {
                        let num = Literal::u32_unsuffixed(*num);
                        self.stmts.push(
                            if self.is_array_now {
                                quote! {
                                    let #current_var = #last_var.iter().map(|val| &val.#num).collect::<Vec<_>>();
                                }
                            } else {
                                quote! {
                                    let #current_var = &#last_var.#num;
                                }
                            },
                        );
                    }
                }
            }
            AstNodeType::Array => {
                let last_var = self.last_var();
                let current_var = self.current_var();
                self.ident_count += 1;
                self.is_array_now = true;
                self.stmts.push(quote! {
                    let #current_var = #last_var.into_iter().collect::<Vec<_>>();
                });
            }
            AstNodeType::Range { start, end, step } => {
                let last_var = self.last_var();
                let start = if let Some(start) = start { *start } else { 0 };
                let end = if let Some(end) = end {
                    quote! { #end }
                } else {
                    quote! { len }
                };
                let step = if let Some(step) = step { *step } else { 1u32 };
                let current_var = self.current_var();
                self.ident_count += 1;

                self.stmts.push(quote! {
                    let #current_var = {
                        let len = #last_var.len() as i64;
                        let start = #start;
                        let end = #end;
                        let start = if start < 0 { (len as i64) + start } else { start };
                        let end = if end < 0 { (len as i64) + end } else { end };
                        if start < 0
                            || start > len
                            || end < 0
                            || end > len
                            || start > end
                            || #step == 0  {
                            unreachable!();
                        } else {
                            let start = start as usize;
                            let end = end as usize;
                            #last_var[start..end]
                                .iter()
                                .step_by(#step as usize)
                                .map(|val| *val)
                                .collect::<Vec<_>>()
                        }
                    };
                });
            }
            AstNodeType::Indexes(indexes) => {
                let last_var = self.last_var();
                let current_var = self.current_var();
                self.ident_count += 1;

                if indexes.len() > 1 {
                    let elements = indexes.iter().map(|index| {
                        if *index < 0 {
                            quote! { &#last_var[(len + (#index)) as usize] }
                        } else {
                            quote! { &#last_var[#index as usize] }
                        }
                    });
                    self.stmts.push(quote! {
                        let #current_var = {
                            let len = #last_var.len() as i64;
                            [#(#elements,)*]
                                .iter()
                                .map(|val| **val)
                                .collect::<Vec<_>>()
                        };
                    })
                } else {
                    debug_assert!(indexes.len() == 1);

                    self.is_array_now = false;
                    let index = Literal::i64_unsuffixed(indexes[0]);
                    self.stmts.push(quote! {
                        let #current_var = {
                            let len = #last_var.len() as i64;
                            let index = if #index >= 0 {
                                #index
                            } else {
                                len + (#index)
                            } as usize;
                            // if the index is greater than the length,
                            // the program will be aborted immediately.
                            &#last_var[index]
                        };
                    });
                }
            }
            AstNodeType::Select => (),
            AstNodeType::Bool(flag) => {
                let current_var = self.current_var();
                self.ident_count += 1;
                self.stmts.push(quote! {
                    let #current_var = &#flag;
                });
            }
            AstNodeType::Integer(int) => {
                let current_var = self.current_var();
                self.ident_count += 1;
                let int = Literal::i64_unsuffixed(*int);
                self.stmts.push(quote! {
                    let #current_var = &#int;
                });
            }
            AstNodeType::Current => {
                let current_var = self.current_var();
                self.ident_count += 1;
                self.stmts.push(quote! {
                    let #current_var = _0;
                });
            }
            AstNodeType::Predicate => {
                let mut nested_visitor = PathVisitor::new(None, self.arena);
                if let Some(right) = node.right {
                    let nested_final_stmts = nested_visitor.eval(right);
                    self.stmts.push(quote! {
                        #nested_final_stmts
                    });
                }
            }
            AstNodeType::RelExpr(op) => {
                let op = match op {
                    RelOp::Equal => quote! { == },
                    RelOp::NotEqual => quote! { != },
                    RelOp::Less => quote! { < },
                    RelOp::LessEqual => quote! { <= },
                    RelOp::Greater => quote! { > },
                    RelOp::GreaterEqual => quote! { >= },
                };

                let current_left_expr = self.current_left_expr.pop().unwrap();
                let current_right_expr = self.last_var();
                let current_var = self.current_var();
                self.ident_count += 1;
                self.stmts.push(quote! {
                    let #current_var = &(#current_left_expr #op #current_right_expr);
                });
            }
            AstNodeType::LogicExpr(op) => {
                let current_var = self.current_var();
                let current_right_expr = self.last_var();
                if matches!(op, LogicOp::Not) {
                    self.current_left_expr.pop().unwrap();
                    self.stmts.push(quote! {
                        let #current_var = &(! *#current_right_expr);
                    });
                } else {
                    let op = match op {
                        LogicOp::And => quote! { && },
                        LogicOp::Or => quote! { || },
                        _ => unreachable!(),
                    };
                    let current_left_expr = self.current_left_expr.pop().unwrap();
                    self.ident_count += 1;
                    self.stmts.push(quote! {
                        let #current_var = &(*#current_left_expr #op *#current_right_expr);
                    });
                }
            }
        }
    }

    fn after_visiting_predicate(&mut self, _nodes: &AstNodeArena, _node: &AstNode) {
        let predicate = self.stmts.drain(self.predicate_start_point..);
        let body = Group::new(Delimiter::Brace, quote! { #(#predicate)* });
        let head = quote! { |&&_0| #body };
        let call = Group::new(Delimiter::Parenthesis, head);
        self.stmts.push(quote! {
            #call.map(|val| *val).collect::<Vec<_>>();
        });
    }

    fn after_visiting_left_expr(&mut self) {
        let last_var = self.last_var();
        self.current_left_expr.push(last_var);
    }

    fn before_visiting_predicate(&mut self) {
        debug_assert!(self.is_array_now);

        let last_var = self.last_var();
        let current_var = self.current_var();
        self.ident_count += 1;

        self.stmts.push(quote! {
            let #current_var = #last_var.iter().filter
        });
        self.predicate_start_point = self.stmts.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::{
        ffi::OsStr,
        fs::File,
        io::Write,
        path::PathBuf,
        process::{Command, Output},
    };
    use tempfile::tempdir;

    fn compile_and_execute(
        obj_ty: &TokenStream2,
        obj_var: &TokenStream2,
        obj_init: &TokenStream2,
        stmts: TokenStream2,
    ) -> String {
        let code = format!(
            "use std::io::Write;#[allow(unused_parens)]fn main() {{ {} let {} = {}; let \
             output = {}; std::io::stdout().write(format!(\"{{:?}}\", \
             output).as_bytes()).unwrap();}}",
            obj_ty, obj_var, obj_init, stmts
        );
        let dir = tempdir().unwrap();
        let code_path = dir.path().join("main.rs");
        let mut code_file = File::create(code_path.clone()).unwrap();
        writeln!(code_file, "{}", code).unwrap();

        let code_path = code_path.as_path();
        let code_path_str = code_path.as_os_str();
        let code_file_name = code_path.file_name().unwrap();

        let mut exe_path_buf = code_path.parent().unwrap().to_path_buf();
        exe_path_buf.push(code_file_name);
        exe_path_buf.set_extension("executable");
        let exe_path = exe_path_buf.as_path();
        let exe_path_str = exe_path.as_os_str();

        let mut cmd = Command::new("rustc");
        let status = cmd
            .args(&[code_path_str, OsStr::new("-o"), exe_path_str])
            .status()
            .expect(&format!("fail to compile {}", code_path.display()));
        assert!(status.success());

        let output = Command::new(exe_path_str)
            .output()
            .expect(&format!("fail to run {}", exe_path.display()));
        assert!(output.status.success());
        drop(dir);
        String::from_utf8(output.stdout).unwrap()
    }

    fn run(
        obj_ty: &TokenStream2,
        obj_var: &TokenStream2,
        obj_init: &TokenStream2,
        selector: &str,
    ) -> String {
        let parser = Parser::new(selector);
        let ast = parser.parse().unwrap();
        let mut visitor = PathVisitor::new(Some(quote! { &#obj_var }), &ast.arena);
        let final_stmts = visitor.eval(ast.root);
        compile_and_execute(obj_ty, obj_var, obj_init, final_stmts)
    }

    #[test]
    #[serial]
    fn ident_field() {
        let (obj_ty, obj_var, obj_init) = (
            quote! {
                struct Foo {
                    foo: u8,
                }
            },
            quote! { bar },
            quote! {
                Foo {
                    foo: 42,
                }
            },
        );

        let result = run(&obj_ty, &obj_var, &obj_init, "$.foo");
        assert_eq!(result, "42");

        let (obj_ty, obj_var, obj_init) = (
            quote! {
                struct Baz {
                    u: u8
                }

                struct Foo {
                    baz: Baz,
                }
            },
            quote! { bar },
            quote! {
                Foo {
                    baz: Baz {
                        u: 42
                    }
                }
            },
        );

        let result = run(&obj_ty, &obj_var, &obj_init, "$.baz.u");
        assert_eq!(result, "42");
    }

    #[test]
    #[serial]
    fn num_field() {
        let (obj_ty, obj_var, obj_init) = (
            quote! {
                struct Foo(u8,);
            },
            quote! { bar },
            quote! {
                Foo(42,)
            },
        );

        let result = run(&obj_ty, &obj_var, &obj_init, "$.0");
        assert_eq!(result, "42");

        let (obj_ty, obj_var, obj_init) = (
            quote! {},
            quote! { bar },
            quote! {
                ((((42),),),)
            },
        );

        let result = run(&obj_ty, &obj_var, &obj_init, "$.0.0.0");
        assert_eq!(result, "42");
    }

    #[test]
    #[serial]
    fn array_range() {
        let (obj_ty, obj_var, obj_init) = (
            quote! {
                struct Baz {
                    u: u8,
                }

                struct Foo {
                    v: Vec<Baz>
                }
            },
            quote! { bar },
            quote! {
                Foo {
                    v: vec![
                        Baz { u: 0 },
                        Baz { u: 1 },
                        Baz { u: 2 },
                        Baz { u: 3 },
                    ],
                }
            },
        );

        let result = run(&obj_ty, &obj_var, &obj_init, "$.v[-3..].u");
        assert_eq!(result, "[1, 2, 3]");

        let result = run(&obj_ty, &obj_var, &obj_init, "$.v[..].u");
        assert_eq!(result, "[0, 1, 2, 3]");

        let result = run(&obj_ty, &obj_var, &obj_init, "$.v[..;2].u");
        assert_eq!(result, "[0, 2]");

        let result = run(&obj_ty, &obj_var, &obj_init, "$.v[0..1;2].u");
        assert_eq!(result, "[0]");

        let result = run(&obj_ty, &obj_var, &obj_init, "$.v[..].u[..-1][..-1]");
        assert_eq!(result, "[0, 1]");
    }

    #[test]
    #[should_panic]
    #[serial]
    fn invalid_array_range_1() {
        let (obj_ty, obj_var, obj_init) =
            (quote! {}, quote! { bar }, quote! { vec![0u32] });

        let _ = run(&obj_ty, &obj_var, &obj_init, "$[2..1]");
    }

    #[test]
    #[should_panic]
    #[serial]
    fn invalid_array_range_2() {
        let (obj_ty, obj_var, obj_init) =
            (quote! {}, quote! { bar }, quote! { vec![0u32] });

        let _ = run(&obj_ty, &obj_var, &obj_init, "$[-2..]");
    }

    #[test]
    #[should_panic]
    #[serial]
    fn invalid_array_range_3() {
        let (obj_ty, obj_var, obj_init) =
            (quote! {}, quote! { bar }, quote! { vec![0u32] });

        let _ = run(&obj_ty, &obj_var, &obj_init, "$[..-2]");
    }

    #[test]
    #[should_panic]
    #[serial]
    fn invalid_array_range_4() {
        let (obj_ty, obj_var, obj_init) =
            (quote! {}, quote! { bar }, quote! { vec![0u32] });

        let _ = run(&obj_ty, &obj_var, &obj_init, "$[..2]");
    }

    #[test]
    #[should_panic]
    #[serial]
    fn invalid_array_range_5() {
        let (obj_ty, obj_var, obj_init) =
            (quote! {}, quote! { bar }, quote! { vec![0u32] });

        let _ = run(&obj_ty, &obj_var, &obj_init, "$[..;0]");
    }

    #[test]
    #[serial]
    fn array_indexes() {
        let (obj_ty, obj_var, obj_init) = (
            quote! {
                struct Baz {
                    u: u8,
                }

                struct Foo {
                    v: Vec<Baz>
                }
            },
            quote! { bar },
            quote! {
                Foo {
                    v: vec![
                        Baz { u: 0 },
                        Baz { u: 1 },
                        Baz { u: 2 },
                        Baz { u: 3 },
                    ],
                }
            },
        );

        let result = run(&obj_ty, &obj_var, &obj_init, "$.v[0].u");
        assert_eq!(result, "0");

        let result = run(&obj_ty, &obj_var, &obj_init, "$.v[-1, -2].u[..1]");
        assert_eq!(result, "[3]");
    }

    #[test]
    #[serial]
    fn predicate() {
        let (obj_ty, obj_var, obj_init) = (
            quote! {
                struct Voter {
                    voted: bool,
                    name: String,
                }
                struct Ballot {
                    voters: Vec<Voter>,
                }
            },
            quote! { ballot },
            quote! {
                Ballot {
                    voters: vec![
                        Voter { voted: true, name: "cat".to_owned() },
                        Voter { voted: false, name: "vita".to_owned() },
                        Voter { voted: true, name: "dounai".to_owned() },
                        Voter { voted: false, name: "wangcai".to_owned() },
                    ]
                }
            },
        );

        let result = run(&obj_ty, &obj_var, &obj_init, "$.voters[..](?@.voted).name");
        assert_eq!(result, "[\"cat\", \"dounai\"]");
    }
}

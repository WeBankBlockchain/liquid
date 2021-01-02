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

use super::{
    tokenizer::{Token, TokenReader, TokenizeError},
    Span,
};
use id_arena::{Arena, Id};
use log::debug;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub msg: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(msg: String, span: Span) -> Self {
        Self { msg, span }
    }
}

impl From<TokenizeError> for ParseError {
    fn from(err: TokenizeError) -> Self {
        match err {
            TokenizeError::IllegalNumber { provided, pos } => Self::new(
                format!("illegal representation of number: `{}`", provided),
                Span::new(pos, provided.len()),
            ),
            TokenizeError::UnexpectedCharacter {
                expected,
                provided,
                pos,
            } => Self::new(
                format!(
                    "found unexpected character: expected `{}` but found `{}`",
                    expected, provided
                ),
                Span::new(pos, 1),
            ),
            TokenizeError::UnexpectedEnd(span) => {
                Self::new("found unexpected end".to_string(), span)
            }
            TokenizeError::UnknownCharacter { provided, pos } => Self::new(
                format!("found unknown character: `{}`", provided),
                Span::new(pos, 1),
            ),
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, PartialEq, Clone)]
pub enum LogicOp {
    And,
    Or,
    Not,
}

impl From<Token> for LogicOp {
    fn from(token: Token) -> Self {
        match token {
            Token::And(..) => Self::And,
            Token::Or(..) => Self::Or,
            Token::Not(..) => Self::Not,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum RelOp {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl From<Token> for RelOp {
    fn from(token: Token) -> Self {
        match token {
            Token::Equal(..) => Self::Equal,
            Token::NotEqual(..) => Self::NotEqual,
            Token::Less(..) => Self::Less,
            Token::LessEqual(..) => Self::LessEqual,
            Token::Greater(..) => Self::Greater,
            Token::GreaterEqual(..) => Self::GreaterEqual,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Field {
    Identifier(String),
    Number(u32),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AstNodeType {
    Root,
    Current,
    Integer(i64),
    Bool(bool),
    Indexes(Vec<i64>),
    Range {
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    },

    Field(Field),
    Array,
    Predicate,
    Select,

    LogicExpr(LogicOp),
    RelExpr(RelOp),
}

pub type AstNodeId = Id<AstNode>;
pub type AstNodeArena = Arena<AstNode>;

#[derive(Clone)]
pub struct Ast {
    pub arena: AstNodeArena,
    pub root: AstNodeId,
}

#[derive(Clone)]
pub struct AstNode {
    pub ty: AstNodeType,
    pub left: Option<AstNodeId>,
    pub right: Option<AstNodeId>,
}

pub struct Parser<'a> {
    nodes: AstNodeArena,
    token_reader: TokenReader<'a>,
    has_met_array: bool,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        assert!(input.is_ascii());

        Self {
            nodes: Arena::<AstNode>::new(),
            token_reader: TokenReader::new(input),
            has_met_array: false,
        }
    }

    pub fn parse(mut self) -> ParseResult<Ast> {
        if let Some(err) = self.token_reader.err {
            return Err(err.into());
        }

        let ast_root = self.object_selector()?;
        Ok(Ast {
            arena: self.nodes,
            root: ast_root,
        })
    }

    // OBJECT_SELECTOR ::= "$" SELECTORS
    fn object_selector(&mut self) -> ParseResult<AstNodeId> {
        debug!("#object_selector");
        match self.token_reader.next_token() {
            Ok(Token::Root(_)) => {
                let node = self.nodes.alloc(AstNode {
                    ty: AstNodeType::Root,
                    left: None,
                    right: None,
                });

                self.selectors(node)
            }
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!("expected one of `$` but found `{}`", other.get_str(),),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // SELECTORS ::= SELECTOR SELECTORS
    //             | ε
    fn selectors(&mut self, prev: AstNodeId) -> ParseResult<AstNodeId> {
        debug!("#selectors");
        match self.token_reader.peek_token() {
            Ok(Token::Dot(..))
            | Ok(Token::LBracket(..))
            | Ok(Token::PredicateStart(..)) => {
                let selector = self.selector()?;
                let node = self.nodes.alloc(AstNode {
                    ty: AstNodeType::Select,
                    left: Some(prev),
                    right: Some(selector),
                });
                self.selectors(node)
            }
            Ok(Token::End(..))
            | Ok(Token::RParen(..))
            | Ok(Token::And(..))
            | Ok(Token::Or(..))
            | Ok(Token::Less(..))
            | Ok(Token::LessEqual(..))
            | Ok(Token::Greater(..))
            | Ok(Token::GreaterEqual(..))
            | Ok(Token::Equal(..))
            | Ok(Token::NotEqual(..)) => Ok(prev),
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!(
                        "expected one of `.`, `[`, `(?`, `END`, `)`, `&&`, `||`, `<`, \
                         `<=`, `>`, `>=`, `==` or `!=`, but found `{}`",
                        other.get_str(),
                    ),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // SELECTOR ::= FIELD_SELECTOR | ARRAY_SELECTOR | PREDICATE_SELECTOR
    fn selector(&mut self) -> ParseResult<AstNodeId> {
        debug!("#selector");
        match self.token_reader.peek_token() {
            Ok(Token::Dot(..)) => self.field_selector(),
            Ok(Token::LBracket(..)) => self.array_selector(),
            Ok(Token::PredicateStart(..)) => self.predicate_selector(),
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!("expected `.`, `[` or `(?` but found `{}`", other.get_str()),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // PREDICATE_SELECTOR ::= "(?" EXPRESSIONS ")"
    fn predicate_selector(&mut self) -> ParseResult<AstNodeId> {
        debug!("#predicate_selector");
        if !self.has_met_array {
            let current_token = self.token_reader.next_token().unwrap();
            match current_token {
                Token::PredicateStart(span) => {
                    return Err(ParseError::new(
                        "predicate selector can't be applied to object which is not \
                         iterable"
                            .to_owned(),
                        span,
                    ));
                }
                _ => unreachable!(),
            }
        }
        self.eat_token();
        let expressions = self.expressions()?;
        match self.token_reader.peek_token() {
            Ok(Token::RParen(..)) => {
                self.eat_token();
                Ok(self.nodes.alloc(AstNode {
                    ty: AstNodeType::Predicate,
                    left: None,
                    right: Some(expressions),
                }))
            }
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!("expected `)` but found `{}`", other.get_str(),),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // EXPRESSIONS ::= EXPRESSION OTHER_EXPRESSIONS
    fn expressions(&mut self) -> ParseResult<AstNodeId> {
        debug!("#expressions");
        let expression = self.expression()?;
        self.other_expressions(expression)
    }

    // OTHER_EXPRESSIONS ::= LOGIC_OP EXPRESSION OTHER_EXPRESSIONS
    //                     | ε
    // LOGIC_OP ::= "&&" | "||"
    fn other_expressions(&mut self, prev: AstNodeId) -> ParseResult<AstNodeId> {
        debug!("#other_expressions");
        match self.token_reader.peek_token() {
            Ok(Token::And(..)) | Ok(Token::Or(..)) => {
                let token = self.token_reader.next_token().unwrap();
                let expression = self.expression()?;
                let node = self.nodes.alloc(AstNode {
                    ty: AstNodeType::LogicExpr(token.into()),
                    left: Some(prev),
                    right: Some(expression),
                });
                self.other_expressions(node)
            }
            Ok(Token::RParen(..)) => Ok(prev),
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!(
                        "expected one of `&&`, `||` or `)` but found `{}`",
                        other.get_str(),
                    ),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // EXPRESSION ::= UNARY OTHER_EXPRESSION
    fn expression(&mut self) -> ParseResult<AstNodeId> {
        debug!("#expression");
        let unary = self.unary()?;
        self.other_expression(unary)
    }

    // OTHER_EXPRESSION ::= REL_OP UNARY OTHER_EXPRESSION
    //                    | ε
    // REL_OP ::= "<=" | "<" | "==" | "!=" | ">" | ">="
    fn other_expression(&mut self, prev: AstNodeId) -> ParseResult<AstNodeId> {
        debug!("#other_expression");
        match self.token_reader.peek_token() {
            Ok(Token::Less(..))
            | Ok(Token::LessEqual(..))
            | Ok(Token::Equal(..))
            | Ok(Token::NotEqual(..))
            | Ok(Token::Greater(..))
            | Ok(Token::GreaterEqual(..)) => {
                let token = self.token_reader.next_token().unwrap();
                let rhs = self.unary()?;
                let node = self.nodes.alloc(AstNode {
                    ty: AstNodeType::RelExpr(token.into()),
                    left: Some(prev),
                    right: Some(rhs),
                });
                self.other_expression(node)
            }
            Ok(Token::And(..)) | Ok(Token::Or(..)) | Ok(Token::RParen(..)) => Ok(prev),
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!(
                        "expected one of `<`, `<=`, `==`, `>`, `>=`, `&&`, `||` or `)` \
                         but found `{}`",
                        other.get_str(),
                    ),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // UNARY ::= "!" UNARY | TERM
    fn unary(&mut self) -> ParseResult<AstNodeId> {
        debug!("#unary");
        match self.token_reader.peek_token() {
            Ok(Token::Not(..)) => {
                self.eat_token();
                let unary = self.unary()?;
                Ok(self.nodes.alloc(AstNode {
                    ty: AstNodeType::LogicExpr(LogicOp::Not),
                    left: None,
                    right: Some(unary),
                }))
            }
            _ => self.term(),
        }
    }

    // TERM ::= INTEGER
    //        | BOOL
    //        | "@" SELECTORS
    //        | "(" EXPRESSIONS ")"
    fn term(&mut self) -> ParseResult<AstNodeId> {
        debug!("#term");
        match self.token_reader.peek_token() {
            Ok(Token::Neg(..)) | Ok(Token::Number(..)) => {
                let integer = self.integer()?;
                Ok(self.nodes.alloc(AstNode {
                    ty: AstNodeType::Integer(integer),
                    left: None,
                    right: None,
                }))
            }
            Ok(Token::Bool(_, flag)) => {
                let node = Ok(self.nodes.alloc(AstNode {
                    ty: AstNodeType::Bool(*flag),
                    left: None,
                    right: None,
                }));
                self.eat_token();
                node
            }
            Ok(Token::At(..)) => {
                self.eat_token();
                let current = self.nodes.alloc(AstNode {
                    ty: AstNodeType::Current,
                    left: None,
                    right: None,
                });
                self.selectors(current)
            }
            Ok(Token::LParen(..)) => {
                self.eat_token();
                let expressions = self.expressions()?;
                match self.token_reader.peek_token() {
                    Ok(Token::RParen(..)) => {
                        self.eat_token();
                        Ok(expressions)
                    }
                    _ => unimplemented!(),
                }
            }
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!(
                        "expected one of `-`, number, boolean, `@` or `(` but found `{}`",
                        other.get_str(),
                    ),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // ARRAY_SELECTOR ::= "[" ELEMENTS "]"
    fn array_selector(&mut self) -> ParseResult<AstNodeId> {
        debug!("#array_selector");
        self.eat_token();
        let elements = self.elements()?;
        match self.token_reader.peek_token() {
            Ok(Token::RBracket(..)) => {
                self.eat_token();
                Ok(self.nodes.alloc(AstNode {
                    ty: AstNodeType::Array,
                    left: None,
                    right: Some(elements),
                }))
            }
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!("expected `]` but found `{}`", other.get_str()),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // ELEMENTS ::= INTEGER OTHER_ELEMENTS
    //            | ".." RANGE_TAIL
    fn elements(&mut self) -> ParseResult<AstNodeId> {
        debug!("#elements");
        match self.token_reader.peek_token() {
            Ok(Token::Neg(..)) | Ok(Token::Number(..)) => {
                let index = self.integer()?;
                self.other_elements(index)
            }
            Ok(Token::DotDot(..)) => {
                self.eat_token();
                self.range_tail(None)
            }
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!(
                        "expected one of `-`, number or `..` but found `{}`",
                        other.get_str(),
                    ),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // OTHER_ELEMENTS ::= ".." RANGE_TAIL
    //                  | "," INDEXES
    fn other_elements(&mut self, prev: i64) -> ParseResult<AstNodeId> {
        debug!("#other_elements");
        match self.token_reader.peek_token() {
            Ok(Token::DotDot(..)) => {
                self.eat_token();
                self.range_tail(Some(prev))
            }
            Ok(Token::Comma(..)) => {
                let indexes = vec![prev];
                self.union_tail(indexes)
            }
            Ok(Token::RBracket(..)) => {
                self.has_met_array = false;
                Ok(self.nodes.alloc(AstNode {
                    ty: AstNodeType::Indexes(vec![prev]),
                    left: None,
                    right: None,
                }))
            }
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!(
                        "expected one of `,`, `..` or `]` but found `{}`",
                        other.get_str(),
                    ),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // RANGE_TAIL ::= RANGE_TO STEP
    fn range_tail(&mut self, start: Option<i64>) -> ParseResult<AstNodeId> {
        debug!("#range_tail");
        self.has_met_array = true;
        let end = self.range_to()?;
        let step = self.step()?;

        Ok(self.nodes.alloc(AstNode {
            ty: AstNodeType::Range { start, end, step },
            left: None,
            right: None,
        }))
    }

    // UNION_TAIL ::= "," INTEGER UNION_TAIL
    //              | ε
    fn union_tail(&mut self, mut indexes: Vec<i64>) -> ParseResult<AstNodeId> {
        debug!("#union_tail");
        self.has_met_array = true;
        match self.token_reader.peek_token() {
            Ok(Token::Comma(..)) => {
                self.eat_token();
                let index = self.integer()?;
                indexes.push(index);
                self.union_tail(indexes)
            }
            Ok(Token::RBracket(..)) => Ok(self.nodes.alloc(AstNode {
                ty: AstNodeType::Indexes(indexes.split_off(0)),
                left: None,
                right: None,
            })),
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!("expected one of `,` or `]` but found `{}`", other.get_str()),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // RANGE_TO := INTEGER
    //           | ε
    fn range_to(&mut self) -> ParseResult<Option<i64>> {
        debug!("#range_to");
        match self.token_reader.peek_token() {
            Ok(Token::Neg(..)) | Ok(Token::Number(..)) => {
                let to = self.integer()?;
                Ok(Some(to))
            }
            Ok(Token::Semicolon(..)) | Ok(Token::RBracket(..)) => Ok(None),
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!(
                        "expected one of `-`, number, `;` or `]` but found `{}`",
                        other.get_str(),
                    ),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // STEP ::= ";" NUMBER
    //        | ε
    fn step(&mut self) -> ParseResult<Option<i64>> {
        debug!("#step");
        match self.token_reader.peek_token() {
            Ok(Token::Semicolon(..)) => {
                self.eat_token();
                let step = self.number()?;
                Ok(Some(step as i64))
            }
            Ok(Token::RBracket(..)) => Ok(None),
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!("expected one of `;` or `]` but found `{}`", other.get_str()),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // INTEGER ::= SIGN NUMBER
    // SIGN ::= "-" | ε
    fn integer(&mut self) -> ParseResult<i64> {
        debug!("#integer");
        match self.token_reader.peek_token() {
            Ok(Token::Neg(_)) => {
                self.eat_token();
                let num = self.number()? as i64;
                Ok(-num)
            }
            Ok(Token::Number(..)) => Ok(self.number()? as i64),
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!(
                        "expected one of `-` or number but found `{}`",
                        other.get_str(),
                    ),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    // FIELD_SELECTOR ::= "." FIELD
    fn field_selector(&mut self) -> ParseResult<AstNodeId> {
        debug!("#field_selector");
        self.eat_token();
        self.field()
    }

    // FIELD ::= IDENTIFIER | NUMBER
    fn field(&mut self) -> ParseResult<AstNodeId> {
        debug!("#field");
        match self.token_reader.peek_token() {
            Ok(Token::Identifier(..)) => {
                let ident = self.identifier()?;
                Ok(self.nodes.alloc(AstNode {
                    ty: AstNodeType::Field(Field::Identifier(ident)),
                    left: None,
                    right: None,
                }))
            }
            Ok(Token::Number(..)) => {
                let num = self.number()?;
                Ok(self.nodes.alloc(AstNode {
                    ty: AstNodeType::Field(Field::Number(num)),
                    left: None,
                    right: None,
                }))
            }
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!(
                        "expected one of identifier or number but found `{}`",
                        other.get_str(),
                    ),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    fn identifier(&mut self) -> ParseResult<String> {
        debug!("#identifier");
        match self.token_reader.next_token() {
            Ok(Token::Identifier(_, ident)) => Ok(ident),
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!("expected identifier but found `{}`", other.get_str()),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    fn number(&mut self) -> ParseResult<u32> {
        debug!("#number");
        match self.token_reader.next_token() {
            Ok(Token::Number(_, num)) => Ok(num),
            Ok(other) => {
                let span = other.get_span();
                Err(ParseError::new(
                    format!("expected number but found `{}`", other.get_str(),),
                    span,
                ))
            }
            Err(err) => Err(err.into()),
        }
    }

    fn eat_token(&mut self) {
        let _ = self.token_reader.next_token();
    }
}

pub trait AstVisitor {
    fn visit(&mut self, nodes: &AstNodeArena, node_id: AstNodeId) {
        let node = &nodes[node_id];
        match node.ty {
            AstNodeType::Integer(..)
            | AstNodeType::Bool(..)
            | AstNodeType::Root
            | AstNodeType::Current
            | AstNodeType::Range { .. }
            | AstNodeType::Indexes(..)
            | AstNodeType::Field(..) => self.visit_token(node),

            AstNodeType::Array => {
                self.visit_token(node);
                if let Some(right) = node.right {
                    self.visit(nodes, right)
                }
            }
            AstNodeType::Select => {
                if let Some(left) = node.left {
                    self.visit(nodes, left);
                }
                self.visit_token(node);
                if let Some(right) = node.right {
                    self.visit(nodes, right)
                }
            }
            AstNodeType::Predicate => {
                self.before_visiting_predicate();
                self.visit_token(node);
                self.after_visiting_predicate(nodes, node);
            }
            AstNodeType::LogicExpr(..) | AstNodeType::RelExpr(..) => {
                if let Some(left) = node.left {
                    self.visit(nodes, left);
                }
                self.after_visiting_left_expr();
                if let Some(right) = node.right {
                    self.visit(nodes, right);
                }
                self.visit_token(node);
            }
        }
    }

    fn visit_token(&mut self, node: &AstNode);

    fn after_visiting_left_expr(&mut self) {}
    fn before_visiting_predicate(&mut self) {}
    fn after_visiting_predicate(&mut self, nodes: &AstNodeArena, node: &AstNode) {
        if let Some(right) = node.right {
            self.visit(nodes, right)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AstVisitorTestImpl<'a> {
        input: &'a str,
        stack: Vec<AstNodeType>,
    }

    impl<'a> AstVisitor for AstVisitorTestImpl<'a> {
        fn visit_token(&mut self, node: &AstNode) {
            self.stack.push(node.ty.clone());
        }
    }

    impl<'a> AstVisitorTestImpl<'a> {
        fn new(input: &'a str) -> Self {
            Self {
                input,
                stack: Vec::new(),
            }
        }

        fn start(&mut self) -> ParseResult<Vec<AstNodeType>> {
            let parser = Parser::new(self.input);
            let ast = parser.parse()?;
            self.visit(&ast.arena, ast.root);
            Ok(self.stack.split_off(0))
        }
    }

    fn run(input: &str) -> ParseResult<Vec<AstNodeType>> {
        let mut interpreter = AstVisitorTestImpl::new(input);
        interpreter.start()
    }

    fn setup() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
    #[test]
    fn parse_field() {
        assert_eq!(run("$"), Ok(vec![AstNodeType::Root]));

        assert_eq!(
            run("$.a"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Field(Field::Identifier("a".to_owned())),
            ])
        );

        assert_eq!(
            run("$.0.a.0"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Field(Field::Number(0)),
                AstNodeType::Select,
                AstNodeType::Field(Field::Identifier("a".to_owned())),
                AstNodeType::Select,
                AstNodeType::Field(Field::Number(0)),
            ])
        );
    }

    #[test]
    fn parse_array() {
        assert_eq!(
            run("$[..]"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Array,
                AstNodeType::Range {
                    start: None,
                    end: None,
                    step: None
                }
            ])
        );

        assert_eq!(
            run("$.a[..-1; 3]"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Field(Field::Identifier("a".to_owned())),
                AstNodeType::Select,
                AstNodeType::Array,
                AstNodeType::Range {
                    start: None,
                    end: Some(-1),
                    step: Some(3)
                }
            ])
        );

        assert_eq!(
            run("$[1,2,3]"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Array,
                AstNodeType::Indexes(vec![1, 2, 3]),
            ])
        );

        assert_eq!(
            run("$[1].0"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Array,
                AstNodeType::Indexes(vec![1]),
                AstNodeType::Select,
                AstNodeType::Field(Field::Number(0)),
            ])
        )
    }

    #[test]
    fn parse_predicate() {
        setup();

        assert_eq!(
            run("$[..](?@.voted).addr"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Array,
                AstNodeType::Range {
                    start: None,
                    end: None,
                    step: None
                },
                AstNodeType::Select,
                AstNodeType::Predicate,
                AstNodeType::Current,
                AstNodeType::Select,
                AstNodeType::Field(Field::Identifier("voted".to_owned())),
                AstNodeType::Select,
                AstNodeType::Field(Field::Identifier("addr".to_owned())),
            ])
        );
        assert_eq!(
            run("$[..](?@.count >= 0 && !@.voted).2"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Array,
                AstNodeType::Range {
                    start: None,
                    end: None,
                    step: None
                },
                AstNodeType::Select,
                AstNodeType::Predicate,
                AstNodeType::Current,
                AstNodeType::Select,
                AstNodeType::Field(Field::Identifier("count".to_owned())),
                AstNodeType::Integer(0),
                AstNodeType::RelExpr(RelOp::GreaterEqual),
                AstNodeType::Current,
                AstNodeType::Select,
                AstNodeType::Field(Field::Identifier("voted".to_owned())),
                AstNodeType::LogicExpr(LogicOp::Not),
                AstNodeType::LogicExpr(LogicOp::And),
                AstNodeType::Select,
                AstNodeType::Field(Field::Number(2)),
            ])
        );

        assert_eq!(
            run("$[..](?!(1 == 2))"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Array,
                AstNodeType::Range {
                    start: None,
                    end: None,
                    step: None
                },
                AstNodeType::Select,
                AstNodeType::Predicate,
                AstNodeType::Integer(1),
                AstNodeType::Integer(2),
                AstNodeType::RelExpr(RelOp::Equal),
                AstNodeType::LogicExpr(LogicOp::Not),
            ])
        );

        assert_eq!(
            run("$[..](?true == !false))"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Array,
                AstNodeType::Range {
                    start: None,
                    end: None,
                    step: None
                },
                AstNodeType::Select,
                AstNodeType::Predicate,
                AstNodeType::Bool(true),
                AstNodeType::Bool(false),
                AstNodeType::LogicExpr(LogicOp::Not),
                AstNodeType::RelExpr(RelOp::Equal),
            ])
        );

        assert_eq!(
            run("$[..](?false != (1 > 2 || (!false && @.count < 2)))"),
            Ok(vec![
                AstNodeType::Root,
                AstNodeType::Select,
                AstNodeType::Array,
                AstNodeType::Range {
                    start: None,
                    end: None,
                    step: None
                },
                AstNodeType::Select,
                AstNodeType::Predicate,
                AstNodeType::Bool(false),
                AstNodeType::Integer(1),
                AstNodeType::Integer(2),
                AstNodeType::RelExpr(RelOp::Greater),
                AstNodeType::Bool(false),
                AstNodeType::LogicExpr(LogicOp::Not),
                AstNodeType::Current,
                AstNodeType::Select,
                AstNodeType::Field(Field::Identifier("count".to_owned())),
                AstNodeType::Integer(2),
                AstNodeType::RelExpr(RelOp::Less),
                AstNodeType::LogicExpr(LogicOp::And),
                AstNodeType::LogicExpr(LogicOp::Or),
                AstNodeType::RelExpr(RelOp::NotEqual),
            ])
        );
    }
}

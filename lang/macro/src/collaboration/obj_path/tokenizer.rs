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

use super::Span;

const CH_DOLLAR: char = '$';
const CH_DOT: char = '.';
const CH_AT: char = '@';
const CH_LPAREN: char = '(';
const CH_RPAREN: char = ')';
const CH_LBRACKET: char = '[';
const CH_RBRACKET: char = ']';
const CH_QUESTION: char = '?';
const CH_COMMA: char = ',';
const CH_EQUAL: char = '=';
const CH_LANGLE: char = '<';
const CH_RANGLE: char = '>';
const CH_AMPERSAND: char = '&';
const CH_PIPE: char = '|';
const CH_MINUS: char = '-';
const CH_EXCLAMATION: char = '!';
const CH_SEMICOLON: char = ';';

#[derive(Debug)]
pub enum Token {
    Root(Span),
    Dot(Span),
    At(Span),
    LParen(Span),
    RParen(Span),
    LBracket(Span),
    RBracket(Span),
    Comma(Span),
    Equal(Span),
    NotEqual(Span),
    Less(Span),
    LessEqual(Span),
    Greater(Span),
    GreaterEqual(Span),
    And(Span),
    Or(Span),
    DotDot(Span),
    Number(Span, u32),
    Bool(Span, bool),
    Identifier(Span, String),
    Neg(Span),
    Not(Span),
    PredicateStart(Span),
    Semicolon(Span),
    End(Span),
}

impl Token {
    pub fn get_span(&self) -> Span {
        let span = match self {
            Self::Root(span) => span,
            Self::Dot(span) => span,
            Self::At(span) => span,
            Self::LParen(span) => span,
            Self::RParen(span) => span,
            Self::LBracket(span) => span,
            Self::RBracket(span) => span,
            Self::Comma(span) => span,
            Self::Equal(span) => span,
            Self::NotEqual(span) => span,
            Self::Less(span) => span,
            Self::LessEqual(span) => span,
            Self::Greater(span) => span,
            Self::GreaterEqual(span) => span,
            Self::And(span) => span,
            Self::Or(span) => span,
            Self::DotDot(span) => span,
            Self::Number(span, _) => span,
            Self::Bool(span, _) => span,
            Self::Identifier(span, _) => span,
            Self::Neg(span) => span,
            Self::Not(span) => span,
            Self::PredicateStart(span) => span,
            Self::Semicolon(span) => span,
            Self::End(span) => span,
        };
        span.clone()
    }

    pub fn get_str(&self) -> String {
        match self {
            Self::Root(_) => "$".to_owned(),
            Self::Dot(_) => ".".to_owned(),
            Self::At(_) => "@".to_owned(),
            Self::LParen(_) => "(".to_owned(),
            Self::RParen(_) => ")".to_owned(),
            Self::LBracket(_) => "[".to_owned(),
            Self::RBracket(_) => "]".to_owned(),
            Self::Comma(_) => ",".to_owned(),
            Self::Equal(_) => "==".to_owned(),
            Self::NotEqual(_) => "!=".to_owned(),
            Self::Less(_) => "<".to_owned(),
            Self::LessEqual(_) => "<=".to_owned(),
            Self::Greater(_) => ">".to_owned(),
            Self::GreaterEqual(_) => ">=".to_owned(),
            Self::And(_) => "&&".to_owned(),
            Self::Or(_) => "||".to_owned(),
            Self::DotDot(_) => "..".to_owned(),
            Self::Number(_, number) => number.to_string(),
            Self::Bool(_, bool) => bool.to_string(),
            Self::Identifier(_, identifier) => identifier.clone(),
            Self::Neg(_) => "-".to_owned(),
            Self::Not(_) => "!".to_owned(),
            Self::PredicateStart(_) => "(?".to_owned(),
            Self::Semicolon(_) => ";".to_owned(),
            Self::End(_) => "END".to_owned(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TokenizeError {
    UnexpectedEnd(Span),
    UnexpectedCharacter {
        expected: char,
        provided: char,
        pos: usize,
    },
    UnknownCharacter {
        provided: char,
        pos: usize,
    },
    IllegalNumber {
        provided: String,
        pos: usize,
    },
}

pub struct Tokenizer<'a> {
    input: &'a [u8],
    cur_pos: usize,
}

pub type TokenizeResult<T> = Result<T, TokenizeError>;

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        assert!(input.is_ascii());

        Self {
            input: input.as_bytes(),
            cur_pos: 0,
        }
    }

    pub fn next_token(&mut self) -> TokenizeResult<Token> {
        // Swallow all whitespace.
        while self.cur_pos < self.input.len()
            && (self.input[self.cur_pos] as char).is_ascii_whitespace()
        {
            self.cur_pos += 1;
        }

        if self.cur_pos >= self.input.len() {
            return Ok(Token::End(Span::new(self.input.len(), 0)));
        }

        let next_ch = self.next_char()?;
        let cur_pos = self.cur_pos - 1;
        let span = Span::new(cur_pos, 1);
        match next_ch {
            CH_DOLLAR => Ok(Token::Root(span)),
            CH_DOT => self.dot(cur_pos),
            CH_AT => Ok(Token::At(span)),
            CH_LPAREN => self.lparen(cur_pos),
            CH_RPAREN => Ok(Token::RParen(span)),
            CH_LBRACKET => Ok(Token::LBracket(span)),
            CH_RBRACKET => Ok(Token::RBracket(span)),
            CH_COMMA => Ok(Token::Comma(span)),
            CH_EQUAL => self.equal(cur_pos),
            CH_LANGLE => self.less(cur_pos),
            CH_RANGLE => self.greater(cur_pos),
            CH_AMPERSAND => self.and(cur_pos),
            CH_PIPE => self.or(cur_pos),
            CH_SEMICOLON => Ok(Token::Semicolon(span)),
            CH_MINUS => Ok(Token::Neg(span)),
            CH_EXCLAMATION => self.not(cur_pos),
            _ if next_ch.is_ascii_alphanumeric() || next_ch == '_' => {
                self.ident_num(next_ch, cur_pos)
            }
            _ => Err(TokenizeError::UnknownCharacter {
                provided: next_ch,
                pos: cur_pos,
            }),
        }
    }

    fn lparen(&mut self, cur_pos: usize) -> TokenizeResult<Token> {
        let next_ch = self.peek_char()?;
        match next_ch {
            CH_QUESTION => {
                self.eat_char();
                Ok(Token::PredicateStart(Span::new(cur_pos, 2)))
            }
            _ => Ok(Token::LParen(Span::new(cur_pos, 1))),
        }
    }

    fn dot(&mut self, cur_pos: usize) -> TokenizeResult<Token> {
        let next_ch = self.peek_char()?;
        match next_ch {
            CH_DOT => {
                self.eat_char();
                Ok(Token::DotDot(Span::new(cur_pos, 2)))
            }
            _ => Ok(Token::Dot(Span::new(cur_pos, 1))),
        }
    }

    fn equal(&mut self, cur_pos: usize) -> TokenizeResult<Token> {
        let next_ch = self.peek_char()?;
        match next_ch {
            CH_EQUAL => {
                self.eat_char();
                Ok(Token::Equal(Span::new(cur_pos, 2)))
            }
            _ => Err(TokenizeError::UnexpectedCharacter {
                expected: CH_EQUAL,
                provided: next_ch,
                pos: cur_pos + 1,
            }),
        }
    }

    fn less(&mut self, cur_pos: usize) -> TokenizeResult<Token> {
        let next_ch = self.peek_char()?;
        match next_ch {
            CH_EQUAL => {
                self.eat_char();
                Ok(Token::LessEqual(Span::new(cur_pos, 2)))
            }
            _ => Ok(Token::Less(Span::new(cur_pos, 1))),
        }
    }

    fn greater(&mut self, cur_pos: usize) -> TokenizeResult<Token> {
        let next_ch = self.peek_char()?;
        match next_ch {
            CH_EQUAL => {
                self.eat_char();
                Ok(Token::GreaterEqual(Span::new(cur_pos, 2)))
            }
            _ => Ok(Token::Greater(Span::new(cur_pos, 1))),
        }
    }

    fn and(&mut self, cur_pos: usize) -> TokenizeResult<Token> {
        let next_ch = self.peek_char()?;
        match next_ch {
            CH_AMPERSAND => {
                self.eat_char();
                Ok(Token::And(Span::new(cur_pos, 2)))
            }
            _ => Err(TokenizeError::UnexpectedCharacter {
                expected: CH_AMPERSAND,
                provided: next_ch,
                pos: cur_pos + 1,
            }),
        }
    }

    fn or(&mut self, cur_pos: usize) -> TokenizeResult<Token> {
        let next_ch = self.peek_char()?;
        match next_ch {
            CH_PIPE => {
                self.eat_char();
                Ok(Token::Or(Span::new(cur_pos, 2)))
            }
            _ => Err(TokenizeError::UnexpectedCharacter {
                expected: CH_PIPE,
                provided: next_ch,
                pos: cur_pos + 1,
            }),
        }
    }

    fn not(&mut self, cur_pos: usize) -> TokenizeResult<Token> {
        let next_ch = self.peek_char()?;
        match next_ch {
            CH_EQUAL => {
                self.eat_char();
                Ok(Token::NotEqual(Span::new(cur_pos, 2)))
            }
            _ => Ok(Token::Not(Span::new(cur_pos, 1))),
        }
    }

    fn ident_num(&mut self, ch: char, cur_pos: usize) -> TokenizeResult<Token> {
        let mut item = String::new();
        item.push(ch);

        loop {
            let next_ch = self.peek_char();
            match next_ch {
                Ok(next_ch) => {
                    if next_ch.is_ascii_alphanumeric() || next_ch == '_' {
                        item.push(next_ch);
                        self.eat_char();
                    } else {
                        break;
                    }
                }
                Err(TokenizeError::UnexpectedEnd(_)) => break,
                _ => {
                    return Err(next_ch.unwrap_err());
                }
            }
        }

        if ch.is_ascii_digit() {
            if let Ok(num) = item.parse::<u32>() {
                return Ok(Token::Number(Span::new(cur_pos, item.len()), num));
            } else {
                return Err(TokenizeError::IllegalNumber {
                    provided: item,
                    pos: cur_pos,
                });
            }
        }

        if item == "true" {
            return Ok(Token::Bool(Span::new(cur_pos, item.len()), true));
        }

        if item == "false" {
            return Ok(Token::Bool(Span::new(cur_pos, item.len()), false));
        }

        Ok(Token::Identifier(Span::new(cur_pos, item.len()), item))
    }

    fn next_char(&mut self) -> TokenizeResult<char> {
        let ch = self.peek_char()?;
        self.eat_char();
        Ok(ch)
    }

    fn peek_char(&self) -> TokenizeResult<char> {
        if self.cur_pos >= self.input.len() {
            return Err(TokenizeError::UnexpectedEnd(Span::new(0, self.input.len())));
        }

        let ch = self.input[self.cur_pos] as char;
        Ok(ch)
    }

    fn eat_char(&mut self) {
        debug_assert!(self.cur_pos < self.input.len());

        self.cur_pos += 1;
    }
}

pub struct TokenReader<'a> {
    input: &'a str,
    tokens: Vec<Token>,
    pub err: Option<TokenizeError>,
}

impl<'a> TokenReader<'a> {
    pub fn new(input: &'a str) -> Self {
        assert!(input.is_ascii());

        let mut tokenizer = Tokenizer::new(input);
        let mut tokens = Vec::new();
        loop {
            let next_token = tokenizer.next_token();
            match next_token {
                Ok(Token::End(span)) => {
                    tokens.push(Token::End(span));
                    tokens.reverse();
                    return Self {
                        input,
                        err: None,
                        tokens,
                    };
                }
                Ok(token) => {
                    tokens.push(token);
                }
                Err(err) => {
                    return Self {
                        input,
                        err: Some(err),
                        tokens,
                    };
                }
            }
        }
    }

    pub fn peek_token(&self) -> TokenizeResult<&Token> {
        match self.tokens.last() {
            Some(token) => Ok(token),
            _ => Err(TokenizeError::UnexpectedEnd(Span::new(0, self.input.len()))),
        }
    }

    pub fn next_token(&mut self) -> TokenizeResult<Token> {
        match self.tokens.pop() {
            Some(token) => Ok(token),
            _ => Err(TokenizeError::UnexpectedEnd(Span::new(0, self.input.len()))),
        }
    }
}

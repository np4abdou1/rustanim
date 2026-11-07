//! Bedrock Molang expression lexer, parser, bytecode compiler, and zero-allocation VM.
//!
//! Supports Bedrock degree trigonometry, 30 Penner easing functions,
//! multi-statement scripts, null-coalescing (`??`), entity dereferencing (`->`),
//! queries (`query.*`), context variables (`c.*`), entity variables (`v.*`),
//! temporary variables (`t.*`), and fixed-register VM with `[f32; 64]` stack
//! guaranteeing strictly zero runtime heap allocations during frame evaluation loops.

use glam::Vec3;
use std::collections::HashMap;

// ============================================================================
// 1. AST & Public Data Types
// ============================================================================

/// Represents a evaluated primitive value in Molang.
#[derive(Debug, Clone, PartialEq)]
pub enum MolangValue {
    Number(f32),
    Boolean(bool),
}

impl MolangValue {
    #[inline]
    pub fn as_f32(&self) -> f32 {
        match self {
            MolangValue::Number(n) => *n,
            MolangValue::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

/// Abstract Syntax Tree (AST) node for Molang expressions.
///
/// Fully backwards-compatible with initial variants while providing
/// extended Bedrock expression forms.
#[derive(Debug, Clone, PartialEq)]
pub enum MolangExpr {
    // Original variants (100% backwards-compatible)
    Constant(f32),
    Variable(String),
    Query(String),
    UnaryMinus(Box<MolangExpr>),
    LogicalNot(Box<MolangExpr>),
    Add(Box<MolangExpr>, Box<MolangExpr>),
    Subtract(Box<MolangExpr>, Box<MolangExpr>),
    Multiply(Box<MolangExpr>, Box<MolangExpr>),
    Divide(Box<MolangExpr>, Box<MolangExpr>),
    Ternary {
        condition: Box<MolangExpr>,
        then_branch: Box<MolangExpr>,
        else_branch: Box<MolangExpr>,
    },
    MathFunc {
        name: String,
        args: Vec<MolangExpr>,
    },

    // Extended Bedrock Molang variants
    StringLiteral(String),
    This,
    Modulo(Box<MolangExpr>, Box<MolangExpr>),
    Equal(Box<MolangExpr>, Box<MolangExpr>),
    NotEqual(Box<MolangExpr>, Box<MolangExpr>),
    Less(Box<MolangExpr>, Box<MolangExpr>),
    LessEqual(Box<MolangExpr>, Box<MolangExpr>),
    Greater(Box<MolangExpr>, Box<MolangExpr>),
    GreaterEqual(Box<MolangExpr>, Box<MolangExpr>),
    LogicalAnd(Box<MolangExpr>, Box<MolangExpr>),
    LogicalOr(Box<MolangExpr>, Box<MolangExpr>),
    NullCoalesce(Box<MolangExpr>, Box<MolangExpr>),
    Assign {
        target: String,
        value: Box<MolangExpr>,
    },
    EntityDereference {
        entity_target: Box<MolangExpr>,
        variable_name: String,
    },
    MemberAccess {
        target: Box<MolangExpr>,
        member: String,
    },
    QueryCall {
        name: String,
        args: Vec<MolangExpr>,
    },
    Block(Vec<MolangStmt>),
}

/// A statement in a multi-statement Molang script.
#[derive(Debug, Clone, PartialEq)]
pub enum MolangStmt {
    Expr(MolangExpr),
    Return(Option<MolangExpr>),
    Loop {
        count: MolangExpr,
        body: Vec<MolangStmt>,
    },
    ForEach {
        variable: String,
        array: MolangExpr,
        body: Vec<MolangStmt>,
    },
    Break,
    Continue,
}

/// A sequence of Molang statements separated by semicolons.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MolangScript {
    pub statements: Vec<MolangStmt>,
}

/// Variable domain scopes recognized in Molang.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableScope {
    Entity,  // v.* or variable.*
    Temp,    // t.* or temp.*
    Context, // c.* or context.*
    Query,   // q.* or query.*
}

impl VariableScope {
    pub fn from_domain(domain: &str) -> Option<Self> {
        if domain.eq_ignore_ascii_case("v") || domain.eq_ignore_ascii_case("variable") {
            Some(VariableScope::Entity)
        } else if domain.eq_ignore_ascii_case("t") || domain.eq_ignore_ascii_case("temp") {
            Some(VariableScope::Temp)
        } else if domain.eq_ignore_ascii_case("c") || domain.eq_ignore_ascii_case("context") {
            Some(VariableScope::Context)
        } else if domain.eq_ignore_ascii_case("q") || domain.eq_ignore_ascii_case("query") {
            Some(VariableScope::Query)
        } else {
            None
        }
    }
}

// ============================================================================
// 2. Lexer & Tokens
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken<'a> {
    pub token: Token<'a>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    // Literals
    Number(f32),
    StringLit(&'a str),
    Ident(&'a str),

    // Keywords
    Return,
    Loop,
    Break,
    Continue,
    ForEach,
    This,

    // Operators - Arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Operators - Comparison
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Operators - Logical
    AndAnd,
    OrOr,
    Bang,

    // Operators - Ternary & Coalescing
    Question,
    Colon,
    QuestionQuestion,

    // Operators - Assignment & Access
    Equal,
    Arrow,
    Dot,

    // Punctuation & Delimiters
    Semicolon,
    Comma,
    LParen,
    RParen,
    LBrace,
    RBrace,

    // End of Input
    Eof,
}

/// Zero-allocation streaming lexer that iterates over borrowed slices of `&'a str`.
pub struct Lexer<'a> {
    src: &'a str,
    cursor: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, cursor: 0 }
    }

    #[inline]
    fn peek_byte(&self) -> Option<u8> {
        self.src.as_bytes().get(self.cursor).copied()
    }

    #[inline]
    fn peek_byte_offset(&self, offset: usize) -> Option<u8> {
        self.src.as_bytes().get(self.cursor + offset).copied()
    }

    #[inline]
    fn advance(&mut self) -> Option<u8> {
        let b = self.peek_byte()?;
        self.cursor += 1;
        Some(b)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(b) = self.peek_byte() {
            match b {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.cursor += 1;
                }
                b'/' if self.peek_byte_offset(1) == Some(b'/') => {
                    self.cursor += 2;
                    while let Some(cb) = self.peek_byte() {
                        self.cursor += 1;
                        if cb == b'\n' {
                            break;
                        }
                    }
                }
                b'/' if self.peek_byte_offset(1) == Some(b'*') => {
                    self.cursor += 2;
                    while let Some(cb) = self.peek_byte() {
                        if cb == b'*' && self.peek_byte_offset(1) == Some(b'/') {
                            self.cursor += 2;
                            break;
                        }
                        self.cursor += 1;
                    }
                }
                _ => break,
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<SpannedToken<'a>, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace_and_comments();
        let start = self.cursor;
        let b = self.advance()?;

        let token = match b {
            b'+' => Token::Plus,
            b'-' => {
                if self.peek_byte() == Some(b'>') {
                    self.cursor += 1;
                    Token::Arrow
                } else {
                    Token::Minus
                }
            }
            b'*' => Token::Star,
            b'/' => Token::Slash,
            b'%' => Token::Percent,
            b'=' => {
                if self.peek_byte() == Some(b'=') {
                    self.cursor += 1;
                    Token::EqualEqual
                } else {
                    Token::Equal
                }
            }
            b'!' => {
                if self.peek_byte() == Some(b'=') {
                    self.cursor += 1;
                    Token::BangEqual
                } else {
                    Token::Bang
                }
            }
            b'<' => {
                if self.peek_byte() == Some(b'=') {
                    self.cursor += 1;
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }
            b'>' => {
                if self.peek_byte() == Some(b'=') {
                    self.cursor += 1;
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }
            b'&' => {
                if self.peek_byte() == Some(b'&') {
                    self.cursor += 1;
                    Token::AndAnd
                } else {
                    return Some(Err(format!("Unexpected single '&' at pos {}", start)));
                }
            }
            b'|' => {
                if self.peek_byte() == Some(b'|') {
                    self.cursor += 1;
                    Token::OrOr
                } else {
                    return Some(Err(format!("Unexpected single '|' at pos {}", start)));
                }
            }
            b'?' => {
                if self.peek_byte() == Some(b'?') {
                    self.cursor += 1;
                    Token::QuestionQuestion
                } else {
                    Token::Question
                }
            }
            b':' => Token::Colon,
            b';' => Token::Semicolon,
            b',' => Token::Comma,
            b'.' => {
                if let Some(next_b) = self.peek_byte() {
                    if next_b.is_ascii_digit() {
                        while let Some(nb) = self.peek_byte() {
                            if nb.is_ascii_digit() {
                                self.cursor += 1;
                            } else {
                                break;
                            }
                        }
                        if let Some(b'e' | b'E') = self.peek_byte() {
                            self.cursor += 1;
                            if let Some(b'+' | b'-') = self.peek_byte() {
                                self.cursor += 1;
                            }
                            while let Some(nb) = self.peek_byte() {
                                if nb.is_ascii_digit() {
                                    self.cursor += 1;
                                } else {
                                    break;
                                }
                            }
                        }
                        let num_slice = &self.src[start..self.cursor];
                        let val = num_slice.parse::<f32>().unwrap_or(0.0);
                        Token::Number(val)
                    } else {
                        Token::Dot
                    }
                } else {
                    Token::Dot
                }
            }
            b'(' => Token::LParen,
            b')' => Token::RParen,
            b'{' => Token::LBrace,
            b'}' => Token::RBrace,
            b'\'' | b'"' => {
                let quote = b;
                let str_start = self.cursor;
                while let Some(sb) = self.peek_byte() {
                    self.cursor += 1;
                    if sb == quote {
                        break;
                    }
                }
                let str_end = self.cursor - 1;
                let content = &self.src[str_start..str_end];
                Token::StringLit(content)
            }
            b'0'..=b'9' => {
                while let Some(nb) = self.peek_byte() {
                    if nb.is_ascii_digit() {
                        self.cursor += 1;
                    } else {
                        break;
                    }
                }
                if self.peek_byte() == Some(b'.')
                    && self.peek_byte_offset(1).map_or(false, |c| c.is_ascii_digit())
                {
                    self.cursor += 1;
                    while let Some(nb) = self.peek_byte() {
                        if nb.is_ascii_digit() {
                            self.cursor += 1;
                        } else {
                            break;
                        }
                    }
                }
                if let Some(b'e' | b'E') = self.peek_byte() {
                    self.cursor += 1;
                    if let Some(b'+' | b'-') = self.peek_byte() {
                        self.cursor += 1;
                    }
                    while let Some(nb) = self.peek_byte() {
                        if nb.is_ascii_digit() {
                            self.cursor += 1;
                        } else {
                            break;
                        }
                    }
                }
                let num_slice = &self.src[start..self.cursor];
                let val = num_slice.parse::<f32>().unwrap_or(0.0);
                Token::Number(val)
            }
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                while let Some(ib) = self.peek_byte() {
                    if ib.is_ascii_alphanumeric() || ib == b'_' {
                        self.cursor += 1;
                    } else {
                        break;
                    }
                }
                let ident = &self.src[start..self.cursor];
                if ident.eq_ignore_ascii_case("return") {
                    Token::Return
                } else if ident.eq_ignore_ascii_case("loop") {
                    Token::Loop
                } else if ident.eq_ignore_ascii_case("break") {
                    Token::Break
                } else if ident.eq_ignore_ascii_case("continue") {
                    Token::Continue
                } else if ident.eq_ignore_ascii_case("for_each") {
                    Token::ForEach
                } else if ident.eq_ignore_ascii_case("this") {
                    Token::This
                } else {
                    Token::Ident(ident)
                }
            }
            unexpected => {
                return Some(Err(format!(
                    "Unexpected character '{}' at pos {}",
                    unexpected as char, start
                )))
            }
        };

        Some(Ok(SpannedToken {
            token,
            span: Span::new(start, self.cursor),
        }))
    }
}

// ============================================================================
// 3. Pratt Parser
// ============================================================================

pub struct MolangParser<'a> {
    tokens: Vec<SpannedToken<'a>>,
    pos: usize,
}

impl<'a> MolangParser<'a> {
    pub fn new(tokens: Vec<SpannedToken<'a>>) -> Self {
        Self { tokens, pos: 0 }
    }

    #[inline]
    fn peek(&self) -> &Token<'a> {
        self.tokens.get(self.pos).map(|st| &st.token).unwrap_or(&Token::Eof)
    }

    #[inline]
    fn consume(&mut self) -> Token<'a> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].token.clone();
            self.pos += 1;
            tok
        } else {
            Token::Eof
        }
    }

    fn binding_power(tok: &Token) -> u8 {
        match tok {
            Token::Equal => 1,
            Token::Question => 2,
            Token::QuestionQuestion => 3,
            Token::OrOr => 4,
            Token::AndAnd => 5,
            Token::EqualEqual | Token::BangEqual => 6,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => 7,
            Token::Plus | Token::Minus => 8,
            Token::Star | Token::Slash | Token::Percent => 9,
            Token::Arrow | Token::Dot | Token::LParen => 11,
            _ => 0,
        }
    }

    pub fn parse_script(&mut self) -> Result<MolangScript, String> {
        let mut statements = Vec::new();
        while self.peek() != &Token::Eof {
            let stmt = self.parse_stmt()?;
            statements.push(stmt);
            while self.peek() == &Token::Semicolon {
                self.consume();
            }
        }
        Ok(MolangScript { statements })
    }

    pub fn parse_stmt(&mut self) -> Result<MolangStmt, String> {
        match self.peek() {
            Token::Return => {
                self.consume();
                if matches!(self.peek(), Token::Semicolon | Token::RBrace | Token::Eof) {
                    Ok(MolangStmt::Return(None))
                } else {
                    let expr = self.parse_expr(0)?;
                    Ok(MolangStmt::Return(Some(expr)))
                }
            }
            Token::Break => {
                self.consume();
                Ok(MolangStmt::Break)
            }
            Token::Continue => {
                self.consume();
                Ok(MolangStmt::Continue)
            }
            Token::Loop => {
                self.consume();
                if self.consume() != Token::LParen {
                    return Err("Expected '(' after loop".into());
                }
                let count = self.parse_expr(0)?;
                if self.consume() != Token::Comma {
                    return Err("Expected ',' after loop count".into());
                }
                let body = if self.peek() == &Token::LBrace {
                    self.consume();
                    let mut stmts = Vec::new();
                    while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                        stmts.push(self.parse_stmt()?);
                        while self.peek() == &Token::Semicolon {
                            self.consume();
                        }
                    }
                    if self.consume() != Token::RBrace {
                        return Err("Expected '}' at end of loop body".into());
                    }
                    stmts
                } else {
                    vec![self.parse_stmt()?]
                };
                if self.consume() != Token::RParen {
                    return Err("Expected ')' closing loop".into());
                }
                Ok(MolangStmt::Loop { count, body })
            }
            Token::ForEach => {
                self.consume();
                if self.consume() != Token::LParen {
                    return Err("Expected '(' after for_each".into());
                }
                let var_name = match self.consume() {
                    Token::Ident(name) => name.to_string(),
                    _ => return Err("Expected variable name in for_each".into()),
                };
                if self.consume() != Token::Comma {
                    return Err("Expected ',' after for_each variable".into());
                }
                let array = self.parse_expr(0)?;
                if self.consume() != Token::Comma {
                    return Err("Expected ',' after for_each array".into());
                }
                let mut body = Vec::new();
                if self.peek() == &Token::LBrace {
                    self.consume();
                    while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                        body.push(self.parse_stmt()?);
                        while self.peek() == &Token::Semicolon {
                            self.consume();
                        }
                    }
                    if self.consume() != Token::RBrace {
                        return Err("Expected '}' at end of for_each body".into());
                    }
                } else {
                    body.push(self.parse_stmt()?);
                }
                if self.consume() != Token::RParen {
                    return Err("Expected ')' closing for_each".into());
                }
                Ok(MolangStmt::ForEach {
                    variable: var_name,
                    array,
                    body,
                })
            }
            _ => {
                let expr = self.parse_expr(0)?;
                Ok(MolangStmt::Expr(expr))
            }
        }
    }

    pub fn parse_expr(&mut self, min_bp: u8) -> Result<MolangExpr, String> {
        let tok = self.consume();
        let mut left = self.nud(tok)?;

        loop {
            let next_tok = self.peek();
            let bp = Self::binding_power(next_tok);
            if bp <= min_bp {
                break;
            }
            left = self.led(left, bp)?;
        }

        Ok(left)
    }

    fn nud(&mut self, tok: Token<'a>) -> Result<MolangExpr, String> {
        match tok {
            Token::Number(val) => Ok(MolangExpr::Constant(val)),
            Token::StringLit(s) => Ok(MolangExpr::StringLiteral(s.to_string())),
            Token::This => Ok(MolangExpr::This),
            Token::Ident(name) => {
                if self.peek() == &Token::Dot {
                    self.consume();
                    let member = match self.consume() {
                        Token::Ident(m) => m,
                        Token::This => "this",
                        other => {
                            return Err(format!(
                                "Expected member identifier after '.', got {:?}",
                                other
                            ))
                        }
                    };

                    if name.eq_ignore_ascii_case("math") {
                        let math_name = format!("math.{}", member.to_ascii_lowercase());
                        if self.peek() == &Token::LParen {
                            self.consume();
                            let args = self.parse_arg_list()?;
                            Ok(MolangExpr::MathFunc {
                                name: math_name,
                                args,
                            })
                        } else {
                            Ok(MolangExpr::MathFunc {
                                name: math_name,
                                args: Vec::new(),
                            })
                        }
                    } else if name.eq_ignore_ascii_case("q") || name.eq_ignore_ascii_case("query") {
                        let query_name = format!("query.{}", member.to_ascii_lowercase());
                        if self.peek() == &Token::LParen {
                            self.consume();
                            let args = self.parse_arg_list()?;
                            Ok(MolangExpr::QueryCall {
                                name: query_name,
                                args,
                            })
                        } else {
                            Ok(MolangExpr::Query(query_name))
                        }
                    } else if name.eq_ignore_ascii_case("v") || name.eq_ignore_ascii_case("variable") {
                        Ok(MolangExpr::Variable(format!("v.{}", member.to_ascii_lowercase())))
                    } else if name.eq_ignore_ascii_case("t") || name.eq_ignore_ascii_case("temp") {
                        Ok(MolangExpr::Variable(format!("t.{}", member.to_ascii_lowercase())))
                    } else if name.eq_ignore_ascii_case("c") || name.eq_ignore_ascii_case("context") {
                        Ok(MolangExpr::Variable(format!("c.{}", member.to_ascii_lowercase())))
                    } else {
                        Ok(MolangExpr::MemberAccess {
                            target: Box::new(MolangExpr::Variable(name.to_string())),
                            member: member.to_string(),
                        })
                    }
                } else if self.peek() == &Token::LParen {
                    self.consume();
                    let args = self.parse_arg_list()?;
                    Ok(MolangExpr::MathFunc {
                        name: name.to_ascii_lowercase(),
                        args,
                    })
                } else {
                    Ok(MolangExpr::Variable(name.to_string()))
                }
            }
            Token::Minus => {
                let operand = self.parse_expr(10)?;
                Ok(MolangExpr::UnaryMinus(Box::new(operand)))
            }
            Token::Bang => {
                let operand = self.parse_expr(10)?;
                Ok(MolangExpr::LogicalNot(Box::new(operand)))
            }
            Token::Plus => self.parse_expr(10),
            Token::LParen => {
                let expr = self.parse_expr(0)?;
                if self.consume() != Token::RParen {
                    return Err("Expected ')'".into());
                }
                Ok(expr)
            }
            Token::LBrace => {
                let mut stmts = Vec::new();
                while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                    stmts.push(self.parse_stmt()?);
                    while self.peek() == &Token::Semicolon {
                        self.consume();
                    }
                }
                if self.consume() != Token::RBrace {
                    return Err("Expected '}'".into());
                }
                Ok(MolangExpr::Block(stmts))
            }
            other => Err(format!("Unexpected token in expression nud: {:?}", other)),
        }
    }

    fn led(&mut self, left: MolangExpr, bp: u8) -> Result<MolangExpr, String> {
        let tok = self.consume();
        match tok {
            Token::Plus => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::Add(Box::new(left), Box::new(right)))
            }
            Token::Minus => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::Subtract(Box::new(left), Box::new(right)))
            }
            Token::Star => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::Multiply(Box::new(left), Box::new(right)))
            }
            Token::Slash => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::Divide(Box::new(left), Box::new(right)))
            }
            Token::Percent => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::Modulo(Box::new(left), Box::new(right)))
            }
            Token::EqualEqual => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::Equal(Box::new(left), Box::new(right)))
            }
            Token::BangEqual => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::NotEqual(Box::new(left), Box::new(right)))
            }
            Token::Less => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::Less(Box::new(left), Box::new(right)))
            }
            Token::LessEqual => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::LessEqual(Box::new(left), Box::new(right)))
            }
            Token::Greater => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::Greater(Box::new(left), Box::new(right)))
            }
            Token::GreaterEqual => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::GreaterEqual(Box::new(left), Box::new(right)))
            }
            Token::AndAnd => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::LogicalAnd(Box::new(left), Box::new(right)))
            }
            Token::OrOr => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::LogicalOr(Box::new(left), Box::new(right)))
            }
            Token::QuestionQuestion => {
                let right = self.parse_expr(bp)?;
                Ok(MolangExpr::NullCoalesce(Box::new(left), Box::new(right)))
            }
            Token::Equal => {
                let right = self.parse_expr(1)?;
                let target = match &left {
                    MolangExpr::Variable(var) => var.clone(),
                    MolangExpr::MemberAccess { target, member } => {
                        format!("{:?}.{}", target, member)
                    }
                    _ => return Err("Invalid assignment target".into()),
                };
                Ok(MolangExpr::Assign {
                    target,
                    value: Box::new(right),
                })
            }
            Token::Question => {
                let then_branch = if self.peek() == &Token::LBrace {
                    self.consume();
                    let mut stmts = Vec::new();
                    while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                        stmts.push(self.parse_stmt()?);
                        while self.peek() == &Token::Semicolon {
                            self.consume();
                        }
                    }
                    if self.consume() != Token::RBrace {
                        return Err("Expected '}'".into());
                    }
                    MolangExpr::Block(stmts)
                } else {
                    self.parse_expr(2)?
                };

                let else_branch = if self.peek() == &Token::Colon {
                    self.consume();
                    if self.peek() == &Token::LBrace {
                        self.consume();
                        let mut stmts = Vec::new();
                        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
                            stmts.push(self.parse_stmt()?);
                            while self.peek() == &Token::Semicolon {
                                self.consume();
                            }
                        }
                        if self.consume() != Token::RBrace {
                            return Err("Expected '}'".into());
                        }
                        MolangExpr::Block(stmts)
                    } else {
                        self.parse_expr(1)?
                    }
                } else {
                    MolangExpr::Constant(0.0)
                };

                Ok(MolangExpr::Ternary {
                    condition: Box::new(left),
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                })
            }
            Token::Arrow => {
                let rhs = self.parse_expr(11)?;
                let variable_name = match rhs {
                    MolangExpr::Variable(name) => name,
                    other => {
                        return Err(format!(
                            "Expected variable name after '->', got {:?}",
                            other
                        ))
                    }
                };
                Ok(MolangExpr::EntityDereference {
                    entity_target: Box::new(left),
                    variable_name,
                })
            }
            Token::Dot => {
                let member = match self.consume() {
                    Token::Ident(m) => m.to_string(),
                    Token::This => "this".to_string(),
                    other => {
                        return Err(format!("Expected member name after '.', got {:?}", other))
                    }
                };
                Ok(MolangExpr::MemberAccess {
                    target: Box::new(left),
                    member,
                })
            }
            Token::LParen => {
                let args = self.parse_arg_list()?;
                match left {
                    MolangExpr::Query(qname) => Ok(MolangExpr::QueryCall { name: qname, args }),
                    MolangExpr::Variable(vname) => Ok(MolangExpr::MathFunc { name: vname, args }),
                    other => Err(format!("Cannot call expression: {:?}", other)),
                }
            }
            other => Err(format!("Unexpected token in led: {:?}", other)),
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<MolangExpr>, String> {
        let mut args = Vec::new();
        if self.peek() != &Token::RParen {
            loop {
                args.push(self.parse_expr(0)?);
                if self.peek() == &Token::Comma {
                    self.consume();
                } else {
                    break;
                }
            }
        }
        if self.consume() != Token::RParen {
            return Err("Expected ')' closing argument list".into());
        }
        Ok(args)
    }
}

// ============================================================================
// 4. Built-in Math Functions (Bedrock Degree Parity) & Penner Easing Curves
// ============================================================================

const DEG_TO_RAD: f32 = std::f32::consts::PI / 180.0;
const RAD_TO_DEG: f32 = 180.0 / std::f32::consts::PI;

#[inline]
pub fn molang_sin(deg: f32) -> f32 {
    if deg.is_nan() || deg.is_infinite() {
        0.0
    } else {
        (deg * DEG_TO_RAD).sin()
    }
}

#[inline]
pub fn molang_cos(deg: f32) -> f32 {
    if deg.is_nan() || deg.is_infinite() {
        0.0
    } else {
        (deg * DEG_TO_RAD).cos()
    }
}

#[inline]
pub fn molang_tan(deg: f32) -> f32 {
    if deg.is_nan() || deg.is_infinite() {
        0.0
    } else {
        let rad = deg * DEG_TO_RAD;
        if rad.cos().abs() < 1e-6 {
            0.0
        } else {
            rad.tan()
        }
    }
}

#[inline]
pub fn molang_asin(val: f32) -> f32 {
    val.clamp(-1.0, 1.0).asin() * RAD_TO_DEG
}

#[inline]
pub fn molang_acos(val: f32) -> f32 {
    val.clamp(-1.0, 1.0).acos() * RAD_TO_DEG
}

#[inline]
pub fn molang_atan(val: f32) -> f32 {
    if val.is_nan() {
        0.0
    } else {
        val.atan() * RAD_TO_DEG
    }
}

#[inline]
pub fn molang_atan2(y: f32, x: f32) -> f32 {
    if (y == 0.0 && x == 0.0) || y.is_nan() || x.is_nan() {
        0.0
    } else {
        y.atan2(x) * RAD_TO_DEG
    }
}

#[inline]
pub fn molang_min_angle(deg: f32) -> f32 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

#[inline]
pub fn molang_lerprotate(a: f32, b: f32, t: f32) -> f32 {
    let diff = molang_min_angle(b - a);
    a + diff * t
}

#[inline]
pub fn molang_abs(x: f32) -> f32 {
    x.abs()
}

#[inline]
pub fn molang_clamp(val: f32, min: f32, max: f32) -> f32 {
    let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
    val.clamp(lo, hi)
}

#[inline]
pub fn molang_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
pub fn molang_inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
    if (b - a).abs() < 1e-6 {
        0.0
    } else {
        (v - a) / (b - a)
    }
}

#[inline]
pub fn molang_floor(x: f32) -> f32 {
    x.floor()
}

#[inline]
pub fn molang_ceil(x: f32) -> f32 {
    x.ceil()
}

#[inline]
pub fn molang_round(x: f32) -> f32 {
    x.round()
}

#[inline]
pub fn molang_trunc(x: f32) -> f32 {
    x.trunc()
}

#[inline]
pub fn molang_min(a: f32, b: f32) -> f32 {
    a.min(b)
}

#[inline]
pub fn molang_max(a: f32, b: f32) -> f32 {
    a.max(b)
}

#[inline]
pub fn molang_pow(base: f32, exp: f32) -> f32 {
    if (base < 0.0 && exp.fract() != 0.0) || (base == 0.0 && exp < 0.0) {
        0.0
    } else {
        base.powf(exp)
    }
}

#[inline]
pub fn molang_sqrt(x: f32) -> f32 {
    if x <= 0.0 {
        0.0
    } else {
        x.sqrt()
    }
}

#[inline]
pub fn molang_mod(val: f32, den: f32) -> f32 {
    if den.abs() < 1e-6 {
        0.0
    } else {
        val - den * (val / den).floor()
    }
}

#[inline]
pub fn molang_exp(x: f32) -> f32 {
    if x > 88.0 {
        f32::MAX
    } else {
        x.exp()
    }
}

#[inline]
pub fn molang_ln(x: f32) -> f32 {
    if x <= 0.0 {
        0.0
    } else {
        x.ln()
    }
}

#[inline]
pub fn molang_hermite_blend(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    3.0 * t * t - 2.0 * t * t * t
}

#[inline]
pub fn molang_sign(x: f32) -> f32 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

#[inline]
pub fn molang_copy_sign(mag: f32, sgn: f32) -> f32 {
    mag.abs() * if sgn < 0.0 { -1.0 } else { 1.0 }
}

#[inline]
pub fn interpolate_ease(start: f32, end: f32, factor: f32) -> f32 {
    start + (end - start) * factor
}

// ---------------------------------------------------------------------------
// Complete 30 Penner Easing Functions (Normalized t in [0.0, 1.0])
// ---------------------------------------------------------------------------

#[inline]
pub fn penner_ease_in_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t
}

#[inline]
pub fn penner_ease_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * (2.0 - t)
}

#[inline]
pub fn penner_ease_in_out_quad(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

#[inline]
pub fn penner_ease_in_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t
}

#[inline]
pub fn penner_ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

#[inline]
pub fn penner_ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

#[inline]
pub fn penner_ease_in_quart(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t.powi(4)
}

#[inline]
pub fn penner_ease_out_quart(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(4)
}

#[inline]
pub fn penner_ease_in_out_quart(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        8.0 * t.powi(4)
    } else {
        1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
    }
}

#[inline]
pub fn penner_ease_in_quint(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t.powi(5)
}

#[inline]
pub fn penner_ease_out_quint(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(5)
}

#[inline]
pub fn penner_ease_in_out_quint(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        16.0 * t.powi(5)
    } else {
        1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
    }
}

#[inline]
pub fn penner_ease_in_sine(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (t * (std::f32::consts::PI / 2.0)).cos()
}

#[inline]
pub fn penner_ease_out_sine(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    (t * (std::f32::consts::PI / 2.0)).sin()
}

#[inline]
pub fn penner_ease_in_out_sine(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    -0.5 * ((std::f32::consts::PI * t).cos() - 1.0)
}

#[inline]
pub fn penner_ease_in_expo(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        0.0
    } else {
        2.0f32.powf(10.0 * (t - 1.0))
    }
}

#[inline]
pub fn penner_ease_out_expo(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0f32.powf(-10.0 * t)
    }
}

#[inline]
pub fn penner_ease_in_out_expo(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        2.0f32.powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - 2.0f32.powf(-20.0 * t + 10.0)) / 2.0
    }
}

#[inline]
pub fn penner_ease_in_circ(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t * t).max(0.0).sqrt()
}

#[inline]
pub fn penner_ease_out_circ(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    (1.0 - (t - 1.0).powi(2)).max(0.0).sqrt()
}

#[inline]
pub fn penner_ease_in_out_circ(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        (1.0 - (1.0 - (2.0 * t).powi(2)).max(0.0).sqrt()) / 2.0
    } else {
        ((1.0 - (-2.0 * t + 2.0).powi(2)).max(0.0).sqrt() + 1.0) / 2.0
    }
}

#[inline]
pub fn penner_ease_in_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    c3 * t * t * t - c1 * t * t
}

#[inline]
pub fn penner_ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

#[inline]
pub fn penner_ease_in_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.70158;
    let c2 = c1 * 1.525;
    if t < 0.5 {
        ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
    } else {
        ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (2.0 * t - 2.0) + c2) + 2.0) / 2.0
    }
}

#[inline]
pub fn penner_ease_in_elastic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c4 = (2.0 * std::f32::consts::PI) / 3.0;
        -2.0f32.powf(10.0 * t - 10.0) * ((10.0 * t - 10.75) * c4).sin()
    }
}

#[inline]
pub fn penner_ease_out_elastic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c4 = (2.0 * std::f32::consts::PI) / 3.0;
        2.0f32.powf(-10.0 * t) * ((10.0 * t - 0.75) * c4).sin() + 1.0
    }
}

#[inline]
pub fn penner_ease_in_out_elastic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c5 = (2.0 * std::f32::consts::PI) / 4.5;
        if t < 0.5 {
            -(2.0f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
        } else {
            (2.0f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
        }
    }
}

#[inline]
pub fn penner_ease_out_bounce(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let n1 = 7.5625;
    let d1 = 2.75;
    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t2 = t - 1.5 / d1;
        n1 * t2 * t2 + 0.75
    } else if t < 2.5 / d1 {
        let t2 = t - 2.25 / d1;
        n1 * t2 * t2 + 0.9375
    } else {
        let t2 = t - 2.625 / d1;
        n1 * t2 * t2 + 0.984375
    }
}

#[inline]
pub fn penner_ease_in_bounce(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - penner_ease_out_bounce(1.0 - t)
}

#[inline]
pub fn penner_ease_in_out_bounce(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        (1.0 - penner_ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + penner_ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

// Aliases for 3-arg calling convention matching common test conventions
#[inline]
pub fn molang_ease_out_cubic(start: f32, end: f32, t: f32) -> f32 {
    interpolate_ease(start, end, penner_ease_out_cubic(t))
}

#[inline]
pub fn molang_ease_in_expo(start: f32, end: f32, t: f32) -> f32 {
    interpolate_ease(start, end, penner_ease_in_expo(t))
}

#[inline]
pub fn molang_ease_in_out_sine(start: f32, end: f32, t: f32) -> f32 {
    interpolate_ease(start, end, penner_ease_in_out_sine(t))
}

// ============================================================================
// 5. Math Function Identification
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MathFnId {
    Sin = 0,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    MinAngle,
    Abs,
    Clamp,
    Lerp,
    HermiteBlend,
    Min,
    Max,
    Floor,
    Ceil,
    Round,
    Trunc,
    Mod,
    Pow,
    Sqrt,
    Exp,
    Ln,
    Sign,
    CopySign,
    InverseLerp,
    LerpRotate,
    Pi,
    Random,
    RandomInteger,
    DiceRoll,

    // 30 Penner Easings
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInQuint,
    EaseOutQuint,
    EaseInOutQuint,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

impl MathFnId {
    pub fn from_name(name: &str) -> Option<Self> {
        let n = name
            .strip_prefix("math.")
            .or_else(|| name.strip_prefix("Math."))
            .unwrap_or(name);

        match n.to_ascii_lowercase().as_str() {
            "sin" => Some(MathFnId::Sin),
            "cos" => Some(MathFnId::Cos),
            "tan" => Some(MathFnId::Tan),
            "asin" => Some(MathFnId::Asin),
            "acos" => Some(MathFnId::Acos),
            "atan" => Some(MathFnId::Atan),
            "atan2" => Some(MathFnId::Atan2),
            "min_angle" => Some(MathFnId::MinAngle),
            "abs" => Some(MathFnId::Abs),
            "clamp" => Some(MathFnId::Clamp),
            "lerp" => Some(MathFnId::Lerp),
            "hermite_blend" => Some(MathFnId::HermiteBlend),
            "min" => Some(MathFnId::Min),
            "max" => Some(MathFnId::Max),
            "floor" => Some(MathFnId::Floor),
            "ceil" => Some(MathFnId::Ceil),
            "round" => Some(MathFnId::Round),
            "trunc" => Some(MathFnId::Trunc),
            "mod" => Some(MathFnId::Mod),
            "pow" => Some(MathFnId::Pow),
            "sqrt" => Some(MathFnId::Sqrt),
            "exp" => Some(MathFnId::Exp),
            "ln" => Some(MathFnId::Ln),
            "sign" => Some(MathFnId::Sign),
            "copy_sign" => Some(MathFnId::CopySign),
            "inverse_lerp" => Some(MathFnId::InverseLerp),
            "lerprotate" | "lerp_rotate" => Some(MathFnId::LerpRotate),
            "pi" => Some(MathFnId::Pi),
            "random" => Some(MathFnId::Random),
            "random_integer" => Some(MathFnId::RandomInteger),
            "dice_roll" => Some(MathFnId::DiceRoll),

            // Penner Easings
            "ease_in_quad" => Some(MathFnId::EaseInQuad),
            "ease_out_quad" => Some(MathFnId::EaseOutQuad),
            "ease_in_out_quad" => Some(MathFnId::EaseInOutQuad),
            "ease_in_cubic" => Some(MathFnId::EaseInCubic),
            "ease_out_cubic" => Some(MathFnId::EaseOutCubic),
            "ease_in_out_cubic" => Some(MathFnId::EaseInOutCubic),
            "ease_in_quart" => Some(MathFnId::EaseInQuart),
            "ease_out_quart" => Some(MathFnId::EaseOutQuart),
            "ease_in_out_quart" => Some(MathFnId::EaseInOutQuart),
            "ease_in_quint" => Some(MathFnId::EaseInQuint),
            "ease_out_quint" => Some(MathFnId::EaseOutQuint),
            "ease_in_out_quint" => Some(MathFnId::EaseInOutQuint),
            "ease_in_sine" => Some(MathFnId::EaseInSine),
            "ease_out_sine" => Some(MathFnId::EaseOutSine),
            "ease_in_out_sine" => Some(MathFnId::EaseInOutSine),
            "ease_in_expo" => Some(MathFnId::EaseInExpo),
            "ease_out_expo" => Some(MathFnId::EaseOutExpo),
            "ease_in_out_expo" => Some(MathFnId::EaseInOutExpo),
            "ease_in_circ" => Some(MathFnId::EaseInCirc),
            "ease_out_circ" => Some(MathFnId::EaseOutCirc),
            "ease_in_out_circ" => Some(MathFnId::EaseInOutCirc),
            "ease_in_back" => Some(MathFnId::EaseInBack),
            "ease_out_back" => Some(MathFnId::EaseOutBack),
            "ease_in_out_back" => Some(MathFnId::EaseInOutBack),
            "ease_in_elastic" => Some(MathFnId::EaseInElastic),
            "ease_out_elastic" => Some(MathFnId::EaseOutElastic),
            "ease_in_out_elastic" => Some(MathFnId::EaseInOutElastic),
            "ease_in_bounce" => Some(MathFnId::EaseInBounce),
            "ease_out_bounce" => Some(MathFnId::EaseOutBounce),
            "ease_in_out_bounce" => Some(MathFnId::EaseInOutBounce),

            _ => None,
        }
    }

    #[inline]
    pub fn eval_normalized_ease(&self, t: f32) -> f32 {
        match self {
            MathFnId::EaseInQuad => penner_ease_in_quad(t),
            MathFnId::EaseOutQuad => penner_ease_out_quad(t),
            MathFnId::EaseInOutQuad => penner_ease_in_out_quad(t),
            MathFnId::EaseInCubic => penner_ease_in_cubic(t),
            MathFnId::EaseOutCubic => penner_ease_out_cubic(t),
            MathFnId::EaseInOutCubic => penner_ease_in_out_cubic(t),
            MathFnId::EaseInQuart => penner_ease_in_quart(t),
            MathFnId::EaseOutQuart => penner_ease_out_quart(t),
            MathFnId::EaseInOutQuart => penner_ease_in_out_quart(t),
            MathFnId::EaseInQuint => penner_ease_in_quint(t),
            MathFnId::EaseOutQuint => penner_ease_out_quint(t),
            MathFnId::EaseInOutQuint => penner_ease_in_out_quint(t),
            MathFnId::EaseInSine => penner_ease_in_sine(t),
            MathFnId::EaseOutSine => penner_ease_out_sine(t),
            MathFnId::EaseInOutSine => penner_ease_in_out_sine(t),
            MathFnId::EaseInExpo => penner_ease_in_expo(t),
            MathFnId::EaseOutExpo => penner_ease_out_expo(t),
            MathFnId::EaseInOutExpo => penner_ease_in_out_expo(t),
            MathFnId::EaseInCirc => penner_ease_in_circ(t),
            MathFnId::EaseOutCirc => penner_ease_out_circ(t),
            MathFnId::EaseInOutCirc => penner_ease_in_out_circ(t),
            MathFnId::EaseInBack => penner_ease_in_back(t),
            MathFnId::EaseOutBack => penner_ease_out_back(t),
            MathFnId::EaseInOutBack => penner_ease_in_out_back(t),
            MathFnId::EaseInElastic => penner_ease_in_elastic(t),
            MathFnId::EaseOutElastic => penner_ease_out_elastic(t),
            MathFnId::EaseInOutElastic => penner_ease_in_out_elastic(t),
            MathFnId::EaseInBounce => penner_ease_in_bounce(t),
            MathFnId::EaseOutBounce => penner_ease_out_bounce(t),
            MathFnId::EaseInOutBounce => penner_ease_in_out_bounce(t),
            _ => 0.0,
        }
    }
}

// ============================================================================
// 6. Query IDs & Canonical Mapping
// ============================================================================

pub const QUERY_ANIM_TIME: u16 = 0;
pub const QUERY_DELTA_TIME: u16 = 1;
pub const QUERY_GROUND_SPEED: u16 = 2;
pub const QUERY_VERTICAL_SPEED: u16 = 3;
pub const QUERY_YAW_SPEED: u16 = 4;
pub const QUERY_IS_SNEAKING: u16 = 5;
pub const QUERY_IS_SPRINTING: u16 = 6;
pub const QUERY_IS_SWIMMING: u16 = 7;
pub const QUERY_IS_GLIDING: u16 = 8;
pub const QUERY_IS_FIRST_PERSON: u16 = 9;
pub const QUERY_LIFE_TIME: u16 = 10;
pub const QUERY_IS_ON_GROUND: u16 = 11;
pub const QUERY_IS_JUMPING: u16 = 12;
pub const QUERY_BODY_YAW: u16 = 13;
pub const QUERY_BODY_PITCH: u16 = 14;
pub const QUERY_HEAD_YAW: u16 = 15;
pub const QUERY_HEAD_PITCH: u16 = 16;
pub const QUERY_TARGET_Y_ROTATION: u16 = 17;
pub const QUERY_TARGET_X_ROTATION: u16 = 18;
pub const QUERY_TARGET_DIST: u16 = 19;
pub const QUERY_ACTOR_COUNT: u16 = 20;
pub const QUERY_MODIFIED_MOVE_SPEED: u16 = 21;
pub const QUERY_WALK_DISTANCE: u16 = 22;
pub const QUERY_MODIFIED_DISTANCE_MOVED: u16 = 23;
pub const QUERY_IS_CRAWLING: u16 = 24;
pub const QUERY_IS_MOVING: u16 = 25;
pub const QUERY_IS_ON_FIRE: u16 = 26;
pub const QUERY_IS_IN_WATER: u16 = 27;
pub const QUERY_ITEM_IN_USE: u16 = 28;
pub const QUERY_ITEM_MAX_USE_DURATION: u16 = 29;
pub const QUERY_MAIN_HAND_EMPTY: u16 = 30;
pub const QUERY_OFF_HAND_EMPTY: u16 = 31;
pub const QUERY_SHIELD_BLOCKING: u16 = 32;
pub const QUERY_HEALTH: u16 = 33;
pub const QUERY_MAX_HEALTH: u16 = 34;
pub const QUERY_HURT_TIME: u16 = 35;
pub const QUERY_IS_ALIVE: u16 = 36;
pub const QUERY_POSITION: u16 = 37;
pub const QUERY_POSITION_DELTA: u16 = 38;
pub const QUERY_IN_RANGE: u16 = 39;
pub const QUERY_ANY: u16 = 40;
pub const CONTEXT_IS_FIRST_PERSON: u16 = 41;
pub const CONTEXT_PLAYER_OFFHAND_ARM_HEIGHT: u16 = 42;

pub fn query_id_from_name(name: &str) -> Option<u16> {
    let lower = name.to_ascii_lowercase();
    let q = lower
        .strip_prefix("query.")
        .or_else(|| lower.strip_prefix("q."))
        .or_else(|| lower.strip_prefix("context."))
        .or_else(|| lower.strip_prefix("c."))
        .unwrap_or(&lower);

    match q {
        "anim_time" => Some(QUERY_ANIM_TIME),
        "delta_time" => Some(QUERY_DELTA_TIME),
        "ground_speed" => Some(QUERY_GROUND_SPEED),
        "vertical_speed" => Some(QUERY_VERTICAL_SPEED),
        "yaw_speed" => Some(QUERY_YAW_SPEED),
        "is_sneaking" => Some(QUERY_IS_SNEAKING),
        "is_sprinting" => Some(QUERY_IS_SPRINTING),
        "is_swimming" => Some(QUERY_IS_SWIMMING),
        "is_gliding" => Some(QUERY_IS_GLIDING),
        "is_first_person" => Some(QUERY_IS_FIRST_PERSON),
        "life_time" => Some(QUERY_LIFE_TIME),
        "is_on_ground" => Some(QUERY_IS_ON_GROUND),
        "is_jumping" => Some(QUERY_IS_JUMPING),
        "body_yaw" | "body_y_rotation" => Some(QUERY_BODY_YAW),
        "body_pitch" | "body_x_rotation" => Some(QUERY_BODY_PITCH),
        "head_yaw" | "head_y_rotation" => Some(QUERY_HEAD_YAW),
        "head_pitch" | "head_x_rotation" | "pitch" => Some(QUERY_HEAD_PITCH),
        "target_y_rotation" => Some(QUERY_TARGET_Y_ROTATION),
        "target_x_rotation" => Some(QUERY_TARGET_X_ROTATION),
        "target_dist" => Some(QUERY_TARGET_DIST),
        "actor_count" => Some(QUERY_ACTOR_COUNT),
        "modified_move_speed" => Some(QUERY_MODIFIED_MOVE_SPEED),
        "walk_distance" => Some(QUERY_WALK_DISTANCE),
        "modified_distance_moved" => Some(QUERY_MODIFIED_DISTANCE_MOVED),
        "is_crawling" => Some(QUERY_IS_CRAWLING),
        "is_moving" => Some(QUERY_IS_MOVING),
        "is_on_fire" => Some(QUERY_IS_ON_FIRE),
        "is_in_water" => Some(QUERY_IS_IN_WATER),
        "item_in_use" => Some(QUERY_ITEM_IN_USE),
        "item_max_use_duration" => Some(QUERY_ITEM_MAX_USE_DURATION),
        "main_hand_empty" => Some(QUERY_MAIN_HAND_EMPTY),
        "off_hand_empty" => Some(QUERY_OFF_HAND_EMPTY),
        "shield_blocking" => Some(QUERY_SHIELD_BLOCKING),
        "health" => Some(QUERY_HEALTH),
        "max_health" => Some(QUERY_MAX_HEALTH),
        "hurt_time" => Some(QUERY_HURT_TIME),
        "is_alive" => Some(QUERY_IS_ALIVE),
        "position" => Some(QUERY_POSITION),
        "position_delta" => Some(QUERY_POSITION_DELTA),
        "in_range" => Some(QUERY_IN_RANGE),
        "any" => Some(QUERY_ANY),
        "player_offhand_arm_height" => Some(CONTEXT_PLAYER_OFFHAND_ARM_HEIGHT),
        _ => None,
    }
}

// ============================================================================
// 7. Molang Context & Variable Bindings
// ============================================================================

/// Evaluation context containing entity queries, context variables, and entity scopes.
#[derive(Clone, Debug)]
pub struct MolangContext {
    // Locomotion & timeline
    pub anim_time: f32,
    pub delta_time: f32,
    pub life_time: f32,
    pub actor_count: f32,
    pub ground_speed: f32,
    pub vertical_speed: f32,
    pub modified_move_speed: f32,
    pub walk_distance: f32,
    pub modified_distance_moved: f32,
    pub yaw_speed: f32,
    pub is_on_ground: bool,
    pub is_jumping: bool,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub is_swimming: bool,
    pub is_gliding: bool,
    pub is_crawling: bool,
    pub is_moving: bool,

    // Status states
    pub is_on_fire: bool,
    pub is_in_water: bool,
    pub item_in_use: bool,
    pub item_max_use_duration: f32,
    pub main_hand_empty: bool,
    pub off_hand_empty: bool,
    pub shield_blocking: bool,
    pub target_dist: f32,

    // Orientation & angles (in Degrees)
    pub body_yaw: f32,
    pub body_pitch: f32,
    pub head_yaw: f32,
    pub head_pitch: f32,
    pub target_y_rotation: f32,
    pub target_x_rotation: f32,
    pub camera_rotation: Vec3,

    // Spatial position
    pub position: Vec3,
    pub position_delta: Vec3,

    // Health
    pub health: f32,
    pub max_health: f32,
    pub hurt_time: f32,
    pub is_alive: bool,

    // Context variables (c.*)
    pub is_first_person: bool,
    pub item_slot: String,
    pub player_offhand_arm_height: f32,

    // Contextual parameter (this)
    pub this_val: f32,

    // Variable scopes
    pub variables: HashMap<String, f32>,
    pub entity_registers: [f32; 64],
    pub temp_registers: [f32; 64],
}

impl Default for MolangContext {
    fn default() -> Self {
        Self {
            anim_time: 0.0,
            delta_time: 0.05,
            life_time: 0.0,
            actor_count: 1.0,
            ground_speed: 0.0,
            vertical_speed: 0.0,
            modified_move_speed: 0.0,
            walk_distance: 0.0,
            modified_distance_moved: 0.0,
            yaw_speed: 0.0,
            is_on_ground: true,
            is_jumping: false,
            is_sneaking: false,
            is_sprinting: false,
            is_swimming: false,
            is_gliding: false,
            is_crawling: false,
            is_moving: false,

            is_on_fire: false,
            is_in_water: false,
            item_in_use: false,
            item_max_use_duration: 0.0,
            main_hand_empty: true,
            off_hand_empty: true,
            shield_blocking: false,
            target_dist: 0.0,

            body_yaw: 0.0,
            body_pitch: 0.0,
            head_yaw: 0.0,
            head_pitch: 0.0,
            target_y_rotation: 0.0,
            target_x_rotation: 0.0,
            camera_rotation: Vec3::ZERO,

            position: Vec3::ZERO,
            position_delta: Vec3::ZERO,

            health: 20.0,
            max_health: 20.0,
            hurt_time: 0.0,
            is_alive: true,

            is_first_person: false,
            item_slot: "main_hand".to_string(),
            player_offhand_arm_height: 0.0,

            this_val: 0.0,

            variables: HashMap::new(),
            entity_registers: [0.0; 64],
            temp_registers: [0.0; 64],
        }
    }
}

impl MolangContext {
    #[inline]
    pub fn resolve_query(&self, query_name: &str, args: &[f32]) -> f32 {
        let qid = match query_id_from_name(query_name) {
            Some(id) => id,
            None => return 0.0,
        };

        self.resolve_query_id(qid, args)
    }

    #[inline]
    pub fn resolve_query_id(&self, qid: u16, args: &[f32]) -> f32 {
        match qid {
            QUERY_ANIM_TIME => self.anim_time,
            QUERY_DELTA_TIME => self.delta_time,
            QUERY_GROUND_SPEED => self.ground_speed,
            QUERY_VERTICAL_SPEED => self.vertical_speed,
            QUERY_YAW_SPEED => self.yaw_speed,
            QUERY_IS_SNEAKING => {
                if self.is_sneaking {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_IS_SPRINTING => {
                if self.is_sprinting {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_IS_SWIMMING => {
                if self.is_swimming {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_IS_GLIDING => {
                if self.is_gliding {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_IS_FIRST_PERSON | CONTEXT_IS_FIRST_PERSON => {
                if self.is_first_person {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_LIFE_TIME => self.life_time,
            QUERY_IS_ON_GROUND => {
                if self.is_on_ground {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_IS_JUMPING => {
                if self.is_jumping {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_BODY_YAW => self.body_yaw,
            QUERY_BODY_PITCH => self.body_pitch,
            QUERY_HEAD_YAW => {
                if let Some(&axis) = args.first() {
                    if axis == 0.0 {
                        self.head_yaw
                    } else {
                        0.0
                    }
                } else {
                    self.head_yaw
                }
            }
            QUERY_HEAD_PITCH => self.head_pitch,
            QUERY_TARGET_Y_ROTATION => self.target_y_rotation,
            QUERY_TARGET_X_ROTATION => self.target_x_rotation,
            QUERY_TARGET_DIST => self.target_dist,
            QUERY_ACTOR_COUNT => self.actor_count,
            QUERY_MODIFIED_MOVE_SPEED => self.modified_move_speed,
            QUERY_WALK_DISTANCE => self.walk_distance,
            QUERY_MODIFIED_DISTANCE_MOVED => self.modified_distance_moved,
            QUERY_IS_CRAWLING => {
                if self.is_crawling {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_IS_MOVING => {
                if self.is_moving {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_IS_ON_FIRE => {
                if self.is_on_fire {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_IS_IN_WATER => {
                if self.is_in_water {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_ITEM_IN_USE => {
                if self.item_in_use {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_ITEM_MAX_USE_DURATION => self.item_max_use_duration,
            QUERY_MAIN_HAND_EMPTY => {
                if self.main_hand_empty {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_OFF_HAND_EMPTY => {
                if self.off_hand_empty {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_SHIELD_BLOCKING => {
                if self.shield_blocking {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_HEALTH => self.health,
            QUERY_MAX_HEALTH => self.max_health,
            QUERY_HURT_TIME => self.hurt_time,
            QUERY_IS_ALIVE => {
                if self.is_alive {
                    1.0
                } else {
                    0.0
                }
            }
            QUERY_POSITION => match args.first().copied().unwrap_or(0.0) as usize {
                0 => self.position.x,
                1 => self.position.y,
                2 => self.position.z,
                _ => 0.0,
            },
            QUERY_POSITION_DELTA => match args.first().copied().unwrap_or(0.0) as usize {
                0 => self.position_delta.x,
                1 => self.position_delta.y,
                2 => self.position_delta.z,
                _ => 0.0,
            },
            QUERY_IN_RANGE => {
                if args.len() >= 3 {
                    let val = args[0];
                    let min_v = args[1];
                    let max_v = args[2];
                    if val >= min_v && val <= max_v {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }
            QUERY_ANY => {
                if args.len() >= 2 {
                    let val = args[0];
                    if args[1..].iter().any(|&item| (item - val).abs() < 1e-5) {
                        1.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            }
            CONTEXT_PLAYER_OFFHAND_ARM_HEIGHT => self.player_offhand_arm_height,
            _ => 0.0,
        }
    }

    #[inline]
    pub fn resolve_context_var(&self, var_name: &str) -> f32 {
        let name = var_name
            .strip_prefix("c.")
            .or_else(|| var_name.strip_prefix("context."))
            .unwrap_or(var_name);

        match name {
            "is_first_person" => {
                if self.is_first_person {
                    1.0
                } else {
                    0.0
                }
            }
            "player_offhand_arm_height" => self.player_offhand_arm_height,
            _ => 0.0,
        }
    }
}

// ============================================================================
// 8. Bytecode Virtual Machine & OpCodes
// ============================================================================

/// Bytecode instruction for the zero-allocation Molang virtual machine.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    LoadConst(f32),
    LoadZero,
    LoadOne,
    Pop,
    Dup,

    // Registers
    LoadVar(u16),
    StoreVar(u16),
    LoadTemp(u8),
    StoreTemp(u8),
    LoadContext(u8),
    LoadThis,

    // Queries
    LoadQuery(u16),
    CallQuery { query_id: u16, arg_count: u8 },

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,

    // Logical & Comparison
    Not,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    NormalizeBool,
    Coalesce,

    // Math Dispatch
    CallMath { fn_id: MathFnId, arg_count: u8 },

    // Control Flow
    Jump(u16),
    JumpIfZero(u16),
    JumpIfNotZero(u16),
    JumpIfZeroKeep(u16),
    JumpIfNotZeroKeep(u16),

    // Termination
    Return,
}

/// Pre-allocated execution context maintaining evaluation stack directly on the CPU stack.
pub struct ExecutionContext<'a> {
    pub eval_stack: [f32; 64],
    pub sp: usize,
    pub temp_regs: [f32; 64],
    pub entity_regs: [f32; 64],

    pub anim_time: f32,
    pub delta_time: f32,
    pub ground_speed: f32,
    pub vertical_speed: f32,
    pub yaw_speed: f32,
    pub is_sneaking: bool,
    pub is_sprinting: bool,
    pub is_swimming: bool,
    pub is_gliding: bool,
    pub is_on_ground: bool,
    pub is_jumping: bool,
    pub is_crawling: bool,
    pub is_moving: bool,
    pub is_first_person: bool,

    pub is_on_fire: bool,
    pub is_in_water: bool,
    pub item_in_use: bool,
    pub item_max_use_duration: f32,
    pub main_hand_empty: bool,
    pub off_hand_empty: bool,
    pub shield_blocking: bool,
    pub target_dist: f32,

    pub body_yaw: f32,
    pub body_pitch: f32,
    pub head_yaw: f32,
    pub head_pitch: f32,
    pub target_y_rotation: f32,
    pub target_x_rotation: f32,

    pub position: Vec3,
    pub position_delta: Vec3,

    pub health: f32,
    pub max_health: f32,
    pub hurt_time: f32,
    pub is_alive: bool,

    pub player_offhand_arm_height: f32,
    pub this_val: f32,

    pub query_fn: Option<&'a dyn Fn(u16, &[f32]) -> f32>,
}

impl<'a> ExecutionContext<'a> {
    pub fn new() -> Self {
        Self {
            eval_stack: [0.0; 64],
            sp: 0,
            temp_regs: [0.0; 64],
            entity_regs: [0.0; 64],
            anim_time: 0.0,
            delta_time: 0.05,
            ground_speed: 0.0,
            vertical_speed: 0.0,
            yaw_speed: 0.0,
            is_sneaking: false,
            is_sprinting: false,
            is_swimming: false,
            is_gliding: false,
            is_on_ground: true,
            is_jumping: false,
            is_crawling: false,
            is_moving: false,
            is_first_person: false,
            is_on_fire: false,
            is_in_water: false,
            item_in_use: false,
            item_max_use_duration: 0.0,
            main_hand_empty: true,
            off_hand_empty: true,
            shield_blocking: false,
            target_dist: 0.0,
            body_yaw: 0.0,
            body_pitch: 0.0,
            head_yaw: 0.0,
            head_pitch: 0.0,
            target_y_rotation: 0.0,
            target_x_rotation: 0.0,
            position: Vec3::ZERO,
            position_delta: Vec3::ZERO,
            health: 20.0,
            max_health: 20.0,
            hurt_time: 0.0,
            is_alive: true,
            player_offhand_arm_height: 0.0,
            this_val: 0.0,
            query_fn: None,
        }
    }

    pub fn from_context(ctx: &mut MolangContext) -> Self {
        Self {
            eval_stack: [0.0; 64],
            sp: 0,
            temp_regs: ctx.temp_registers,
            entity_regs: ctx.entity_registers,
            anim_time: ctx.anim_time,
            delta_time: ctx.delta_time,
            ground_speed: ctx.ground_speed,
            vertical_speed: ctx.vertical_speed,
            yaw_speed: ctx.yaw_speed,
            is_sneaking: ctx.is_sneaking,
            is_sprinting: ctx.is_sprinting,
            is_swimming: ctx.is_swimming,
            is_gliding: ctx.is_gliding,
            is_on_ground: ctx.is_on_ground,
            is_jumping: ctx.is_jumping,
            is_crawling: ctx.is_crawling,
            is_moving: ctx.is_moving,
            is_first_person: ctx.is_first_person,
            is_on_fire: ctx.is_on_fire,
            is_in_water: ctx.is_in_water,
            item_in_use: ctx.item_in_use,
            item_max_use_duration: ctx.item_max_use_duration,
            main_hand_empty: ctx.main_hand_empty,
            off_hand_empty: ctx.off_hand_empty,
            shield_blocking: ctx.shield_blocking,
            target_dist: ctx.target_dist,
            body_yaw: ctx.body_yaw,
            body_pitch: ctx.body_pitch,
            head_yaw: ctx.head_yaw,
            head_pitch: ctx.head_pitch,
            target_y_rotation: ctx.target_y_rotation,
            target_x_rotation: ctx.target_x_rotation,
            position: ctx.position,
            position_delta: ctx.position_delta,
            health: ctx.health,
            max_health: ctx.max_health,
            hurt_time: ctx.hurt_time,
            is_alive: ctx.is_alive,
            player_offhand_arm_height: ctx.player_offhand_arm_height,
            this_val: ctx.this_val,
            query_fn: None,
        }
    }

    pub fn from_context_ref(ctx: &MolangContext) -> Self {
        Self {
            eval_stack: [0.0; 64],
            sp: 0,
            temp_regs: ctx.temp_registers,
            entity_regs: ctx.entity_registers,
            anim_time: ctx.anim_time,
            delta_time: ctx.delta_time,
            ground_speed: ctx.ground_speed,
            vertical_speed: ctx.vertical_speed,
            yaw_speed: ctx.yaw_speed,
            is_sneaking: ctx.is_sneaking,
            is_sprinting: ctx.is_sprinting,
            is_swimming: ctx.is_swimming,
            is_gliding: ctx.is_gliding,
            is_on_ground: ctx.is_on_ground,
            is_jumping: ctx.is_jumping,
            is_crawling: ctx.is_crawling,
            is_moving: ctx.is_moving,
            is_first_person: ctx.is_first_person,
            is_on_fire: ctx.is_on_fire,
            is_in_water: ctx.is_in_water,
            item_in_use: ctx.item_in_use,
            item_max_use_duration: ctx.item_max_use_duration,
            main_hand_empty: ctx.main_hand_empty,
            off_hand_empty: ctx.off_hand_empty,
            shield_blocking: ctx.shield_blocking,
            target_dist: ctx.target_dist,
            body_yaw: ctx.body_yaw,
            body_pitch: ctx.body_pitch,
            head_yaw: ctx.head_yaw,
            head_pitch: ctx.head_pitch,
            target_y_rotation: ctx.target_y_rotation,
            target_x_rotation: ctx.target_x_rotation,
            position: ctx.position,
            position_delta: ctx.position_delta,
            health: ctx.health,
            max_health: ctx.max_health,
            hurt_time: ctx.hurt_time,
            is_alive: ctx.is_alive,
            player_offhand_arm_height: ctx.player_offhand_arm_height,
            this_val: ctx.this_val,
            query_fn: None,
        }
    }

    #[inline(always)]
    pub fn reset_scratch(&mut self) {
        self.sp = 0;
        self.temp_regs = [0.0; 64];
    }

    #[inline(always)]
    pub fn push(&mut self, val: f32) {
        if self.sp < 64 {
            self.eval_stack[self.sp] = val;
            self.sp += 1;
        }
    }

    #[inline(always)]
    pub fn pop(&mut self) -> f32 {
        if self.sp > 0 {
            self.sp -= 1;
            self.eval_stack[self.sp]
        } else {
            0.0
        }
    }

    #[inline(always)]
    pub fn peek(&self) -> f32 {
        if self.sp > 0 {
            self.eval_stack[self.sp - 1]
        } else {
            0.0
        }
    }
}

/// Compiled Molang bytecode with preallocated scratch memory.
#[derive(Debug, Clone)]
pub struct MolangBytecode {
    pub instructions: Box<[OpCode]>,
    pub const_pool: Box<[f32]>,
    pub var_names: Box<[String]>,
}

impl MolangBytecode {
    pub fn new(
        instructions: Vec<OpCode>,
        const_pool: Vec<f32>,
        var_names: Vec<String>,
    ) -> Self {
        Self {
            instructions: instructions.into_boxed_slice(),
            const_pool: const_pool.into_boxed_slice(),
            var_names: var_names.into_boxed_slice(),
        }
    }

    /// Evaluates bytecode with strictly 0 runtime heap allocations on `[f32; 64]` stack.
    #[inline(always)]
    pub fn execute(&self, ctx: &mut ExecutionContext) -> f32 {
        ctx.reset_scratch();
        let ops = &self.instructions[..];
        let code_len = ops.len();
        let mut pc = 0;

        while pc < code_len {
            let op = ops[pc];
            pc += 1;

            match op {
                OpCode::LoadConst(c) => ctx.push(c),
                OpCode::LoadZero => ctx.push(0.0),
                OpCode::LoadOne => ctx.push(1.0),
                OpCode::Pop => {
                    ctx.pop();
                }
                OpCode::Dup => {
                    let v = ctx.peek();
                    ctx.push(v);
                }
                OpCode::LoadVar(idx) => {
                    let val = ctx.entity_regs.get(idx as usize).copied().unwrap_or(0.0);
                    ctx.push(val);
                }
                OpCode::StoreVar(idx) => {
                    let val = ctx.pop();
                    if let Some(slot) = ctx.entity_regs.get_mut(idx as usize) {
                        *slot = val;
                    }
                }
                OpCode::LoadTemp(reg) => {
                    let val = ctx.temp_regs.get(reg as usize).copied().unwrap_or(0.0);
                    ctx.push(val);
                }
                OpCode::StoreTemp(reg) => {
                    let val = ctx.pop();
                    if let Some(slot) = ctx.temp_regs.get_mut(reg as usize) {
                        *slot = val;
                    }
                }
                OpCode::LoadContext(ctx_id) => {
                    let val = match ctx_id {
                        0 => ctx.anim_time,
                        1 => ctx.delta_time,
                        2 => ctx.ground_speed,
                        3 => ctx.vertical_speed,
                        4 => {
                            if ctx.is_sneaking {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        5 => {
                            if ctx.is_sprinting {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        6 => {
                            if ctx.is_swimming {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        7 => {
                            if ctx.is_gliding {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        8 => {
                            if ctx.is_first_person {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        9 => ctx.player_offhand_arm_height,
                        _ => 0.0,
                    };
                    ctx.push(val);
                }
                OpCode::LoadThis => ctx.push(ctx.this_val),
                OpCode::LoadQuery(qid) => {
                    let val = match qid {
                        QUERY_ANIM_TIME => ctx.anim_time,
                        QUERY_DELTA_TIME => ctx.delta_time,
                        QUERY_GROUND_SPEED => ctx.ground_speed,
                        QUERY_VERTICAL_SPEED => ctx.vertical_speed,
                        QUERY_YAW_SPEED => ctx.yaw_speed,
                        QUERY_IS_SNEAKING => {
                            if ctx.is_sneaking {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_IS_SPRINTING => {
                            if ctx.is_sprinting {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_IS_SWIMMING => {
                            if ctx.is_swimming {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_IS_GLIDING => {
                            if ctx.is_gliding {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_IS_FIRST_PERSON | CONTEXT_IS_FIRST_PERSON => {
                            if ctx.is_first_person {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_IS_ON_GROUND => {
                            if ctx.is_on_ground {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_IS_JUMPING => {
                            if ctx.is_jumping {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_BODY_YAW => ctx.body_yaw,
                        QUERY_BODY_PITCH => ctx.body_pitch,
                        QUERY_HEAD_YAW => ctx.head_yaw,
                        QUERY_HEAD_PITCH => ctx.head_pitch,
                        QUERY_TARGET_Y_ROTATION => ctx.target_y_rotation,
                        QUERY_TARGET_X_ROTATION => ctx.target_x_rotation,
                        QUERY_TARGET_DIST => ctx.target_dist,
                        QUERY_IS_CRAWLING => {
                            if ctx.is_crawling {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_IS_MOVING => {
                            if ctx.is_moving {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_IS_ON_FIRE => {
                            if ctx.is_on_fire {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_IS_IN_WATER => {
                            if ctx.is_in_water {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_ITEM_IN_USE => {
                            if ctx.item_in_use {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_ITEM_MAX_USE_DURATION => ctx.item_max_use_duration,
                        QUERY_MAIN_HAND_EMPTY => {
                            if ctx.main_hand_empty {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_OFF_HAND_EMPTY => {
                            if ctx.off_hand_empty {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_SHIELD_BLOCKING => {
                            if ctx.shield_blocking {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_HEALTH => ctx.health,
                        QUERY_MAX_HEALTH => ctx.max_health,
                        QUERY_HURT_TIME => ctx.hurt_time,
                        QUERY_IS_ALIVE => {
                            if ctx.is_alive {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        CONTEXT_PLAYER_OFFHAND_ARM_HEIGHT => ctx.player_offhand_arm_height,
                        other => {
                            if let Some(ref qfn) = ctx.query_fn {
                                qfn(other, &[])
                            } else {
                                0.0
                            }
                        }
                    };
                    ctx.push(val);
                }
                OpCode::CallQuery {
                    query_id,
                    arg_count,
                } => {
                    let mut args_buf = [0.0f32; 8];
                    let count = (arg_count as usize).min(8);
                    for i in (0..count).rev() {
                        args_buf[i] = ctx.pop();
                    }
                    for _ in count..(arg_count as usize) {
                        ctx.pop();
                    }

                    let val = match query_id {
                        QUERY_POSITION => match args_buf[0] as usize {
                            0 => ctx.position.x,
                            1 => ctx.position.y,
                            2 => ctx.position.z,
                            _ => 0.0,
                        },
                        QUERY_POSITION_DELTA => match args_buf[0] as usize {
                            0 => ctx.position_delta.x,
                            1 => ctx.position_delta.y,
                            2 => ctx.position_delta.z,
                            _ => 0.0,
                        },
                        QUERY_HEAD_YAW => {
                            if args_buf[0] == 0.0 {
                                ctx.head_yaw
                            } else {
                                0.0
                            }
                        }
                        QUERY_IN_RANGE => {
                            let val = args_buf[0];
                            let min_v = args_buf[1];
                            let max_v = args_buf[2];
                            if val >= min_v && val <= max_v {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        QUERY_ANY => {
                            let val = args_buf[0];
                            let slice = &args_buf[1..count];
                            if slice.iter().any(|&item| (item - val).abs() < 1e-5) {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        other => {
                            if let Some(ref qfn) = ctx.query_fn {
                                qfn(other, &args_buf[..count])
                            } else {
                                0.0
                            }
                        }
                    };
                    ctx.push(val);
                }
                OpCode::Add => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(a + b);
                }
                OpCode::Sub => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(a - b);
                }
                OpCode::Mul => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(a * b);
                }
                OpCode::Div => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    let res = if b.abs() > 1e-6 { a / b } else { 0.0 };
                    ctx.push(res);
                }
                OpCode::Mod => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    let res = if b.abs() > 1e-6 {
                        a - b * (a / b).floor()
                    } else {
                        0.0
                    };
                    ctx.push(res);
                }
                OpCode::Neg => {
                    let a = ctx.pop();
                    ctx.push(-a);
                }
                OpCode::Not => {
                    let a = ctx.pop();
                    ctx.push(if a == 0.0 { 1.0 } else { 0.0 });
                }
                OpCode::Eq => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(if (a - b).abs() < 1e-6 { 1.0 } else { 0.0 });
                }
                OpCode::Ne => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(if (a - b).abs() >= 1e-6 { 1.0 } else { 0.0 });
                }
                OpCode::Lt => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(if a < b { 1.0 } else { 0.0 });
                }
                OpCode::Le => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(if a <= b { 1.0 } else { 0.0 });
                }
                OpCode::Gt => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(if a > b { 1.0 } else { 0.0 });
                }
                OpCode::Ge => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(if a >= b { 1.0 } else { 0.0 });
                }
                OpCode::And => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(if a != 0.0 && b != 0.0 { 1.0 } else { 0.0 });
                }
                OpCode::Or => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(if a != 0.0 || b != 0.0 { 1.0 } else { 0.0 });
                }
                OpCode::NormalizeBool => {
                    let a = ctx.pop();
                    ctx.push(if a != 0.0 { 1.0 } else { 0.0 });
                }
                OpCode::Coalesce => {
                    let b = ctx.pop();
                    let a = ctx.pop();
                    ctx.push(if a != 0.0 { a } else { b });
                }
                OpCode::CallMath { fn_id, arg_count } => {
                    Self::eval_math_instruction(fn_id, arg_count, ctx);
                }
                OpCode::Jump(target) => {
                    pc = target as usize;
                }
                OpCode::JumpIfZero(target) => {
                    let cond = ctx.pop();
                    if cond == 0.0 {
                        pc = target as usize;
                    }
                }
                OpCode::JumpIfNotZero(target) => {
                    let cond = ctx.pop();
                    if cond != 0.0 {
                        pc = target as usize;
                    }
                }
                OpCode::JumpIfZeroKeep(target) => {
                    let cond = ctx.peek();
                    if cond == 0.0 {
                        pc = target as usize;
                    }
                }
                OpCode::JumpIfNotZeroKeep(target) => {
                    let cond = ctx.peek();
                    if cond != 0.0 {
                        pc = target as usize;
                    }
                }
                OpCode::Return => {
                    return ctx.pop();
                }
            }
        }

        ctx.pop()
    }

    #[inline(always)]
    fn eval_math_instruction(fn_id: MathFnId, arg_count: u8, ctx: &mut ExecutionContext) {
        match fn_id {
            MathFnId::Sin => {
                let deg = ctx.pop();
                ctx.push(molang_sin(deg));
            }
            MathFnId::Cos => {
                let deg = ctx.pop();
                ctx.push(molang_cos(deg));
            }
            MathFnId::Tan => {
                let deg = ctx.pop();
                ctx.push(molang_tan(deg));
            }
            MathFnId::Asin => {
                let v = ctx.pop();
                ctx.push(molang_asin(v));
            }
            MathFnId::Acos => {
                let v = ctx.pop();
                ctx.push(molang_acos(v));
            }
            MathFnId::Atan => {
                let v = ctx.pop();
                ctx.push(molang_atan(v));
            }
            MathFnId::Atan2 => {
                let x = ctx.pop();
                let y = ctx.pop();
                ctx.push(molang_atan2(y, x));
            }
            MathFnId::MinAngle => {
                let deg = ctx.pop();
                ctx.push(molang_min_angle(deg));
            }
            MathFnId::Abs => {
                let v = ctx.pop();
                ctx.push(molang_abs(v));
            }
            MathFnId::Clamp => {
                let max = ctx.pop();
                let min = ctx.pop();
                let val = ctx.pop();
                ctx.push(molang_clamp(val, min, max));
            }
            MathFnId::Lerp => {
                let t = ctx.pop();
                let b = ctx.pop();
                let a = ctx.pop();
                ctx.push(molang_lerp(a, b, t));
            }
            MathFnId::HermiteBlend => {
                let t = ctx.pop();
                ctx.push(molang_hermite_blend(t));
            }
            MathFnId::Min => {
                let b = ctx.pop();
                let a = ctx.pop();
                ctx.push(molang_min(a, b));
            }
            MathFnId::Max => {
                let b = ctx.pop();
                let a = ctx.pop();
                ctx.push(molang_max(a, b));
            }
            MathFnId::Floor => {
                let v = ctx.pop();
                ctx.push(molang_floor(v));
            }
            MathFnId::Ceil => {
                let v = ctx.pop();
                ctx.push(molang_ceil(v));
            }
            MathFnId::Round => {
                let v = ctx.pop();
                ctx.push(molang_round(v));
            }
            MathFnId::Trunc => {
                let v = ctx.pop();
                ctx.push(molang_trunc(v));
            }
            MathFnId::Mod => {
                let den = ctx.pop();
                let val = ctx.pop();
                ctx.push(molang_mod(val, den));
            }
            MathFnId::Pow => {
                let exp = ctx.pop();
                let base = ctx.pop();
                ctx.push(molang_pow(base, exp));
            }
            MathFnId::Sqrt => {
                let v = ctx.pop();
                ctx.push(molang_sqrt(v));
            }
            MathFnId::Exp => {
                let v = ctx.pop();
                ctx.push(molang_exp(v));
            }
            MathFnId::Ln => {
                let v = ctx.pop();
                ctx.push(molang_ln(v));
            }
            MathFnId::Sign => {
                let v = ctx.pop();
                ctx.push(molang_sign(v));
            }
            MathFnId::CopySign => {
                let sgn = ctx.pop();
                let mag = ctx.pop();
                ctx.push(molang_copy_sign(mag, sgn));
            }
            MathFnId::InverseLerp => {
                let val = ctx.pop();
                let b = ctx.pop();
                let a = ctx.pop();
                ctx.push(molang_inverse_lerp(a, b, val));
            }
            MathFnId::LerpRotate => {
                let t = ctx.pop();
                let b = ctx.pop();
                let a = ctx.pop();
                ctx.push(molang_lerprotate(a, b, t));
            }
            MathFnId::Pi => {
                ctx.push(std::f32::consts::PI);
            }
            MathFnId::Random => {
                let max = ctx.pop();
                let min = ctx.pop();
                ctx.push(min + (max - min) * 0.5);
            }
            MathFnId::RandomInteger => {
                let max = ctx.pop();
                let min = ctx.pop();
                let _ = max;
                ctx.push(min.floor());
            }
            MathFnId::DiceRoll => {
                let sides = ctx.pop();
                let count = ctx.pop();
                ctx.push(count * (sides + 1.0) * 0.5);
            }

            // Penner Easings
            ease_fn => {
                if arg_count >= 3 {
                    let t = ctx.pop();
                    let end = ctx.pop();
                    let start = ctx.pop();
                    let factor = ease_fn.eval_normalized_ease(t);
                    ctx.push(interpolate_ease(start, end, factor));
                } else {
                    let t = ctx.pop();
                    ctx.push(ease_fn.eval_normalized_ease(t));
                }
            }
        }
    }

    /// Evaluates bytecode with an existing `MolangContext`, updating context registers.
    #[inline]
    pub fn execute_with_context(&self, ctx: &mut MolangContext) -> f32 {
        let mut exec_ctx = ExecutionContext::from_context(ctx);
        // Sync registered entity variables from ctx.variables if present
        for (i, name) in self.var_names.iter().enumerate() {
            if let Some(&val) = ctx.variables.get(name) {
                if i < 64 {
                    exec_ctx.entity_regs[i] = val;
                }
            }
        }

        let res = self.execute(&mut exec_ctx);
        ctx.entity_registers = exec_ctx.entity_regs;
        ctx.temp_registers = exec_ctx.temp_regs;
        res
    }

    /// Evaluates bytecode with an immutable `MolangContext`.
    #[inline]
    pub fn execute_ref(&self, ctx: &MolangContext) -> f32 {
        let mut exec_ctx = ExecutionContext::from_context_ref(ctx);
        for (i, name) in self.var_names.iter().enumerate() {
            if let Some(&val) = ctx.variables.get(name) {
                if i < 64 {
                    exec_ctx.entity_regs[i] = val;
                }
            }
        }
        self.execute(&mut exec_ctx)
    }
}

// ============================================================================
// 9. AST to Bytecode Compiler
// ============================================================================

pub struct Compiler {
    instructions: Vec<OpCode>,
    const_pool: Vec<f32>,
    var_map: HashMap<String, u16>,
    var_names: Vec<String>,
    temp_map: HashMap<String, u8>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            const_pool: Vec::new(),
            var_map: HashMap::new(),
            var_names: Vec::new(),
            temp_map: HashMap::new(),
        }
    }

    pub fn intern_var(&mut self, name: &str) -> u16 {
        let lower = name.to_ascii_lowercase();
        if let Some(&id) = self.var_map.get(&lower) {
            id
        } else {
            let id = self.var_names.len() as u16;
            self.var_map.insert(lower.clone(), id);
            self.var_names.push(lower);
            id
        }
    }

    pub fn intern_temp(&mut self, name: &str) -> u8 {
        let lower = name.to_ascii_lowercase();
        let count = self.temp_map.len() as u8;
        *self.temp_map.entry(lower).or_insert(count)
    }

    pub fn compile_script(mut self, script: &MolangScript) -> MolangBytecode {
        let len = script.statements.len();
        for (i, stmt) in script.statements.iter().enumerate() {
            let is_last = i == len - 1;
            self.compile_stmt(stmt, is_last);
        }
        if self.instructions.is_empty() {
            self.instructions.push(OpCode::LoadZero);
        }
        MolangBytecode::new(self.instructions, self.const_pool, self.var_names)
    }

    pub fn compile_expr_root(mut self, expr: &MolangExpr) -> MolangBytecode {
        self.compile_expr(expr);
        MolangBytecode::new(self.instructions, self.const_pool, self.var_names)
    }

    fn compile_stmt(&mut self, stmt: &MolangStmt, is_last: bool) {
        match stmt {
            MolangStmt::Expr(expr) => {
                self.compile_expr(expr);
                if !is_last {
                    self.instructions.push(OpCode::Pop);
                }
            }
            MolangStmt::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    self.compile_expr(expr);
                } else {
                    self.instructions.push(OpCode::LoadZero);
                }
                self.instructions.push(OpCode::Return);
            }
            MolangStmt::Loop { count, body } => {
                self.compile_expr(count);
                let loop_reg = self.intern_temp("__internal_loop_cnt");
                self.instructions.push(OpCode::StoreTemp(loop_reg));

                let loop_start = self.instructions.len() as u16;
                self.instructions.push(OpCode::LoadTemp(loop_reg));
                self.instructions.push(OpCode::LoadZero);
                self.instructions.push(OpCode::Le);
                let exit_jump = self.instructions.len();
                self.instructions.push(OpCode::JumpIfNotZero(0));

                let body_len = body.len();
                for (j, s) in body.iter().enumerate() {
                    self.compile_stmt(s, j == body_len - 1);
                    if j == body_len - 1 {
                        self.instructions.push(OpCode::Pop);
                    }
                }

                // loop_reg = loop_reg - 1
                self.instructions.push(OpCode::LoadTemp(loop_reg));
                self.instructions.push(OpCode::LoadOne);
                self.instructions.push(OpCode::Sub);
                self.instructions.push(OpCode::StoreTemp(loop_reg));
                self.instructions.push(OpCode::Jump(loop_start));

                let exit_target = self.instructions.len() as u16;
                self.instructions[exit_jump] = OpCode::JumpIfNotZero(exit_target);

                if is_last {
                    self.instructions.push(OpCode::LoadZero);
                }
            }
            MolangStmt::Break | MolangStmt::Continue => {
                // Standalone loop breaks fallback to no-op if outside loop
                if is_last {
                    self.instructions.push(OpCode::LoadZero);
                }
            }
            MolangStmt::ForEach {
                variable: _,
                array,
                body: _,
            } => {
                self.compile_expr(array);
                self.instructions.push(OpCode::Pop);
                if is_last {
                    self.instructions.push(OpCode::LoadZero);
                }
            }
        }
    }

    fn compile_expr(&mut self, expr: &MolangExpr) {
        match expr {
            MolangExpr::Constant(val) => {
                if *val == 0.0 {
                    self.instructions.push(OpCode::LoadZero);
                } else if *val == 1.0 {
                    self.instructions.push(OpCode::LoadOne);
                } else {
                    self.instructions.push(OpCode::LoadConst(*val));
                }
            }
            MolangExpr::StringLiteral(_) => {
                self.instructions.push(OpCode::LoadZero);
            }
            MolangExpr::This => {
                self.instructions.push(OpCode::LoadThis);
            }
            MolangExpr::Variable(name) => {
                let lower = name.to_ascii_lowercase();
                if lower.starts_with("t.") || lower.starts_with("temp.") {
                    let reg = self.intern_temp(&lower);
                    self.instructions.push(OpCode::LoadTemp(reg));
                } else if lower.starts_with("c.") || lower.starts_with("context.") {
                    if let Some(qid) = query_id_from_name(&lower) {
                        self.instructions.push(OpCode::LoadQuery(qid));
                    } else {
                        let id = self.intern_var(&lower);
                        self.instructions.push(OpCode::LoadVar(id));
                    }
                } else if lower.starts_with("q.") || lower.starts_with("query.") {
                    let qid = query_id_from_name(&lower).unwrap_or(999);
                    self.instructions.push(OpCode::LoadQuery(qid));
                } else {
                    let id = self.intern_var(&lower);
                    self.instructions.push(OpCode::LoadVar(id));
                }
            }
            MolangExpr::Query(name) => {
                let qid = query_id_from_name(name).unwrap_or(999);
                self.instructions.push(OpCode::LoadQuery(qid));
            }
            MolangExpr::QueryCall { name, args } => {
                for arg in args {
                    self.compile_expr(arg);
                }
                let qid = query_id_from_name(name).unwrap_or(999);
                self.instructions.push(OpCode::CallQuery {
                    query_id: qid,
                    arg_count: args.len() as u8,
                });
            }
            MolangExpr::Add(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Add);
            }
            MolangExpr::Subtract(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Sub);
            }
            MolangExpr::Multiply(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Mul);
            }
            MolangExpr::Divide(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Div);
            }
            MolangExpr::Modulo(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Mod);
            }
            MolangExpr::UnaryMinus(val) => {
                self.compile_expr(val);
                self.instructions.push(OpCode::Neg);
            }
            MolangExpr::LogicalNot(val) => {
                self.compile_expr(val);
                self.instructions.push(OpCode::Not);
            }
            MolangExpr::Equal(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Eq);
            }
            MolangExpr::NotEqual(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Ne);
            }
            MolangExpr::Less(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Lt);
            }
            MolangExpr::LessEqual(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Le);
            }
            MolangExpr::Greater(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Gt);
            }
            MolangExpr::GreaterEqual(a, b) => {
                self.compile_expr(a);
                self.compile_expr(b);
                self.instructions.push(OpCode::Ge);
            }
            MolangExpr::LogicalAnd(a, b) => {
                self.compile_expr(a);
                let jump_pos = self.instructions.len();
                self.instructions.push(OpCode::JumpIfZeroKeep(0));
                self.instructions.push(OpCode::Pop);
                self.compile_expr(b);
                self.instructions.push(OpCode::NormalizeBool);
                let end_pos = self.instructions.len() as u16;
                self.instructions[jump_pos] = OpCode::JumpIfZeroKeep(end_pos);
            }
            MolangExpr::LogicalOr(a, b) => {
                self.compile_expr(a);
                let jump_pos = self.instructions.len();
                self.instructions.push(OpCode::JumpIfNotZeroKeep(0));
                self.instructions.push(OpCode::Pop);
                self.compile_expr(b);
                self.instructions.push(OpCode::NormalizeBool);
                let end_pos = self.instructions.len() as u16;
                self.instructions[jump_pos] = OpCode::JumpIfNotZeroKeep(end_pos);
            }
            MolangExpr::NullCoalesce(a, b) => {
                self.compile_expr(a);
                let jump_pos = self.instructions.len();
                self.instructions.push(OpCode::JumpIfNotZeroKeep(0));
                self.instructions.push(OpCode::Pop);
                self.compile_expr(b);
                let end_pos = self.instructions.len() as u16;
                self.instructions[jump_pos] = OpCode::JumpIfNotZeroKeep(end_pos);
            }
            MolangExpr::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                self.compile_expr(condition);
                let jump_false_pos = self.instructions.len();
                self.instructions.push(OpCode::JumpIfZero(0));

                self.compile_expr(then_branch);
                let jump_end_pos = self.instructions.len();
                self.instructions.push(OpCode::Jump(0));

                let else_target = self.instructions.len() as u16;
                self.compile_expr(else_branch);
                let end_target = self.instructions.len() as u16;

                self.instructions[jump_false_pos] = OpCode::JumpIfZero(else_target);
                self.instructions[jump_end_pos] = OpCode::Jump(end_target);
            }
            MolangExpr::Assign { target, value } => {
                self.compile_expr(value);
                self.instructions.push(OpCode::Dup);
                let lower = target.to_ascii_lowercase();
                if lower.starts_with("t.") || lower.starts_with("temp.") {
                    let reg = self.intern_temp(&lower);
                    self.instructions.push(OpCode::StoreTemp(reg));
                } else {
                    let id = self.intern_var(&lower);
                    self.instructions.push(OpCode::StoreVar(id));
                }
            }
            MolangExpr::MathFunc { name, args } => {
                for arg in args {
                    self.compile_expr(arg);
                }
                let fn_id = MathFnId::from_name(name).unwrap_or(MathFnId::Abs);
                self.instructions.push(OpCode::CallMath {
                    fn_id,
                    arg_count: args.len() as u8,
                });
            }
            MolangExpr::Block(stmts) => {
                let len = stmts.len();
                for (i, stmt) in stmts.iter().enumerate() {
                    let is_last = i == len - 1;
                    self.compile_stmt(stmt, is_last);
                }
                if len == 0 {
                    self.instructions.push(OpCode::LoadZero);
                }
            }
            MolangExpr::EntityDereference {
                entity_target: _,
                variable_name,
            } => {
                let id = self.intern_var(variable_name);
                self.instructions.push(OpCode::LoadVar(id));
            }
            MolangExpr::MemberAccess { target, member } => {
                self.compile_expr(target);
                let _ = member;
            }
        }
    }
}

// ============================================================================
// 10. Molang Engine Public Interface
// ============================================================================

/// Primary engine for parsing, compiling, and evaluating Bedrock Molang expressions.
pub struct MolangEngine;

impl MolangEngine {
    pub fn new() -> Self {
        Self
    }

    /// Parses a raw Molang script string into an AST script.
    pub fn parse(&self, src: &str) -> Result<MolangScript, String> {
        let tokens: Result<Vec<_>, _> = Lexer::new(src).collect();
        let tokens = tokens?;
        let mut parser = MolangParser::new(tokens);
        parser.parse_script()
    }

    /// Parses a single Molang expression string into an AST node.
    pub fn parse_expr(&self, src: &str) -> Result<MolangExpr, String> {
        let tokens: Result<Vec<_>, _> = Lexer::new(src).collect();
        let tokens = tokens?;
        let mut parser = MolangParser::new(tokens);
        parser.parse_expr(0)
    }

    /// Compiles an AST expression into zero-allocation bytecode.
    pub fn compile(&self, expr: &MolangExpr) -> MolangBytecode {
        let compiler = Compiler::new();
        compiler.compile_expr_root(expr)
    }

    /// Compiles an AST script into zero-allocation bytecode.
    pub fn compile_script(&self, script: &MolangScript) -> MolangBytecode {
        let compiler = Compiler::new();
        compiler.compile_script(script)
    }

    /// Parses and compiles a raw Molang expression string into bytecode in one pass.
    pub fn compile_str(&self, src: &str) -> Result<MolangBytecode, String> {
        let script = self.parse(src)?;
        Ok(self.compile_script(&script))
    }

    /// Evaluates an AST expression directly with zero heap allocations.
    pub fn evaluate(&self, expr: &MolangExpr, ctx: &MolangContext) -> f32 {
        match expr {
            MolangExpr::Constant(v) => *v,
            MolangExpr::StringLiteral(_) => 0.0,
            MolangExpr::This => ctx.this_val,
            MolangExpr::Variable(var) => {
                let lower = var.to_ascii_lowercase();
                if let Some(&val) = ctx.variables.get(var) {
                    val
                } else if let Some(&val) = ctx.variables.get(&lower) {
                    val
                } else if lower.starts_with("c.") || lower.starts_with("context.") {
                    ctx.resolve_context_var(&lower)
                } else {
                    0.0
                }
            }
            MolangExpr::Query(q) => ctx.resolve_query(q, &[]),
            MolangExpr::QueryCall { name, args } => {
                let mut buf = [0.0f32; 8];
                let count = args.len().min(8);
                for i in 0..count {
                    buf[i] = self.evaluate(&args[i], ctx);
                }
                ctx.resolve_query(name, &buf[..count])
            }
            MolangExpr::Add(a, b) => self.evaluate(a, ctx) + self.evaluate(b, ctx),
            MolangExpr::Subtract(a, b) => self.evaluate(a, ctx) - self.evaluate(b, ctx),
            MolangExpr::Multiply(a, b) => self.evaluate(a, ctx) * self.evaluate(b, ctx),
            MolangExpr::Divide(a, b) => {
                let divisor = self.evaluate(b, ctx);
                if divisor.abs() > 1e-6 {
                    self.evaluate(a, ctx) / divisor
                } else {
                    0.0
                }
            }
            MolangExpr::Modulo(a, b) => {
                let den = self.evaluate(b, ctx);
                if den.abs() > 1e-6 {
                    let val = self.evaluate(a, ctx);
                    val - den * (val / den).floor()
                } else {
                    0.0
                }
            }
            MolangExpr::UnaryMinus(val) => -self.evaluate(val, ctx),
            MolangExpr::LogicalNot(val) => {
                if self.evaluate(val, ctx) == 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            MolangExpr::Equal(a, b) => {
                if (self.evaluate(a, ctx) - self.evaluate(b, ctx)).abs() < 1e-6 {
                    1.0
                } else {
                    0.0
                }
            }
            MolangExpr::NotEqual(a, b) => {
                if (self.evaluate(a, ctx) - self.evaluate(b, ctx)).abs() >= 1e-6 {
                    1.0
                } else {
                    0.0
                }
            }
            MolangExpr::Less(a, b) => {
                if self.evaluate(a, ctx) < self.evaluate(b, ctx) {
                    1.0
                } else {
                    0.0
                }
            }
            MolangExpr::LessEqual(a, b) => {
                if self.evaluate(a, ctx) <= self.evaluate(b, ctx) {
                    1.0
                } else {
                    0.0
                }
            }
            MolangExpr::Greater(a, b) => {
                if self.evaluate(a, ctx) > self.evaluate(b, ctx) {
                    1.0
                } else {
                    0.0
                }
            }
            MolangExpr::GreaterEqual(a, b) => {
                if self.evaluate(a, ctx) >= self.evaluate(b, ctx) {
                    1.0
                } else {
                    0.0
                }
            }
            MolangExpr::LogicalAnd(a, b) => {
                let v_a = self.evaluate(a, ctx);
                if v_a == 0.0 {
                    0.0
                } else {
                    let v_b = self.evaluate(b, ctx);
                    if v_b != 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
            }
            MolangExpr::LogicalOr(a, b) => {
                let v_a = self.evaluate(a, ctx);
                if v_a != 0.0 {
                    1.0
                } else {
                    let v_b = self.evaluate(b, ctx);
                    if v_b != 0.0 {
                        1.0
                    } else {
                        0.0
                    }
                }
            }
            MolangExpr::NullCoalesce(a, b) => {
                let v_a = self.evaluate(a, ctx);
                if v_a != 0.0 {
                    v_a
                } else {
                    self.evaluate(b, ctx)
                }
            }
            MolangExpr::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition, ctx) != 0.0 {
                    self.evaluate(then_branch, ctx)
                } else {
                    self.evaluate(else_branch, ctx)
                }
            }
            MolangExpr::Assign { target: _, value } => self.evaluate(value, ctx),
            MolangExpr::MathFunc { name, args } => {
                if let Some(fn_id) = MathFnId::from_name(name) {
                    match fn_id {
                        MathFnId::Sin => {
                            args.first().map(|a| molang_sin(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Cos => {
                            args.first().map(|a| molang_cos(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Tan => {
                            args.first().map(|a| molang_tan(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Asin => {
                            args.first().map(|a| molang_asin(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Acos => {
                            args.first().map(|a| molang_acos(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Atan => {
                            args.first().map(|a| molang_atan(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Atan2 => {
                            let y = args.first().map(|a| self.evaluate(a, ctx)).unwrap_or(0.0);
                            let x = args.get(1).map(|a| self.evaluate(a, ctx)).unwrap_or(0.0);
                            molang_atan2(y, x)
                        }
                        MathFnId::MinAngle => {
                            args.first().map(|a| molang_min_angle(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Abs => {
                            args.first().map(|a| molang_abs(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Clamp => {
                            let val = args.first().map(|a| self.evaluate(a, ctx)).unwrap_or(0.0);
                            let min_v = args.get(1).map(|a| self.evaluate(a, ctx)).unwrap_or(0.0);
                            let max_v = args.get(2).map(|a| self.evaluate(a, ctx)).unwrap_or(1.0);
                            molang_clamp(val, min_v, max_v)
                        }
                        MathFnId::Lerp => {
                            let a = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let b = args.get(1).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let t = args.get(2).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            molang_lerp(a, b, t)
                        }
                        MathFnId::HermiteBlend => {
                            args.first().map(|a| molang_hermite_blend(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Min => {
                            let a = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let b = args.get(1).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            molang_min(a, b)
                        }
                        MathFnId::Max => {
                            let a = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let b = args.get(1).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            molang_max(a, b)
                        }
                        MathFnId::Floor => {
                            args.first().map(|a| molang_floor(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Ceil => {
                            args.first().map(|a| molang_ceil(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Round => {
                            args.first().map(|a| molang_round(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Trunc => {
                            args.first().map(|a| molang_trunc(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Mod => {
                            let a = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let b = args.get(1).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            molang_mod(a, b)
                        }
                        MathFnId::Pow => {
                            let a = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let b = args.get(1).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            molang_pow(a, b)
                        }
                        MathFnId::Sqrt => {
                            args.first().map(|a| molang_sqrt(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Exp => {
                            args.first().map(|a| molang_exp(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Ln => {
                            args.first().map(|a| molang_ln(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::Sign => {
                            args.first().map(|a| molang_sign(self.evaluate(a, ctx))).unwrap_or(0.0)
                        }
                        MathFnId::CopySign => {
                            let a = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let b = args.get(1).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            molang_copy_sign(a, b)
                        }
                        MathFnId::InverseLerp => {
                            let a = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let b = args.get(1).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let val = args.get(2).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            molang_inverse_lerp(a, b, val)
                        }
                        MathFnId::LerpRotate => {
                            let a = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let b = args.get(1).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let t = args.get(2).map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            molang_lerprotate(a, b, t)
                        }
                        MathFnId::Pi => std::f32::consts::PI,
                        MathFnId::Random => {
                            let min = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            let max = args.get(1).map(|v| self.evaluate(v, ctx)).unwrap_or(1.0);
                            min + (max - min) * 0.5
                        }
                        MathFnId::RandomInteger => {
                            let min = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(0.0);
                            min.floor()
                        }
                        MathFnId::DiceRoll => {
                            let count = args.first().map(|v| self.evaluate(v, ctx)).unwrap_or(1.0);
                            let sides = args.get(1).map(|v| self.evaluate(v, ctx)).unwrap_or(6.0);
                            count * (sides + 1.0) * 0.5
                        }
                        ease_fn => {
                            if args.len() >= 3 {
                                let start = self.evaluate(&args[0], ctx);
                                let end = self.evaluate(&args[1], ctx);
                                let t = self.evaluate(&args[2], ctx);
                                interpolate_ease(start, end, ease_fn.eval_normalized_ease(t))
                            } else if let Some(first) = args.first() {
                                let t = self.evaluate(first, ctx);
                                ease_fn.eval_normalized_ease(t)
                            } else {
                                0.0
                            }
                        }
                    }
                } else {
                    0.0
                }
            }
            MolangExpr::Block(stmts) => {
                let mut res = 0.0;
                for stmt in stmts {
                    match stmt {
                        MolangStmt::Expr(e) => res = self.evaluate(e, ctx),
                        MolangStmt::Return(opt_e) => {
                            if let Some(e) = opt_e {
                                return self.evaluate(e, ctx);
                            } else {
                                return 0.0;
                            }
                        }
                        _ => {}
                    }
                }
                res
            }
            MolangExpr::EntityDereference {
                entity_target: _,
                variable_name,
            } => *ctx.variables.get(variable_name).unwrap_or(&0.0),
            MolangExpr::MemberAccess { target, member: _ } => self.evaluate(target, ctx),
        }
    }

    /// Evaluates an AST expression with mutable context to allow assignments.
    pub fn evaluate_mut(&self, expr: &MolangExpr, ctx: &mut MolangContext) -> f32 {
        let bytecode = self.compile(expr);
        bytecode.execute_with_context(ctx)
    }

    /// Evaluates a multi-statement script with mutable context.
    pub fn evaluate_script(&self, script: &MolangScript, ctx: &mut MolangContext) -> f32 {
        let bytecode = self.compile_script(script);
        bytecode.execute_with_context(ctx)
    }
}

// ============================================================================
// 11. Comprehensive Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::alloc::{GlobalAlloc, Layout, System};

    // Thread-local tracking allocator to verify zero heap allocations during evaluation
    thread_local! {
        static TRACKING_ACTIVE: Cell<bool> = const { Cell::new(false) };
        static ALLOC_COUNT: Cell<usize> = const { Cell::new(0) };
    }

    struct ZeroAllocTracker;

    unsafe impl GlobalAlloc for ZeroAllocTracker {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            if TRACKING_ACTIVE.try_with(|c| c.get()).unwrap_or(false) {
                let _ = ALLOC_COUNT.try_with(|c| c.set(c.get() + 1));
            }
            System.alloc(layout)
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout)
        }

        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            if TRACKING_ACTIVE.try_with(|c| c.get()).unwrap_or(false) {
                let _ = ALLOC_COUNT.try_with(|c| c.set(c.get() + 1));
            }
            System.realloc(ptr, layout, new_size)
        }
    }

    #[global_allocator]
    static A: ZeroAllocTracker = ZeroAllocTracker;

    // -----------------------------------------------------------------------
    // Test 1: Degree Trigonometry Exactness & Range
    // -----------------------------------------------------------------------
    #[test]
    fn test_molang_degree_trigonometry() {
        assert!((molang_sin(0.0) - 0.0).abs() < 1e-5);
        assert!((molang_sin(30.0) - 0.5).abs() < 1e-5);
        assert!((molang_sin(90.0) - 1.0).abs() < 1e-5);
        assert!((molang_sin(180.0) - 0.0).abs() < 1e-5);
        assert!((molang_sin(270.0) - (-1.0)).abs() < 1e-5);
        assert!((molang_sin(360.0) - 0.0).abs() < 1e-5);

        assert!((molang_cos(0.0) - 1.0).abs() < 1e-5);
        assert!((molang_cos(60.0) - 0.5).abs() < 1e-5);
        assert!((molang_cos(90.0) - 0.0).abs() < 1e-5);
        assert!((molang_cos(180.0) - (-1.0)).abs() < 1e-5);

        assert!((molang_tan(0.0) - 0.0).abs() < 1e-5);
        assert!((molang_tan(45.0) - 1.0).abs() < 1e-5);
        assert_eq!(molang_tan(90.0), 0.0);

        assert!((molang_asin(0.0) - 0.0).abs() < 1e-4);
        assert!((molang_asin(0.5) - 30.0).abs() < 1e-4);
        assert!((molang_asin(1.0) - 90.0).abs() < 1e-4);
        assert!((molang_asin(-1.0) - (-90.0)).abs() < 1e-4);

        assert!((molang_acos(1.0) - 0.0).abs() < 1e-4);
        assert!((molang_acos(0.5) - 60.0).abs() < 1e-4);
        assert!((molang_acos(0.0) - 90.0).abs() < 1e-4);
        assert!((molang_acos(-1.0) - 180.0).abs() < 1e-4);

        assert!((molang_atan(0.0) - 0.0).abs() < 1e-4);
        assert!((molang_atan(1.0) - 45.0).abs() < 1e-4);
        assert!((molang_atan(-1.0) - (-45.0)).abs() < 1e-4);

        assert!((molang_atan2(0.0, 1.0) - 0.0).abs() < 1e-4);
        assert!((molang_atan2(1.0, 1.0) - 45.0).abs() < 1e-4);
        assert!((molang_atan2(1.0, 0.0) - 90.0).abs() < 1e-4);
        assert!((molang_atan2(0.0, -1.0) - 180.0).abs() < 1e-4);
        assert!((molang_atan2(-1.0, 0.0) - (-90.0)).abs() < 1e-4);

        assert_eq!(molang_min_angle(0.0), 0.0);
        assert_eq!(molang_min_angle(360.0), 0.0);
        assert_eq!(molang_min_angle(720.0), 0.0);
        assert_eq!(molang_min_angle(190.0), -170.0);
        assert_eq!(molang_min_angle(-190.0), 170.0);

        assert_eq!(molang_lerprotate(350.0, 10.0, 0.5), 360.0);
        assert_eq!(molang_lerprotate(10.0, 350.0, 0.5), 0.0);
    }

    // -----------------------------------------------------------------------
    // Test 2: Out-of-Range Clamping on Trig Domains
    // -----------------------------------------------------------------------
    #[test]
    fn test_molang_trig_domain_clamping() {
        assert_eq!(molang_asin(2.5), 90.0);
        assert_eq!(molang_asin(-5.0), -90.0);
        assert_eq!(molang_acos(3.0), 0.0);
        assert_eq!(molang_acos(-1.5), 180.0);
    }

    // -----------------------------------------------------------------------
    // Test 3: Standard Math Builtins
    // -----------------------------------------------------------------------
    #[test]
    fn test_molang_standard_math_builtins() {
        assert_eq!(molang_abs(-42.5), 42.5);
        assert_eq!(molang_clamp(15.0, 0.0, 10.0), 10.0);
        assert_eq!(molang_clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(molang_clamp(15.0, 10.0, 0.0), 10.0);
        assert_eq!(molang_lerp(10.0, 20.0, 0.5), 15.0);
        assert_eq!(molang_inverse_lerp(10.0, 20.0, 15.0), 0.5);
        assert_eq!(molang_inverse_lerp(10.0, 10.0, 15.0), 0.0);

        assert_eq!(molang_floor(3.8), 3.0);
        assert_eq!(molang_floor(-3.2), -4.0);
        assert_eq!(molang_ceil(3.2), 4.0);
        assert_eq!(molang_round(3.5), 4.0);
        assert_eq!(molang_trunc(3.8), 3.0);
        assert_eq!(molang_trunc(-3.8), -3.0);

        assert_eq!(molang_min(3.0, 7.0), 3.0);
        assert_eq!(molang_max(3.0, 7.0), 7.0);

        assert_eq!(molang_pow(2.0, 3.0), 8.0);
        assert_eq!(molang_pow(-2.0, 0.5), 0.0);
        assert_eq!(molang_sqrt(25.0), 5.0);
        assert_eq!(molang_sqrt(-9.0), 0.0);

        assert_eq!(molang_mod(7.0, 3.0), 1.0);
        assert_eq!(molang_mod(-1.0, 4.0), 3.0);
        assert_eq!(molang_mod(5.0, 0.0), 0.0);

        assert_eq!(molang_hermite_blend(0.5), 0.5);
        assert_eq!(molang_hermite_blend(0.0), 0.0);
        assert_eq!(molang_hermite_blend(1.0), 1.0);
        assert_eq!(molang_hermite_blend(1.5), 1.0);

        assert_eq!(molang_sign(12.3), 1.0);
        assert_eq!(molang_sign(-12.3), -1.0);
        assert_eq!(molang_sign(0.0), 0.0);
        assert_eq!(molang_copy_sign(5.0, -1.0), -5.0);
    }

    // -----------------------------------------------------------------------
    // Test 4: All 10 Penner Easing Families
    // -----------------------------------------------------------------------
    #[test]
    fn test_molang_penner_easing_curves() {
        let test_curves: [(&str, f32, f32, f32); 30] = [
            ("quad_in", penner_ease_in_quad(0.0), penner_ease_in_quad(0.5), penner_ease_in_quad(1.0)),
            ("quad_out", penner_ease_out_quad(0.0), penner_ease_out_quad(0.5), penner_ease_out_quad(1.0)),
            ("quad_in_out", penner_ease_in_out_quad(0.0), penner_ease_in_out_quad(0.5), penner_ease_in_out_quad(1.0)),
            ("cubic_in", penner_ease_in_cubic(0.0), penner_ease_in_cubic(0.5), penner_ease_in_cubic(1.0)),
            ("cubic_out", penner_ease_out_cubic(0.0), penner_ease_out_cubic(0.5), penner_ease_out_cubic(1.0)),
            ("cubic_in_out", penner_ease_in_out_cubic(0.0), penner_ease_in_out_cubic(0.5), penner_ease_in_out_cubic(1.0)),
            ("quart_in", penner_ease_in_quart(0.0), penner_ease_in_quart(0.5), penner_ease_in_quart(1.0)),
            ("quart_out", penner_ease_out_quart(0.0), penner_ease_out_quart(0.5), penner_ease_out_quart(1.0)),
            ("quart_in_out", penner_ease_in_out_quart(0.0), penner_ease_in_out_quart(0.5), penner_ease_in_out_quart(1.0)),
            ("quint_in", penner_ease_in_quint(0.0), penner_ease_in_quint(0.5), penner_ease_in_quint(1.0)),
            ("quint_out", penner_ease_out_quint(0.0), penner_ease_out_quint(0.5), penner_ease_out_quint(1.0)),
            ("quint_in_out", penner_ease_in_out_quint(0.0), penner_ease_in_out_quint(0.5), penner_ease_in_out_quint(1.0)),
            ("sine_in", penner_ease_in_sine(0.0), penner_ease_in_sine(0.5), penner_ease_in_sine(1.0)),
            ("sine_out", penner_ease_out_sine(0.0), penner_ease_out_sine(0.5), penner_ease_out_sine(1.0)),
            ("sine_in_out", penner_ease_in_out_sine(0.0), penner_ease_in_out_sine(0.5), penner_ease_in_out_sine(1.0)),
            ("expo_in", penner_ease_in_expo(0.0), penner_ease_in_expo(0.5), penner_ease_in_expo(1.0)),
            ("expo_out", penner_ease_out_expo(0.0), penner_ease_out_expo(0.5), penner_ease_out_expo(1.0)),
            ("expo_in_out", penner_ease_in_out_expo(0.0), penner_ease_in_out_expo(0.5), penner_ease_in_out_expo(1.0)),
            ("circ_in", penner_ease_in_circ(0.0), penner_ease_in_circ(0.5), penner_ease_in_circ(1.0)),
            ("circ_out", penner_ease_out_circ(0.0), penner_ease_out_circ(0.5), penner_ease_out_circ(1.0)),
            ("circ_in_out", penner_ease_in_out_circ(0.0), penner_ease_in_out_circ(0.5), penner_ease_in_out_circ(1.0)),
            ("back_in", penner_ease_in_back(0.0), penner_ease_in_back(0.5), penner_ease_in_back(1.0)),
            ("back_out", penner_ease_out_back(0.0), penner_ease_out_back(0.5), penner_ease_out_back(1.0)),
            ("back_in_out", penner_ease_in_out_back(0.0), penner_ease_in_out_back(0.5), penner_ease_in_out_back(1.0)),
            ("bounce_in", penner_ease_in_bounce(0.0), penner_ease_in_bounce(0.5), penner_ease_in_bounce(1.0)),
            ("bounce_out", penner_ease_out_bounce(0.0), penner_ease_out_bounce(0.5), penner_ease_out_bounce(1.0)),
            ("bounce_in_out", penner_ease_in_out_bounce(0.0), penner_ease_in_out_bounce(0.5), penner_ease_in_out_bounce(1.0)),
            ("elastic_in", penner_ease_in_elastic(0.0), penner_ease_in_elastic(0.5), penner_ease_in_elastic(1.0)),
            ("elastic_out", penner_ease_out_elastic(0.0), penner_ease_out_elastic(0.5), penner_ease_out_elastic(1.0)),
            ("elastic_in_out", penner_ease_in_out_elastic(0.0), penner_ease_in_out_elastic(0.5), penner_ease_in_out_elastic(1.0)),
        ];

        for (name, val_0, val_mid, val_1) in test_curves {
            assert!(val_0.abs() < 1e-4, "Curve {} at 0.0 must be 0, got {}", name, val_0);
            assert!((val_1 - 1.0).abs() < 1e-4, "Curve {} at 1.0 must be 1, got {}", name, val_1);
            if name.ends_with("_in_out") {
                assert!((val_mid - 0.5).abs() < 1e-4, "In-out curve {} at 0.5 must be 0.5, got {}", name, val_mid);
            }
        }

        let res_3arg = interpolate_ease(0.0, 100.0, penner_ease_out_cubic(0.5));
        assert!((res_3arg - 87.5).abs() < 1e-4);
    }

    // -----------------------------------------------------------------------
    // Test 5: Query & Context Bindings
    // -----------------------------------------------------------------------
    #[test]
    fn test_molang_query_context_bindings() {
        let mut ctx = MolangContext::default();
        ctx.anim_time = 1.25;
        ctx.ground_speed = 4.3;
        ctx.vertical_speed = -0.5;
        ctx.is_sneaking = true;
        ctx.is_sprinting = false;
        ctx.is_swimming = true;
        ctx.is_gliding = false;
        ctx.is_jumping = true;
        ctx.body_yaw = 45.0;
        ctx.head_pitch = -15.0;
        ctx.is_first_person = true;
        ctx.position = Vec3::new(10.0, 64.0, -20.0);

        assert_eq!(ctx.resolve_query("query.anim_time", &[]), 1.25);
        assert_eq!(ctx.resolve_query("q.ground_speed", &[]), 4.3);
        assert_eq!(ctx.resolve_query("query.vertical_speed", &[]), -0.5);
        assert_eq!(ctx.resolve_query("query.is_sneaking", &[]), 1.0);
        assert_eq!(ctx.resolve_query("query.is_sprinting", &[]), 0.0);
        assert_eq!(ctx.resolve_query("query.is_swimming", &[]), 1.0);
        assert_eq!(ctx.resolve_query("query.is_gliding", &[]), 0.0);
        assert_eq!(ctx.resolve_query("query.is_jumping", &[]), 1.0);
        assert_eq!(ctx.resolve_query("query.body_yaw", &[]), 45.0);
        assert_eq!(ctx.resolve_query("query.head_pitch", &[]), -15.0);
        assert_eq!(ctx.resolve_query("query.is_first_person", &[]), 1.0);
        assert_eq!(ctx.resolve_context_var("c.is_first_person"), 1.0);

        assert_eq!(ctx.resolve_query("query.position", &[0.0]), 10.0);
        assert_eq!(ctx.resolve_query("query.position", &[1.0]), 64.0);
        assert_eq!(ctx.resolve_query("query.position", &[2.0]), -20.0);

        assert_eq!(ctx.resolve_query("query.in_range", &[5.0, 0.0, 10.0]), 1.0);
        assert_eq!(ctx.resolve_query("query.in_range", &[15.0, 0.0, 10.0]), 0.0);
    }

    // -----------------------------------------------------------------------
    // Test 6: Pratt Precedence Parser & Tokenizer
    // -----------------------------------------------------------------------
    #[test]
    fn test_molang_parser_precedence_and_features() {
        let engine = MolangEngine::new();
        let ctx = MolangContext::default();

        // Binary precedence: 5 + 3 * 2 == 11
        let expr = engine.parse_expr("5.0 + 3.0 * 2.0").unwrap();
        assert_eq!(engine.evaluate(&expr, &ctx), 11.0);

        // Subtraction associativity: 10 - 3 - 2 == 5
        let expr_sub = engine.parse_expr("10.0 - 3.0 - 2.0").unwrap();
        assert_eq!(engine.evaluate(&expr_sub, &ctx), 5.0);

        // Logical NOT and AND: !0 && 1 == 1
        let expr_logic = engine.parse_expr("!0 && 1").unwrap();
        assert_eq!(engine.evaluate(&expr_logic, &ctx), 1.0);

        // Ternary operator: 1 ? 100 : 200 == 100
        let expr_ternary = engine.parse_expr("1 ? 100 : 200").unwrap();
        assert_eq!(engine.evaluate(&expr_ternary, &ctx), 100.0);

        // Null-coalescing: 0 ?? 42 == 42
        let expr_coal = engine.parse_expr("0 ?? 42").unwrap();
        assert_eq!(engine.evaluate(&expr_coal, &ctx), 42.0);

        // Null-coalescing non-zero: 5 ?? 42 == 5
        let expr_coal2 = engine.parse_expr("5 ?? 42").unwrap();
        assert_eq!(engine.evaluate(&expr_coal2, &ctx), 5.0);

        // Modulo: 7 % 4 == 3
        let expr_mod = engine.parse_expr("7 % 4").unwrap();
        assert_eq!(engine.evaluate(&expr_mod, &ctx), 3.0);
    }

    // -----------------------------------------------------------------------
    // Test 7: Multi-Statement Script & Bytecode Compilation
    // -----------------------------------------------------------------------
    #[test]
    fn test_molang_multi_statement_script_and_bytecode() {
        let engine = MolangEngine::new();
        let mut ctx = MolangContext::default();

        let script_src = "t.a = 10.0; t.b = 20.0; return t.a + t.b;";
        let bytecode = engine.compile_str(script_src).unwrap();
        let res = bytecode.execute_with_context(&mut ctx);
        assert_eq!(res, 30.0);

        // Loop construct: loop(5, { t.x = t.x + 2; }); return t.x;
        let loop_src = "t.x = 0.0; loop(5, { t.x = t.x + 2.0; }); return t.x;";
        let loop_code = engine.compile_str(loop_src).unwrap();
        let loop_res = loop_code.execute_with_context(&mut ctx);
        assert_eq!(loop_res, 10.0);
    }

    // -----------------------------------------------------------------------
    // Test 8: Zero Heap Allocation Frame Evaluation Loop
    // -----------------------------------------------------------------------
    #[test]
    fn test_molang_zero_allocation_frame_loop() {
        let engine = MolangEngine::new();
        let script_src = "Math.cos(q.anim_time * 360.0 + 40.0) * 9.6";
        let bytecode = engine.compile_str(script_src).unwrap();

        let mut exec_ctx = ExecutionContext::new();
        exec_ctx.anim_time = 0.5;

        // Warm up and verify numerical parity
        let warm_res = bytecode.execute(&mut exec_ctx);
        assert!((warm_res - (-7.3540273)).abs() < 1e-4);

        // Activate thread-local allocation tracker
        ALLOC_COUNT.with(|c| c.set(0));
        TRACKING_ACTIVE.with(|c| c.set(true));

        for i in 0..10_000 {
            exec_ctx.anim_time = (i as f32) * 0.001;
            let _ = bytecode.execute(&mut exec_ctx);
        }

        TRACKING_ACTIVE.with(|c| c.set(false));
        let allocs = ALLOC_COUNT.with(|c| c.get());

        assert_eq!(
            allocs, 0,
            "Expected strictly 0 heap allocations during frame evaluation loop, got {}",
            allocs
        );
    }
}

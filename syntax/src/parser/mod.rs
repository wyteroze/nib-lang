// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::errors::messages::ErrorMessage;
use crate::errors::parse_error::ParseError;
use crate::lexer::token::{Token, TokenType};
use crate::literal_value::LiteralValue;
use crate::parser::expr::Expr;
use crate::parser::stmt::Stmt;

pub mod expr;
pub mod stmt;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

macro_rules! match_token {
    ($self:ident, $($token_type:expr),+) => {{
        let token_types = [$($token_type),+];

        let mut matched = false;
        for token_type in token_types {
            if $self.check(token_type)? {
                $self.advance()?;

                matched = true;
                break;
            }
        }

        matched
    }};
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    // Parser entrypoint
    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            stmts.push(self.declaration()?)
        }

        Ok(stmts)
    }

    // Statement parsers
    // These can only appear in the top level of a program:
    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        self.statement()
    }

    // These can appear inside code blocks, or the top level of a program.
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if match_token!(self, TokenType::Var) {
            return self.var_declaration();
        } else if match_token!(self, TokenType::Const) {
            return self.const_declaration();
        } else if match_token!(self, TokenType::Return) {
            return self.return_statement();
        } else if match_token!(self, TokenType::If) {
            return self.if_statement();
        } else if match_token!(self, TokenType::Func) {
            return self.func_declaration();
        }

        self.expression_statement()
    }

    // This handles things like `x = 5`, or wraps the expression in a Stmt
    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let target = self.expression()?;

        if match_token!(self, TokenType::Equals) {
            let value = self.expression()?;

            self.consume(TokenType::Semicolon, ErrorMessage::ExpectedChar(';'))?;
            return Ok(Stmt::Assign { target, value });
        }

        self.consume(TokenType::Semicolon, ErrorMessage::ExpectedChar(';'))?;
        Ok(Stmt::Expr(target))
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(TokenType::Identifier, ErrorMessage::ExpectedIdentifier("var"))?
            .clone();

        let initializer = if match_token!(self, TokenType::Equals) { Some(self.expression()?) } else { None };

        self.consume(TokenType::Semicolon, ErrorMessage::ExpectedChar(';'))?;
        Ok(Stmt::Var { name, initializer })
    }

    fn const_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(TokenType::Identifier, ErrorMessage::ExpectedIdentifier("const"))?
            .clone();

        self.consume(TokenType::Equals, ErrorMessage::ExpectedChar('='))?;

        let initializer = self.expression()?;
        self.consume(TokenType::Semicolon, ErrorMessage::ExpectedChar(';'))?;

        Ok(Stmt::Const { name, initializer })
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let return_value = if !self.check(TokenType::Semicolon)? { Some(self.expression()?) } else { None };

        self.consume(TokenType::Semicolon, ErrorMessage::ExpectedChar(';'))?;
        Ok(Stmt::Return { return_value })
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.expression()?;
        let mut if_branch = Vec::new();
        let mut else_branch = None;

        self.advance()?; // skip opening brace
        while !self.check(TokenType::RightBrace)? {
            if_branch.push(self.statement()?);
        }

        self.advance()?; // skip closing brace

        if self.check(TokenType::Else)? {
            self.advance()?; // skip `else`

            if self.check(TokenType::If)? {
                self.advance()?; // skip `if`
                let nested = self.if_statement()?;

                else_branch = Some(vec![nested])
            } else {
                let mut branch = Vec::new();

                self.advance()?; // skip opening brace
                while !self.check(TokenType::RightBrace)? {
                    branch.push(self.statement()?);
                }

                self.advance()?; // skip closing brace

                else_branch = Some(branch)
            }
        }

        Ok(Stmt::If { condition, if_branch, else_branch })
    }

    fn func_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(TokenType::Identifier, ErrorMessage::ExpectedIdentifier("func"))?
            .clone();

        let params = self.parse_params()?;

        let mut body = Vec::new();
        self.consume(TokenType::LeftBrace, ErrorMessage::ExpectedChar('{'))?;
        while !self.check(TokenType::RightBrace)? {
            body.push(self.statement()?);
        }
        self.consume(TokenType::RightBrace, ErrorMessage::ExpectedChar('}'))?;

        Ok(Stmt::Func { name, params, body })
    }

    // Expression parsers

    // Highest level parser
    // Math: The parser chain is essentially just reverse PEMDAS. (SADMEP)
    // First S/A (term), then D/M (factor), then E (exponentiation), then P (primary).
    // Left associative functions call the next highest precedence in a loop (ex. S/A calls D/M),
    // while right associative functions call themselves.
    // Left associative is `Add(Add(1,2),3)`, while right associative is `Exp(2,Exp(3,4))`
    fn expression(&mut self) -> Result<Expr, ParseError> {
        if self.check(TokenType::Func)? {
            return self.anonymous_func();
        }

        self.or()
    }

    fn anonymous_func(&mut self) -> Result<Expr, ParseError> {
        self.consume(TokenType::Func, ErrorMessage::ExpectedIdentifier("func"))?;
        let params = self.parse_params()?;
        let mut body = Vec::new();

        self.consume(TokenType::LeftBrace, ErrorMessage::ExpectedChar('{'))?;
        while !self.check(TokenType::RightBrace)? {
            body.push(self.statement()?);
        }
        self.consume(TokenType::RightBrace, ErrorMessage::ExpectedChar('}'))?;

        Ok(Expr::Func { params, body })
    }

    fn exponentiation(&mut self) -> Result<Expr, ParseError> {
        let left = self.unary()?;

        if match_token!(self, TokenType::Carat) {
            let right = self.exponentiation()?;

            return Ok(Expr::Exponentiation { left: left.into(), right: right.into() });
        }

        Ok(left)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.exponentiation()?;

        while match_token!(self, TokenType::Star, TokenType::Slash, TokenType::Percent) {
            let operation = self.previous().clone();
            let right = self.exponentiation()?;

            expr = match operation.token_type {
                TokenType::Star => Expr::Multiply { left: expr.into(), right: right.into() },

                TokenType::Slash => Expr::Divide { left: expr.into(), right: right.into() },

                TokenType::Percent => Expr::Modulo { left: expr.into(), right: right.into() },

                _ => unimplemented!(),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while match_token!(self, TokenType::Plus, TokenType::Minus) {
            let operation = self.previous().clone();
            let right = self.factor()?;

            expr = match operation.token_type {
                TokenType::Plus => Expr::Add { left: expr.into(), right: right.into() },

                TokenType::Minus => Expr::Subtract { left: expr.into(), right: right.into() },

                _ => unreachable!(),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;

        while match_token!(self, TokenType::Or) {
            let right = self.and()?;

            expr = Expr::Or { left: expr.into(), right: right.into() }
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;

        while match_token!(self, TokenType::And) {
            let right = self.equality()?;

            expr = Expr::And { left: expr.into(), right: right.into() }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while match_token!(self, TokenType::BangEqual, TokenType::EqualEqual) {
            let operation = self.previous().clone();
            let right = self.comparison()?;

            expr = match operation.token_type {
                TokenType::BangEqual => Expr::NotEquals { left: expr.into(), right: right.into() },

                TokenType::EqualEqual => Expr::Equals { left: expr.into(), right: right.into() },

                _ => unreachable!(),
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;

        while match_token!(self, TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual) {
            let operation = self.previous().clone();
            let right = self.term()?;

            expr = match operation.token_type {
                TokenType::Greater => Expr::GreaterThan { left: expr.into(), right: right.into() },

                TokenType::GreaterEqual => Expr::GreaterEqualThan { left: expr.into(), right: right.into() },

                TokenType::Less => Expr::LessThan { left: expr.into(), right: right.into() },

                TokenType::LessEqual => Expr::LessEqualThan { left: expr.into(), right: right.into() },

                _ => unreachable!(),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if match_token!(self, TokenType::Not) {
            let operand = self.unary()?;
            return Ok(Expr::Not { operand: operand.into() });
        }

        if match_token!(self, TokenType::Minus) {
            let operand = self.unary()?;
            return Ok(Expr::Negate { operand: operand.into() });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        while self.check(TokenType::LeftParen)? {
            let args = self.parse_args()?;
            expr = Expr::Call { callee: expr.into(), args }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if match_token!(self, TokenType::Number) {
            return Ok(Expr::Literal(self.previous().literal.clone().unwrap()));
        } else if match_token!(self, TokenType::True) {
            return Ok(Expr::Literal(LiteralValue::Bool(true)));
        } else if match_token!(self, TokenType::False) {
            return Ok(Expr::Literal(LiteralValue::Bool(false)));
        } else if match_token!(self, TokenType::Nil) {
            return Ok(Expr::Literal(LiteralValue::Nil));
        } else if match_token!(self, TokenType::String) {
            return Ok(Expr::Literal(self.previous().literal.clone().unwrap()));
        } else if match_token!(self, TokenType::Identifier) {
            return Ok(Expr::Identifier(self.previous().clone()));
        } else if match_token!(self, TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, ErrorMessage::ExpectedChar(')'))?;

            return Ok(Expr::Group { expr: expr.into() });
        }

        Err(ParseError { token: self.peek().clone(), message: ErrorMessage::ExpectedExpression })
    }

    fn parse_params(&mut self) -> Result<Vec<Token>, ParseError> {
        let mut params = Vec::new();
        self.consume(TokenType::LeftParen, ErrorMessage::ExpectedChar('('))?;

        if !self.check(TokenType::RightParen)? {
            loop {
                params.push(
                    self.consume(TokenType::Identifier, ErrorMessage::ExpectedIdentifier("param"))?
                        .clone(),
                );

                if self.check(TokenType::RightParen)? {
                    break;
                } else {
                    self.consume(TokenType::Comma, ErrorMessage::ExpectedChar(','))?;
                }
            }
        }

        self.consume(TokenType::RightParen, ErrorMessage::ExpectedChar(')'))?;
        Ok(params)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        self.consume(TokenType::LeftParen, ErrorMessage::ExpectedChar('('))?;

        if !self.check(TokenType::RightParen)? {
            loop {
                args.push(self.expression()?);

                if self.check(TokenType::RightParen)? {
                    break;
                } else {
                    self.consume(TokenType::Comma, ErrorMessage::ExpectedChar(','))?;
                }
            }
        }

        self.consume(TokenType::RightParen, ErrorMessage::ExpectedChar(')'))?;
        Ok(args)
    }

    // Helper methods
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            .unwrap_or_else(|| self.tokens.last().expect("Token stream is empty"))
    }

    fn previous(&self) -> &Token {
        self.tokens
            .get(self.current - 1)
            .unwrap_or_else(|| self.tokens.last().expect("Token stream is empty"))
    }

    fn advance(&mut self) -> Result<&Token, ParseError> {
        if !self.is_at_end() {
            self.current += 1;

            Ok(self.tokens.get(self.current - 1).unwrap())
        } else {
            Err(ParseError { token: self.peek().clone(), message: ErrorMessage::UnexpectedEof })
        }
    }

    fn check(&self, token_type: TokenType) -> Result<bool, ParseError> {
        if self.is_at_end() {
            return Ok(false);
        }

        Ok(self.peek().token_type == token_type)
    }

    fn consume(&mut self, token_type: TokenType, message: ErrorMessage) -> Result<&Token, ParseError> {
        if self.check(token_type)? { self.advance() } else { Err(ParseError { token: self.peek().clone(), message }) }
    }
}

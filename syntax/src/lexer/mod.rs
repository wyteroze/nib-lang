// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::errors::lex_error::LexError;
use crate::errors::messages::ErrorMessage;
use crate::lexer::token::{Token, TokenType};
use crate::literal_value::LiteralValue;
use std::collections::HashMap;

pub mod token;

pub struct Lexer {
    tkn_start: usize,
    source_idx: usize,
    source: Vec<char>,

    line: usize,
    keywords: HashMap<String, TokenType>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        let keywords: HashMap<String, TokenType> = HashMap::from([
            ("var".into(), TokenType::Var),
            ("true".into(), TokenType::True),
            ("false".into(), TokenType::False),
            ("nil".into(), TokenType::Nil),
            ("return".into(), TokenType::Return),
            ("if".into(), TokenType::If),
            ("else".into(), TokenType::Else),
            ("and".into(), TokenType::And),
            ("or".into(), TokenType::Or),
            ("not".into(), TokenType::Not),
            ("const".into(), TokenType::Const),
            ("func".into(), TokenType::Func),
        ]);

        Self { source: source.chars().collect(), keywords, line: 0, source_idx: 0, tkn_start: 0 }
    }

    // Lexer entrypoint
    pub fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.tkn_start = self.source_idx;

            if let Some(t) = self.lex_token()? {
                tokens.push(t)
            }
        }

        tokens.push(Token::eof(self.source.len()));
        Ok(tokens)
    }

    // Lex methods
    fn lex_token(&mut self) -> Result<Option<Token>, LexError> {
        let mut literal: Option<LiteralValue> = None;

        let c = self.advance()?;
        let token = match c {
            ';' => Some(TokenType::Semicolon),
            '{' => Some(TokenType::LeftBrace),
            '}' => Some(TokenType::RightBrace),
            '(' => Some(TokenType::LeftParen),
            ')' => Some(TokenType::RightParen),
            ',' => Some(TokenType::Comma),

            // Math operators
            '+' => Some(TokenType::Plus),
            '-' => Some(TokenType::Minus),
            '/' => Some(TokenType::Slash),
            '*' => Some(TokenType::Star),
            '^' => Some(TokenType::Carat),
            '%' => Some(TokenType::Percent),

            // Whitespace/special characters
            ' ' | '\r' | '\t' => None,
            '\n' => {
                self.line += 1;
                None
            }

            // Strings
            '"' => {
                let string = self.string()?;

                literal = Some(LiteralValue::String(string));
                Some(TokenType::String)
            }

            // Multi-character symbols
            '=' => {
                if *self.peek()? == '=' {
                    self.advance()?;
                    Some(TokenType::EqualEqual)
                } else {
                    Some(TokenType::Equals)
                }
            }
            '>' => {
                if *self.peek()? == '=' {
                    self.advance()?;
                    Some(TokenType::GreaterEqual)
                } else {
                    Some(TokenType::Greater)
                }
            }
            '<' => {
                if *self.peek()? == '=' {
                    self.advance()?;
                    Some(TokenType::LessEqual)
                } else {
                    Some(TokenType::Less)
                }
            }
            '!' => {
                if *self.peek()? == '=' {
                    self.advance()?;
                    Some(TokenType::BangEqual)
                } else {
                    None
                }
            }

            // not a symbol
            _ => {
                if self.is_digit(&c) {
                    let number = self.number()?;

                    literal = Some(LiteralValue::Number(number));
                    Some(TokenType::Number)
                } else if self.is_letter(&c) {
                    let (identifier, token_type) = self.identifier()?;

                    literal = Some(LiteralValue::String(identifier));
                    Some(token_type)
                } else {
                    return Err(LexError { line: self.line, message: ErrorMessage::UnexpectedChar(c) });
                }
            }
        };

        if let Some(token_type) = token {
            let lexeme = self.source[self.tkn_start..self.source_idx]
                .iter()
                .collect();
            let length = self.source_idx - self.tkn_start;

            Ok(Some(Token { start: self.tkn_start, lexeme, token_type, length, literal }))
        } else {
            Ok(None)
        }
    }

    // Lex helpers
    fn consume_digits(&mut self) -> Result<(), LexError> {
        while self.is_digit(self.peek()?) {
            self.advance()?;
        }

        Ok(())
    }

    fn number(&mut self) -> Result<f64, LexError> {
        // first part of number
        self.consume_digits()?;

        // decimal part
        if *self.peek()? == '.' && self.is_digit(self.peek_next()?) {
            self.advance()?; // consume "."
            self.consume_digits()?;
        }

        let num_str: String = self.source[self.tkn_start..self.source_idx]
            .iter()
            .collect();
        num_str
            .parse::<f64>()
            .map_err(|_| LexError { line: self.line, message: ErrorMessage::MalformedNumber(num_str) })
    }

    fn consume_letters(&mut self) -> Result<(), LexError> {
        while self.is_letter(self.peek()?) {
            self.advance()?;
        }

        Ok(())
    }

    fn identifier(&mut self) -> Result<(String, TokenType), LexError> {
        self.consume_letters()?;

        let word = self.source[self.tkn_start..self.source_idx]
            .iter()
            .collect();
        if let Some(kw) = self.keywords.get(&word) { Ok((word, kw.clone())) } else { Ok((word, TokenType::Identifier)) }
    }

    fn string(&mut self) -> Result<String, LexError> {
        while !self.is_at_end() && *self.peek()? != '"' {
            if *self.peek()? == '\n' {
                self.line += 1;
            }

            self.advance()?;
        }

        if self.is_at_end() {
            return Err(LexError { line: self.line, message: ErrorMessage::UnterminatedString });
        }

        self.advance()?; // closing quote

        // +1 to skip starting quote, -1 to skip closing quote
        let literal = self.source[self.tkn_start + 1..self.source_idx - 1]
            .iter()
            .collect();

        Ok(literal)
    }

    // Utils
    fn is_at_end(&self) -> bool {
        self.source_idx >= self.source.len()
    }

    fn peek(&self) -> Result<&char, LexError> {
        self.source
            .get(self.source_idx)
            .ok_or(LexError { line: self.source_idx, message: ErrorMessage::UnexpectedEof })
    }

    fn peek_next(&self) -> Result<&char, LexError> {
        self.source
            .get(self.source_idx + 1)
            .ok_or(LexError { line: self.source_idx, message: ErrorMessage::UnexpectedEof })
    }

    fn advance(&mut self) -> Result<char, LexError> {
        let last = *self.peek()?;
        self.source_idx += 1;

        Ok(last)
    }

    // Type checkers
    fn is_letter(&self, character: &char) -> bool {
        matches!(character, 'a'..='z' | 'A'..='Z' | '_')
    }

    fn is_digit(&self, character: &char) -> bool {
        character.is_ascii_digit()
    }
}

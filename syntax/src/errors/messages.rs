// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum ErrorMessage {
    UnexpectedChar(char),
    UnexpectedEof,
    ExpectedIdentifier(&'static str),
    MalformedNumber(String),
    ExpectedChar(char),
    ExpectedExpression,
    UnterminatedString,
}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedChar(c) => write!(f, "Unexpected character '{}'", c),
            Self::UnexpectedEof => write!(f, "Unexpected EOF"),
            Self::ExpectedIdentifier(s) => write!(f, "Expected identifier after '{}'", s),
            Self::MalformedNumber(s) => write!(f, "Malformed number '{}'", s),
            Self::ExpectedChar(c) => write!(f, "Expected character '{}'", c),
            Self::ExpectedExpression => write!(f, "Expected expression"),
            Self::UnterminatedString => write!(f, "Unterminated string"),
        }
    }
}

// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::errors::messages::ErrorMessage;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct LexError {
    pub line: usize,
    pub message: ErrorMessage,
}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] {}", self.line, self.message)
    }
}

impl Error for LexError {}

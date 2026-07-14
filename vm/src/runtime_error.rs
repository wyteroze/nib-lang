// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum RuntimeError {
    IllegalOperation(String),
    InvalidOperation(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::IllegalOperation(msg) => write!(f, "Illegal operation: {}", msg),
            RuntimeError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl Error for RuntimeError {}

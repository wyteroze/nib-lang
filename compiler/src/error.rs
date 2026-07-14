// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CompileError {
    TooManyConstants,
    UnknownLocal(String),
    ImmutableLocal(String),
    InvalidAssignment,
}

impl Display for CompileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::TooManyConstants => write!(f, "Too many constants"),
            CompileError::UnknownLocal(s) => write!(f, "Unknown local {}", s),
            CompileError::ImmutableLocal(s) => write!(f, "Immutable local {}", s),
            CompileError::InvalidAssignment => write!(f, "Invalid assignment"),
        }
    }
}

impl Error for CompileError {}

// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use bytecode::constant::Constant;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    HeapObj(usize),
    Bool(bool),
    Number(f64),
    Nil,
    Void,
}

impl From<Constant> for Value {
    fn from(value: Constant) -> Self {
        match value {
            Constant::Number(n) => Value::Number(n),
            Constant::Bool(b) => Value::Bool(b),
            Constant::Nil => Value::Nil,
            Constant::String(_) => panic!("String is incompatible with Value, put it in the heap"),
            Constant::Chunk(_) => panic!("Chunk is incompatible with Value, put it in the heap"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::HeapObj(idx) => write!(f, "heap[{}]", idx),
        }
    }
}

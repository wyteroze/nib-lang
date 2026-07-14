// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

#[derive(Clone, Debug)]
pub enum LiteralValue {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

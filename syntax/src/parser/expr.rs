// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::lexer::token::Token;
use crate::literal_value::LiteralValue;
use crate::parser::stmt::Stmt;

#[derive(Debug)]
pub enum Expr {
    Literal(LiteralValue),
    Identifier(Token),
    Add { left: Box<Expr>, right: Box<Expr> },
    Subtract { left: Box<Expr>, right: Box<Expr> },
    Multiply { left: Box<Expr>, right: Box<Expr> },
    Divide { left: Box<Expr>, right: Box<Expr> },
    Exponentiation { left: Box<Expr>, right: Box<Expr> },

    Or { left: Box<Expr>, right: Box<Expr> },
    And { left: Box<Expr>, right: Box<Expr> },
    Equals { left: Box<Expr>, right: Box<Expr> },
    NotEquals { left: Box<Expr>, right: Box<Expr> },
    GreaterThan { left: Box<Expr>, right: Box<Expr> },
    GreaterEqualThan { left: Box<Expr>, right: Box<Expr> },
    LessThan { left: Box<Expr>, right: Box<Expr> },
    LessEqualThan { left: Box<Expr>, right: Box<Expr> },
    Not { operand: Box<Expr> },
    Negate { operand: Box<Expr> },
    Modulo { left: Box<Expr>, right: Box<Expr> },
    Group { expr: Box<Expr> },
    Func { params: Vec<Token>, body: Vec<Stmt> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
}

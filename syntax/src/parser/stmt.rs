// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::lexer::token::Token;
use crate::parser::expr::Expr;

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Var { name: Token, initializer: Option<Expr> },
    Const { name: Token, initializer: Expr },
    Assign { target: Expr, value: Expr },
    Return { return_value: Option<Expr> },
    If { condition: Expr, if_branch: Vec<Stmt>, else_branch: Option<Vec<Stmt>> },
    Func { name: Token, params: Vec<Token>, body: Vec<Stmt> },
}

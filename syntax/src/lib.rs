// Copyright 2026 wyteroze. Licensed under the Do What The Fuck You Want To Public License Version 2.

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::parser::stmt::Stmt;

mod errors;
pub mod lexer;
pub mod literal_value;
pub mod parser;

pub fn parse_source(source: String) -> Result<Vec<Stmt>, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex()?;

    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;

    Ok(ast)
}

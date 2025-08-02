use rust_decimal::Decimal;

use crate::calc::{error::CalcResult, parser::Parser};

mod token;
mod tokenizer;
mod error;
mod ast;
mod parser;

pub fn calculate(expression: &str) -> CalcResult<Decimal> {
    let mut parser = Parser::new(expression)?;
    let ast = parser.parse()?;
    Ok(ast.eval())
}
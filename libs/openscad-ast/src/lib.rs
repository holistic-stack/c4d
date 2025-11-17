mod span;
mod error;
mod ast;
mod parser;
mod visitor;
mod printer;
mod validate;

pub use ast::*;
pub use error::*;
pub use span::*;
pub use visitor::*;
pub use printer::*;
pub use validate::*;

pub fn parse(source: &str) -> Result<Ast, ParseError> {
    parser::parse(source)
}

pub fn parse_strict(source: &str) -> Result<Ast, ParseError> {
    let ast = parser::parse(source)?;
    validate(&ast)?;
    Ok(ast)
}
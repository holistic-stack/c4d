//! # Expression Parsing
//!
//! Facade module for parsing OpenSCAD expressions.
//!
//! ## Module Structure (SRP)
//!
//! - `operators` - Binary, unary, ternary operators with precedence
//! - `primaries` - Literals and identifiers
//! - `postfix` - Call, index, member access
//! - `collections` - List and range parsing
//!
//! ## Operator Precedence (from OpenSCAD)
//!
//! | Precedence | Operators | Associativity |
//! |------------|-----------|---------------|
//! | 1 | ?: (ternary) | Right |
//! | 2 | \|\| | Left |
//! | 3 | && | Left |
//! | 4 | == != | Left |
//! | 5 | < > <= >= | Left |
//! | 6 | + - | Left |
//! | 7 | * / % | Left |
//! | 8 | ^ | Right |
//! | 9 | ! - + (unary) | Right |
//! | 10 | () [] . | Left |
//!
//! ## Example
//!
//! ```rust,ignore
//! let expr = parser.parse_expression()?;
//! ```

use super::Parser;
use super::operators::Precedence;
use crate::cst::CstNode;
use crate::error::ParseError;

impl<'a> Parser<'a> {
    /// Parse an expression.
    ///
    /// Entry point for expression parsing. Uses precedence climbing algorithm.
    ///
    /// ## Example
    ///
    /// ```text
    /// 1 + 2 * 3
    /// x > 0 ? 1 : 0
    /// [1, 2, 3]
    /// ```
    pub(super) fn parse_expression(&mut self) -> Result<CstNode, ParseError> {
        self.parse_precedence(Precedence::Ternary)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::cst::NodeKind;

    fn parse_expr(source: &str) -> crate::cst::CstNode {
        let full = format!("x = {};", source);
        let tokens = Lexer::new(&full).tokenize();
        let mut parser = Parser::new(&full, tokens);
        let cst = parser.parse();
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        cst.root.children[0].children[1].clone()
    }

    #[test]
    fn test_expression_dispatch() {
        // Test that expressions parse through the facade
        let expr = parse_expr("1 + 2 * 3");
        assert_eq!(expr.kind, NodeKind::BinaryExpression);
    }
}

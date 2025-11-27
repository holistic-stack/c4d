//! # Primary Expression Parsing
//!
//! Parses primary expressions: literals, identifiers, parenthesized expressions.
//!
//! ## Responsibilities
//!
//! - Number literals: `42`, `3.14`
//! - String literals: `"hello"`
//! - Boolean literals: `true`, `false`
//! - Undef: `undef`
//! - Identifiers: `x`, `myVar`
//! - Special variables: `$fn`, `$fa`, `$fs`
//! - Parenthesized expressions: `(1 + 2)`
//!
//! ## Example
//!
//! ```rust,ignore
//! let expr = parser.parse_primary()?;
//! ```

use super::Parser;
use crate::cst::{CstNode, NodeKind};
use crate::error::ParseError;
use crate::lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse primary expression.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// primary = number | string | boolean | undef | identifier
    ///         | special_variable | list | range | "(" expression ")"
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// 42
    /// "hello"
    /// true
    /// undef
    /// x
    /// $fn
    /// [1, 2, 3]
    /// (1 + 2)
    /// ```
    pub(super) fn parse_primary(&mut self) -> Result<CstNode, ParseError> {
        let token = self.peek().clone();
        let start = self.current_position();

        match token.kind {
            // Number
            TokenKind::Number => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::Number, self.span_from(start), token.text))
            }

            // String
            TokenKind::String => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::String, self.span_from(start), token.text))
            }

            // Boolean
            TokenKind::True => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::Boolean, self.span_from(start), "true"))
            }
            TokenKind::False => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::Boolean, self.span_from(start), "false"))
            }

            // Undef
            TokenKind::Undef => {
                self.advance();
                Ok(CstNode::new(NodeKind::Undef, self.span_from(start)))
            }

            // Identifier
            TokenKind::Identifier => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::Identifier, self.span_from(start), token.text))
            }

            // Special variable ($fn, $fa, etc.)
            TokenKind::SpecialVariable => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::SpecialVariable, self.span_from(start), token.text))
            }

            // List or range: [...]
            TokenKind::LBracket => {
                self.parse_list_or_range()
            }

            // Parenthesized expression
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }

            _ => Err(ParseError::unexpected_token(
                &token.text,
                "expression",
            ).with_span(token.span)),
        }
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
    fn test_parse_number() {
        let expr = parse_expr("42");
        assert_eq!(expr.kind, NodeKind::Number);
        assert_eq!(expr.text_or_empty(), "42");
    }

    #[test]
    fn test_parse_float() {
        let expr = parse_expr("3.14");
        assert_eq!(expr.kind, NodeKind::Number);
        assert_eq!(expr.text_or_empty(), "3.14");
    }

    #[test]
    fn test_parse_string() {
        let expr = parse_expr("\"hello\"");
        assert_eq!(expr.kind, NodeKind::String);
    }

    #[test]
    fn test_parse_boolean_true() {
        let expr = parse_expr("true");
        assert_eq!(expr.kind, NodeKind::Boolean);
        assert_eq!(expr.text_or_empty(), "true");
    }

    #[test]
    fn test_parse_boolean_false() {
        let expr = parse_expr("false");
        assert_eq!(expr.kind, NodeKind::Boolean);
        assert_eq!(expr.text_or_empty(), "false");
    }

    #[test]
    fn test_parse_undef() {
        let expr = parse_expr("undef");
        assert_eq!(expr.kind, NodeKind::Undef);
    }

    #[test]
    fn test_parse_identifier() {
        let expr = parse_expr("myVar");
        assert_eq!(expr.kind, NodeKind::Identifier);
        assert_eq!(expr.text_or_empty(), "myVar");
    }

    #[test]
    fn test_parse_special_variable() {
        let expr = parse_expr("$fn");
        assert_eq!(expr.kind, NodeKind::SpecialVariable);
        assert_eq!(expr.text_or_empty(), "$fn");
    }

    #[test]
    fn test_parse_parenthesized() {
        let expr = parse_expr("(1 + 2)");
        // Should return the inner expression, not a wrapper
        assert_eq!(expr.kind, NodeKind::BinaryExpression);
    }
}

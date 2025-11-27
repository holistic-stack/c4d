//! # Collection Parsing
//!
//! Parses list and range expressions.
//!
//! ## Responsibilities
//!
//! - List literals: `[1, 2, 3]`
//! - Range expressions: `[0:10]`, `[0:2:10]`
//!
//! ## Example
//!
//! ```rust,ignore
//! let expr = parser.parse_list_or_range()?;
//! ```

use super::Parser;
use crate::cst::{CstNode, NodeKind};
use crate::error::ParseError;
use crate::lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse list or range.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// list = "[" (expression ("," expression)*)? "]"
    /// range = "[" expression ":" expression (":" expression)? "]"
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// []                  // empty list
    /// [1, 2, 3]           // list
    /// [1, 2, 3,]          // list with trailing comma
    /// [0:10]              // range (start:end)
    /// [0:2:10]            // range (start:step:end)
    /// ```
    pub(super) fn parse_list_or_range(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.expect(TokenKind::LBracket)?;

        // Empty list
        if self.check(TokenKind::RBracket) {
            self.advance();
            return Ok(CstNode::with_children(NodeKind::List, self.span_from(start), vec![]));
        }

        // First element
        let first = self.parse_expression()?;

        // Check for range syntax
        if self.check(TokenKind::Colon) {
            return self.parse_range(start, first);
        }

        // List
        self.parse_list(start, first)
    }

    /// Parse list (after first element).
    ///
    /// ## Grammar
    ///
    /// ```text
    /// list = "[" expression ("," expression)* ","? "]"
    /// ```
    fn parse_list(&mut self, start: crate::span::Position, first: CstNode) -> Result<CstNode, ParseError> {
        let mut elements = vec![first];
        
        while self.match_token(TokenKind::Comma) {
            // Allow trailing comma
            if self.check(TokenKind::RBracket) {
                break;
            }
            elements.push(self.parse_expression()?);
        }

        self.expect(TokenKind::RBracket)?;
        Ok(CstNode::with_children(NodeKind::List, self.span_from(start), elements))
    }

    /// Parse range.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// range = "[" start ":" end "]"
    ///       | "[" start ":" step ":" end "]"
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// [0:10]      // start=0, end=10
    /// [0:2:10]    // start=0, step=2, end=10
    /// ```
    fn parse_range(&mut self, start: crate::span::Position, first: CstNode) -> Result<CstNode, ParseError> {
        self.expect(TokenKind::Colon)?;
        let second = self.parse_expression()?;

        let children = if self.check(TokenKind::Colon) {
            self.advance();
            let third = self.parse_expression()?;
            // [start : step : end]
            vec![first, second, third]
        } else {
            // [start : end]
            vec![first, second]
        };

        self.expect(TokenKind::RBracket)?;
        Ok(CstNode::with_children(NodeKind::Range, self.span_from(start), children))
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
    fn test_parse_empty_list() {
        let expr = parse_expr("[]");
        assert_eq!(expr.kind, NodeKind::List);
        assert!(expr.children.is_empty());
    }

    #[test]
    fn test_parse_list() {
        let expr = parse_expr("[1, 2, 3]");
        assert_eq!(expr.kind, NodeKind::List);
        assert_eq!(expr.children.len(), 3);
    }

    #[test]
    fn test_parse_list_trailing_comma() {
        let expr = parse_expr("[1, 2, 3,]");
        assert_eq!(expr.kind, NodeKind::List);
        assert_eq!(expr.children.len(), 3);
    }

    #[test]
    fn test_parse_nested_list() {
        let expr = parse_expr("[[1, 2], [3, 4]]");
        assert_eq!(expr.kind, NodeKind::List);
        assert_eq!(expr.children.len(), 2);
        assert_eq!(expr.children[0].kind, NodeKind::List);
    }

    #[test]
    fn test_parse_range() {
        let expr = parse_expr("[0:10]");
        assert_eq!(expr.kind, NodeKind::Range);
        assert_eq!(expr.children.len(), 2);
    }

    #[test]
    fn test_parse_range_with_step() {
        let expr = parse_expr("[0:2:10]");
        assert_eq!(expr.kind, NodeKind::Range);
        assert_eq!(expr.children.len(), 3);
    }

    #[test]
    fn test_parse_range_with_expressions() {
        let expr = parse_expr("[1+1:5*2]");
        assert_eq!(expr.kind, NodeKind::Range);
        assert_eq!(expr.children.len(), 2);
        // Children should be binary expressions
        assert_eq!(expr.children[0].kind, NodeKind::BinaryExpression);
        assert_eq!(expr.children[1].kind, NodeKind::BinaryExpression);
    }

    #[test]
    fn test_parse_list_with_expressions() {
        let expr = parse_expr("[1+2, 3*4, 5/6]");
        assert_eq!(expr.kind, NodeKind::List);
        assert_eq!(expr.children.len(), 3);
        // All children should be binary expressions
        for child in &expr.children {
            assert_eq!(child.kind, NodeKind::BinaryExpression);
        }
    }
}

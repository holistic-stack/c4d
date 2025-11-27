//! # Postfix Expression Parsing
//!
//! Parses postfix expressions: function calls, index access, member access.
//!
//! ## Responsibilities
//!
//! - Function calls: `sin(x)`
//! - Index access: `arr[0]`
//! - Member access: `v.x`
//!
//! ## Example
//!
//! ```rust,ignore
//! let expr = parser.parse_postfix()?;
//! ```

use super::Parser;
use crate::cst::{CstNode, NodeKind};
use crate::error::ParseError;
use crate::lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse postfix expressions (call, index, dot).
    ///
    /// ## Grammar
    ///
    /// ```text
    /// postfix = primary ("(" args ")" | "[" expr "]" | "." identifier)*
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// sin(x)
    /// arr[0]
    /// vec.x
    /// func(a)(b)
    /// arr[i].length
    /// ```
    pub(super) fn parse_postfix(&mut self) -> Result<CstNode, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek_kind() {
                // Function call
                TokenKind::LParen => {
                    expr = self.parse_function_call(expr)?;
                }
                // Index access
                TokenKind::LBracket => {
                    expr = self.parse_index_access(expr)?;
                }
                // Dot access
                TokenKind::Dot => {
                    expr = self.parse_member_access(expr)?;
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse function call expression.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// function_call = expression "(" arguments? ")"
    /// ```
    fn parse_function_call(&mut self, callee: CstNode) -> Result<CstNode, ParseError> {
        let start = callee.span.start;
        self.advance(); // (
        let args = self.parse_arguments()?;
        self.expect(TokenKind::RParen)?;

        Ok(CstNode::with_children(
            NodeKind::FunctionCall,
            self.span_from(start),
            vec![callee, args],
        ))
    }

    /// Parse index access expression.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// index_access = expression "[" expression "]"
    /// ```
    fn parse_index_access(&mut self, object: CstNode) -> Result<CstNode, ParseError> {
        let start = object.span.start;
        self.advance(); // [
        let index = self.parse_expression()?;
        self.expect(TokenKind::RBracket)?;

        Ok(CstNode::with_children(
            NodeKind::IndexExpression,
            self.span_from(start),
            vec![object, index],
        ))
    }

    /// Parse member access expression.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// member_access = expression "." identifier
    /// ```
    fn parse_member_access(&mut self, object: CstNode) -> Result<CstNode, ParseError> {
        let start = object.span.start;
        self.advance(); // .
        let name = self.expect(TokenKind::Identifier)?.clone();

        Ok(CstNode::with_children(
            NodeKind::DotExpression,
            self.span_from(start),
            vec![
                object,
                CstNode::with_text(NodeKind::Identifier, name.span, name.text),
            ],
        ))
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
    fn test_parse_function_call() {
        let expr = parse_expr("sin(x)");
        assert_eq!(expr.kind, NodeKind::FunctionCall);
        
        // First child should be identifier
        assert_eq!(expr.children[0].kind, NodeKind::Identifier);
        assert_eq!(expr.children[0].text_or_empty(), "sin");
    }

    #[test]
    fn test_parse_function_call_multiple_args() {
        let expr = parse_expr("max(a, b, c)");
        assert_eq!(expr.kind, NodeKind::FunctionCall);
        
        let args = &expr.children[1];
        assert_eq!(args.kind, NodeKind::Arguments);
        assert_eq!(args.children.len(), 3);
    }

    #[test]
    fn test_parse_index_access() {
        let expr = parse_expr("arr[0]");
        assert_eq!(expr.kind, NodeKind::IndexExpression);
        
        // First child should be identifier
        assert_eq!(expr.children[0].kind, NodeKind::Identifier);
        // Second child should be index
        assert_eq!(expr.children[1].kind, NodeKind::Number);
    }

    #[test]
    fn test_parse_member_access() {
        let expr = parse_expr("vec.x");
        assert_eq!(expr.kind, NodeKind::DotExpression);
        
        // First child should be object
        assert_eq!(expr.children[0].kind, NodeKind::Identifier);
        // Second child should be member name
        assert_eq!(expr.children[1].kind, NodeKind::Identifier);
        assert_eq!(expr.children[1].text_or_empty(), "x");
    }

    #[test]
    fn test_parse_chained_access() {
        let expr = parse_expr("arr[0].x");
        assert_eq!(expr.kind, NodeKind::DotExpression);
        
        // First child should be index expression
        assert_eq!(expr.children[0].kind, NodeKind::IndexExpression);
    }

    #[test]
    fn test_parse_chained_calls() {
        let expr = parse_expr("f(1)(2)");
        assert_eq!(expr.kind, NodeKind::FunctionCall);
        
        // First child should be another function call
        assert_eq!(expr.children[0].kind, NodeKind::FunctionCall);
    }
}

//! # Control Flow Parsing
//!
//! Parses control flow statements: for, if/else, let, blocks.
//!
//! ## Responsibilities
//!
//! - For loops: `for (i = [0:10]) { ... }`
//! - If/else: `if (x > 0) { ... } else { ... }`
//! - Let blocks: `let (x = 10) { ... }`
//! - Blocks: `{ ... }`
//!
//! ## Example
//!
//! ```rust,ignore
//! let node = parser.parse_for_block()?;
//! ```

use super::Parser;
use crate::cst::{CstNode, NodeKind};
use crate::error::ParseError;
use crate::lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse block.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// block = "{" statement* "}"
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// { cube(10); sphere(5); }
    /// ```
    pub(super) fn parse_block(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.expect(TokenKind::LBrace)?;

        let mut children = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => children.push(stmt),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(CstNode::with_children(NodeKind::Block, self.span_from(start), children))
    }

    /// Parse for loop block.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// for_block = "for" "(" for_assignments ")" statement
    /// for_assignments = for_assignment ("," for_assignment)*
    /// for_assignment = identifier "=" expression
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// for (i = [0:10]) cube(i);
    /// for (i = [0:10], j = [0:5]) translate([i, j, 0]) cube(1);
    /// ```
    pub(super) fn parse_for_block(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // for
        self.expect(TokenKind::LParen)?;
        
        // Parse for assignments
        let assignments = self.parse_for_assignments()?;
        
        self.expect(TokenKind::RParen)?;
        let body = self.parse_statement()?;
        
        Ok(CstNode::with_children(NodeKind::ForBlock, self.span_from(start), vec![assignments, body]))
    }

    /// Parse for loop assignments.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// for_assignments = for_assignment ("," for_assignment)*
    /// ```
    fn parse_for_assignments(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        let mut children = Vec::new();

        // First assignment
        children.push(self.parse_for_assignment()?);

        // Additional assignments
        while self.match_token(TokenKind::Comma) {
            children.push(self.parse_for_assignment()?);
        }

        Ok(CstNode::with_children(NodeKind::ForAssignments, self.span_from(start), children))
    }

    /// Parse single for assignment: identifier = expression
    ///
    /// ## Grammar
    ///
    /// ```text
    /// for_assignment = identifier "=" expression
    /// ```
    fn parse_for_assignment(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        
        let name_token = self.expect(TokenKind::Identifier)?;
        let name_span = name_token.span;
        let name_text = name_token.text.clone();
        
        self.expect(TokenKind::Eq)?;
        let value = self.parse_expression()?;

        Ok(CstNode::with_children(
            NodeKind::ForAssignment,
            self.span_from(start),
            vec![
                CstNode::with_text(NodeKind::Identifier, name_span, name_text),
                value,
            ],
        ))
    }

    /// Parse if/else block.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// if_block = "if" "(" expression ")" statement ("else" statement)?
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// if (x > 0) cube(x);
    /// if (x > 0) cube(x); else sphere(5);
    /// ```
    pub(super) fn parse_if_block(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // if
        self.expect(TokenKind::LParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RParen)?;
        let then_body = self.parse_statement()?;
        
        let mut children = vec![condition, then_body];
        
        if self.match_token(TokenKind::Else) {
            let else_body = self.parse_statement()?;
            children.push(else_body);
        }
        
        Ok(CstNode::with_children(NodeKind::IfBlock, self.span_from(start), children))
    }

    /// Parse let block.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// let_block = "let" "(" assignments ")" statement
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// let (x = 10) cube(x);
    /// let (x = 10, y = 20) translate([x, y, 0]) cube(5);
    /// ```
    pub(super) fn parse_let_block(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // let
        self.expect(TokenKind::LParen)?;
        
        // TODO: Parse let assignments properly
        while !self.check(TokenKind::RParen) && !self.is_at_end() {
            self.advance();
        }
        
        self.expect(TokenKind::RParen)?;
        let body = self.parse_statement()?;
        
        Ok(CstNode::with_children(NodeKind::LetBlock, self.span_from(start), vec![body]))
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

    fn parse(source: &str) -> crate::cst::Cst {
        let tokens = Lexer::new(source).tokenize();
        let mut parser = Parser::new(source, tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_block() {
        let cst = parse("{ cube(10); sphere(5); }");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let block = &cst.root.children[0];
        assert_eq!(block.kind, NodeKind::Block);
        assert_eq!(block.children.len(), 2);
    }

    #[test]
    fn test_parse_for_loop() {
        let cst = parse("for (i = [0:10]) cube(i);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let for_block = &cst.root.children[0];
        assert_eq!(for_block.kind, NodeKind::ForBlock);
        
        // Should have ForAssignments and body
        assert_eq!(for_block.children.len(), 2);
        assert_eq!(for_block.children[0].kind, NodeKind::ForAssignments);
    }

    #[test]
    fn test_parse_for_with_multiple_assignments() {
        let cst = parse("for (i = [0:10], j = [0:5]) cube(i);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let for_block = &cst.root.children[0];
        let assignments = &for_block.children[0];
        assert_eq!(assignments.children.len(), 2);
    }

    #[test]
    fn test_parse_if() {
        let cst = parse("if (true) cube(10);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let if_block = &cst.root.children[0];
        assert_eq!(if_block.kind, NodeKind::IfBlock);
        
        // condition + then_body
        assert_eq!(if_block.children.len(), 2);
    }

    #[test]
    fn test_parse_if_else() {
        let cst = parse("if (false) cube(10); else sphere(5);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let if_block = &cst.root.children[0];
        assert_eq!(if_block.kind, NodeKind::IfBlock);
        
        // condition + then_body + else_body
        assert_eq!(if_block.children.len(), 3);
    }

    #[test]
    fn test_parse_nested_if() {
        let cst = parse("if (x > 0) if (y > 0) cube(10);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let outer_if = &cst.root.children[0];
        assert_eq!(outer_if.kind, NodeKind::IfBlock);
        
        // Nested if should be in then_body
        let inner_if = &outer_if.children[1];
        assert_eq!(inner_if.kind, NodeKind::IfBlock);
    }
}

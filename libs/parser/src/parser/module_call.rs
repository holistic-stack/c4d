//! # Module Call Parsing
//!
//! Parses module calls and arguments.
//!
//! ## Responsibilities
//!
//! - Module call parsing: `cube(10);`
//! - Transform with child: `translate([1,2,3]) cube(5);`
//! - Argument parsing (positional and named)
//!
//! ## Example
//!
//! ```rust,ignore
//! let node = parser.parse_module_call(start, name_token)?;
//! ```

use super::Parser;
use crate::cst::{CstNode, NodeKind};
use crate::error::ParseError;
use crate::lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse module call.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// module_call = identifier "(" arguments? ")" (";" | block | statement)
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// cube(10);
    /// cube(10, center=true);
    /// translate([1,2,3]) cube(5);
    /// union() { cube(10); sphere(5); }
    /// ```
    pub(super) fn parse_module_call(
        &mut self,
        start: crate::span::Position,
        name_token: crate::lexer::Token,
    ) -> Result<CstNode, ParseError> {
        let mut children = Vec::new();

        // Module name
        children.push(CstNode::with_text(
            NodeKind::Identifier,
            name_token.span,
            name_token.text,
        ));

        // Arguments
        self.expect(TokenKind::LParen)?;
        let args = self.parse_arguments()?;
        children.push(args);
        self.expect(TokenKind::RParen)?;

        // Body: semicolon, block, or child statement
        if self.check(TokenKind::Semicolon) {
            self.advance();
        } else if self.check(TokenKind::LBrace) {
            let block = self.parse_block()?;
            children.push(block);
        } else {
            // Child statement (for transforms like translate)
            let child = self.parse_statement()?;
            children.push(child);
        }

        Ok(CstNode::with_children(NodeKind::ModuleCall, self.span_from(start), children))
    }

    /// Parse arguments list.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// arguments = (argument ("," argument)*)?
    /// argument = expression | identifier "=" expression
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// (10, 20, 30)
    /// (size=10, center=true)
    /// (5, $fn=32)
    /// ```
    pub(super) fn parse_arguments(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        let mut children = Vec::new();

        // Empty arguments
        if self.check(TokenKind::RParen) {
            return Ok(CstNode::with_children(NodeKind::Arguments, self.span_from(start), children));
        }

        // First argument
        children.push(self.parse_argument()?);

        // Remaining arguments
        while self.match_token(TokenKind::Comma) {
            // Allow trailing comma
            if self.check(TokenKind::RParen) {
                break;
            }
            children.push(self.parse_argument()?);
        }

        Ok(CstNode::with_children(NodeKind::Arguments, self.span_from(start), children))
    }

    /// Parse single argument.
    ///
    /// Handles both positional and named arguments.
    /// Named arguments can use identifiers or special variables ($fn, $fa, $fs).
    ///
    /// ## Grammar
    ///
    /// ```text
    /// argument = expression                    // positional
    ///          | identifier "=" expression     // named
    ///          | special_var "=" expression    // special ($fn=32)
    /// ```
    fn parse_argument(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();

        // Check for named argument: identifier "=" or special_variable "="
        let is_named = (self.check(TokenKind::Identifier) || self.check(TokenKind::SpecialVariable))
            && self.peek_next_is(TokenKind::Eq);

        if is_named {
            return self.parse_named_argument(start);
        }

        // Positional argument
        let expr = self.parse_expression()?;
        Ok(CstNode::with_children(NodeKind::Argument, self.span_from(start), vec![expr]))
    }

    /// Parse named argument (name=value).
    fn parse_named_argument(&mut self, start: crate::span::Position) -> Result<CstNode, ParseError> {
        let name = self.advance().clone();
        self.expect(TokenKind::Eq)?;
        let value = self.parse_expression()?;

        let name_kind = if name.kind == TokenKind::SpecialVariable {
            NodeKind::SpecialVariable
        } else {
            NodeKind::Identifier
        };

        Ok(CstNode::with_children(
            NodeKind::NamedArgument,
            self.span_from(start),
            vec![
                CstNode::with_text(name_kind, name.span, name.text),
                value,
            ],
        ))
    }

    /// Check if next token (after current) is of given kind.
    pub(super) fn peek_next_is(&self, kind: TokenKind) -> bool {
        self.tokens.get(self.current + 1)
            .map(|t| t.kind == kind)
            .unwrap_or(false)
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
    fn test_parse_module_call() {
        let cst = parse("cube(10);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let call = &cst.root.children[0];
        assert_eq!(call.kind, NodeKind::ModuleCall);
        
        // Check name
        let name = call.find_child(NodeKind::Identifier).unwrap();
        assert_eq!(name.text_or_empty(), "cube");
        
        // Check arguments
        let args = call.find_child(NodeKind::Arguments).unwrap();
        assert_eq!(args.children.len(), 1);
    }

    #[test]
    fn test_parse_named_argument() {
        let cst = parse("cube(10, center=true);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let call = &cst.root.children[0];
        let args = call.find_child(NodeKind::Arguments).unwrap();
        assert_eq!(args.children.len(), 2);
        
        // Second argument should be named
        assert_eq!(args.children[1].kind, NodeKind::NamedArgument);
    }

    #[test]
    fn test_parse_special_variable_argument() {
        let cst = parse("sphere(5, $fn=32);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let call = &cst.root.children[0];
        let args = call.find_child(NodeKind::Arguments).unwrap();
        
        // Second argument should be named with SpecialVariable
        let named = &args.children[1];
        assert_eq!(named.kind, NodeKind::NamedArgument);
        
        let name_node = named.find_child(NodeKind::SpecialVariable).unwrap();
        assert_eq!(name_node.text_or_empty(), "$fn");
    }

    #[test]
    fn test_parse_transform_with_child() {
        let cst = parse("translate([1,2,3]) cube(5);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let translate = &cst.root.children[0];
        assert_eq!(translate.kind, NodeKind::ModuleCall);
        
        // Should have child statement (cube)
        assert!(translate.children.len() >= 3);
    }

    #[test]
    fn test_parse_multiple_arguments() {
        let cst = parse("cylinder(10, 5, 3, center=true);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let call = &cst.root.children[0];
        let args = call.find_child(NodeKind::Arguments).unwrap();
        assert_eq!(args.children.len(), 4);
    }
}

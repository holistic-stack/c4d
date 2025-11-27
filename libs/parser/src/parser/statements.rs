//! # Statement Parsing
//!
//! Facade module for parsing OpenSCAD statements.
//!
//! ## Module Structure (SRP)
//!
//! - `module_call` - Module call and argument parsing
//! - `control_flow` - For, if/else, let, blocks
//! - `declarations` - Module/function declarations, assignments
//!
//! ## Grammar (from OpenSCAD)
//!
//! ```text
//! statement = module_call | assignment | for_block | if_block | block | ...
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! let node = parser.parse_statement()?;
//! ```

use super::Parser;
use crate::cst::{CstNode, NodeKind};
use crate::error::ParseError;
use crate::lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse a statement.
    ///
    /// Dispatches to the appropriate parsing function based on the current token.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// statement = module_call | assignment | for_block | if_block | block
    ///           | module_declaration | function_declaration
    ///           | include_statement | use_statement
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// cube(10);
    /// x = 10;
    /// for (i = [0:10]) cube(i);
    /// if (x > 0) cube(x);
    /// ```
    pub(super) fn parse_statement(&mut self) -> Result<CstNode, ParseError> {
        // Check for modifier (* ! # %)
        let modifier = self.parse_modifier();

        let stmt = match self.peek_kind() {
            // Declarations
            TokenKind::Module => self.parse_module_declaration(),
            TokenKind::Function => self.parse_function_declaration(),

            // Control flow
            TokenKind::For => self.parse_for_block(),
            TokenKind::If => self.parse_if_block(),
            TokenKind::Let => self.parse_let_block(),

            // Include/use
            TokenKind::Include => self.parse_include_statement(),
            TokenKind::Use => self.parse_use_statement(),

            // Block
            TokenKind::LBrace => self.parse_block(),

            // Identifier: could be module call or assignment
            TokenKind::Identifier | TokenKind::SpecialVariable => {
                self.parse_identifier_statement()
            }

            // Semicolon (empty statement)
            TokenKind::Semicolon => {
                let start = self.current_position();
                self.advance();
                Ok(CstNode::new(NodeKind::Semicolon, self.span_from(start)))
            }

            _ => {
                let token = self.peek().clone();
                Err(ParseError::unexpected_token(
                    &token.text,
                    "statement",
                ).with_span(token.span))
            }
        }?;

        // Wrap with modifier if present
        self.wrap_with_modifier(modifier, stmt)
    }

    /// Parse optional modifier (* ! # %).
    ///
    /// Modifiers change how geometry is rendered:
    /// - `*` - Disable (don't render)
    /// - `!` - Root (only render this)
    /// - `#` - Debug (highlight)
    /// - `%` - Background (transparent)
    fn parse_modifier(&mut self) -> Option<CstNode> {
        match self.peek_kind() {
            TokenKind::Star | TokenKind::Bang | TokenKind::Hash | TokenKind::Percent => {
                let start = self.current_position();
                let text = self.peek().text.clone();
                self.advance();
                Some(CstNode::with_text(NodeKind::Modifier, self.span_from(start), text))
            }
            _ => None,
        }
    }

    /// Wrap statement with modifier if present.
    fn wrap_with_modifier(&self, modifier: Option<CstNode>, stmt: CstNode) -> Result<CstNode, ParseError> {
        if let Some(mod_node) = modifier {
            let span = mod_node.span;
            let mut wrapper = CstNode::with_children(
                NodeKind::Modifier,
                span,
                vec![mod_node, stmt],
            );
            wrapper.span.end = self.previous().span.end;
            Ok(wrapper)
        } else {
            Ok(stmt)
        }
    }

    /// Parse statement starting with identifier.
    ///
    /// Could be module call `cube(10);` or assignment `x = 10;`
    fn parse_identifier_statement(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        let name_token = self.advance().clone();

        // Check for assignment
        if self.check(TokenKind::Eq) {
            return self.parse_assignment(start, name_token);
        }

        // Otherwise it's a module call
        self.parse_module_call(start, name_token)
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
    fn test_parse_statement_dispatch() {
        let cst = parse("cube(10);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        assert_eq!(cst.root.children[0].kind, NodeKind::ModuleCall);
    }

    #[test]
    fn test_parse_modifier() {
        let cst = parse("* cube(10);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        assert_eq!(cst.root.children[0].kind, NodeKind::Modifier);
    }

    #[test]
    fn test_parse_assignment() {
        let cst = parse("x = 10;");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let assign = &cst.root.children[0];
        assert_eq!(assign.kind, NodeKind::Assignment);
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
    fn test_parse_block() {
        let cst = parse("{ cube(10); sphere(5); }");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let block = &cst.root.children[0];
        assert_eq!(block.kind, NodeKind::Block);
        assert_eq!(block.children.len(), 2);
    }
}

//! # Statement Parsing
//!
//! Parses OpenSCAD statements: module calls, assignments, etc.
//!
//! ## Grammar (from grammar.js)
//!
//! ```text
//! statement = module_call | assignment | for_block | if_block | ...
//! module_call = identifier "(" arguments? ")" (";" | block)
//! ```

use super::Parser;
use crate::cst::{CstNode, NodeKind};
use crate::error::ParseError;
use crate::lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse a statement.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// statement = module_call | assignment | for_block | if_block | block
    /// ```
    ///
    /// ## Example
    ///
    /// ```rust
    /// // Parses: cube(10);
    /// let node = parser.parse_statement()?;
    /// assert_eq!(node.kind, NodeKind::ModuleCall);
    /// ```
    pub(super) fn parse_statement(&mut self) -> Result<CstNode, ParseError> {
        // Check for modifier (* ! # %)
        let modifier = self.parse_modifier();

        let stmt = match self.peek_kind() {
            // Module/function definition
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

    /// Parse optional modifier (* ! # %).
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

    /// Parse module call.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// module_call = identifier "(" arguments? ")" (";" | block)
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// cube(10);
    /// cube(10, center=true);
    /// translate([1,2,3]) cube(5);
    /// ```
    fn parse_module_call(&mut self, start: crate::span::Position, name_token: crate::lexer::Token) -> Result<CstNode, ParseError> {
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

        // Body: semicolon or block or child statement
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

    /// Parse assignment statement.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// assignment = identifier "=" expression ";"
    /// ```
    fn parse_assignment(&mut self, start: crate::span::Position, name_token: crate::lexer::Token) -> Result<CstNode, ParseError> {
        let mut children = Vec::new();

        // Variable name
        children.push(CstNode::with_text(
            NodeKind::Identifier,
            name_token.span,
            name_token.text,
        ));

        // =
        self.expect(TokenKind::Eq)?;

        // Value expression
        let value = self.parse_expression()?;
        children.push(value);

        // ;
        self.expect(TokenKind::Semicolon)?;

        Ok(CstNode::with_children(NodeKind::Assignment, self.span_from(start), children))
    }

    /// Parse arguments list.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// arguments = (argument ("," argument)*)?
    /// argument = expression | identifier "=" expression
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
    fn parse_argument(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();

        // Check for named argument: identifier "="
        if self.check(TokenKind::Identifier) && self.peek_next_is(TokenKind::Eq) {
            let name = self.advance().clone();
            self.expect(TokenKind::Eq)?;
            let value = self.parse_expression()?;

            let mut node = CstNode::with_children(
                NodeKind::NamedArgument,
                self.span_from(start),
                vec![
                    CstNode::with_text(NodeKind::Identifier, name.span, name.text),
                    value,
                ],
            );
            return Ok(node);
        }

        // Positional argument
        let expr = self.parse_expression()?;
        Ok(CstNode::with_children(NodeKind::Argument, self.span_from(start), vec![expr]))
    }

    /// Check if next token (after current) is of given kind.
    fn peek_next_is(&self, kind: TokenKind) -> bool {
        self.tokens.get(self.current + 1)
            .map(|t| t.kind == kind)
            .unwrap_or(false)
    }

    /// Parse block.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// block = "{" statement* "}"
    /// ```
    fn parse_block(&mut self) -> Result<CstNode, ParseError> {
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

    // =========================================================================
    // PLACEHOLDERS (to be implemented)
    // =========================================================================

    fn parse_module_declaration(&mut self) -> Result<CstNode, ParseError> {
        // TODO: Implement module declaration parsing
        let start = self.current_position();
        self.advance(); // module
        let name = self.expect(TokenKind::Identifier)?.clone();
        self.expect(TokenKind::LParen)?;
        // Skip parameters for now
        while !self.check(TokenKind::RParen) && !self.is_at_end() {
            self.advance();
        }
        self.expect(TokenKind::RParen)?;
        let body = self.parse_block()?;
        
        Ok(CstNode::with_children(
            NodeKind::ModuleDeclaration,
            self.span_from(start),
            vec![
                CstNode::with_text(NodeKind::Identifier, name.span, name.text),
                body,
            ],
        ))
    }

    fn parse_function_declaration(&mut self) -> Result<CstNode, ParseError> {
        // TODO: Implement function declaration parsing
        let start = self.current_position();
        self.advance(); // function
        let name = self.expect(TokenKind::Identifier)?.clone();
        self.expect(TokenKind::LParen)?;
        while !self.check(TokenKind::RParen) && !self.is_at_end() {
            self.advance();
        }
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Eq)?;
        let body = self.parse_expression()?;
        self.expect(TokenKind::Semicolon)?;
        
        Ok(CstNode::with_children(
            NodeKind::FunctionDeclaration,
            self.span_from(start),
            vec![
                CstNode::with_text(NodeKind::Identifier, name.span, name.text),
                body,
            ],
        ))
    }

    fn parse_for_block(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // for
        self.expect(TokenKind::LParen)?;
        // Skip for now - parse assignments
        while !self.check(TokenKind::RParen) && !self.is_at_end() {
            self.advance();
        }
        self.expect(TokenKind::RParen)?;
        let body = self.parse_statement()?;
        
        Ok(CstNode::with_children(NodeKind::ForBlock, self.span_from(start), vec![body]))
    }

    fn parse_if_block(&mut self) -> Result<CstNode, ParseError> {
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

    fn parse_let_block(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // let
        self.expect(TokenKind::LParen)?;
        while !self.check(TokenKind::RParen) && !self.is_at_end() {
            self.advance();
        }
        self.expect(TokenKind::RParen)?;
        let body = self.parse_statement()?;
        
        Ok(CstNode::with_children(NodeKind::LetBlock, self.span_from(start), vec![body]))
    }

    fn parse_include_statement(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // include
        // Skip path parsing for now
        while !self.check(TokenKind::Semicolon) && !self.is_at_end() {
            self.advance();
        }
        self.expect(TokenKind::Semicolon)?;
        
        Ok(CstNode::new(NodeKind::IncludeStatement, self.span_from(start)))
    }

    fn parse_use_statement(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // use
        while !self.check(TokenKind::Semicolon) && !self.is_at_end() {
            self.advance();
        }
        self.expect(TokenKind::Semicolon)?;
        
        Ok(CstNode::new(NodeKind::UseStatement, self.span_from(start)))
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use super::*;

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

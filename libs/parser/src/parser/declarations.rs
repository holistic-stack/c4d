//! # Declaration Parsing
//!
//! Parses declarations: modules, functions, assignments, include/use.
//!
//! ## Responsibilities
//!
//! - Module declarations: `module foo() { ... }`
//! - Function declarations: `function foo() = ...;`
//! - Variable assignments: `x = 10;`
//! - Include/use statements
//!
//! ## Example
//!
//! ```rust,ignore
//! let node = parser.parse_module_declaration()?;
//! ```

use super::Parser;
use crate::cst::{CstNode, NodeKind};
use crate::error::ParseError;
use crate::lexer::TokenKind;

impl<'a> Parser<'a> {
    /// Parse assignment statement.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// assignment = identifier "=" expression ";"
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// x = 10;
    /// size = [10, 20, 30];
    /// $fn = 32;
    /// ```
    pub(super) fn parse_assignment(
        &mut self,
        start: crate::span::Position,
        name_token: crate::lexer::Token,
    ) -> Result<CstNode, ParseError> {
        let mut children = Vec::new();

        // Variable name (could be identifier or special variable)
        let name_kind = if name_token.kind == TokenKind::SpecialVariable {
            NodeKind::SpecialVariable
        } else {
            NodeKind::Identifier
        };
        
        children.push(CstNode::with_text(name_kind, name_token.span, name_token.text));

        // =
        self.expect(TokenKind::Eq)?;

        // Value expression
        let value = self.parse_expression()?;
        children.push(value);

        // ;
        self.expect(TokenKind::Semicolon)?;

        Ok(CstNode::with_children(NodeKind::Assignment, self.span_from(start), children))
    }

    /// Parse module declaration.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// module_declaration = "module" identifier "(" parameters? ")" block
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// module foo() { cube(10); }
    /// module bar(size=10) { cube(size); }
    /// ```
    pub(super) fn parse_module_declaration(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // module
        
        let name = self.expect(TokenKind::Identifier)?.clone();
        self.expect(TokenKind::LParen)?;
        
        // TODO: Parse parameters properly
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

    /// Parse function declaration.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// function_declaration = "function" identifier "(" parameters? ")" "=" expression ";"
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// function foo() = 10;
    /// function bar(x) = x * 2;
    /// function add(a, b) = a + b;
    /// ```
    pub(super) fn parse_function_declaration(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // function
        
        let name = self.expect(TokenKind::Identifier)?.clone();
        self.expect(TokenKind::LParen)?;
        
        // Parse parameters
        let params = self.parse_parameters()?;
        
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Eq)?;
        let body = self.parse_expression()?;
        self.expect(TokenKind::Semicolon)?;
        
        Ok(CstNode::with_children(
            NodeKind::FunctionDeclaration,
            self.span_from(start),
            vec![
                CstNode::with_text(NodeKind::Identifier, name.span, name.text),
                params,
                body,
            ],
        ))
    }

    /// Parse function/module parameters.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// parameters = (parameter ("," parameter)*)?
    /// parameter = identifier ("=" expression)?
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// (x)
    /// (x, y)
    /// (x = 10)
    /// (x, y = 20)
    /// ```
    fn parse_parameters(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        let mut children = Vec::new();

        // Empty parameters
        if self.check(TokenKind::RParen) {
            return Ok(CstNode::with_children(NodeKind::Parameters, self.span_from(start), children));
        }

        // First parameter
        children.push(self.parse_parameter()?);

        // Remaining parameters
        while self.match_token(TokenKind::Comma) {
            // Allow trailing comma
            if self.check(TokenKind::RParen) {
                break;
            }
            children.push(self.parse_parameter()?);
        }

        Ok(CstNode::with_children(NodeKind::Parameters, self.span_from(start), children))
    }

    /// Parse single parameter.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// parameter = identifier ("=" expression)?
    /// ```
    fn parse_parameter(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        let name = self.expect(TokenKind::Identifier)?.clone();
        let name_node = CstNode::with_text(NodeKind::Identifier, name.span, name.text);

        // Check for default value
        if self.match_token(TokenKind::Eq) {
            let default = self.parse_expression()?;
            Ok(CstNode::with_children(
                NodeKind::Parameter,
                self.span_from(start),
                vec![name_node, default],
            ))
        } else {
            Ok(CstNode::with_children(
                NodeKind::Parameter,
                self.span_from(start),
                vec![name_node],
            ))
        }
    }

    /// Parse include statement.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// include_statement = "include" "<" path ">"
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// include <MCAD/boxes.scad>
    /// ```
    pub(super) fn parse_include_statement(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // include
        
        // TODO: Parse path properly
        while !self.check(TokenKind::Semicolon) && !self.is_at_end() {
            self.advance();
        }
        
        if self.check(TokenKind::Semicolon) {
            self.advance();
        }
        
        Ok(CstNode::new(NodeKind::IncludeStatement, self.span_from(start)))
    }

    /// Parse use statement.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// use_statement = "use" "<" path ">"
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// use <MCAD/boxes.scad>
    /// ```
    pub(super) fn parse_use_statement(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.advance(); // use
        
        // TODO: Parse path properly
        while !self.check(TokenKind::Semicolon) && !self.is_at_end() {
            self.advance();
        }
        
        if self.check(TokenKind::Semicolon) {
            self.advance();
        }
        
        Ok(CstNode::new(NodeKind::UseStatement, self.span_from(start)))
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
    fn test_parse_assignment() {
        let cst = parse("x = 10;");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let assign = &cst.root.children[0];
        assert_eq!(assign.kind, NodeKind::Assignment);
        
        // Should have identifier and value
        let name = assign.find_child(NodeKind::Identifier).unwrap();
        assert_eq!(name.text_or_empty(), "x");
    }

    #[test]
    fn test_parse_special_variable_assignment() {
        let cst = parse("$fn = 32;");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let assign = &cst.root.children[0];
        assert_eq!(assign.kind, NodeKind::Assignment);
        
        // Should have special variable
        let name = assign.find_child(NodeKind::SpecialVariable).unwrap();
        assert_eq!(name.text_or_empty(), "$fn");
    }

    #[test]
    fn test_parse_module_declaration() {
        let cst = parse("module foo() { cube(10); }");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let module = &cst.root.children[0];
        assert_eq!(module.kind, NodeKind::ModuleDeclaration);
        
        let name = module.find_child(NodeKind::Identifier).unwrap();
        assert_eq!(name.text_or_empty(), "foo");
        
        let body = module.find_child(NodeKind::Block).unwrap();
        assert!(!body.children.is_empty());
    }

    #[test]
    fn test_parse_function_declaration() {
        let cst = parse("function foo() = 10;");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let func = &cst.root.children[0];
        assert_eq!(func.kind, NodeKind::FunctionDeclaration);
        
        let name = func.find_child(NodeKind::Identifier).unwrap();
        assert_eq!(name.text_or_empty(), "foo");
    }

    #[test]
    fn test_parse_function_with_params() {
        let cst = parse("function double(x) = x * 2;");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let func = &cst.root.children[0];
        assert_eq!(func.kind, NodeKind::FunctionDeclaration);
        
        // Should have Parameters node
        let params = func.find_child(NodeKind::Parameters).expect("Should have Parameters");
        assert_eq!(params.children.len(), 1, "Should have 1 parameter");
        
        // Parameter should have identifier
        let param = &params.children[0];
        assert_eq!(param.kind, NodeKind::Parameter);
        let param_name = param.find_child(NodeKind::Identifier).unwrap();
        assert_eq!(param_name.text_or_empty(), "x");
    }

    #[test]
    fn test_parse_function_with_default_param() {
        let cst = parse("function size(x=10) = x;");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let func = &cst.root.children[0];
        let params = func.find_child(NodeKind::Parameters).expect("Should have Parameters");
        let param = &params.children[0];
        
        // Should have 2 children: identifier and default expression
        assert_eq!(param.children.len(), 2, "Should have name and default");
    }

    #[test]
    fn test_parse_list_assignment() {
        let cst = parse("size = [10, 20, 30];");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        let assign = &cst.root.children[0];
        assert_eq!(assign.kind, NodeKind::Assignment);
    }
}

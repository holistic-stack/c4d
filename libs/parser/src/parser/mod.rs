//! # OpenSCAD Parser
//!
//! Recursive descent parser for OpenSCAD.
//! Produces a Concrete Syntax Tree (CST).
//!
//! ## Example
//!
//! ```rust
//! use openscad_parser::parser::Parser;
//! use openscad_parser::lexer::Lexer;
//!
//! let tokens = Lexer::new("cube(10);").tokenize();
//! let mut parser = Parser::new("cube(10);", tokens);
//! let cst = parser.parse();
//! ```

mod statements;
mod expressions;

use crate::cst::{Cst, CstNode, NodeKind};
use crate::error::{ParseError, ParseErrorKind};
use crate::lexer::{Token, TokenKind};
use crate::span::{Position, Span};

// =============================================================================
// PARSER
// =============================================================================

/// Recursive descent parser for OpenSCAD.
///
/// ## Example
///
/// ```rust
/// let tokens = Lexer::new("cube(10);").tokenize();
/// let mut parser = Parser::new("cube(10);", tokens);
/// let cst = parser.parse();
/// assert!(cst.errors.is_empty());
/// ```
pub struct Parser<'a> {
    /// Source text (for error messages).
    source: &'a str,
    /// Token stream.
    tokens: Vec<Token>,
    /// Current token index.
    current: usize,
    /// Collected parse errors.
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    /// Create a new parser.
    ///
    /// ## Parameters
    ///
    /// - `source`: Original source text
    /// - `tokens`: Tokens from lexer
    ///
    /// ## Example
    ///
    /// ```rust
    /// let tokens = Lexer::new("cube(10);").tokenize();
    /// let parser = Parser::new("cube(10);", tokens);
    /// ```
    pub fn new(source: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    /// Parse the entire source into a CST.
    ///
    /// ## Returns
    ///
    /// CST with root node and any parse errors
    ///
    /// ## Example
    ///
    /// ```rust
    /// let cst = parser.parse();
    /// if cst.errors.is_empty() {
    ///     println!("Parsed successfully!");
    /// }
    /// ```
    pub fn parse(&mut self) -> Cst {
        let start = self.current_position();
        let mut children = Vec::new();

        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(node) => children.push(node),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize();
                }
            }
        }

        let end = self.current_position();
        let root = CstNode::with_children(NodeKind::SourceFile, Span::new(start, end), children);

        Cst::new(root, std::mem::take(&mut self.errors))
    }

    // =========================================================================
    // TOKEN ACCESS
    // =========================================================================

    /// Get current token.
    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or_else(|| {
            self.tokens.last().expect("Token stream should have at least EOF")
        })
    }

    /// Get current token kind.
    fn peek_kind(&self) -> TokenKind {
        self.peek().kind
    }

    /// Check if current token matches kind.
    fn check(&self, kind: TokenKind) -> bool {
        self.peek_kind() == kind
    }

    /// Check if at end of file.
    fn is_at_end(&self) -> bool {
        self.peek_kind() == TokenKind::Eof
    }

    /// Get current position.
    fn current_position(&self) -> Position {
        self.peek().span.start
    }

    /// Advance to next token.
    ///
    /// ## Returns
    ///
    /// The token that was consumed
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Get previous token.
    fn previous(&self) -> &Token {
        &self.tokens[self.current.saturating_sub(1)]
    }

    /// Consume token if it matches expected kind.
    ///
    /// ## Parameters
    ///
    /// - `kind`: Expected token kind
    ///
    /// ## Returns
    ///
    /// Ok with consumed token, or Err with parse error
    fn expect(&mut self, kind: TokenKind) -> Result<&Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken {
                    found: self.peek().text.clone(),
                    expected: kind.display().to_string(),
                },
                self.peek().span,
            ))
        }
    }

    /// Try to consume token if it matches.
    ///
    /// ## Parameters
    ///
    /// - `kind`: Token kind to match
    ///
    /// ## Returns
    ///
    /// true if token was consumed, false otherwise
    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    // =========================================================================
    // ERROR RECOVERY
    // =========================================================================

    /// Synchronize parser state after an error.
    ///
    /// Skips tokens until a statement boundary is found.
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            // Stop after semicolon
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }

            // Stop at statement starters
            match self.peek_kind() {
                TokenKind::Module
                | TokenKind::Function
                | TokenKind::For
                | TokenKind::If
                | TokenKind::Let
                | TokenKind::Include
                | TokenKind::Use
                | TokenKind::RBrace => return,
                _ => {}
            }

            self.advance();
        }
    }

    // =========================================================================
    // HELPERS
    // =========================================================================

    /// Create span from start to current position.
    fn span_from(&self, start: Position) -> Span {
        Span::new(start, self.previous().span.end)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Cst {
        let tokens = Lexer::new(source).tokenize();
        let mut parser = Parser::new(source, tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_empty() {
        let cst = parse("");
        assert!(cst.errors.is_empty());
        assert_eq!(cst.root.kind, NodeKind::SourceFile);
        assert!(cst.root.children.is_empty());
    }

    #[test]
    fn test_parse_simple_cube() {
        let cst = parse("cube(10);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        assert_eq!(cst.root.kind, NodeKind::SourceFile);
        assert_eq!(cst.root.children.len(), 1);
        assert_eq!(cst.root.children[0].kind, NodeKind::ModuleCall);
    }

    #[test]
    fn test_parse_cube_with_center() {
        let cst = parse("cube(10, center=true);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
    }

    #[test]
    fn test_parse_multiple_statements() {
        let cst = parse("cube(10); sphere(5);");
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        assert_eq!(cst.root.children.len(), 2);
    }

    #[test]
    fn test_parse_recovers_from_error() {
        let cst = parse("cube(; sphere(5);");
        // Should have errors but recover
        assert!(!cst.errors.is_empty());
        // Should still parse sphere
        assert!(!cst.root.children.is_empty());
    }
}

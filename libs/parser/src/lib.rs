//! # OpenSCAD Parser (Pure Rust)
//!
//! A pure Rust parser for OpenSCAD source code.
//! No C dependencies - compiles directly to WASM.
//!
//! ## Architecture
//!
//! ```text
//! Source Text → Lexer → Tokens → Parser → CST
//! ```
//!
//! ## Example
//!
//! ```rust
//! use openscad_parser::parse;
//!
//! let result = parse("cube(10);");
//! assert!(result.errors.is_empty());
//! ```
//!
//! ## Pipeline Integration
//!
//! This crate is the first layer in the OpenSCAD pipeline:
//!
//! ```text
//! openscad-parser → openscad-ast → openscad-eval → openscad-mesh → wasm
//! ```

pub mod lexer;
pub mod parser;
pub mod cst;
pub mod error;
pub mod span;

// Re-export public API
pub use cst::{Cst, CstNode, NodeKind};
pub use error::{ParseError, ParseErrorKind};
pub use span::{Position, Span, Spanned};

// =============================================================================
// PUBLIC API
// =============================================================================

/// Parse OpenSCAD source code into a Concrete Syntax Tree.
///
/// This is the main entry point for the parser.
///
/// ## Parameters
///
/// - `source`: OpenSCAD source code string
///
/// ## Returns
///
/// `Cst` containing the root node and any parse errors
///
/// ## Example
///
/// ```rust
/// use openscad_parser::parse;
///
/// let cst = parse("cube(10);");
/// assert!(cst.is_ok());
/// assert_eq!(cst.root.kind, openscad_parser::NodeKind::SourceFile);
/// ```
///
/// ## Error Handling
///
/// The parser attempts to recover from errors and continue parsing.
/// Errors are collected in `cst.errors`. Check `cst.is_ok()` for success.
///
/// ```rust
/// let cst = parse("cube(;"); // Syntax error
/// assert!(!cst.is_ok());
/// println!("Errors: {:?}", cst.errors);
/// ```
pub fn parse(source: &str) -> Cst {
    let tokens = lexer::Lexer::new(source).tokenize();
    let mut parser = parser::Parser::new(source, tokens);
    parser.parse()
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test parsing simple cube call.
    #[test]
    fn test_parse_cube() {
        let cst = parse("cube(10);");
        assert!(cst.is_ok(), "Expected no errors, got: {:?}", cst.errors);
        assert_eq!(cst.root.kind, NodeKind::SourceFile);
        assert!(!cst.root.children.is_empty(), "Expected children in source file");
    }

    /// Test parsing cube with center parameter.
    #[test]
    fn test_parse_cube_center() {
        let cst = parse("cube(10, center=true);");
        assert!(cst.is_ok(), "Expected no errors, got: {:?}", cst.errors);
    }

    /// Test parsing cube with array size.
    #[test]
    fn test_parse_cube_array() {
        let cst = parse("cube([10, 20, 30]);");
        assert!(cst.is_ok(), "Expected no errors, got: {:?}", cst.errors);
    }

    /// Test parsing multiple statements.
    #[test]
    fn test_parse_multiple() {
        let cst = parse("cube(10); sphere(5);");
        assert!(cst.is_ok(), "Expected no errors, got: {:?}", cst.errors);
        assert_eq!(cst.root.children.len(), 2);
    }

    /// Test parsing transform with child.
    #[test]
    fn test_parse_transform() {
        let cst = parse("translate([1, 2, 3]) cube(10);");
        assert!(cst.is_ok(), "Expected no errors, got: {:?}", cst.errors);
    }

    /// Test parsing union.
    #[test]
    fn test_parse_union() {
        let cst = parse("union() { cube(10); sphere(5); }");
        assert!(cst.is_ok(), "Expected no errors, got: {:?}", cst.errors);
    }

    /// Test error recovery.
    #[test]
    fn test_error_recovery() {
        let cst = parse("cube(; sphere(5);");
        // Should have errors
        assert!(!cst.is_ok());
        // But should still parse sphere
        assert!(!cst.root.children.is_empty());
    }
}

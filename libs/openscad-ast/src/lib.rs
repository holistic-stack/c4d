//! # OpenSCAD AST
//!
//! Abstract Syntax Tree types for OpenSCAD.
//!
//! ## Architecture
//!
//! ```text
//! Source → openscad-parser (CST) → openscad-ast (AST) → openscad-eval
//! ```
//!
//! ## Example
//!
//! ```rust
//! use openscad_ast::parse;
//!
//! let ast = parse("cube(10);").unwrap();
//! assert!(!ast.statements.is_empty());
//! ```

pub mod ast;
pub mod error;
pub mod visitor;

// Re-export public API
pub use ast::{Ast, Statement, Expression, Argument, BinaryOp, UnaryOp};
pub use error::AstError;
pub use openscad_parser::{Span, Position};

// =============================================================================
// PUBLIC API
// =============================================================================

/// Parse OpenSCAD source code into an Abstract Syntax Tree.
///
/// This is the main entry point for the AST crate.
///
/// ## Parameters
///
/// - `source`: OpenSCAD source code string
///
/// ## Returns
///
/// `Result<Ast, AstError>` - AST on success, error on failure
///
/// ## Example
///
/// ```rust
/// use openscad_ast::parse;
///
/// let ast = parse("cube(10);").unwrap();
/// assert_eq!(ast.statements.len(), 1);
/// ```
pub fn parse(source: &str) -> Result<Ast, AstError> {
    // Parse to CST using openscad-parser
    let cst = openscad_parser::parse(source);
    
    // Check for parse errors
    if !cst.is_ok() {
        return Err(AstError::ParseError(
            cst.errors.iter()
                .map(|e| format!("{}", e))
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    
    // Transform CST to AST using visitor
    visitor::cst_to_ast::transform(&cst)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test parsing simple cube.
    #[test]
    fn test_parse_cube() {
        let ast = parse("cube(10);").unwrap();
        assert_eq!(ast.statements.len(), 1);
        
        match &ast.statements[0] {
            Statement::ModuleCall { name, .. } => {
                assert_eq!(name, "cube");
            }
            _ => panic!("Expected ModuleCall"),
        }
    }

    /// Test parsing cube with named argument.
    #[test]
    fn test_parse_cube_center() {
        let ast = parse("cube(10, center=true);").unwrap();
        assert_eq!(ast.statements.len(), 1);
    }

    /// Test parsing multiple statements.
    #[test]
    fn test_parse_multiple() {
        let ast = parse("cube(10); sphere(5);").unwrap();
        assert_eq!(ast.statements.len(), 2);
    }

    /// Test parsing transform with child.
    #[test]
    fn test_parse_transform() {
        let ast = parse("translate([1, 2, 3]) cube(10);").unwrap();
        assert_eq!(ast.statements.len(), 1);
        
        match &ast.statements[0] {
            Statement::ModuleCall { name, children, .. } => {
                assert_eq!(name, "translate");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected ModuleCall"),
        }
    }
}

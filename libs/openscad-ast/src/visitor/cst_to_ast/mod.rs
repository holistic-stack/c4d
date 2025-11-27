//! # CST to AST Transformation
//!
//! Transforms Concrete Syntax Tree to Abstract Syntax Tree.
//!
//! ## Module Structure (SRP)
//!
//! - `statements` - Statement transformation facade
//! - `expressions` - Expression transformation facade
//! - `arguments` - Shared argument transformation
//! - `literals` - Literal transformations (number, string, boolean)
//! - `operators` - Operator transformations (binary, unary, ternary)
//! - `control_flow` - Control flow (for, if/else, blocks)
//! - `declarations` - Declarations (assignment, module, function)
//!
//! ## Example
//!
//! ```rust
//! use openscad_parser::parse as parse_cst;
//! use openscad_ast::visitor::cst_to_ast::transform;
//!
//! let cst = parse_cst("cube(10);");
//! let ast = transform(&cst).unwrap();
//! ```

// Core modules
mod statements;
mod expressions;

// SRP modules
mod arguments;
mod literals;
mod operators;
mod control_flow;
mod declarations;

use crate::ast::Ast;
use crate::error::AstError;
use openscad_parser::Cst;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Transform CST to AST.
///
/// ## Parameters
///
/// - `cst`: Concrete Syntax Tree from parser
///
/// ## Returns
///
/// `Result<Ast, AstError>` - AST on success, error on failure
///
/// ## Example
///
/// ```rust
/// let cst = openscad_parser::parse("cube(10);");
/// let ast = transform(&cst).unwrap();
/// assert_eq!(ast.statements.len(), 1);
/// ```
pub fn transform(cst: &Cst) -> Result<Ast, AstError> {
    let statements = statements::transform_statements(&cst.root.children)?;
    Ok(Ast::with_statements(statements))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use openscad_parser::parse as parse_cst;

    #[test]
    fn test_transform_cube() {
        let cst = parse_cst("cube(10);");
        let ast = transform(&cst).unwrap();
        assert_eq!(ast.statements.len(), 1);
    }

    #[test]
    fn test_transform_multiple() {
        let cst = parse_cst("cube(10); sphere(5);");
        let ast = transform(&cst).unwrap();
        assert_eq!(ast.statements.len(), 2);
    }
}

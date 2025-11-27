//! # OpenSCAD Eval
//!
//! AST evaluation and geometry IR generation.
//!
//! ## Architecture
//!
//! ```text
//! Source → openscad-ast (AST) → openscad-eval (GeometryNode) → openscad-mesh
//! ```
//!
//! ## Example
//!
//! ```rust
//! use openscad_eval::evaluate;
//!
//! let result = evaluate("cube(10);").unwrap();
//! // result.geometry is a GeometryNode::Cube
//! ```

pub mod geometry;
pub mod error;
pub mod visitor;
pub mod value;

// Re-export public API
pub use geometry::{GeometryNode, EvaluatedAst};
pub use error::EvalError;
pub use value::Value;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Evaluate OpenSCAD source code to geometry.
///
/// This is the main entry point for the evaluator.
///
/// ## Parameters
///
/// - `source`: OpenSCAD source code string
///
/// ## Returns
///
/// `Result<EvaluatedAst, EvalError>` - Evaluated geometry on success
///
/// ## Example
///
/// ```rust
/// use openscad_eval::evaluate;
///
/// let result = evaluate("cube(10);").unwrap();
/// ```
pub fn evaluate(source: &str) -> Result<EvaluatedAst, EvalError> {
    // Parse to AST using openscad-ast
    let ast = openscad_ast::parse(source)
        .map_err(|e| EvalError::ParseError(e.to_string()))?;
    
    // Evaluate AST to geometry
    visitor::evaluator::evaluate_ast(&ast)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test evaluating simple cube.
    #[test]
    fn test_evaluate_cube() {
        let result = evaluate("cube(10);").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, center } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
                assert!(!center);
            }
            _ => panic!("Expected Cube, got {:?}", result.geometry),
        }
    }

    /// Test evaluating cube with center.
    #[test]
    fn test_evaluate_cube_center() {
        let result = evaluate("cube(10, center=true);").unwrap();
        match result.geometry {
            GeometryNode::Cube { center, .. } => {
                assert!(center);
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test evaluating cube with array size.
    #[test]
    fn test_evaluate_cube_array() {
        let result = evaluate("cube([10, 20, 30]);").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 20.0, 30.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test evaluating union.
    #[test]
    fn test_evaluate_union() {
        let result = evaluate("union() { cube(10); cube(5); }").unwrap();
        match result.geometry {
            GeometryNode::Union { children } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected Union"),
        }
    }
}

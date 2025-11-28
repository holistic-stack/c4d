//! # Evaluation Visitors
//!
//! Visitor pattern implementations for AST evaluation.
//!
//! ## Module Structure (SRP)
//!
//! - `context` - Evaluator state and statement evaluation
//! - `expressions` - Expression evaluation
//! - `primitives` - 3D and 2D primitive evaluators
//! - `boolean` - Boolean operation evaluators
//! - `transforms` - Transform evaluators
//! - `extrusions` - Extrusion evaluators
//! - `ops_2d` - 2D operations (offset, projection)
//!
//! ## Example
//!
//! ```rust,ignore
//! use openscad_eval::visitor::evaluate_ast;
//! use openscad_ast::parse;
//!
//! let ast = parse("cube(10);").unwrap();
//! let result = evaluate_ast(&ast).unwrap();
//! ```

pub mod context;
pub mod expressions;
pub mod primitives;
pub mod boolean;
pub mod transforms;
pub mod extrusions;
pub mod ops_2d;

// Re-export public API
pub use context::{EvalContext, evaluate_statements};

use crate::error::EvalError;
use crate::geometry::EvaluatedAst;
use openscad_ast::Ast;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Evaluate AST to geometry.
///
/// This is the main entry point for AST evaluation.
///
/// ## Parameters
///
/// - `ast`: Abstract Syntax Tree from openscad-ast
///
/// ## Returns
///
/// `Result<EvaluatedAst, EvalError>` - Evaluated geometry tree with warnings
///
/// ## Example
///
/// ```rust,ignore
/// use openscad_eval::visitor::evaluate_ast;
/// use openscad_ast::parse;
///
/// let ast = parse("cube(10);").unwrap();
/// let result = evaluate_ast(&ast).unwrap();
/// ```
pub fn evaluate_ast(ast: &Ast) -> Result<EvaluatedAst, EvalError> {
    let mut ctx = EvalContext::new();
    let geometry = evaluate_statements(&mut ctx, &ast.statements)?;
    Ok(EvaluatedAst::with_warnings(geometry, ctx.warnings))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::GeometryNode;

    fn eval(source: &str) -> EvaluatedAst {
        let ast = openscad_ast::parse(source).unwrap();
        evaluate_ast(&ast).unwrap()
    }

    #[test]
    fn test_eval_cube() {
        let result = eval("cube(10);");
        match result.geometry {
            GeometryNode::Cube { size, center } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
                assert!(!center);
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_eval_cube_center() {
        let result = eval("cube(10, center=true);");
        match result.geometry {
            GeometryNode::Cube { center, .. } => assert!(center),
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_eval_cube_vec() {
        let result = eval("cube([10, 20, 30]);");
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 20.0, 30.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_eval_sphere() {
        let result = eval("sphere(r=5);");
        match result.geometry {
            GeometryNode::Sphere { radius, .. } => {
                assert_eq!(radius, 5.0);
            }
            _ => panic!("Expected Sphere"),
        }
    }

    #[test]
    fn test_eval_union() {
        let result = eval("union() { cube(10); sphere(5); }");
        match result.geometry {
            GeometryNode::Union { children } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected Union"),
        }
    }

    #[test]
    fn test_eval_translate() {
        let result = eval("translate([1, 2, 3]) cube(10);");
        match result.geometry {
            GeometryNode::Translate { offset, .. } => {
                assert_eq!(offset, [1.0, 2.0, 3.0]);
            }
            _ => panic!("Expected Translate"),
        }
    }
}

//! # 2D Operation Evaluators
//!
//! Evaluators for 2D operations like offset and projection.
//!
//! ## Operations
//!
//! - `offset(r)` or `offset(delta, chamfer)` - Expand/shrink 2D polygon
//! - `projection(cut)` - 3D to 2D projection

use crate::error::EvalError;
use crate::geometry::GeometryNode;
use openscad_ast::{Argument, Statement};

use super::context::{EvalContext, evaluate_statements};
use super::expressions::eval_expr;

// =============================================================================
// OFFSET
// =============================================================================

/// Evaluate offset() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// offset(r = number) child;
/// offset(delta = number, chamfer = false) child;
/// ```
///
/// ## Parameters
///
/// - `r`: Round offset (uses circular arcs at corners)
/// - `delta`: Straight offset (uses mitered or chamfered corners)
/// - `chamfer`: If true with delta, use beveled corners
///
/// ## Example
///
/// ```text
/// offset(r = 5) circle(10);
/// offset(delta = 2, chamfer = true) square(10);
/// ```
pub fn eval_offset(
    ctx: &mut EvalContext,
    args: &[Argument],
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let mut delta = 1.0;
    let mut chamfer = false;
    let mut use_radius = false;

    for arg in args {
        match arg {
            Argument::Positional(expr) => {
                // First positional arg is r (radius)
                delta = eval_expr(ctx, expr)?.as_number()?;
                use_radius = true;
            }
            Argument::Named { name, value } => match name.as_str() {
                "r" => {
                    delta = eval_expr(ctx, value)?.as_number()?;
                    use_radius = true;
                }
                "delta" => {
                    delta = eval_expr(ctx, value)?.as_number()?;
                    use_radius = false;
                }
                "chamfer" => {
                    chamfer = eval_expr(ctx, value)?.as_boolean();
                }
                _ => {}
            },
        }
    }

    // If using radius mode, chamfer is always false
    if use_radius {
        chamfer = false;
    }

    let child = evaluate_statements(ctx, children)?;
    Ok(GeometryNode::Offset {
        delta,
        chamfer,
        child: Box::new(child),
    })
}

// =============================================================================
// PROJECTION
// =============================================================================

/// Evaluate projection() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// projection(cut = false) child;
/// ```
///
/// ## Parameters
///
/// - `cut`: If true, project only the XY cross-section at Z=0
///
/// ## Example
///
/// ```text
/// projection() sphere(10);
/// projection(cut = true) cube([20, 20, 10]);
/// ```
pub fn eval_projection(
    ctx: &mut EvalContext,
    args: &[Argument],
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let mut cut = false;

    for arg in args {
        if let Argument::Named { name, value } = arg {
            if name == "cut" {
                cut = eval_expr(ctx, value)?.as_boolean();
            }
        }
    }

    let child = evaluate_statements(ctx, children)?;
    Ok(GeometryNode::Projection {
        cut,
        child: Box::new(child),
    })
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use openscad_ast::Expression;

    fn ctx() -> EvalContext {
        EvalContext::new()
    }

    #[test]
    fn test_eval_offset_default() {
        let mut ctx = ctx();
        let node = eval_offset(&mut ctx, &[], &[]).unwrap();
        match node {
            GeometryNode::Offset { delta, chamfer, .. } => {
                assert_eq!(delta, 1.0);
                assert!(!chamfer);
            }
            _ => panic!("Expected Offset"),
        }
    }

    #[test]
    fn test_eval_offset_with_r() {
        let mut ctx = ctx();
        let args = vec![Argument::Named {
            name: "r".to_string(),
            value: Expression::Number(5.0),
        }];
        let node = eval_offset(&mut ctx, &args, &[]).unwrap();
        match node {
            GeometryNode::Offset { delta, chamfer, .. } => {
                assert_eq!(delta, 5.0);
                assert!(!chamfer);
            }
            _ => panic!("Expected Offset"),
        }
    }

    #[test]
    fn test_eval_offset_with_delta_chamfer() {
        let mut ctx = ctx();
        let args = vec![
            Argument::Named {
                name: "delta".to_string(),
                value: Expression::Number(3.0),
            },
            Argument::Named {
                name: "chamfer".to_string(),
                value: Expression::Boolean(true),
            },
        ];
        let node = eval_offset(&mut ctx, &args, &[]).unwrap();
        match node {
            GeometryNode::Offset { delta, chamfer, .. } => {
                assert_eq!(delta, 3.0);
                assert!(chamfer);
            }
            _ => panic!("Expected Offset"),
        }
    }

    #[test]
    fn test_eval_projection_default() {
        let mut ctx = ctx();
        let node = eval_projection(&mut ctx, &[], &[]).unwrap();
        match node {
            GeometryNode::Projection { cut, .. } => {
                assert!(!cut);
            }
            _ => panic!("Expected Projection"),
        }
    }

    #[test]
    fn test_eval_projection_with_cut() {
        let mut ctx = ctx();
        let args = vec![Argument::Named {
            name: "cut".to_string(),
            value: Expression::Boolean(true),
        }];
        let node = eval_projection(&mut ctx, &args, &[]).unwrap();
        match node {
            GeometryNode::Projection { cut, .. } => {
                assert!(cut);
            }
            _ => panic!("Expected Projection"),
        }
    }
}

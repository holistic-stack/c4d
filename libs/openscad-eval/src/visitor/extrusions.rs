//! # Extrusion Evaluators
//!
//! Evaluators for 2D to 3D extrusion operations.
//!
//! ## Operations
//!
//! - `linear_extrude(height, twist, scale, slices, center)` - Extrude 2D shape along Z
//! - `rotate_extrude(angle, $fn)` - Rotate 2D shape around Z axis
//!
//! ## Example
//!
//! ```rust,ignore
//! let node = eval_linear_extrude(&mut ctx, &args, &children)?;
//! ```

use crate::error::EvalError;
use crate::geometry::GeometryNode;
use crate::value::Value;
use openscad_ast::{Argument, Statement};

use super::context::{EvalContext, evaluate_statements};
use super::expressions::eval_expr;

// =============================================================================
// EXTRUSIONS
// =============================================================================

/// Evaluate linear_extrude() call.
///
/// Extrudes a 2D shape along the Z axis.
///
/// ## OpenSCAD Signature
///
/// ```text
/// linear_extrude(height, center, convexity, twist, slices, scale) child;
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Extrusion arguments
/// - `children`: 2D child shapes to extrude
///
/// ## Example
///
/// ```text
/// linear_extrude(height=10, twist=90)
///     circle(5);
/// ```
pub fn eval_linear_extrude(
    ctx: &mut EvalContext,
    args: &[Argument],
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let mut height = 1.0;
    let mut twist = 0.0;
    let mut scale = [1.0, 1.0];
    let mut slices = 1;
    let mut center = false;

    for arg in args {
        match arg {
            Argument::Positional(expr) => {
                height = eval_expr(ctx, expr)?.as_number()?;
            }
            Argument::Named { name, value } => match name.as_str() {
                "height" => height = eval_expr(ctx, value)?.as_number()?,
                "twist" => twist = eval_expr(ctx, value)?.as_number()?,
                "scale" => scale = eval_expr(ctx, value)?.as_vec2()?,
                "slices" => slices = eval_expr(ctx, value)?.as_number()? as u32,
                "center" => center = eval_expr(ctx, value)?.as_boolean(),
                _ => {}
            },
        }
    }

    let child = evaluate_statements(ctx, children)?;
    Ok(GeometryNode::LinearExtrude {
        height,
        twist,
        scale,
        slices,
        center,
        child: Box::new(child),
    })
}

/// Evaluate rotate_extrude() call.
///
/// Rotates a 2D shape around the Z axis.
///
/// ## OpenSCAD Signature
///
/// ```text
/// rotate_extrude(angle, convexity, $fn, $fa, $fs) child;
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Extrusion arguments
/// - `children`: 2D child shapes to extrude
///
/// ## Example
///
/// ```text
/// rotate_extrude(angle=360, $fn=100)
///     translate([10, 0]) circle(3);
/// ```
pub fn eval_rotate_extrude(
    ctx: &mut EvalContext,
    args: &[Argument],
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let mut angle = 360.0;

    for arg in args {
        if let Argument::Named { name, value } = arg {
            match name.as_str() {
                "angle" => angle = eval_expr(ctx, value)?.as_number()?,
                "$fn" => {
                    let fn_val = eval_expr(ctx, value)?.as_number()?;
                    ctx.scope.define("$fn", Value::Number(fn_val));
                }
                _ => {}
            }
        }
    }

    let fn_ = ctx.scope.fn_value().max(3);
    let child = evaluate_statements(ctx, children)?;
    Ok(GeometryNode::RotateExtrude {
        angle,
        fn_,
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
    fn test_eval_linear_extrude_default() {
        let mut ctx = ctx();
        let node = eval_linear_extrude(&mut ctx, &[], &[]).unwrap();
        match node {
            GeometryNode::LinearExtrude { height, twist, scale, slices, center, .. } => {
                assert_eq!(height, 1.0);
                assert_eq!(twist, 0.0);
                assert_eq!(scale, [1.0, 1.0]);
                assert_eq!(slices, 1);
                assert!(!center);
            }
            _ => panic!("Expected LinearExtrude"),
        }
    }

    #[test]
    fn test_eval_linear_extrude_with_height() {
        let mut ctx = ctx();
        let args = vec![Argument::Named {
            name: "height".to_string(),
            value: Expression::Number(20.0),
        }];
        let node = eval_linear_extrude(&mut ctx, &args, &[]).unwrap();
        match node {
            GeometryNode::LinearExtrude { height, .. } => {
                assert_eq!(height, 20.0);
            }
            _ => panic!("Expected LinearExtrude"),
        }
    }

    #[test]
    fn test_eval_rotate_extrude_default() {
        let mut ctx = ctx();
        let node = eval_rotate_extrude(&mut ctx, &[], &[]).unwrap();
        match node {
            GeometryNode::RotateExtrude { angle, fn_, .. } => {
                assert_eq!(angle, 360.0);
                assert!(fn_ >= 3);
            }
            _ => panic!("Expected RotateExtrude"),
        }
    }

    #[test]
    fn test_eval_rotate_extrude_with_angle() {
        let mut ctx = ctx();
        let args = vec![Argument::Named {
            name: "angle".to_string(),
            value: Expression::Number(180.0),
        }];
        let node = eval_rotate_extrude(&mut ctx, &args, &[]).unwrap();
        match node {
            GeometryNode::RotateExtrude { angle, .. } => {
                assert_eq!(angle, 180.0);
            }
            _ => panic!("Expected RotateExtrude"),
        }
    }
}

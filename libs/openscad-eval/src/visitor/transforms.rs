//! # Transform Evaluators
//!
//! Evaluators for geometry transformations.
//!
//! ## Transforms
//!
//! - `translate([x, y, z])` - Move geometry
//! - `rotate([x, y, z])` - Rotate geometry
//! - `scale([x, y, z])` - Scale geometry
//! - `mirror([x, y, z])` - Mirror geometry
//! - `color([r, g, b, a])` - Color geometry
//!
//! ## Example
//!
//! ```rust,ignore
//! let node = eval_translate(&mut ctx, &args, &children)?;
//! ```

use crate::error::EvalError;
use crate::geometry::GeometryNode;
use openscad_ast::{Argument, Statement};

use super::context::{EvalContext, evaluate_statements};
use super::expressions::eval_expr;

// =============================================================================
// TRANSFORMS
// =============================================================================

/// Evaluate translate() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// translate([x, y, z]) child;
/// translate(v=[x, y, z]) child;
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Translation offset arguments
/// - `children`: Child statements to transform
pub fn eval_translate(
    ctx: &mut EvalContext,
    args: &[Argument],
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let offset = args.first()
        .map(|a| match a {
            Argument::Positional(e) => eval_expr(ctx, e),
            Argument::Named { value, .. } => eval_expr(ctx, value),
        })
        .transpose()?
        .map(|v| v.as_vec3())
        .transpose()?
        .unwrap_or([0.0, 0.0, 0.0]);

    let child = evaluate_statements(ctx, children)?;
    Ok(GeometryNode::Translate {
        offset,
        child: Box::new(child),
    })
}

/// Evaluate rotate() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// rotate([x, y, z]) child;       // Euler angles in degrees
/// rotate(a, v=[x, y, z]) child;  // Angle around axis (not yet supported)
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Rotation angles arguments
/// - `children`: Child statements to transform
pub fn eval_rotate(
    ctx: &mut EvalContext,
    args: &[Argument],
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let angles = args.first()
        .map(|a| match a {
            Argument::Positional(e) => eval_expr(ctx, e),
            Argument::Named { value, .. } => eval_expr(ctx, value),
        })
        .transpose()?
        .map(|v| v.as_vec3())
        .transpose()?
        .unwrap_or([0.0, 0.0, 0.0]);

    let child = evaluate_statements(ctx, children)?;
    Ok(GeometryNode::Rotate {
        angles,
        child: Box::new(child),
    })
}

/// Evaluate scale() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// scale([x, y, z]) child;
/// scale(v=[x, y, z]) child;
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Scale factor arguments
/// - `children`: Child statements to transform
pub fn eval_scale(
    ctx: &mut EvalContext,
    args: &[Argument],
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let factors = args.first()
        .map(|a| match a {
            Argument::Positional(e) => eval_expr(ctx, e),
            Argument::Named { value, .. } => eval_expr(ctx, value),
        })
        .transpose()?
        .map(|v| v.as_vec3())
        .transpose()?
        .unwrap_or([1.0, 1.0, 1.0]);

    let child = evaluate_statements(ctx, children)?;
    Ok(GeometryNode::Scale {
        factors,
        child: Box::new(child),
    })
}

/// Evaluate mirror() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// mirror([x, y, z]) child;
/// mirror(v=[x, y, z]) child;
/// ```
///
/// The vector specifies the normal of the mirror plane passing through origin.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Mirror plane normal arguments
/// - `children`: Child statements to transform
pub fn eval_mirror(
    ctx: &mut EvalContext,
    args: &[Argument],
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let normal = args.first()
        .map(|a| match a {
            Argument::Positional(e) => eval_expr(ctx, e),
            Argument::Named { value, .. } => eval_expr(ctx, value),
        })
        .transpose()?
        .map(|v| v.as_vec3())
        .transpose()?
        .unwrap_or([1.0, 0.0, 0.0]);

    let child = evaluate_statements(ctx, children)?;
    Ok(GeometryNode::Mirror {
        normal,
        child: Box::new(child),
    })
}

/// Evaluate color() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// color([r, g, b]) child;
/// color([r, g, b, a]) child;
/// color("colorname") child;  // Not yet supported
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Color arguments (RGBA, 0.0-1.0)
/// - `children`: Child statements to color
pub fn eval_color(
    ctx: &mut EvalContext,
    args: &[Argument],
    children: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let mut rgba = [1.0, 1.0, 1.0, 1.0];

    if let Some(arg) = args.first() {
        if let Argument::Positional(expr) = arg {
            let value = eval_expr(ctx, expr)?;
            let nums = value.as_number_list()?;
            for (i, n) in nums.iter().take(4).enumerate() {
                rgba[i] = *n;
            }
        }
    }

    let child = evaluate_statements(ctx, children)?;
    Ok(GeometryNode::Color {
        rgba,
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
    fn test_eval_translate_default() {
        let mut ctx = ctx();
        let node = eval_translate(&mut ctx, &[], &[]).unwrap();
        match node {
            GeometryNode::Translate { offset, .. } => {
                assert_eq!(offset, [0.0, 0.0, 0.0]);
            }
            _ => panic!("Expected Translate"),
        }
    }

    #[test]
    fn test_eval_translate_with_offset() {
        let mut ctx = ctx();
        let args = vec![Argument::Positional(Expression::List(vec![
            Expression::Number(1.0),
            Expression::Number(2.0),
            Expression::Number(3.0),
        ]))];
        let node = eval_translate(&mut ctx, &args, &[]).unwrap();
        match node {
            GeometryNode::Translate { offset, .. } => {
                assert_eq!(offset, [1.0, 2.0, 3.0]);
            }
            _ => panic!("Expected Translate"),
        }
    }

    #[test]
    fn test_eval_rotate_default() {
        let mut ctx = ctx();
        let node = eval_rotate(&mut ctx, &[], &[]).unwrap();
        match node {
            GeometryNode::Rotate { angles, .. } => {
                assert_eq!(angles, [0.0, 0.0, 0.0]);
            }
            _ => panic!("Expected Rotate"),
        }
    }

    #[test]
    fn test_eval_scale_default() {
        let mut ctx = ctx();
        let node = eval_scale(&mut ctx, &[], &[]).unwrap();
        match node {
            GeometryNode::Scale { factors, .. } => {
                assert_eq!(factors, [1.0, 1.0, 1.0]);
            }
            _ => panic!("Expected Scale"),
        }
    }

    #[test]
    fn test_eval_mirror_default() {
        let mut ctx = ctx();
        let node = eval_mirror(&mut ctx, &[], &[]).unwrap();
        match node {
            GeometryNode::Mirror { normal, .. } => {
                assert_eq!(normal, [1.0, 0.0, 0.0]);
            }
            _ => panic!("Expected Mirror"),
        }
    }

    #[test]
    fn test_eval_color_default() {
        let mut ctx = ctx();
        let node = eval_color(&mut ctx, &[], &[]).unwrap();
        match node {
            GeometryNode::Color { rgba, .. } => {
                assert_eq!(rgba, [1.0, 1.0, 1.0, 1.0]);
            }
            _ => panic!("Expected Color"),
        }
    }
}

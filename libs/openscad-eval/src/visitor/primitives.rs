//! # Primitive Evaluators
//!
//! Evaluators for 3D and 2D OpenSCAD primitives.
//!
//! ## 3D Primitives
//!
//! - `cube(size, center)` - Box primitive
//! - `sphere(r)` - Sphere primitive
//! - `cylinder(h, r1, r2, center)` - Cylinder/cone primitive
//!
//! ## 2D Primitives
//!
//! - `circle(r)` - Circle primitive
//! - `square(size, center)` - Rectangle primitive
//!
//! ## Example
//!
//! ```rust,ignore
//! let node = eval_cube(&mut ctx, &args)?;
//! ```

use crate::error::EvalError;
use crate::geometry::GeometryNode;
use crate::value::Value;
use openscad_ast::Argument;

use super::context::EvalContext;
use super::expressions::eval_expr;

// =============================================================================
// 3D PRIMITIVES
// =============================================================================

/// Evaluate cube() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// cube(size);
/// cube(size, center);
/// cube([x, y, z], center);
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Arguments from the module call
///
/// ## Example
///
/// ```text
/// cube(10);              // 10x10x10 cube at origin
/// cube([10, 20, 30]);    // Rectangular box
/// cube(10, center=true); // Centered cube
/// ```
pub fn eval_cube(ctx: &mut EvalContext, args: &[Argument]) -> Result<GeometryNode, EvalError> {
    let mut size = [1.0, 1.0, 1.0];
    let mut center = false;

    // Process arguments
    for (i, arg) in args.iter().enumerate() {
        match arg {
            Argument::Positional(expr) => {
                if i == 0 {
                    let value = eval_expr(ctx, expr)?;
                    size = value.as_vec3()?;
                } else if i == 1 {
                    center = eval_expr(ctx, expr)?.as_boolean();
                }
            }
            Argument::Named { name, value } => match name.as_str() {
                "size" => size = eval_expr(ctx, value)?.as_vec3()?,
                "center" => center = eval_expr(ctx, value)?.as_boolean(),
                _ => ctx.warn(format!("Unknown argument for cube: {}", name)),
            },
        }
    }

    Ok(GeometryNode::Cube { size, center })
}

/// Evaluate sphere() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// sphere(r);
/// sphere(r, $fn);
/// sphere(d=diameter);
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Arguments from the module call
pub fn eval_sphere(ctx: &mut EvalContext, args: &[Argument]) -> Result<GeometryNode, EvalError> {
    let mut radius = 1.0;

    for (i, arg) in args.iter().enumerate() {
        match arg {
            Argument::Positional(expr) => {
                if i == 0 {
                    radius = eval_expr(ctx, expr)?.as_number()?;
                }
            }
            Argument::Named { name, value } => match name.as_str() {
                "r" | "radius" => radius = eval_expr(ctx, value)?.as_number()?,
                "d" | "diameter" => radius = eval_expr(ctx, value)?.as_number()? / 2.0,
                "$fn" => {
                    let fn_val = eval_expr(ctx, value)?.as_number()?;
                    ctx.scope.define("$fn", Value::Number(fn_val));
                }
                _ => {}
            },
        }
    }

    let fn_ = ctx.calculate_fragments(radius);
    Ok(GeometryNode::Sphere { radius, fn_ })
}

/// Evaluate cylinder() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// cylinder(h, r, center);
/// cylinder(h, r1, r2, center);
/// cylinder(h, d, center);
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Arguments from the module call
pub fn eval_cylinder(ctx: &mut EvalContext, args: &[Argument]) -> Result<GeometryNode, EvalError> {
    let mut height = 1.0;
    let mut radius1 = 1.0;
    let mut radius2 = 1.0;
    let mut center = false;

    for (i, arg) in args.iter().enumerate() {
        match arg {
            Argument::Positional(expr) => {
                if i == 0 {
                    height = eval_expr(ctx, expr)?.as_number()?;
                } else if i == 1 {
                    let r = eval_expr(ctx, expr)?.as_number()?;
                    radius1 = r;
                    radius2 = r;
                }
            }
            Argument::Named { name, value } => match name.as_str() {
                "h" | "height" => height = eval_expr(ctx, value)?.as_number()?,
                "r" | "radius" => {
                    let r = eval_expr(ctx, value)?.as_number()?;
                    radius1 = r;
                    radius2 = r;
                }
                "r1" => radius1 = eval_expr(ctx, value)?.as_number()?,
                "r2" => radius2 = eval_expr(ctx, value)?.as_number()?,
                "d" | "diameter" => {
                    let r = eval_expr(ctx, value)?.as_number()? / 2.0;
                    radius1 = r;
                    radius2 = r;
                }
                "d1" => radius1 = eval_expr(ctx, value)?.as_number()? / 2.0,
                "d2" => radius2 = eval_expr(ctx, value)?.as_number()? / 2.0,
                "center" => center = eval_expr(ctx, value)?.as_boolean(),
                "$fn" => {
                    let fn_val = eval_expr(ctx, value)?.as_number()?;
                    ctx.scope.define("$fn", Value::Number(fn_val));
                }
                _ => {}
            },
        }
    }

    let fn_ = ctx.calculate_fragments(radius1.max(radius2));
    Ok(GeometryNode::Cylinder {
        height,
        radius1,
        radius2,
        center,
        fn_,
    })
}

// =============================================================================
// 2D PRIMITIVES
// =============================================================================

/// Evaluate circle() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// circle(r);
/// circle(r, $fn);
/// circle(d=diameter);
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Arguments from the module call
pub fn eval_circle(ctx: &mut EvalContext, args: &[Argument]) -> Result<GeometryNode, EvalError> {
    let mut radius = 1.0;

    for (i, arg) in args.iter().enumerate() {
        match arg {
            Argument::Positional(expr) => {
                if i == 0 {
                    radius = eval_expr(ctx, expr)?.as_number()?;
                }
            }
            Argument::Named { name, value } => match name.as_str() {
                "r" | "radius" => radius = eval_expr(ctx, value)?.as_number()?,
                "d" | "diameter" => radius = eval_expr(ctx, value)?.as_number()? / 2.0,
                "$fn" => {
                    let fn_val = eval_expr(ctx, value)?.as_number()?;
                    ctx.scope.define("$fn", Value::Number(fn_val));
                }
                _ => {}
            },
        }
    }

    let fn_ = ctx.calculate_fragments(radius);
    Ok(GeometryNode::Circle { radius, fn_ })
}

/// Evaluate square() call.
///
/// ## OpenSCAD Signature
///
/// ```text
/// square(size);
/// square(size, center);
/// square([x, y], center);
/// ```
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `args`: Arguments from the module call
pub fn eval_square(ctx: &mut EvalContext, args: &[Argument]) -> Result<GeometryNode, EvalError> {
    let mut size = [1.0, 1.0];
    let mut center = false;

    for (i, arg) in args.iter().enumerate() {
        match arg {
            Argument::Positional(expr) => {
                if i == 0 {
                    size = eval_expr(ctx, expr)?.as_vec2()?;
                } else if i == 1 {
                    center = eval_expr(ctx, expr)?.as_boolean();
                }
            }
            Argument::Named { name, value } => match name.as_str() {
                "size" => size = eval_expr(ctx, value)?.as_vec2()?,
                "center" => center = eval_expr(ctx, value)?.as_boolean(),
                _ => {}
            },
        }
    }

    Ok(GeometryNode::Square { size, center })
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
    fn test_eval_cube_default() {
        let mut ctx = ctx();
        let node = eval_cube(&mut ctx, &[]).unwrap();
        match node {
            GeometryNode::Cube { size, center } => {
                assert_eq!(size, [1.0, 1.0, 1.0]);
                assert!(!center);
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_eval_cube_size() {
        let mut ctx = ctx();
        let args = vec![Argument::Positional(Expression::Number(10.0))];
        let node = eval_cube(&mut ctx, &args).unwrap();
        match node {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_eval_sphere_default() {
        let mut ctx = ctx();
        let node = eval_sphere(&mut ctx, &[]).unwrap();
        match node {
            GeometryNode::Sphere { radius, fn_ } => {
                assert_eq!(radius, 1.0);
                assert!(fn_ >= 3);
            }
            _ => panic!("Expected Sphere"),
        }
    }

    #[test]
    fn test_eval_cylinder_default() {
        let mut ctx = ctx();
        let node = eval_cylinder(&mut ctx, &[]).unwrap();
        match node {
            GeometryNode::Cylinder { height, radius1, radius2, center, .. } => {
                assert_eq!(height, 1.0);
                assert_eq!(radius1, 1.0);
                assert_eq!(radius2, 1.0);
                assert!(!center);
            }
            _ => panic!("Expected Cylinder"),
        }
    }

    #[test]
    fn test_eval_circle_default() {
        let mut ctx = ctx();
        let node = eval_circle(&mut ctx, &[]).unwrap();
        match node {
            GeometryNode::Circle { radius, fn_ } => {
                assert_eq!(radius, 1.0);
                assert!(fn_ >= 3);
            }
            _ => panic!("Expected Circle"),
        }
    }

    #[test]
    fn test_eval_square_default() {
        let mut ctx = ctx();
        let node = eval_square(&mut ctx, &[]).unwrap();
        match node {
            GeometryNode::Square { size, center } => {
                assert_eq!(size, [1.0, 1.0]);
                assert!(!center);
            }
            _ => panic!("Expected Square"),
        }
    }
}

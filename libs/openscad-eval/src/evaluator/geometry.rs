use crate::context::EvaluationContext;
use crate::error::EvalError;
use crate::ir::{BooleanOperation, GeometryNode, OffsetAmount};
use crate::resolution::{compute_segments, compute_segments_with_override};
use config::constants::DEFAULT_CONVEXITY;
use glam::{DMat4, DVec3};
use openscad_ast::{Expression, OffsetAmount as AstOffsetAmount, Span, Statement};
use super::expressions::{eval_expr_f64, eval_vec3};
use super::evaluate_children;

// 3D Primitives

pub fn eval_cube(size: &Expression, center: bool, span: Span, ctx: &EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let size_val = eval_vec3(size, ctx);
    Ok(Some(GeometryNode::Cube {
        size: size_val,
        center,
        span,
    }))
}

pub fn eval_sphere(radius: &Expression, fn_override: Option<f64>, span: Span, ctx: &EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let r = eval_expr_f64(radius, ctx).unwrap_or(1.0);
    let segments = compute_segments_with_override(r, fn_override, ctx);
    Ok(Some(GeometryNode::Sphere {
        radius: r,
        segments,
        span,
    }))
}

pub fn eval_cylinder(
    height: &Expression,
    radius_bottom: &Expression,
    radius_top: &Expression,
    center: bool,
    fn_override: Option<f64>,
    span: Span,
    ctx: &EvaluationContext
) -> Result<Option<GeometryNode>, EvalError> {
    let h = eval_expr_f64(height, ctx).unwrap_or(1.0);
    let r_bottom = eval_expr_f64(radius_bottom, ctx).unwrap_or(1.0);
    let r_top = eval_expr_f64(radius_top, ctx).unwrap_or(1.0);
    let max_radius = r_bottom.max(r_top);
    let segments = compute_segments_with_override(max_radius, fn_override, ctx);
    Ok(Some(GeometryNode::Cylinder {
        height: h,
        radius_bottom: r_bottom,
        radius_top: r_top,
        center,
        segments,
        span,
    }))
}

pub fn eval_polyhedron(
    points: &Vec<DVec3>,
    faces: &Vec<Vec<u32>>,
    convexity: u32,
    span: Span
) -> Result<Option<GeometryNode>, EvalError> {
    Ok(Some(GeometryNode::Polyhedron {
        points: points.clone(),
        faces: faces.clone(),
        convexity,
        span,
    }))
}

// 2D Primitives

pub fn eval_square(size: &[f64; 2], center: bool, span: Span) -> Result<Option<GeometryNode>, EvalError> {
    Ok(Some(GeometryNode::Square {
        size: *size,
        center,
        span,
    }))
}

pub fn eval_circle(radius: f64, _segments: u32, span: Span, ctx: &EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    // Note: segments from AST is ignored here because we compute it from $fn/$fa/$fs
    // unless the AST segments IS the resolved $fn?
    // In AST, Circle has `segments: u32`.
    // If it's passed, maybe we should use it if it's non-zero?
    // For now, use compute_segments.
    let segments = compute_segments(radius, ctx);
    Ok(Some(GeometryNode::Circle {
        radius,
        segments,
        span,
    }))
}

pub fn eval_polygon(points: &Vec<[f64; 2]>, paths: &Option<Vec<Vec<u32>>>, span: Span) -> Result<Option<GeometryNode>, EvalError> {
    Ok(Some(GeometryNode::Polygon {
        points: points.clone(),
        paths: paths.clone(),
        span,
    }))
}

// Transformations

pub fn eval_translate(vector: &Expression, children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let v = eval_vec3(vector, ctx);
    let matrix = DMat4::from_translation(v);
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Transform {
        matrix,
        children: child_nodes,
        span,
    }))
}

pub fn eval_rotate(angles: &Expression, axis: &Option<Expression>, children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let ang = eval_vec3(angles, ctx);
    let matrix = if let Some(ax_expr) = axis {
        let ax = eval_vec3(ax_expr, ctx);
        let angle_rad = eval_expr_f64(angles, ctx).unwrap_or(0.0).to_radians();
        DMat4::from_axis_angle(ax.normalize(), angle_rad)
    } else {
        let rx = DMat4::from_rotation_x(ang.x.to_radians());
        let ry = DMat4::from_rotation_y(ang.y.to_radians());
        let rz = DMat4::from_rotation_z(ang.z.to_radians());
        rz * ry * rx
    };
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Transform {
        matrix,
        children: child_nodes,
        span,
    }))
}

pub fn eval_scale(factors: &Expression, children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let f = eval_vec3(factors, ctx);
    let matrix = DMat4::from_scale(f);
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Transform {
        matrix,
        children: child_nodes,
        span,
    }))
}

pub fn eval_mirror(normal: &Expression, children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let n = eval_vec3(normal, ctx).normalize();
    let matrix = DMat4::from_cols_array(&[
        1.0 - 2.0 * n.x * n.x, -2.0 * n.x * n.y, -2.0 * n.x * n.z, 0.0,
        -2.0 * n.y * n.x, 1.0 - 2.0 * n.y * n.y, -2.0 * n.y * n.z, 0.0,
        -2.0 * n.z * n.x, -2.0 * n.z * n.y, 1.0 - 2.0 * n.z * n.z, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]);
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Transform {
        matrix,
        children: child_nodes,
        span,
    }))
}

pub fn eval_multmatrix(matrix: &[[f64; 4]; 4], children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let m = DMat4::from_cols_array(&[
        matrix[0][0], matrix[1][0], matrix[2][0], matrix[3][0],
        matrix[0][1], matrix[1][1], matrix[2][1], matrix[3][1],
        matrix[0][2], matrix[1][2], matrix[2][2], matrix[3][2],
        matrix[0][3], matrix[1][3], matrix[2][3], matrix[3][3],
    ]);
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Transform {
        matrix: m,
        children: child_nodes,
        span,
    }))
}

pub fn eval_resize(new_size: &DVec3, auto_scale: &[bool; 3], children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Resize {
        new_size: *new_size,
        auto_scale: *auto_scale,
        convexity: DEFAULT_CONVEXITY,
        children: child_nodes,
        span,
    }))
}

pub fn eval_color(color: &[f32; 4], children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Color {
        color: *color,
        children: child_nodes,
        span,
    }))
}

// Boolean Operations

pub fn eval_boolean(op: BooleanOperation, children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Boolean {
        operation: op,
        children: child_nodes,
        span,
    }))
}

// Extrusions

pub fn eval_linear_extrude(
    height: f64,
    center: bool,
    twist: f64,
    slices: u32,
    scale: &[f64; 2],
    children: &[Statement],
    span: Span,
    ctx: &mut EvaluationContext
) -> Result<Option<GeometryNode>, EvalError> {
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::LinearExtrude {
        height,
        center,
        twist,
        slices,
        scale: *scale,
        children: child_nodes,
        span,
    }))
}

pub fn eval_rotate_extrude(
    angle: f64,
    convexity: u32,
    children: &[Statement],
    span: Span,
    ctx: &mut EvaluationContext
) -> Result<Option<GeometryNode>, EvalError> {
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::RotateExtrude {
        angle,
        convexity,
        children: child_nodes,
        span,
    }))
}

// Advanced

pub fn eval_hull(children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Hull {
        children: child_nodes,
        span,
    }))
}

pub fn eval_minkowski(convexity: u32, children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Minkowski {
        convexity,
        children: child_nodes,
        span,
    }))
}

pub fn eval_offset(amount: &AstOffsetAmount, chamfer: bool, children: &[Statement], span: Span, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    let offset_amount = match amount {
        AstOffsetAmount::Radius(r) => OffsetAmount::Radius(*r),
        AstOffsetAmount::Delta(d) => OffsetAmount::Delta(*d),
    };
    
    let child_nodes = evaluate_children(children, ctx)?;
    Ok(Some(GeometryNode::Offset {
        amount: offset_amount,
        chamfer,
        children: child_nodes,
        span,
    }))
}

pub mod expressions;
pub mod geometry;
pub mod control_flow;
pub mod modules;

use crate::context::EvaluationContext;
use crate::error::EvalError;
use crate::ir::GeometryNode;
use openscad_ast::{SerializedNode, Statement};

/// Evaluates OpenSCAD source code and returns geometry IR.
#[cfg(feature = "native-parser")]
pub fn evaluate(source: &str) -> Result<Vec<GeometryNode>, EvalError> {
    let statements = openscad_ast::parse_to_ast(source)?;
    let mut ctx = EvaluationContext::new();
    evaluate_statements(&statements, &mut ctx)
}

/// Evaluates a serialized CST and returns geometry IR.
pub fn evaluate_from_cst(cst: &SerializedNode) -> Result<Vec<GeometryNode>, EvalError> {
    let statements = openscad_ast::parse_from_cst(cst)?;
    let mut ctx = EvaluationContext::new();
    evaluate_statements(&statements, &mut ctx)
}

/// Evaluates a list of statements with the given context.
pub fn evaluate_statements(
    statements: &[Statement],
    ctx: &mut EvaluationContext,
) -> Result<Vec<GeometryNode>, EvalError> {
    // Pass 1: Register definitions (hoisting)
    for stmt in statements {
        match stmt {
            Statement::ModuleDefinition { name, parameters, body, .. } => {
                modules::register_module(name, parameters, body, ctx)?;
            }
            Statement::FunctionDefinition { name, parameters, body, .. } => {
                modules::register_function(name, parameters, body, ctx)?;
            }
            _ => {}
        }
    }

    // Pass 2: Evaluate geometry and assignments
    let mut nodes = Vec::new();
    for stmt in statements {
        match stmt {
            Statement::ModuleDefinition { .. } | Statement::FunctionDefinition { .. } => {
                continue;
            }
            _ => {
                if let Some(node) = evaluate_statement(stmt, ctx)? {
                    nodes.push(node);
                }
            }
        }
    }
    Ok(nodes)
}

fn evaluate_statement(
    stmt: &Statement,
    ctx: &mut EvaluationContext,
) -> Result<Option<GeometryNode>, EvalError> {
    match stmt {
        Statement::Cube { size, center, span } => geometry::eval_cube(size, *center, *span, ctx),
        Statement::Sphere { radius, fn_override, span } => geometry::eval_sphere(radius, *fn_override, *span, ctx),
        Statement::Cylinder { height, radius_bottom, radius_top, center, fn_override, span } => 
            geometry::eval_cylinder(height, radius_bottom, radius_top, *center, *fn_override, *span, ctx),
        Statement::Polyhedron { points, faces, convexity, span } => 
            geometry::eval_polyhedron(points, faces, *convexity, *span),
        Statement::Square { size, center, span } => geometry::eval_square(size, *center, *span),
        Statement::Circle { radius, segments, span } => geometry::eval_circle(*radius, *segments, *span, ctx),
        Statement::Polygon { points, paths, span } => geometry::eval_polygon(points, paths, *span),
        
        Statement::Translate { vector, children, span } => geometry::eval_translate(vector, children, *span, ctx),
        Statement::Rotate { angles, axis, children, span } => geometry::eval_rotate(angles, axis, children, *span, ctx),
        Statement::Scale { factors, children, span } => geometry::eval_scale(factors, children, *span, ctx),
        Statement::Mirror { normal, children, span } => geometry::eval_mirror(normal, children, *span, ctx),
        Statement::Multmatrix { matrix, children, span } => geometry::eval_multmatrix(matrix, children, *span, ctx),
        Statement::Resize { new_size, auto_scale, children, span } => geometry::eval_resize(new_size, auto_scale, children, *span, ctx),
        Statement::Color { color, children, span } => geometry::eval_color(color, children, *span, ctx),
        
        Statement::Union { children, span } => geometry::eval_boolean(crate::ir::BooleanOperation::Union, children, *span, ctx),
        Statement::Difference { children, span } => geometry::eval_boolean(crate::ir::BooleanOperation::Difference, children, *span, ctx),
        Statement::Intersection { children, span } => geometry::eval_boolean(crate::ir::BooleanOperation::Intersection, children, *span, ctx),
        
        Statement::LinearExtrude { height, center, twist, slices, scale, children, span } => 
            geometry::eval_linear_extrude(*height, *center, *twist, *slices, scale, children, *span, ctx),
        Statement::RotateExtrude { angle, convexity, children, span } => 
            geometry::eval_rotate_extrude(*angle, *convexity, children, *span, ctx),
            
        Statement::Hull { children, span } => geometry::eval_hull(children, *span, ctx),
        Statement::Minkowski { convexity, children, span } => geometry::eval_minkowski(*convexity, children, *span, ctx),
        Statement::Offset { amount, chamfer, children, span } => geometry::eval_offset(amount, *chamfer, children, *span, ctx),
        
        Statement::Assignment { name, value, .. } => modules::eval_assignment(name, value, ctx),
        Statement::ModuleCall { name, arguments, children, span } => modules::eval_module_call(name, arguments, children, *span, ctx),
        
        Statement::ForLoop { variable, range, body, span } => control_flow::eval_for_loop(variable, range, body, *span, ctx),
        Statement::If { condition, then_branch, else_branch, span } => control_flow::eval_if(condition, then_branch, else_branch, *span, ctx),
        
        Statement::ModuleDefinition { name, parameters, body, .. } => modules::register_module(name, parameters, body, ctx),
        Statement::FunctionDefinition { name, parameters, body, .. } => modules::register_function(name, parameters, body, ctx),

        Statement::Echo { .. } => Ok(None),
        Statement::Modified { modifier, child, span } => {
            use openscad_ast::Modifier;
            match modifier {
                Modifier::Disable => Ok(None),
                Modifier::ShowOnly => evaluate_statement(child, ctx),
                Modifier::Highlight => {
                    if let Some(geometry) = evaluate_statement(child, ctx)? {
                        Ok(Some(GeometryNode::Color {
                            color: [1.0, 0.0, 1.0, 0.5],
                            children: vec![geometry],
                            span: *span,
                        }))
                    } else {
                        Ok(None)
                    }
                }
                Modifier::Transparent => {
                    if let Some(geometry) = evaluate_statement(child, ctx)? {
                        Ok(Some(GeometryNode::Color {
                            color: [0.0, 0.5, 0.5, 0.3],
                            children: vec![geometry],
                            span: *span,
                        }))
                    } else {
                        Ok(None)
                    }
                }
            }
        }
    }
}

pub fn evaluate_children(
    children: &[Statement],
    ctx: &mut EvaluationContext,
) -> Result<Vec<GeometryNode>, EvalError> {
    ctx.push_scope();
    let result = evaluate_statements(children, ctx);
    ctx.pop_scope();
    result
}

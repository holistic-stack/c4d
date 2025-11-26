//! # Evaluator
//!
//! Transforms OpenSCAD AST into Geometry IR by evaluating expressions,
//! resolving variables, and computing transforms.
//!
//! ## Architecture
//!
//! ### Native (CLI/Server)
//!
//! ```text
//! OpenSCAD Source → parse_to_ast → evaluate_statements → Geometry IR
//! ```
//!
//! ### Browser (WASM)
//!
//! ```text
//! Serialized CST → parse_from_cst → evaluate_statements → Geometry IR
//! ```

use crate::context::{EvaluationContext, FunctionDefinition, ModuleDefinition};
use crate::error::EvalError;
use crate::ir::{BooleanOperation, GeometryNode, OffsetAmount};
use crate::resolution::{compute_segments, compute_segments_with_override};
use config::constants::DEFAULT_CONVEXITY;
use glam::{DMat4, DVec3};
use openscad_ast::{Expression, SerializedNode, Statement};

/// Evaluates OpenSCAD source code and returns geometry IR.
///
/// This function uses the native tree-sitter parser.
/// For browser/WASM use, see `evaluate_from_cst`.
///
/// # Arguments
///
/// * `source` - The OpenSCAD source code
///
/// # Returns
///
/// A vector of geometry nodes, or an error if evaluation fails.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_eval::evaluate;
///
/// let source = "cube(10);";
/// let geometry = evaluate(source)?;
/// ```
#[cfg(feature = "native-parser")]
pub fn evaluate(source: &str) -> Result<Vec<GeometryNode>, EvalError> {
    let statements = openscad_ast::parse_to_ast(source)?;
    let mut ctx = EvaluationContext::new();
    evaluate_statements(&statements, &mut ctx)
}

/// Evaluates a serialized CST and returns geometry IR.
///
/// This function is browser-safe and accepts a pre-parsed CST
/// from web-tree-sitter.
///
/// # Arguments
///
/// * `cst` - The serialized CST from web-tree-sitter
///
/// # Returns
///
/// A vector of geometry nodes, or an error if evaluation fails.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_eval::evaluate_from_cst;
/// use openscad_ast::SerializedNode;
///
/// let cst: SerializedNode = serde_json::from_str(json)?;
/// let geometry = evaluate_from_cst(&cst)?;
/// ```
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
    let mut nodes = Vec::new();
    for stmt in statements {
        if let Some(node) = evaluate_statement(stmt, ctx)? {
            nodes.push(node);
        }
    }
    Ok(nodes)
}

/// Evaluates a single statement.
fn evaluate_statement(
    stmt: &Statement,
    ctx: &mut EvaluationContext,
) -> Result<Option<GeometryNode>, EvalError> {
    match stmt {
        // 3D Primitives - evaluate expressions at runtime
        Statement::Cube { size, center, span } => {
            // Evaluate size expression to DVec3 at runtime
            let size_val = eval_vec3(size, ctx);
            Ok(Some(GeometryNode::Cube {
                size: size_val,
                center: *center,
                span: *span,
            }))
        }

        Statement::Sphere { radius, fn_override, span } => {
            // Evaluate radius expression at runtime
            let r = eval_expr(radius, ctx).unwrap_or(1.0);
            // Use local $fn override if specified, otherwise use context's $fn
            let segments = compute_segments_with_override(r, *fn_override, ctx);
            Ok(Some(GeometryNode::Sphere {
                radius: r,
                segments,
                span: *span,
            }))
        }

        Statement::Cylinder {
            height,
            radius_bottom,
            radius_top,
            center,
            fn_override,
            span,
        } => {
            // Evaluate dimension expressions at runtime
            let h = eval_expr(height, ctx).unwrap_or(1.0);
            let r_bottom = eval_expr(radius_bottom, ctx).unwrap_or(1.0);
            let r_top = eval_expr(radius_top, ctx).unwrap_or(1.0);
            let max_radius = r_bottom.max(r_top);
            // Use local $fn override if specified, otherwise use context's $fn
            let segments = compute_segments_with_override(max_radius, *fn_override, ctx);
            Ok(Some(GeometryNode::Cylinder {
                height: h,
                radius_bottom: r_bottom,
                radius_top: r_top,
                center: *center,
                segments,
                span: *span,
            }))
        }

        Statement::Polyhedron {
            points,
            faces,
            convexity,
            span,
        } => {
            Ok(Some(GeometryNode::Polyhedron {
                points: points.clone(),
                faces: faces.clone(),
                convexity: *convexity,
                span: *span,
            }))
        }

        // 2D Primitives
        Statement::Square { size, center, span } => {
            Ok(Some(GeometryNode::Square {
                size: *size,
                center: *center,
                span: *span,
            }))
        }

        Statement::Circle { radius, span, .. } => {
            let segments = compute_segments(*radius, ctx);
            Ok(Some(GeometryNode::Circle {
                radius: *radius,
                segments,
                span: *span,
            }))
        }

        Statement::Polygon { points, paths, span } => {
            Ok(Some(GeometryNode::Polygon {
                points: points.clone(),
                paths: paths.clone(),
                span: *span,
            }))
        }

        // Transformations - evaluate expressions to concrete values
        Statement::Translate { vector, children, span } => {
            let v = eval_vec3(vector, ctx);
            let matrix = DMat4::from_translation(v);
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Transform {
                matrix,
                children: child_nodes,
                span: *span,
            }))
        }

        Statement::Rotate { angles, axis, children, span } => {
            let ang = eval_vec3(angles, ctx);
            let matrix = if let Some(ax_expr) = axis {
                // Axis-angle rotation: angles.x is the angle, axis is the axis
                let ax = eval_vec3(ax_expr, ctx);
                let angle_rad = eval_expr(angles, ctx).unwrap_or(0.0).to_radians();
                DMat4::from_axis_angle(ax.normalize(), angle_rad)
            } else {
                // Euler angles (XYZ order, degrees)
                let rx = DMat4::from_rotation_x(ang.x.to_radians());
                let ry = DMat4::from_rotation_y(ang.y.to_radians());
                let rz = DMat4::from_rotation_z(ang.z.to_radians());
                // OpenSCAD order: rotate around X, then Y, then Z
                rz * ry * rx
            };
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Transform {
                matrix,
                children: child_nodes,
                span: *span,
            }))
        }

        Statement::Scale { factors, children, span } => {
            let f = eval_vec3(factors, ctx);
            let matrix = DMat4::from_scale(f);
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Transform {
                matrix,
                children: child_nodes,
                span: *span,
            }))
        }

        Statement::Mirror { normal, children, span } => {
            // Mirror matrix: I - 2 * n * n^T
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
                span: *span,
            }))
        }

        Statement::Multmatrix { matrix, children, span } => {
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
                span: *span,
            }))
        }

        Statement::Resize { new_size, auto_scale, children, span } => {
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Resize {
                new_size: *new_size,
                auto_scale: *auto_scale,
                convexity: DEFAULT_CONVEXITY,
                children: child_nodes,
                span: *span,
            }))
        }

        Statement::Color { color, children, span } => {
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Color {
                color: *color,
                children: child_nodes,
                span: *span,
            }))
        }

        // Boolean Operations
        Statement::Union { children, span } => {
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Boolean {
                operation: BooleanOperation::Union,
                children: child_nodes,
                span: *span,
            }))
        }

        Statement::Difference { children, span } => {
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Boolean {
                operation: BooleanOperation::Difference,
                children: child_nodes,
                span: *span,
            }))
        }

        Statement::Intersection { children, span } => {
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Boolean {
                operation: BooleanOperation::Intersection,
                children: child_nodes,
                span: *span,
            }))
        }

        // Extrusions
        Statement::LinearExtrude {
            height,
            center,
            twist,
            slices,
            scale,
            children,
            span,
        } => {
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::LinearExtrude {
                height: *height,
                center: *center,
                twist: *twist,
                slices: *slices,
                scale: *scale,
                children: child_nodes,
                span: *span,
            }))
        }

        Statement::RotateExtrude {
            angle,
            convexity,
            children,
            span,
        } => {
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::RotateExtrude {
                angle: *angle,
                convexity: *convexity,
                children: child_nodes,
                span: *span,
            }))
        }

        // Advanced Operations
        Statement::Hull { children, span } => {
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Hull {
                children: child_nodes,
                span: *span,
            }))
        }

        Statement::Minkowski { convexity, children, span } => {
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Minkowski {
                convexity: *convexity,
                children: child_nodes,
                span: *span,
            }))
        }

        Statement::Offset { amount, chamfer, children, span } => {
            let offset_amount = match amount {
                openscad_ast::OffsetAmount::Radius(r) => OffsetAmount::Radius(*r),
                openscad_ast::OffsetAmount::Delta(d) => OffsetAmount::Delta(*d),
            };
            let child_nodes = evaluate_children(children, ctx)?;
            Ok(Some(GeometryNode::Offset {
                amount: offset_amount,
                chamfer: *chamfer,
                children: child_nodes,
                span: *span,
            }))
        }

        // Variables
        Statement::Assignment { name, value, .. } => {
            // Evaluate the expression using the current context (which has registered functions)
            if let Some(val) = eval_expr(value, ctx) {
                ctx.set_variable(name, val);
            }
            Ok(None) // Assignments don't produce geometry
        }

        // Module calls - look up user-defined modules
        Statement::ModuleCall { name, arguments, children, span } => {
            // Check for user-defined module
            if let Some(module_def) = ctx.get_module(name).cloned() {
                // Create new scope and bind arguments to parameters
                ctx.push_scope();
                
                // Bind arguments to parameters
                for (i, (param_name, default_val)) in module_def.parameters.iter().enumerate() {
                    let value = arguments.get(i)
                        .map(|arg| eval_expr(&arg.value, ctx).unwrap_or(0.0))
                        .or(*default_val)
                        .unwrap_or(0.0);
                    ctx.set_variable(param_name, value);
                }
                
                // Evaluate module body
                let result = evaluate_statements(&module_def.body, ctx);
                ctx.pop_scope();
                
                // Return geometry from module body
                let nodes = result?;
                if nodes.is_empty() {
                    Ok(None)
                } else if nodes.len() == 1 {
                    Ok(Some(nodes.into_iter().next().unwrap()))
                } else {
                    Ok(Some(GeometryNode::Boolean {
                        operation: BooleanOperation::Union,
                        children: nodes,
                        span: *span,
                    }))
                }
            } else {
                Err(EvalError::UnknownModule {
                    name: name.clone(),
                    span: *span,
                })
            }
        }

        // Control flow - for loop with iteration
        Statement::ForLoop { variable, range, body, span } => {
            let mut all_nodes = Vec::new();
            
            // Evaluate range bounds
            let (start, step, end) = match range {
                Expression::Range { start, step, end } => {
                    let s = eval_expr(start, ctx).unwrap_or(0.0);
                    let st = step.as_ref().map(|e| eval_expr(e, ctx).unwrap_or(1.0)).unwrap_or(1.0);
                    let e = eval_expr(end, ctx).unwrap_or(0.0);
                    (s, st, e)
                }
                Expression::Vector(items) => {
                    // Iterate over vector items
                    for item in items {
                        if let Some(val) = eval_expr(item, ctx) {
                            ctx.push_scope();
                            ctx.set_variable(variable, val);
                            let nodes = evaluate_statements(body, ctx)?;
                            all_nodes.extend(nodes);
                            ctx.pop_scope();
                        }
                    }
                    return if all_nodes.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(GeometryNode::Boolean {
                            operation: BooleanOperation::Union,
                            children: all_nodes,
                            span: *span,
                        }))
                    };
                }
                _ => {
                    let val = eval_expr(range, ctx).unwrap_or(0.0);
                    (0.0, 1.0, val)
                }
            };
            
            // Iterate with range [start:step:end]
            let mut i = start;
            let max_iterations = 10000; // Safety limit
            let mut count = 0;
            while (step > 0.0 && i <= end) || (step < 0.0 && i >= end) {
                if count >= max_iterations { break; }
                ctx.push_scope();
                ctx.set_variable(variable, i);
                let nodes = evaluate_statements(body, ctx)?;
                all_nodes.extend(nodes);
                ctx.pop_scope();
                i += step;
                count += 1;
            }
            
            if all_nodes.is_empty() {
                Ok(None)
            } else {
                Ok(Some(GeometryNode::Boolean {
                    operation: BooleanOperation::Union,
                    children: all_nodes,
                    span: *span,
                }))
            }
        }

        // If statement with condition evaluation
        Statement::If { condition, then_branch, else_branch, span } => {
            let cond_value = eval_expr(condition, ctx).unwrap_or(0.0);
            let branch = if cond_value != 0.0 { 
                then_branch 
            } else { 
                else_branch.as_ref().map(|v| v.as_slice()).unwrap_or(&[])
            };
            
            let child_nodes = evaluate_children(branch, ctx)?;
            if child_nodes.is_empty() {
                Ok(None)
            } else if child_nodes.len() == 1 {
                Ok(Some(child_nodes.into_iter().next().unwrap()))
            } else {
                Ok(Some(GeometryNode::Boolean {
                    operation: BooleanOperation::Union,
                    children: child_nodes,
                    span: *span,
                }))
            }
        }

        // Module definitions - register in context
        Statement::ModuleDefinition { name, parameters, body, .. } => {
            let params: Vec<(String, Option<f64>)> = parameters.iter()
                .map(|p| {
                    let default = p.default.as_ref().and_then(|e| eval_expr(e, ctx));
                    (p.name.clone(), default)
                })
                .collect();
            ctx.register_module(name.clone(), ModuleDefinition {
                parameters: params,
                body: body.clone(),
            });
            Ok(None) // Definitions don't produce geometry
        }

        // Function definitions - register in context
        Statement::FunctionDefinition { name, parameters, body, .. } => {
            let params: Vec<(String, Option<f64>)> = parameters.iter()
                .map(|p| {
                    let default = p.default.as_ref().and_then(|e| eval_expr(e, ctx));
                    (p.name.clone(), default)
                })
                .collect();
            ctx.register_function(name.clone(), FunctionDefinition {
                parameters: params,
                body: body.clone(),
            });
            Ok(None) // Definitions don't produce geometry
        }

        Statement::Echo { .. } => Ok(None), // Echo doesn't produce geometry

        // Modified statements (*, !, #, %)
        // These affect rendering behavior but still produce geometry
        Statement::Modified { modifier, child, span } => {
            use openscad_ast::Modifier;
            match modifier {
                // Disable (*): Don't render this object at all
                Modifier::Disable => Ok(None),
                
                // ShowOnly (!): In preview, only show this object
                // For evaluation, we still generate the geometry
                Modifier::ShowOnly => {
                    // TODO: Track "show only" flag for renderer
                    evaluate_statement(child, ctx)
                }
                
                // Highlight (#): Render in magenta for debugging
                // We wrap the geometry with a highlight color
                Modifier::Highlight => {
                    if let Some(geometry) = evaluate_statement(child, ctx)? {
                        // Wrap in magenta color [1.0, 0.0, 1.0, 0.5]
                        Ok(Some(GeometryNode::Color {
                            color: [1.0, 0.0, 1.0, 0.5],
                            children: vec![geometry],
                            span: *span,
                        }))
                    } else {
                        Ok(None)
                    }
                }
                
                // Transparent (%): Render semi-transparent
                // We wrap the geometry with a transparent color
                Modifier::Transparent => {
                    if let Some(geometry) = evaluate_statement(child, ctx)? {
                        // Wrap in teal color with transparency [0.0, 0.5, 0.5, 0.3]
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

/// Evaluates children statements.
fn evaluate_children(
    children: &[Statement],
    ctx: &mut EvaluationContext,
) -> Result<Vec<GeometryNode>, EvalError> {
    ctx.push_scope();
    let result = evaluate_statements(children, ctx);
    ctx.pop_scope();
    result
}

/// Evaluates an expression to a f64 value using the context for variables.
/// For vector special variables ($vpr, $vpt), returns the first component.
fn eval_expr(expr: &Expression, ctx: &EvaluationContext) -> Option<f64> {
    match expr {
        Expression::Number(n) => Some(*n),
        Expression::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
        Expression::Variable(name) => {
            // Handle special variables first
            match name.as_str() {
                "$t" => Some(ctx.t_value()),
                "$children" => Some(ctx.children_count() as f64),
                "$fn" => Some(ctx.fn_value()),
                "$fa" => Some(ctx.fa_value()),
                "$fs" => Some(ctx.fs_value()),
                "$vpd" => Some(ctx.vpd_value()),
                "$vpf" => Some(ctx.vpf_value()),
                // Return first component for vector variables in scalar context
                "$vpr" => Some(ctx.vpr_value()[0]),
                "$vpt" => Some(ctx.vpt_value()[0]),
                _ => ctx.get_variable(name),
            }
        }
        Expression::Binary { left, operator, right } => {
            let l = eval_expr(left, ctx)?;
            let r = eval_expr(right, ctx)?;
            use openscad_ast::BinaryOp;
            Some(match operator {
                BinaryOp::Add => l + r,
                BinaryOp::Subtract => l - r,
                BinaryOp::Multiply => l * r,
                BinaryOp::Divide => if r != 0.0 { l / r } else { f64::NAN },
                BinaryOp::Modulo => l % r,
                BinaryOp::Power => l.powf(r),
                BinaryOp::Less => if l < r { 1.0 } else { 0.0 },
                BinaryOp::Greater => if l > r { 1.0 } else { 0.0 },
                BinaryOp::LessEqual => if l <= r { 1.0 } else { 0.0 },
                BinaryOp::GreaterEqual => if l >= r { 1.0 } else { 0.0 },
                BinaryOp::Equal => if (l - r).abs() < 1e-10 { 1.0 } else { 0.0 },
                BinaryOp::NotEqual => if (l - r).abs() >= 1e-10 { 1.0 } else { 0.0 },
                BinaryOp::And => if l != 0.0 && r != 0.0 { 1.0 } else { 0.0 },
                BinaryOp::Or => if l != 0.0 || r != 0.0 { 1.0 } else { 0.0 },
            })
        }
        Expression::Unary { operator, operand } => {
            let v = eval_expr(operand, ctx)?;
            use openscad_ast::UnaryOp;
            Some(match operator {
                UnaryOp::Negate => -v,
                UnaryOp::Not => if v == 0.0 { 1.0 } else { 0.0 },
            })
        }
        Expression::Ternary { condition, then_expr, else_expr } => {
            let c = eval_expr(condition, ctx)?;
            if c != 0.0 { eval_expr(then_expr, ctx) } else { eval_expr(else_expr, ctx) }
        }
        Expression::Range { start, step: _, end: _ } => {
            // For range as a single value, return start (typically used in for loops)
            eval_expr(start, ctx)
        }
        Expression::FunctionCall { name, arguments } => {
            eval_function_call(name, arguments, ctx)
        }
        _ => None, // Vector not yet supported as scalar
    }
}

/// Evaluates a function call (built-in or user-defined).
fn eval_function_call(name: &str, arguments: &[openscad_ast::Argument], ctx: &EvaluationContext) -> Option<f64> {
    // Evaluate arguments
    let args: Vec<f64> = arguments.iter()
        .filter_map(|a| eval_expr(&a.value, ctx))
        .collect();
    
    // Check for user-defined function first
    if let Some(func_def) = ctx.get_function(name).cloned() {
        // Create temporary context with function parameters bound
        let mut func_ctx = ctx.clone();
        func_ctx.push_scope();
        
        // Bind arguments to parameters
        for (i, (param_name, default_val)) in func_def.parameters.iter().enumerate() {
            let value = args.get(i).copied()
                .or(*default_val)
                .unwrap_or(0.0);
            func_ctx.set_variable(param_name, value);
        }
        
        // Evaluate function body
        let result = eval_expr(&func_def.body, &func_ctx);
        func_ctx.pop_scope();
        return result;
    }
    
    // Built-in math functions
    match name {
        // Trigonometric (degrees)
        "sin" => args.first().map(|x| x.to_radians().sin()),
        "cos" => args.first().map(|x| x.to_radians().cos()),
        "tan" => args.first().map(|x| x.to_radians().tan()),
        "asin" => args.first().map(|x| x.asin().to_degrees()),
        "acos" => args.first().map(|x| x.acos().to_degrees()),
        "atan" => args.first().map(|x| x.atan().to_degrees()),
        "atan2" => args.get(0).and_then(|y| args.get(1).map(|x| y.atan2(*x).to_degrees())),
        
        // Math functions
        "abs" => args.first().map(|x| x.abs()),
        "ceil" => args.first().map(|x| x.ceil()),
        "floor" => args.first().map(|x| x.floor()),
        "round" => args.first().map(|x| x.round()),
        "sqrt" => args.first().map(|x| x.sqrt()),
        "exp" => args.first().map(|x| x.exp()),
        "ln" => args.first().map(|x| x.ln()),
        "log" => args.first().map(|x| x.log10()),
        "pow" => args.get(0).and_then(|base| args.get(1).map(|exp| base.powf(*exp))),
        "sign" => args.first().map(|x| if *x > 0.0 { 1.0 } else if *x < 0.0 { -1.0 } else { 0.0 }),
        "min" => args.iter().copied().reduce(f64::min),
        "max" => args.iter().copied().reduce(f64::max),
        
        // Vector/list functions (return length for now)
        "len" => Some(args.len() as f64),
        
        _ => None, // Unknown function
    }
}

// NOTE: evaluate_expression_to_f64 was removed - it created a new empty context
// which didn't have registered functions. Use eval_expr with the current context instead.

/// Evaluates an expression to a DVec3 using the context for variables.
/// Handles vectors, single numbers (broadcasts to all components), and expressions.
/// Special vector variables ($vpr, $vpt) return their full 3-component values.
fn eval_vec3(expr: &Expression, ctx: &EvaluationContext) -> DVec3 {
    match expr {
        Expression::Number(n) => DVec3::splat(*n),
        Expression::Vector(items) => {
            let x = items.first().map(|e| eval_expr(e, ctx).unwrap_or(0.0)).unwrap_or(0.0);
            let y = items.get(1).map(|e| eval_expr(e, ctx).unwrap_or(0.0)).unwrap_or(0.0);
            let z = items.get(2).map(|e| eval_expr(e, ctx).unwrap_or(0.0)).unwrap_or(0.0);
            DVec3::new(x, y, z)
        }
        Expression::Variable(name) => {
            // Handle vector special variables
            match name.as_str() {
                "$vpr" => {
                    let v = ctx.vpr_value();
                    DVec3::new(v[0], v[1], v[2])
                }
                "$vpt" => {
                    let v = ctx.vpt_value();
                    DVec3::new(v[0], v[1], v[2])
                }
                _ => {
                    // Try to get as scalar and broadcast
                    ctx.get_variable(name).map(DVec3::splat).unwrap_or(DVec3::ZERO)
                }
            }
        }
        _ => {
            // Try to evaluate as a scalar and broadcast
            eval_expr(expr, ctx).map(DVec3::splat).unwrap_or(DVec3::ZERO)
        }
    }
}

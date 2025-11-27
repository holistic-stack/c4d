//! # AST Evaluator
//!
//! Evaluates AST to produce geometry nodes.
//!
//! ## Example
//!
//! ```rust
//! use openscad_eval::visitor::evaluator::evaluate_ast;
//! use openscad_ast::parse;
//!
//! let ast = parse("cube(10);").unwrap();
//! let result = evaluate_ast(&ast).unwrap();
//! ```

use crate::error::EvalError;
use crate::geometry::{EvaluatedAst, GeometryNode};
use crate::value::Value;
use openscad_ast::{Ast, Statement, Expression, Argument, BinaryOp, UnaryOp};

// =============================================================================
// CONSTANTS - OpenSCAD defaults
// =============================================================================

/// Default $fn value.
const DEFAULT_FN: u32 = 0;
/// Default $fa value (degrees).
const DEFAULT_FA: f64 = 12.0;
/// Default $fs value (mm).
const DEFAULT_FS: f64 = 2.0;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Evaluate AST to geometry.
///
/// ## Parameters
///
/// - `ast`: Abstract Syntax Tree from openscad-ast
///
/// ## Returns
///
/// `Result<EvaluatedAst, EvalError>` - Evaluated geometry tree
pub fn evaluate_ast(ast: &Ast) -> Result<EvaluatedAst, EvalError> {
    let mut evaluator = Evaluator::new();
    let geometry = evaluator.evaluate_statements(&ast.statements)?;
    Ok(EvaluatedAst::with_warnings(geometry, evaluator.warnings))
}

// =============================================================================
// EVALUATOR
// =============================================================================

/// AST evaluator state.
struct Evaluator {
    /// Collected warnings.
    warnings: Vec<String>,
    /// Current $fn value.
    fn_: u32,
    /// Current $fa value.
    fa: f64,
    /// Current $fs value.
    fs: f64,
}

impl Evaluator {
    /// Create new evaluator with default settings.
    fn new() -> Self {
        Self {
            warnings: Vec::new(),
            fn_: DEFAULT_FN,
            fa: DEFAULT_FA,
            fs: DEFAULT_FS,
        }
    }

    /// Calculate number of fragments for circular shapes.
    fn calculate_fragments(&self, radius: f64) -> u32 {
        if self.fn_ > 0 {
            self.fn_.max(3)
        } else {
            let fa_fragments = (360.0 / self.fa).ceil() as u32;
            let fs_fragments = ((2.0 * std::f64::consts::PI * radius) / self.fs).ceil() as u32;
            fa_fragments.min(fs_fragments).max(3)
        }
    }

    /// Evaluate a list of statements.
    fn evaluate_statements(&mut self, statements: &[Statement]) -> Result<GeometryNode, EvalError> {
        let mut children = Vec::new();

        for stmt in statements {
            if let Some(node) = self.evaluate_statement(stmt)? {
                if !node.is_empty() {
                    children.push(node);
                }
            }
        }

        match children.len() {
            0 => Ok(GeometryNode::Empty),
            1 => Ok(children.remove(0)),
            _ => Ok(GeometryNode::Group { children }),
        }
    }

    /// Evaluate a single statement.
    fn evaluate_statement(&mut self, stmt: &Statement) -> Result<Option<GeometryNode>, EvalError> {
        match stmt {
            Statement::ModuleCall { name, args, children, .. } => {
                self.evaluate_module_call(name, args, children)
            }
            Statement::Block { statements, .. } => {
                Ok(Some(self.evaluate_statements(statements)?))
            }
            Statement::Assignment { .. } => {
                // TODO: Handle variable assignments
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Evaluate a module call.
    fn evaluate_module_call(
        &mut self,
        name: &str,
        args: &[Argument],
        children: &[Statement],
    ) -> Result<Option<GeometryNode>, EvalError> {
        match name {
            // 3D Primitives
            "cube" => Ok(Some(self.eval_cube(args)?)),
            "sphere" => Ok(Some(self.eval_sphere(args)?)),
            "cylinder" => Ok(Some(self.eval_cylinder(args)?)),

            // 2D Primitives
            "circle" => Ok(Some(self.eval_circle(args)?)),
            "square" => Ok(Some(self.eval_square(args)?)),

            // Boolean operations
            "union" => Ok(Some(self.eval_union(children)?)),
            "difference" => Ok(Some(self.eval_difference(children)?)),
            "intersection" => Ok(Some(self.eval_intersection(children)?)),

            // Transforms
            "translate" => Ok(Some(self.eval_translate(args, children)?)),
            "rotate" => Ok(Some(self.eval_rotate(args, children)?)),
            "scale" => Ok(Some(self.eval_scale(args, children)?)),
            "mirror" => Ok(Some(self.eval_mirror(args, children)?)),
            "color" => Ok(Some(self.eval_color(args, children)?)),

            // Extrusions
            "linear_extrude" => Ok(Some(self.eval_linear_extrude(args, children)?)),
            "rotate_extrude" => Ok(Some(self.eval_rotate_extrude(args, children)?)),

            // Unknown module - warn and skip
            _ => {
                self.warnings.push(format!("Unknown module: {}", name));
                Ok(None)
            }
        }
    }

    // =========================================================================
    // 3D PRIMITIVES
    // =========================================================================

    /// Evaluate cube() call.
    ///
    /// ## OpenSCAD Signature
    ///
    /// ```text
    /// cube(size);
    /// cube(size, center);
    /// cube([x, y, z], center);
    /// ```
    fn eval_cube(&mut self, args: &[Argument]) -> Result<GeometryNode, EvalError> {
        let mut size = [1.0, 1.0, 1.0];
        let mut center = false;

        // Process arguments
        for (i, arg) in args.iter().enumerate() {
            match arg {
                Argument::Positional(expr) => {
                    if i == 0 {
                        let value = self.eval_expr(expr)?;
                        size = value.as_vec3()?;
                    } else if i == 1 {
                        center = self.eval_expr(expr)?.as_boolean();
                    }
                }
                Argument::Named { name, value } => match name.as_str() {
                    "size" => size = self.eval_expr(value)?.as_vec3()?,
                    "center" => center = self.eval_expr(value)?.as_boolean(),
                    _ => self.warnings.push(format!("Unknown argument for cube: {}", name)),
                },
            }
        }

        Ok(GeometryNode::Cube { size, center })
    }

    /// Evaluate sphere() call.
    fn eval_sphere(&mut self, args: &[Argument]) -> Result<GeometryNode, EvalError> {
        let mut radius = 1.0;

        for (i, arg) in args.iter().enumerate() {
            match arg {
                Argument::Positional(expr) => {
                    if i == 0 {
                        radius = self.eval_expr(expr)?.as_number()?;
                    }
                }
                Argument::Named { name, value } => match name.as_str() {
                    "r" | "radius" => radius = self.eval_expr(value)?.as_number()?,
                    "d" | "diameter" => radius = self.eval_expr(value)?.as_number()? / 2.0,
                    "$fn" => self.fn_ = self.eval_expr(value)?.as_number()? as u32,
                    _ => {}
                },
            }
        }

        let fn_ = self.calculate_fragments(radius);
        Ok(GeometryNode::Sphere { radius, fn_ })
    }

    /// Evaluate cylinder() call.
    fn eval_cylinder(&mut self, args: &[Argument]) -> Result<GeometryNode, EvalError> {
        let mut height = 1.0;
        let mut radius1 = 1.0;
        let mut radius2 = 1.0;
        let mut center = false;

        for (i, arg) in args.iter().enumerate() {
            match arg {
                Argument::Positional(expr) => {
                    if i == 0 {
                        height = self.eval_expr(expr)?.as_number()?;
                    } else if i == 1 {
                        let r = self.eval_expr(expr)?.as_number()?;
                        radius1 = r;
                        radius2 = r;
                    }
                }
                Argument::Named { name, value } => match name.as_str() {
                    "h" | "height" => height = self.eval_expr(value)?.as_number()?,
                    "r" | "radius" => {
                        let r = self.eval_expr(value)?.as_number()?;
                        radius1 = r;
                        radius2 = r;
                    }
                    "r1" => radius1 = self.eval_expr(value)?.as_number()?,
                    "r2" => radius2 = self.eval_expr(value)?.as_number()?,
                    "d" | "diameter" => {
                        let r = self.eval_expr(value)?.as_number()? / 2.0;
                        radius1 = r;
                        radius2 = r;
                    }
                    "d1" => radius1 = self.eval_expr(value)?.as_number()? / 2.0,
                    "d2" => radius2 = self.eval_expr(value)?.as_number()? / 2.0,
                    "center" => center = self.eval_expr(value)?.as_boolean(),
                    "$fn" => self.fn_ = self.eval_expr(value)?.as_number()? as u32,
                    _ => {}
                },
            }
        }

        let fn_ = self.calculate_fragments(radius1.max(radius2));
        Ok(GeometryNode::Cylinder {
            height,
            radius1,
            radius2,
            center,
            fn_,
        })
    }

    // =========================================================================
    // 2D PRIMITIVES
    // =========================================================================

    fn eval_circle(&mut self, args: &[Argument]) -> Result<GeometryNode, EvalError> {
        let mut radius = 1.0;

        for (i, arg) in args.iter().enumerate() {
            match arg {
                Argument::Positional(expr) => {
                    if i == 0 {
                        radius = self.eval_expr(expr)?.as_number()?;
                    }
                }
                Argument::Named { name, value } => match name.as_str() {
                    "r" | "radius" => radius = self.eval_expr(value)?.as_number()?,
                    "d" | "diameter" => radius = self.eval_expr(value)?.as_number()? / 2.0,
                    "$fn" => self.fn_ = self.eval_expr(value)?.as_number()? as u32,
                    _ => {}
                },
            }
        }

        let fn_ = self.calculate_fragments(radius);
        Ok(GeometryNode::Circle { radius, fn_ })
    }

    fn eval_square(&mut self, args: &[Argument]) -> Result<GeometryNode, EvalError> {
        let mut size = [1.0, 1.0];
        let mut center = false;

        for (i, arg) in args.iter().enumerate() {
            match arg {
                Argument::Positional(expr) => {
                    if i == 0 {
                        size = self.eval_expr(expr)?.as_vec2()?;
                    } else if i == 1 {
                        center = self.eval_expr(expr)?.as_boolean();
                    }
                }
                Argument::Named { name, value } => match name.as_str() {
                    "size" => size = self.eval_expr(value)?.as_vec2()?,
                    "center" => center = self.eval_expr(value)?.as_boolean(),
                    _ => {}
                },
            }
        }

        Ok(GeometryNode::Square { size, center })
    }

    // =========================================================================
    // BOOLEAN OPERATIONS
    // =========================================================================

    fn eval_union(&mut self, children: &[Statement]) -> Result<GeometryNode, EvalError> {
        let child_nodes = self.flatten_children(children)?;

        match child_nodes.len() {
            0 => Ok(GeometryNode::Empty),
            1 => Ok(child_nodes.into_iter().next().unwrap()),
            _ => Ok(GeometryNode::Union { children: child_nodes }),
        }
    }

    /// Flatten children, extracting Block contents.
    ///
    /// When a module call has a block body like `union() { ... }`,
    /// the children contain a single Block statement that needs to be expanded.
    fn flatten_children(&mut self, children: &[Statement]) -> Result<Vec<GeometryNode>, EvalError> {
        let mut result = Vec::new();

        for stmt in children {
            match stmt {
                // Extract statements from blocks
                Statement::Block { statements, .. } => {
                    for inner in statements {
                        if let Some(node) = self.evaluate_statement(inner)? {
                            if !node.is_empty() {
                                result.push(node);
                            }
                        }
                    }
                }
                // Regular statements
                _ => {
                    if let Some(node) = self.evaluate_statement(stmt)? {
                        if !node.is_empty() {
                            result.push(node);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    fn eval_difference(&mut self, children: &[Statement]) -> Result<GeometryNode, EvalError> {
        let child_nodes = self.flatten_children(children)?;

        if child_nodes.is_empty() {
            Ok(GeometryNode::Empty)
        } else {
            Ok(GeometryNode::Difference { children: child_nodes })
        }
    }

    fn eval_intersection(&mut self, children: &[Statement]) -> Result<GeometryNode, EvalError> {
        let child_nodes = self.flatten_children(children)?;

        if child_nodes.is_empty() {
            Ok(GeometryNode::Empty)
        } else {
            Ok(GeometryNode::Intersection { children: child_nodes })
        }
    }

    // =========================================================================
    // TRANSFORMS
    // =========================================================================

    fn eval_translate(&mut self, args: &[Argument], children: &[Statement]) -> Result<GeometryNode, EvalError> {
        let offset = args.first()
            .map(|a| match a {
                Argument::Positional(e) => self.eval_expr(e),
                Argument::Named { value, .. } => self.eval_expr(value),
            })
            .transpose()?
            .map(|v| v.as_vec3())
            .transpose()?
            .unwrap_or([0.0, 0.0, 0.0]);

        let child = self.evaluate_statements(children)?;
        Ok(GeometryNode::Translate {
            offset,
            child: Box::new(child),
        })
    }

    fn eval_rotate(&mut self, args: &[Argument], children: &[Statement]) -> Result<GeometryNode, EvalError> {
        let angles = args.first()
            .map(|a| match a {
                Argument::Positional(e) => self.eval_expr(e),
                Argument::Named { value, .. } => self.eval_expr(value),
            })
            .transpose()?
            .map(|v| v.as_vec3())
            .transpose()?
            .unwrap_or([0.0, 0.0, 0.0]);

        let child = self.evaluate_statements(children)?;
        Ok(GeometryNode::Rotate {
            angles,
            child: Box::new(child),
        })
    }

    fn eval_scale(&mut self, args: &[Argument], children: &[Statement]) -> Result<GeometryNode, EvalError> {
        let factors = args.first()
            .map(|a| match a {
                Argument::Positional(e) => self.eval_expr(e),
                Argument::Named { value, .. } => self.eval_expr(value),
            })
            .transpose()?
            .map(|v| v.as_vec3())
            .transpose()?
            .unwrap_or([1.0, 1.0, 1.0]);

        let child = self.evaluate_statements(children)?;
        Ok(GeometryNode::Scale {
            factors,
            child: Box::new(child),
        })
    }

    fn eval_mirror(&mut self, args: &[Argument], children: &[Statement]) -> Result<GeometryNode, EvalError> {
        let normal = args.first()
            .map(|a| match a {
                Argument::Positional(e) => self.eval_expr(e),
                Argument::Named { value, .. } => self.eval_expr(value),
            })
            .transpose()?
            .map(|v| v.as_vec3())
            .transpose()?
            .unwrap_or([1.0, 0.0, 0.0]);

        let child = self.evaluate_statements(children)?;
        Ok(GeometryNode::Mirror {
            normal,
            child: Box::new(child),
        })
    }

    fn eval_color(&mut self, args: &[Argument], children: &[Statement]) -> Result<GeometryNode, EvalError> {
        let mut rgba = [1.0, 1.0, 1.0, 1.0];

        if let Some(arg) = args.first() {
            match arg {
                Argument::Positional(expr) => {
                    let value = self.eval_expr(expr)?;
                    let nums = value.as_number_list()?;
                    for (i, n) in nums.iter().take(4).enumerate() {
                        rgba[i] = *n;
                    }
                }
                _ => {}
            }
        }

        let child = self.evaluate_statements(children)?;
        Ok(GeometryNode::Color {
            rgba,
            child: Box::new(child),
        })
    }

    // =========================================================================
    // EXTRUSIONS
    // =========================================================================

    fn eval_linear_extrude(&mut self, args: &[Argument], children: &[Statement]) -> Result<GeometryNode, EvalError> {
        let mut height = 1.0;
        let mut twist = 0.0;
        let mut scale = [1.0, 1.0];
        let mut slices = 1;
        let mut center = false;

        for arg in args {
            match arg {
                Argument::Positional(expr) => {
                    height = self.eval_expr(expr)?.as_number()?;
                }
                Argument::Named { name, value } => match name.as_str() {
                    "height" => height = self.eval_expr(value)?.as_number()?,
                    "twist" => twist = self.eval_expr(value)?.as_number()?,
                    "scale" => scale = self.eval_expr(value)?.as_vec2()?,
                    "slices" => slices = self.eval_expr(value)?.as_number()? as u32,
                    "center" => center = self.eval_expr(value)?.as_boolean(),
                    _ => {}
                },
            }
        }

        let child = self.evaluate_statements(children)?;
        Ok(GeometryNode::LinearExtrude {
            height,
            twist,
            scale,
            slices,
            center,
            child: Box::new(child),
        })
    }

    fn eval_rotate_extrude(&mut self, args: &[Argument], children: &[Statement]) -> Result<GeometryNode, EvalError> {
        let mut angle = 360.0;

        for arg in args {
            if let Argument::Named { name, value } = arg {
                match name.as_str() {
                    "angle" => angle = self.eval_expr(value)?.as_number()?,
                    "$fn" => self.fn_ = self.eval_expr(value)?.as_number()? as u32,
                    _ => {}
                }
            }
        }

        let fn_ = self.fn_.max(3);
        let child = self.evaluate_statements(children)?;
        Ok(GeometryNode::RotateExtrude {
            angle,
            fn_,
            child: Box::new(child),
        })
    }

    // =========================================================================
    // EXPRESSION EVALUATION
    // =========================================================================

    /// Evaluate an expression to a value.
    fn eval_expr(&mut self, expr: &Expression) -> Result<Value, EvalError> {
        match expr {
            Expression::Number(n) => Ok(Value::Number(*n)),
            Expression::Boolean(b) => Ok(Value::Boolean(*b)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Undef => Ok(Value::Undef),
            Expression::Identifier(_) => Ok(Value::Undef), // TODO: Variable lookup
            Expression::SpecialVariable(name) => self.eval_special_var(name),
            Expression::List(items) => {
                let values: Result<Vec<_>, _> = items.iter()
                    .map(|e| self.eval_expr(e))
                    .collect();
                Ok(Value::List(values?))
            }
            Expression::Range { start, end, step } => {
                let s = self.eval_expr(start)?.as_number()?;
                let e = self.eval_expr(end)?.as_number()?;
                let st = step.as_ref()
                    .map(|x| self.eval_expr(x))
                    .transpose()?
                    .map(|v| v.as_number())
                    .transpose()?;
                Ok(Value::Range {
                    start: s,
                    end: e,
                    step: st,
                })
            }
            Expression::BinaryOp { op, left, right } => {
                self.eval_binary_op(*op, left, right)
            }
            Expression::UnaryOp { op, operand } => {
                self.eval_unary_op(*op, operand)
            }
            Expression::Ternary { condition, then_expr, else_expr } => {
                if self.eval_expr(condition)?.as_boolean() {
                    self.eval_expr(then_expr)
                } else {
                    self.eval_expr(else_expr)
                }
            }
            Expression::FunctionCall { name, args } => {
                self.eval_function_call(name, args)
            }
            Expression::Index { object, index } => {
                let obj = self.eval_expr(object)?;
                let idx = self.eval_expr(index)?.as_number()? as usize;
                match obj {
                    Value::List(items) => {
                        items.get(idx).cloned().ok_or_else(|| {
                            EvalError::InvalidArgument(format!("Index {} out of bounds", idx))
                        })
                    }
                    _ => Err(EvalError::TypeError("Cannot index non-list".to_string())),
                }
            }
            Expression::Member { object, member } => {
                let obj = self.eval_expr(object)?;
                match (&obj, member.as_str()) {
                    (Value::List(items), "x") => items.first().cloned().ok_or_else(|| EvalError::InvalidArgument("No x".to_string())),
                    (Value::List(items), "y") => items.get(1).cloned().ok_or_else(|| EvalError::InvalidArgument("No y".to_string())),
                    (Value::List(items), "z") => items.get(2).cloned().ok_or_else(|| EvalError::InvalidArgument("No z".to_string())),
                    _ => Err(EvalError::TypeError(format!("Unknown member: {}", member))),
                }
            }
        }
    }

    fn eval_special_var(&self, name: &str) -> Result<Value, EvalError> {
        match name {
            "$fn" => Ok(Value::Number(self.fn_ as f64)),
            "$fa" => Ok(Value::Number(self.fa)),
            "$fs" => Ok(Value::Number(self.fs)),
            "$t" => Ok(Value::Number(0.0)), // Animation time
            _ => Ok(Value::Undef),
        }
    }

    fn eval_binary_op(&mut self, op: BinaryOp, left: &Expression, right: &Expression) -> Result<Value, EvalError> {
        let l = self.eval_expr(left)?;
        let r = self.eval_expr(right)?;

        match op {
            BinaryOp::Add => Ok(Value::Number(l.as_number()? + r.as_number()?)),
            BinaryOp::Sub => Ok(Value::Number(l.as_number()? - r.as_number()?)),
            BinaryOp::Mul => Ok(Value::Number(l.as_number()? * r.as_number()?)),
            BinaryOp::Div => {
                let divisor = r.as_number()?;
                if divisor == 0.0 {
                    Err(EvalError::DivisionByZero)
                } else {
                    Ok(Value::Number(l.as_number()? / divisor))
                }
            }
            BinaryOp::Mod => Ok(Value::Number(l.as_number()? % r.as_number()?)),
            BinaryOp::Pow => Ok(Value::Number(l.as_number()?.powf(r.as_number()?))),
            BinaryOp::Lt => Ok(Value::Boolean(l.as_number()? < r.as_number()?)),
            BinaryOp::Gt => Ok(Value::Boolean(l.as_number()? > r.as_number()?)),
            BinaryOp::Le => Ok(Value::Boolean(l.as_number()? <= r.as_number()?)),
            BinaryOp::Ge => Ok(Value::Boolean(l.as_number()? >= r.as_number()?)),
            BinaryOp::Eq => Ok(Value::Boolean(l.as_number()? == r.as_number()?)),
            BinaryOp::Ne => Ok(Value::Boolean(l.as_number()? != r.as_number()?)),
            BinaryOp::And => Ok(Value::Boolean(l.as_boolean() && r.as_boolean())),
            BinaryOp::Or => Ok(Value::Boolean(l.as_boolean() || r.as_boolean())),
        }
    }

    fn eval_unary_op(&mut self, op: UnaryOp, operand: &Expression) -> Result<Value, EvalError> {
        let v = self.eval_expr(operand)?;
        match op {
            UnaryOp::Neg => Ok(Value::Number(-v.as_number()?)),
            UnaryOp::Not => Ok(Value::Boolean(!v.as_boolean())),
            UnaryOp::Pos => Ok(Value::Number(v.as_number()?)),
        }
    }

    fn eval_function_call(&mut self, name: &str, args: &[Argument]) -> Result<Value, EvalError> {
        let arg_values: Vec<_> = args.iter()
            .filter_map(|a| match a {
                Argument::Positional(e) => self.eval_expr(e).ok(),
                Argument::Named { value, .. } => self.eval_expr(value).ok(),
            })
            .collect();

        match name {
            "sin" => Ok(Value::Number(arg_values.first().map(|v| v.as_number().unwrap_or(0.0).to_radians().sin()).unwrap_or(0.0))),
            "cos" => Ok(Value::Number(arg_values.first().map(|v| v.as_number().unwrap_or(0.0).to_radians().cos()).unwrap_or(0.0))),
            "tan" => Ok(Value::Number(arg_values.first().map(|v| v.as_number().unwrap_or(0.0).to_radians().tan()).unwrap_or(0.0))),
            "abs" => Ok(Value::Number(arg_values.first().map(|v| v.as_number().unwrap_or(0.0).abs()).unwrap_or(0.0))),
            "sqrt" => Ok(Value::Number(arg_values.first().map(|v| v.as_number().unwrap_or(0.0).sqrt()).unwrap_or(0.0))),
            "floor" => Ok(Value::Number(arg_values.first().map(|v| v.as_number().unwrap_or(0.0).floor()).unwrap_or(0.0))),
            "ceil" => Ok(Value::Number(arg_values.first().map(|v| v.as_number().unwrap_or(0.0).ceil()).unwrap_or(0.0))),
            "round" => Ok(Value::Number(arg_values.first().map(|v| v.as_number().unwrap_or(0.0).round()).unwrap_or(0.0))),
            "len" => {
                match arg_values.first() {
                    Some(Value::List(l)) => Ok(Value::Number(l.len() as f64)),
                    Some(Value::String(s)) => Ok(Value::Number(s.len() as f64)),
                    _ => Ok(Value::Undef),
                }
            }
            _ => {
                self.warnings.push(format!("Unknown function: {}", name));
                Ok(Value::Undef)
            }
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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

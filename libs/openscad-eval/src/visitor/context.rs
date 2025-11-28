//! # Evaluator Context
//!
//! Core evaluator state and statement evaluation logic.
//!
//! ## Responsibilities
//!
//! - Maintains variable scope for lexical scoping
//! - Collects warnings during evaluation
//! - Evaluates statements (assignments, blocks, loops, conditionals)
//!
//! ## Example
//!
//! ```rust,ignore
//! use crate::visitor::context::EvalContext;
//!
//! let mut ctx = EvalContext::new();
//! ctx.scope.define("x", Value::Number(10.0));
//! ```

use crate::error::EvalError;
use crate::geometry::GeometryNode;
use crate::scope::Scope;
use crate::value::Value;
use openscad_ast::{Statement, Expression, Argument};
use openscad_ast::ast::Parameter;
use std::collections::HashMap;

use super::expressions::eval_expr;
use super::primitives::{eval_cube, eval_sphere, eval_cylinder, eval_polyhedron, eval_circle, eval_square, eval_polygon};
use super::boolean::{eval_union, eval_difference, eval_intersection, eval_hull, eval_minkowski};
use super::transforms::{eval_translate, eval_rotate, eval_scale, eval_mirror, eval_color};
use super::extrusions::{eval_linear_extrude, eval_rotate_extrude};
use super::ops_2d::{eval_offset, eval_projection};

// =============================================================================
// USER-DEFINED FUNCTIONS
// =============================================================================

/// A user-defined function.
///
/// Stores the function's parameters and body expression for later evaluation.
///
/// ## Example
///
/// ```text
/// function double(x) = x * 2;
/// // Stored as: FunctionDef { params: [x], body: x * 2 }
/// ```
#[derive(Debug, Clone)]
pub struct FunctionDef {
    /// Function parameters.
    pub params: Vec<Parameter>,
    /// Body expression.
    pub body: Expression,
}

// =============================================================================
// USER-DEFINED MODULES
// =============================================================================

/// A user-defined module.
///
/// Stores the module's parameters and body statements for later evaluation.
/// Modules differ from functions in that they produce geometry and can
/// access `children()` passed to them.
///
/// ## Example
///
/// ```text
/// module box(size=10) { cube(size); }
/// // Stored as: ModuleDef { params: [size=10], body: [cube(size)] }
///
/// module wrapper() { color("red") children(); }
/// // Module that wraps children in a color
/// ```
#[derive(Debug, Clone)]
pub struct ModuleDef {
    /// Module parameters with optional defaults.
    pub params: Vec<Parameter>,
    /// Body statements (geometry-producing statements).
    pub body: Vec<Statement>,
}

// =============================================================================
// EVALUATOR CONTEXT
// =============================================================================

/// Evaluation context maintaining state during AST traversal.
///
/// ## Fields
///
/// - `warnings`: Collected warnings during evaluation
/// - `scope`: Variable scope for lexical scoping
/// - `functions`: User-defined functions
/// - `modules`: User-defined modules
/// - `children_stack`: Stack of children for nested module calls
pub struct EvalContext {
    /// Collected warnings (undefined variables, unknown modules, etc.).
    pub warnings: Vec<String>,
    /// Variable scope for lexical scoping.
    pub scope: Scope,
    /// User-defined functions.
    pub functions: HashMap<String, FunctionDef>,
    /// User-defined modules.
    pub modules: HashMap<String, ModuleDef>,
    /// Stack of children statements for nested module calls.
    /// Each level represents the children passed to the current module.
    pub children_stack: Vec<Vec<Statement>>,
}

impl EvalContext {
    /// Create new evaluation context with default settings.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// let ctx = EvalContext::new();
    /// assert!(ctx.warnings.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
            scope: Scope::new(),
            functions: HashMap::new(),
            modules: HashMap::new(),
            children_stack: Vec::new(),
        }
    }

    /// Define a user-defined function.
    ///
    /// ## Parameters
    ///
    /// - `name`: Function name
    /// - `params`: Function parameters
    /// - `body`: Body expression
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// ctx.define_function("double", vec![param("x")], expr);
    /// ```
    pub fn define_function(&mut self, name: String, params: Vec<Parameter>, body: Expression) {
        self.functions.insert(name, FunctionDef { params, body });
    }

    /// Get a user-defined function by name.
    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.get(name)
    }

    /// Define a user-defined module.
    ///
    /// ## Parameters
    ///
    /// - `name`: Module name
    /// - `params`: Module parameters with optional defaults
    /// - `body`: Body statements
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// ctx.define_module("box", vec![param("size")], body_stmts);
    /// ```
    pub fn define_module(&mut self, name: String, params: Vec<Parameter>, body: Vec<Statement>) {
        self.modules.insert(name, ModuleDef { params, body });
    }

    /// Get a user-defined module by name.
    pub fn get_module(&self, name: &str) -> Option<&ModuleDef> {
        self.modules.get(name)
    }

    /// Push children onto the stack for module evaluation.
    ///
    /// Called when entering a user-defined module with children.
    pub fn push_children(&mut self, children: Vec<Statement>) {
        self.children_stack.push(children);
    }

    /// Pop children from the stack after module evaluation.
    pub fn pop_children(&mut self) {
        self.children_stack.pop();
    }

    /// Get the current children (for `children()` call inside modules).
    ///
    /// Returns empty slice if no children are available.
    pub fn current_children(&self) -> &[Statement] {
        self.children_stack.last().map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get number of current children (for `$children` special variable).
    pub fn children_count(&self) -> usize {
        self.current_children().len()
    }

    /// Calculate number of fragments for circular shapes.
    ///
    /// Delegates to scope which handles $fn/$fa/$fs calculation.
    ///
    /// ## Parameters
    ///
    /// - `radius`: Radius of the circular shape
    ///
    /// ## Returns
    ///
    /// Number of segments to use (minimum 3)
    pub fn calculate_fragments(&self, radius: f64) -> u32 {
        self.scope.calculate_fragments(radius)
    }

    /// Add a warning message.
    ///
    /// ## Parameters
    ///
    /// - `msg`: Warning message to add
    pub fn warn(&mut self, msg: String) {
        self.warnings.push(msg);
    }
}

impl Default for EvalContext {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// STATEMENT EVALUATION
// =============================================================================

/// Evaluate a list of statements.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `statements`: Statements to evaluate
///
/// ## Returns
///
/// Combined geometry from all statements
pub fn evaluate_statements(
    ctx: &mut EvalContext,
    statements: &[Statement],
) -> Result<GeometryNode, EvalError> {
    let mut children = Vec::new();

    for stmt in statements {
        if let Some(node) = evaluate_statement(ctx, stmt)? {
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
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `stmt`: Statement to evaluate
///
/// ## Returns
///
/// Optional geometry node (assignments return None)
pub fn evaluate_statement(
    ctx: &mut EvalContext,
    stmt: &Statement,
) -> Result<Option<GeometryNode>, EvalError> {
    match stmt {
        Statement::ModuleCall { name, args, children, .. } => {
            evaluate_module_call(ctx, name, args, children)
        }
        Statement::Block { statements, .. } => {
            // Block creates a new scope
            ctx.scope.push();
            let result = evaluate_statements(ctx, statements)?;
            ctx.scope.pop();
            Ok(Some(result))
        }
        Statement::Assignment { name, value, .. } => {
            // Evaluate the value and store in scope
            let val = eval_expr(ctx, value)?;
            ctx.scope.define(name, val);
            Ok(None)
        }
        Statement::ForLoop { assignments, body, .. } => {
            evaluate_for_loop(ctx, assignments, body)
        }
        Statement::IfElse { condition, then_body, else_body, .. } => {
            evaluate_if_else(ctx, condition, then_body, else_body.as_deref())
        }
        Statement::FunctionDeclaration { name, params, body, .. } => {
            // Register the function for later evaluation
            ctx.define_function(name.clone(), params.clone(), body.clone());
            Ok(None)
        }
        Statement::ModuleDeclaration { name, params, body, .. } => {
            // Register the module for later evaluation
            ctx.define_module(name.clone(), params.clone(), body.clone());
            Ok(None)
        }
    }
}

/// Evaluate a module call.
///
/// ## Evaluation Order
///
/// 1. Special modules (children)
/// 2. User-defined modules
/// 3. Built-in primitives, transforms, booleans, extrusions
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `name`: Module name (e.g., "cube", "translate", or user-defined)
/// - `args`: Module arguments
/// - `children`: Child statements (for transforms/booleans/user modules)
fn evaluate_module_call(
    ctx: &mut EvalContext,
    name: &str,
    args: &[Argument],
    children: &[Statement],
) -> Result<Option<GeometryNode>, EvalError> {
    // Special module: children() - evaluates the children passed to current module
    if name == "children" {
        return eval_children_call(ctx, args);
    }

    // Check for user-defined module first
    if let Some(module) = ctx.get_module(name).cloned() {
        return eval_user_module(ctx, &module, args, children);
    }

    // Built-in modules
    match name {
        // 3D Primitives
        "cube" => Ok(Some(eval_cube(ctx, args)?)),
        "sphere" => Ok(Some(eval_sphere(ctx, args)?)),
        "cylinder" => Ok(Some(eval_cylinder(ctx, args)?)),
        "polyhedron" => Ok(Some(eval_polyhedron(ctx, args)?)),

        // 2D Primitives
        "circle" => Ok(Some(eval_circle(ctx, args)?)),
        "square" => Ok(Some(eval_square(ctx, args)?)),
        "polygon" => Ok(Some(eval_polygon(ctx, args)?)),

        // Boolean operations
        "union" => Ok(Some(eval_union(ctx, children)?)),
        "difference" => Ok(Some(eval_difference(ctx, children)?)),
        "intersection" => Ok(Some(eval_intersection(ctx, children)?)),

        // Advanced geometry operations
        "hull" => Ok(Some(eval_hull(ctx, children)?)),
        "minkowski" => Ok(Some(eval_minkowski(ctx, children)?)),

        // Transforms
        "translate" => Ok(Some(eval_translate(ctx, args, children)?)),
        "rotate" => Ok(Some(eval_rotate(ctx, args, children)?)),
        "scale" => Ok(Some(eval_scale(ctx, args, children)?)),
        "mirror" => Ok(Some(eval_mirror(ctx, args, children)?)),
        "color" => Ok(Some(eval_color(ctx, args, children)?)),

        // Extrusions
        "linear_extrude" => Ok(Some(eval_linear_extrude(ctx, args, children)?)),
        "rotate_extrude" => Ok(Some(eval_rotate_extrude(ctx, args, children)?)),

        // 2D Operations
        "offset" => Ok(Some(eval_offset(ctx, args, children)?)),
        "projection" => Ok(Some(eval_projection(ctx, args, children)?)),

        // Unknown module - warn and skip
        _ => {
            ctx.warn(format!("Unknown module: {}", name));
            Ok(None)
        }
    }
}

// =============================================================================
// USER-DEFINED MODULES
// =============================================================================

/// Evaluate a user-defined module call.
///
/// Creates a new scope, binds parameters to arguments, pushes children onto
/// the stack, evaluates the module body, and pops the children stack.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `module`: The module definition
/// - `args`: Arguments passed to the module call
/// - `children`: Child statements passed to the module
///
/// ## Example
///
/// ```text
/// module wrapper() { color("red") children(); }
/// wrapper() cube(10);  // Cube wrapped in red color
/// ```
fn eval_user_module(
    ctx: &mut EvalContext,
    module: &ModuleDef,
    args: &[Argument],
    children: &[Statement],
) -> Result<Option<GeometryNode>, EvalError> {
    // Evaluate all arguments first
    let mut arg_values: Vec<crate::value::Value> = Vec::new();
    let mut named_args: std::collections::HashMap<String, crate::value::Value> = std::collections::HashMap::new();

    for arg in args {
        match arg {
            Argument::Positional(e) => {
                arg_values.push(eval_expr(ctx, e)?);
            }
            Argument::Named { name, value } => {
                named_args.insert(name.clone(), eval_expr(ctx, value)?);
            }
        }
    }

    // Create new scope for module evaluation
    ctx.scope.push();

    // Push children onto the stack for children() access
    ctx.push_children(children.to_vec());

    // Set $children special variable
    ctx.scope.define("$children", crate::value::Value::Number(children.len() as f64));

    // Bind parameters to arguments
    for (i, param) in module.params.iter().enumerate() {
        // Check for named argument first
        let value = if let Some(v) = named_args.get(&param.name) {
            v.clone()
        } else if i < arg_values.len() {
            // Use positional argument
            arg_values[i].clone()
        } else if let Some(default) = &param.default {
            // Use default value
            eval_expr(ctx, default)?
        } else {
            // No value provided
            crate::value::Value::Undef
        };

        ctx.scope.define(&param.name, value);
    }

    // Evaluate module body
    let result = evaluate_statements(ctx, &module.body);

    // Pop children stack
    ctx.pop_children();

    // Pop module scope
    ctx.scope.pop();

    result.map(Some)
}

/// Evaluate children() call inside a module.
///
/// Returns the geometry from evaluating the children passed to the current module.
///
/// ## Syntax
///
/// - `children()` - Evaluate all children
/// - `children(i)` - Evaluate specific child by index
/// - `children([start:end])` - Evaluate range of children (future)
///
/// ## Example
///
/// ```text
/// module wrapper() { translate([10, 0, 0]) children(); }
/// wrapper() { cube(5); sphere(3); }  // Both translated
/// ```
fn eval_children_call(
    ctx: &mut EvalContext,
    args: &[Argument],
) -> Result<Option<GeometryNode>, EvalError> {
    let children = ctx.current_children().to_vec();

    if children.is_empty() {
        return Ok(Some(GeometryNode::Empty));
    }

    // Check for index argument: children(i)
    if let Some(arg) = args.first() {
        let index = match arg {
            Argument::Positional(e) => eval_expr(ctx, e)?.as_number()? as usize,
            Argument::Named { value, .. } => eval_expr(ctx, value)?.as_number()? as usize,
        };

        // Return specific child
        if index < children.len() {
            return evaluate_statement(ctx, &children[index]);
        } else {
            return Ok(Some(GeometryNode::Empty));
        }
    }

    // Evaluate all children
    evaluate_statements(ctx, &children).map(Some)
}

// =============================================================================
// CONTROL FLOW
// =============================================================================

/// Evaluate a for loop.
///
/// Each iteration creates a new scope with the loop variable.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `assignments`: Loop variable assignments
/// - `body`: Loop body statements
///
/// ## Example
///
/// ```text
/// for (i = [0:2]) cube(i);  // Creates 3 cubes
/// ```
fn evaluate_for_loop(
    ctx: &mut EvalContext,
    assignments: &[(String, Expression)],
    body: &[Statement],
) -> Result<Option<GeometryNode>, EvalError> {
    let mut children = Vec::new();

    // Handle single assignment (most common case)
    if let Some((var_name, range_expr)) = assignments.first() {
        let range_val = eval_expr(ctx, range_expr)?;
        
        // Get iteration values
        let values = match range_val {
            Value::List(items) => items,
            Value::Range { start, end, step } => {
                let mut vals = Vec::new();
                let mut current = start;
                let step_val = step.unwrap_or(1.0);
                if step_val > 0.0 {
                    while current <= end {
                        vals.push(Value::Number(current));
                        current += step_val;
                    }
                } else if step_val < 0.0 {
                    while current >= end {
                        vals.push(Value::Number(current));
                        current += step_val;
                    }
                }
                vals
            }
            _ => vec![range_val],
        };

        // Iterate
        for val in values {
            ctx.scope.push();
            ctx.scope.define(var_name, val);
            
            if let Ok(node) = evaluate_statements(ctx, body) {
                if !node.is_empty() {
                    children.push(node);
                }
            }
            
            ctx.scope.pop();
        }
    }

    match children.len() {
        0 => Ok(None),
        1 => Ok(Some(children.remove(0))),
        _ => Ok(Some(GeometryNode::Group { children })),
    }
}

/// Evaluate an if/else statement.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `condition`: Condition expression
/// - `then_body`: Statements if condition is true
/// - `else_body`: Optional statements if condition is false
fn evaluate_if_else(
    ctx: &mut EvalContext,
    condition: &Expression,
    then_body: &[Statement],
    else_body: Option<&[Statement]>,
) -> Result<Option<GeometryNode>, EvalError> {
    let cond_val = eval_expr(ctx, condition)?;
    
    if cond_val.as_boolean() {
        ctx.scope.push();
        let result = evaluate_statements(ctx, then_body)?;
        ctx.scope.pop();
        Ok(Some(result))
    } else if let Some(else_stmts) = else_body {
        ctx.scope.push();
        let result = evaluate_statements(ctx, else_stmts)?;
        ctx.scope.pop();
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = EvalContext::new();
        assert!(ctx.warnings.is_empty());
    }

    #[test]
    fn test_context_warn() {
        let mut ctx = EvalContext::new();
        ctx.warn("Test warning".to_string());
        assert_eq!(ctx.warnings.len(), 1);
    }

    #[test]
    fn test_context_fragments() {
        let ctx = EvalContext::new();
        let fragments = ctx.calculate_fragments(10.0);
        assert!(fragments >= 3);
    }
}

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
use super::primitives::{eval_cube, eval_sphere, eval_cylinder, eval_circle, eval_square};
use super::boolean::{eval_union, eval_difference, eval_intersection};
use super::transforms::{eval_translate, eval_rotate, eval_scale, eval_mirror, eval_color};
use super::extrusions::{eval_linear_extrude, eval_rotate_extrude};

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
// EVALUATOR CONTEXT
// =============================================================================

/// Evaluation context maintaining state during AST traversal.
///
/// ## Fields
///
/// - `warnings`: Collected warnings during evaluation
/// - `scope`: Variable scope for lexical scoping
/// - `functions`: User-defined functions
pub struct EvalContext {
    /// Collected warnings (undefined variables, unknown modules, etc.).
    pub warnings: Vec<String>,
    /// Variable scope for lexical scoping.
    pub scope: Scope,
    /// User-defined functions.
    pub functions: HashMap<String, FunctionDef>,
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
        Statement::ModuleDeclaration { .. } => {
            // TODO: Implement user-defined modules
            Ok(None)
        }
    }
}

/// Evaluate a module call.
///
/// Dispatches to the appropriate primitive, boolean, transform, or extrusion evaluator.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `name`: Module name (e.g., "cube", "translate")
/// - `args`: Module arguments
/// - `children`: Child statements
fn evaluate_module_call(
    ctx: &mut EvalContext,
    name: &str,
    args: &[Argument],
    children: &[Statement],
) -> Result<Option<GeometryNode>, EvalError> {
    match name {
        // 3D Primitives
        "cube" => Ok(Some(eval_cube(ctx, args)?)),
        "sphere" => Ok(Some(eval_sphere(ctx, args)?)),
        "cylinder" => Ok(Some(eval_cylinder(ctx, args)?)),

        // 2D Primitives
        "circle" => Ok(Some(eval_circle(ctx, args)?)),
        "square" => Ok(Some(eval_square(ctx, args)?)),

        // Boolean operations
        "union" => Ok(Some(eval_union(ctx, children)?)),
        "difference" => Ok(Some(eval_difference(ctx, children)?)),
        "intersection" => Ok(Some(eval_intersection(ctx, children)?)),

        // Transforms
        "translate" => Ok(Some(eval_translate(ctx, args, children)?)),
        "rotate" => Ok(Some(eval_rotate(ctx, args, children)?)),
        "scale" => Ok(Some(eval_scale(ctx, args, children)?)),
        "mirror" => Ok(Some(eval_mirror(ctx, args, children)?)),
        "color" => Ok(Some(eval_color(ctx, args, children)?)),

        // Extrusions
        "linear_extrude" => Ok(Some(eval_linear_extrude(ctx, args, children)?)),
        "rotate_extrude" => Ok(Some(eval_rotate_extrude(ctx, args, children)?)),

        // Unknown module - warn and skip
        _ => {
            ctx.warn(format!("Unknown module: {}", name));
            Ok(None)
        }
    }
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

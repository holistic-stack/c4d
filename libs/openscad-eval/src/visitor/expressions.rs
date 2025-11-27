//! # Expression Evaluation
//!
//! Evaluates OpenSCAD expressions to runtime values.
//!
//! ## Responsibilities
//!
//! - Literal evaluation (numbers, booleans, strings)
//! - Variable and special variable lookup
//! - Binary and unary operations
//! - Function calls (built-in functions)
//! - List and range expressions
//!
//! ## Example
//!
//! ```rust,ignore
//! use crate::visitor::expressions::eval_expr;
//!
//! let value = eval_expr(&mut ctx, &Expression::Number(42.0))?;
//! assert_eq!(value, Value::Number(42.0));
//! ```

use crate::error::EvalError;
use crate::value::Value;
use openscad_ast::{Expression, Argument, BinaryOp, UnaryOp};

use super::context::EvalContext;

// =============================================================================
// EXPRESSION EVALUATION
// =============================================================================

/// Evaluate an expression to a value.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `expr`: Expression to evaluate
///
/// ## Returns
///
/// Evaluated value
///
/// ## Example
///
/// ```rust,ignore
/// let result = eval_expr(&mut ctx, &Expression::Number(5.0))?;
/// assert_eq!(result, Value::Number(5.0));
/// ```
pub fn eval_expr(ctx: &mut EvalContext, expr: &Expression) -> Result<Value, EvalError> {
    match expr {
        Expression::Number(n) => Ok(Value::Number(*n)),
        Expression::Boolean(b) => Ok(Value::Boolean(*b)),
        Expression::String(s) => Ok(Value::String(s.clone())),
        Expression::Undef => Ok(Value::Undef),
        Expression::Identifier(name) => eval_identifier(ctx, name),
        Expression::SpecialVariable(name) => eval_special_var(ctx, name),
        Expression::List(items) => eval_list(ctx, items),
        Expression::Range { start, end, step } => eval_range(ctx, start, end, step.as_deref()),
        Expression::BinaryOp { op, left, right } => eval_binary_op(ctx, *op, left, right),
        Expression::UnaryOp { op, operand } => eval_unary_op(ctx, *op, operand),
        Expression::Ternary { condition, then_expr, else_expr } => {
            eval_ternary(ctx, condition, then_expr, else_expr)
        }
        Expression::FunctionCall { name, args } => eval_function_call(ctx, name, args),
        Expression::Index { object, index } => eval_index(ctx, object, index),
        Expression::Member { object, member } => eval_member(ctx, object, member),
    }
}

// =============================================================================
// VARIABLE EVALUATION
// =============================================================================

/// Evaluate a special variable ($fn, $fa, $fs, etc.).
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `name`: Variable name (e.g., "$fn")
///
/// ## Returns
///
/// Variable value or Undef if not found
fn eval_special_var(ctx: &EvalContext, name: &str) -> Result<Value, EvalError> {
    // Look up in scope first
    if let Some(val) = ctx.scope.get(name) {
        return Ok(val.clone());
    }
    // Return undef for unknown special variables
    Ok(Value::Undef)
}

/// Evaluate an identifier (variable reference).
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `name`: Variable name
///
/// ## Returns
///
/// Variable value or Undef if not defined (with warning)
fn eval_identifier(ctx: &mut EvalContext, name: &str) -> Result<Value, EvalError> {
    if let Some(val) = ctx.scope.get(name) {
        Ok(val.clone())
    } else {
        // Undefined variable returns undef (OpenSCAD behavior)
        ctx.warn(format!("Undefined variable: {}", name));
        Ok(Value::Undef)
    }
}

// =============================================================================
// COMPOUND EXPRESSIONS
// =============================================================================

/// Evaluate a list expression.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `items`: List items
fn eval_list(ctx: &mut EvalContext, items: &[Expression]) -> Result<Value, EvalError> {
    let values: Result<Vec<_>, _> = items.iter()
        .map(|e| eval_expr(ctx, e))
        .collect();
    Ok(Value::List(values?))
}

/// Evaluate a range expression.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `start`: Start expression
/// - `end`: End expression
/// - `step`: Optional step expression
fn eval_range(
    ctx: &mut EvalContext,
    start: &Expression,
    end: &Expression,
    step: Option<&Expression>,
) -> Result<Value, EvalError> {
    let s = eval_expr(ctx, start)?.as_number()?;
    let e = eval_expr(ctx, end)?.as_number()?;
    let st = step
        .map(|x| eval_expr(ctx, x))
        .transpose()?
        .map(|v| v.as_number())
        .transpose()?;
    Ok(Value::Range { start: s, end: e, step: st })
}

/// Evaluate a ternary expression.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `condition`: Condition expression
/// - `then_expr`: Expression if true
/// - `else_expr`: Expression if false
fn eval_ternary(
    ctx: &mut EvalContext,
    condition: &Expression,
    then_expr: &Expression,
    else_expr: &Expression,
) -> Result<Value, EvalError> {
    if eval_expr(ctx, condition)?.as_boolean() {
        eval_expr(ctx, then_expr)
    } else {
        eval_expr(ctx, else_expr)
    }
}

/// Evaluate index access (e.g., arr[0]).
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `object`: Object to index
/// - `index`: Index expression
fn eval_index(
    ctx: &mut EvalContext,
    object: &Expression,
    index: &Expression,
) -> Result<Value, EvalError> {
    let obj = eval_expr(ctx, object)?;
    let idx = eval_expr(ctx, index)?.as_number()? as usize;
    match obj {
        Value::List(items) => {
            items.get(idx).cloned().ok_or_else(|| {
                EvalError::InvalidArgument(format!("Index {} out of bounds", idx))
            })
        }
        _ => Err(EvalError::TypeError("Cannot index non-list".to_string())),
    }
}

/// Evaluate member access (e.g., vec.x).
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `object`: Object to access
/// - `member`: Member name
fn eval_member(
    ctx: &mut EvalContext,
    object: &Expression,
    member: &str,
) -> Result<Value, EvalError> {
    let obj = eval_expr(ctx, object)?;
    match (&obj, member) {
        (Value::List(items), "x") => items.first().cloned()
            .ok_or_else(|| EvalError::InvalidArgument("No x".to_string())),
        (Value::List(items), "y") => items.get(1).cloned()
            .ok_or_else(|| EvalError::InvalidArgument("No y".to_string())),
        (Value::List(items), "z") => items.get(2).cloned()
            .ok_or_else(|| EvalError::InvalidArgument("No z".to_string())),
        _ => Err(EvalError::TypeError(format!("Unknown member: {}", member))),
    }
}

// =============================================================================
// OPERATORS
// =============================================================================

/// Evaluate a binary operation.
///
/// ## Supported Operations
///
/// - Arithmetic: +, -, *, /, %, ^
/// - Comparison: <, >, <=, >=, ==, !=
/// - Logical: &&, ||
fn eval_binary_op(
    ctx: &mut EvalContext,
    op: BinaryOp,
    left: &Expression,
    right: &Expression,
) -> Result<Value, EvalError> {
    let l = eval_expr(ctx, left)?;
    let r = eval_expr(ctx, right)?;

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

/// Evaluate a unary operation.
///
/// ## Supported Operations
///
/// - Negation: -x
/// - Logical not: !x
/// - Positive: +x
fn eval_unary_op(
    ctx: &mut EvalContext,
    op: UnaryOp,
    operand: &Expression,
) -> Result<Value, EvalError> {
    let v = eval_expr(ctx, operand)?;
    match op {
        UnaryOp::Neg => Ok(Value::Number(-v.as_number()?)),
        UnaryOp::Not => Ok(Value::Boolean(!v.as_boolean())),
        UnaryOp::Pos => Ok(Value::Number(v.as_number()?)),
    }
}

// =============================================================================
// BUILT-IN FUNCTIONS
// =============================================================================

/// Evaluate a function call.
///
/// ## Evaluation Order
///
/// 1. User-defined functions (defined with `function name(params) = expr;`)
/// 2. Built-in functions (sin, cos, abs, etc.)
///
/// ## Supported Built-in Functions
///
/// - Trigonometric: sin, cos, tan
/// - Math: abs, sqrt, floor, ceil, round
/// - List: len
fn eval_function_call(
    ctx: &mut EvalContext,
    name: &str,
    args: &[Argument],
) -> Result<Value, EvalError> {
    // First, check for user-defined functions
    if let Some(func) = ctx.get_function(name).cloned() {
        return eval_user_function(ctx, &func, args);
    }

    // Evaluate arguments for built-in functions
    let arg_values: Vec<_> = args.iter()
        .filter_map(|a| match a {
            Argument::Positional(e) => eval_expr(ctx, e).ok(),
            Argument::Named { value, .. } => eval_expr(ctx, value).ok(),
        })
        .collect();

    match name {
        // Trigonometric (angles in degrees)
        "sin" => {
            let angle = arg_values.first()
                .map(|v| v.as_number().unwrap_or(0.0).to_radians().sin())
                .unwrap_or(0.0);
            Ok(Value::Number(angle))
        }
        "cos" => {
            let angle = arg_values.first()
                .map(|v| v.as_number().unwrap_or(0.0).to_radians().cos())
                .unwrap_or(0.0);
            Ok(Value::Number(angle))
        }
        "tan" => {
            let angle = arg_values.first()
                .map(|v| v.as_number().unwrap_or(0.0).to_radians().tan())
                .unwrap_or(0.0);
            Ok(Value::Number(angle))
        }
        
        // Math functions
        "abs" => {
            let val = arg_values.first()
                .map(|v| v.as_number().unwrap_or(0.0).abs())
                .unwrap_or(0.0);
            Ok(Value::Number(val))
        }
        "sqrt" => {
            let val = arg_values.first()
                .map(|v| v.as_number().unwrap_or(0.0).sqrt())
                .unwrap_or(0.0);
            Ok(Value::Number(val))
        }
        "floor" => {
            let val = arg_values.first()
                .map(|v| v.as_number().unwrap_or(0.0).floor())
                .unwrap_or(0.0);
            Ok(Value::Number(val))
        }
        "ceil" => {
            let val = arg_values.first()
                .map(|v| v.as_number().unwrap_or(0.0).ceil())
                .unwrap_or(0.0);
            Ok(Value::Number(val))
        }
        "round" => {
            let val = arg_values.first()
                .map(|v| v.as_number().unwrap_or(0.0).round())
                .unwrap_or(0.0);
            Ok(Value::Number(val))
        }
        
        // List functions
        "len" => {
            match arg_values.first() {
                Some(Value::List(l)) => Ok(Value::Number(l.len() as f64)),
                Some(Value::String(s)) => Ok(Value::Number(s.len() as f64)),
                _ => Ok(Value::Undef),
            }
        }
        
        // Unknown function
        _ => {
            ctx.warn(format!("Unknown function: {}", name));
            Ok(Value::Undef)
        }
    }
}

// =============================================================================
// USER-DEFINED FUNCTIONS
// =============================================================================

/// Evaluate a user-defined function call.
///
/// Creates a new scope with the function parameters bound to argument values,
/// then evaluates the function body expression.
///
/// ## Parameters
///
/// - `ctx`: Evaluation context
/// - `func`: The user-defined function definition
/// - `args`: Arguments passed to the function
///
/// ## Example
///
/// ```text
/// function double(x) = x * 2;
/// double(5);  // Returns 10
/// ```
fn eval_user_function(
    ctx: &mut EvalContext,
    func: &super::context::FunctionDef,
    args: &[Argument],
) -> Result<Value, EvalError> {
    // Evaluate all arguments first
    let mut arg_values: Vec<Value> = Vec::new();
    let mut named_args: std::collections::HashMap<String, Value> = std::collections::HashMap::new();

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

    // Create a new scope for function evaluation
    ctx.scope.push();

    // Bind parameters to arguments
    for (i, param) in func.params.iter().enumerate() {
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
            Value::Undef
        };

        ctx.scope.define(&param.name, value);
    }

    // Evaluate function body
    let result = eval_expr(ctx, &func.body);

    // Pop function scope
    ctx.scope.pop();

    result
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> EvalContext {
        EvalContext::new()
    }

    #[test]
    fn test_eval_number() {
        let mut ctx = ctx();
        let result = eval_expr(&mut ctx, &Expression::Number(42.0)).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_eval_boolean() {
        let mut ctx = ctx();
        let result = eval_expr(&mut ctx, &Expression::Boolean(true)).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_eval_string() {
        let mut ctx = ctx();
        let result = eval_expr(&mut ctx, &Expression::String("hello".to_string())).unwrap();
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_eval_binary_add() {
        let mut ctx = ctx();
        let expr = Expression::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(Expression::Number(2.0)),
            right: Box::new(Expression::Number(3.0)),
        };
        let result = eval_expr(&mut ctx, &expr).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_eval_unary_neg() {
        let mut ctx = ctx();
        let expr = Expression::UnaryOp {
            op: UnaryOp::Neg,
            operand: Box::new(Expression::Number(5.0)),
        };
        let result = eval_expr(&mut ctx, &expr).unwrap();
        assert_eq!(result, Value::Number(-5.0));
    }

    #[test]
    fn test_eval_identifier_undefined() {
        let mut ctx = ctx();
        let result = eval_expr(&mut ctx, &Expression::Identifier("x".to_string())).unwrap();
        assert_eq!(result, Value::Undef);
        assert!(!ctx.warnings.is_empty());
    }

    #[test]
    fn test_eval_identifier_defined() {
        let mut ctx = ctx();
        ctx.scope.define("x", Value::Number(10.0));
        let result = eval_expr(&mut ctx, &Expression::Identifier("x".to_string())).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }
}

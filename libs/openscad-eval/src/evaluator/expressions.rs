use crate::context::EvaluationContext;
use crate::value::Value;
use glam::DVec3;
use openscad_ast::{BinaryOp, Expression, UnaryOp};

/// Evaluates an expression to a Value.
pub fn eval_expr(expr: &Expression, ctx: &EvaluationContext) -> Option<Value> {
    match expr {
        Expression::Number(n) => Some(Value::Number(*n)),
        Expression::Boolean(b) => Some(Value::Boolean(*b)),
        Expression::String(s) => Some(Value::String(s.clone())),
        Expression::Variable(name) => ctx.get_variable(name),
        Expression::Vector(items) => {
            let values: Vec<Value> = items.iter()
                .filter_map(|item| eval_expr(item, ctx))
                .collect();
            Some(Value::Vector(values))
        }
        Expression::Range { start, step, end } => {
            let s = eval_expr(start, ctx)?.as_f64()?;
            let e = eval_expr(end, ctx)?.as_f64()?;
            let st = if let Some(step_expr) = step {
                eval_expr(step_expr, ctx)?.as_f64()?
            } else {
                1.0
            };
            Some(Value::Range { start: s, step: st, end: e })
        }
        Expression::Binary { left, operator, right } => {
            let l = eval_expr(left, ctx)?;
            let r = eval_expr(right, ctx)?;
            eval_binary_op(l, *operator, r)
        }
        Expression::Unary { operator, operand } => {
             let v = eval_expr(operand, ctx)?;
             eval_unary_op(*operator, v)
        }
        Expression::Ternary { condition, then_expr, else_expr } => {
            let c = eval_expr(condition, ctx)?;
            if c.is_truthy() {
                eval_expr(then_expr, ctx)
            } else {
                eval_expr(else_expr, ctx)
            }
        }
        Expression::FunctionCall { name, arguments } => {
            eval_function_call(name, arguments, ctx)
        }
        _ => None,
    }
}

/// Evaluates an expression to an f64.
pub fn eval_expr_f64(expr: &Expression, ctx: &EvaluationContext) -> Option<f64> {
    eval_expr(expr, ctx).and_then(|v| v.as_f64())
}

fn eval_binary_op(left: Value, op: BinaryOp, right: Value) -> Option<Value> {
    // Strings
    if let (Value::String(l), Value::String(r)) = (&left, &right) {
        if op == BinaryOp::Add {
            return Some(Value::String(format!("{}{}", l, r)));
        }
        return Some(match op {
            BinaryOp::Equal => Value::Boolean(l == r),
            BinaryOp::NotEqual => Value::Boolean(l != r),
            BinaryOp::Less => Value::Boolean(l < r),
            BinaryOp::Greater => Value::Boolean(l > r),
            BinaryOp::LessEqual => Value::Boolean(l <= r),
            BinaryOp::GreaterEqual => Value::Boolean(l >= r),
            _ => Value::Undef,
        });
    }

    // Vectors
    if let (Value::Vector(l_vec), Value::Vector(r_vec)) = (&left, &right) {
        if l_vec.len() == r_vec.len() {
            let mut result = Vec::with_capacity(l_vec.len());
            for (l_item, r_item) in l_vec.iter().zip(r_vec.iter()) {
                let res = eval_binary_op(l_item.clone(), op, r_item.clone())?;
                result.push(res);
            }
            return Some(Value::Vector(result));
        }
    }
    
    // Vector-Scalar
    if let (Value::Vector(l_vec), r_val) = (&left, &right) {
        if r_val.as_f64().is_some() {
             // Scalar on right
             let mut result = Vec::with_capacity(l_vec.len());
             for l_item in l_vec {
                 let res = eval_binary_op(l_item.clone(), op, r_val.clone())?;
                 result.push(res);
             }
             return Some(Value::Vector(result));
        }
    }
    
    if let (l_val, Value::Vector(r_vec)) = (&left, &right) {
        if l_val.as_f64().is_some() {
             // Scalar on left
             let mut result = Vec::with_capacity(r_vec.len());
             for r_item in r_vec {
                 let res = eval_binary_op(l_val.clone(), op, r_item.clone())?;
                 result.push(res);
             }
             return Some(Value::Vector(result));
        }
    }

    // Try to treat as numbers first
    let l_f64 = left.as_f64();
    let r_f64 = right.as_f64();

    if let (Some(l), Some(r)) = (l_f64, r_f64) {
        return Some(match op {
            BinaryOp::Add => Value::Number(l + r),
            BinaryOp::Subtract => Value::Number(l - r),
            BinaryOp::Multiply => Value::Number(l * r),
            BinaryOp::Divide => Value::Number(if r != 0.0 { l / r } else { f64::NAN }),
            BinaryOp::Modulo => Value::Number(l % r),
            BinaryOp::Power => Value::Number(l.powf(r)),
            BinaryOp::Less => Value::Boolean(l < r),
            BinaryOp::Greater => Value::Boolean(l > r),
            BinaryOp::LessEqual => Value::Boolean(l <= r),
            BinaryOp::GreaterEqual => Value::Boolean(l >= r),
            BinaryOp::Equal => Value::Boolean((l - r).abs() < 1e-10),
            BinaryOp::NotEqual => Value::Boolean((l - r).abs() >= 1e-10),
            BinaryOp::And => Value::Boolean(l != 0.0 && r != 0.0),
            BinaryOp::Or => Value::Boolean(l != 0.0 || r != 0.0),
        });
    }
    
    // Handle equality for other types
    match op {
        BinaryOp::Equal => Some(Value::Boolean(left == right)),
        BinaryOp::NotEqual => Some(Value::Boolean(left != right)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openscad_ast::BinaryOp;

    #[test]
    fn test_eval_binary_op_numbers() {
        assert_eq!(eval_binary_op(Value::Number(1.0), BinaryOp::Add, Value::Number(2.0)), Some(Value::Number(3.0)));
        assert_eq!(eval_binary_op(Value::Number(10.0), BinaryOp::Divide, Value::Number(2.0)), Some(Value::Number(5.0)));
    }
    
    #[test]
    fn test_eval_binary_op_strings() {
        assert_eq!(
            eval_binary_op(Value::String("a".into()), BinaryOp::Add, Value::String("b".into())),
            Some(Value::String("ab".into()))
        );
        assert_eq!(
            eval_binary_op(Value::String("a".into()), BinaryOp::Less, Value::String("b".into())),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            eval_binary_op(Value::String("b".into()), BinaryOp::Greater, Value::String("a".into())),
            Some(Value::Boolean(true))
        );
        assert_eq!(
            eval_binary_op(Value::String("a".into()), BinaryOp::Equal, Value::String("a".into())),
            Some(Value::Boolean(true))
        );
    }
    
    #[test]
    fn test_eval_len_string() {
        let ctx = EvaluationContext::new();
        // len("hello") -> 5
        let expr = Expression::FunctionCall {
            name: "len".to_string(),
            arguments: vec![
                openscad_ast::Argument { name: None, value: Expression::String("hello".to_string()) }
            ]
        };
        assert_eq!(eval_expr(&expr, &ctx), Some(Value::Number(5.0)));
    }

    #[test]
    fn test_eval_len_vector() {
        let ctx = EvaluationContext::new();
        // len([1, 2, 3]) -> 3
        let expr = Expression::FunctionCall {
            name: "len".to_string(),
            arguments: vec![
                openscad_ast::Argument { name: None, value: Expression::Vector(vec![
                    Expression::Number(1.0), Expression::Number(2.0), Expression::Number(3.0)
                ]) }
            ]
        };
        assert_eq!(eval_expr(&expr, &ctx), Some(Value::Number(3.0)));
    }
    
    #[test]
    fn test_eval_binary_op_vectors() {
        let v1 = Value::Vector(vec![Value::Number(1.0), Value::Number(2.0)]);
        let v2 = Value::Vector(vec![Value::Number(3.0), Value::Number(4.0)]);
        
        // [1,2] + [3,4] = [4,6]
        let res = eval_binary_op(v1.clone(), BinaryOp::Add, v2.clone());
        match res {
            Some(Value::Vector(v)) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], Value::Number(4.0));
                assert_eq!(v[1], Value::Number(6.0));
            }
            _ => panic!("Expected vector result for Add"),
        }
        
        // [1,2] - [3,4] = [-2,-2]
        let res_sub = eval_binary_op(v1.clone(), BinaryOp::Subtract, v2.clone());
        match res_sub {
            Some(Value::Vector(v)) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], Value::Number(-2.0));
                assert_eq!(v[1], Value::Number(-2.0));
            }
            _ => panic!("Expected vector result for Subtract"),
        }

        // [1,2] * 2 = [2,4]
        let res_scalar = eval_binary_op(v1.clone(), BinaryOp::Multiply, Value::Number(2.0));
         match res_scalar {
            Some(Value::Vector(v)) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], Value::Number(2.0));
                assert_eq!(v[1], Value::Number(4.0));
            }
            _ => panic!("Expected vector result for Multiply"),
        }

        // [2,4] / 2 = [1,2]
        let v3 = Value::Vector(vec![Value::Number(2.0), Value::Number(4.0)]);
        let res_div = eval_binary_op(v3, BinaryOp::Divide, Value::Number(2.0));
        match res_div {
            Some(Value::Vector(v)) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0], Value::Number(1.0));
                assert_eq!(v[1], Value::Number(2.0));
            }
            _ => panic!("Expected vector result for Divide"),
        }
    }
    
    /// Tests version() function returns [year, month, day] vector
    #[test]
    fn test_eval_version() {
        let ctx = EvaluationContext::new();
        let expr = Expression::FunctionCall {
            name: "version".to_string(),
            arguments: vec![]
        };
        let result = eval_expr(&expr, &ctx);
        match result {
            Some(Value::Vector(v)) => {
                assert_eq!(v.len(), 3, "version() should return 3-element vector");
                assert!(matches!(v[0], Value::Number(_)), "Year should be number");
                assert!(matches!(v[1], Value::Number(_)), "Month should be number");
                assert!(matches!(v[2], Value::Number(_)), "Day should be number");
            }
            _ => panic!("version() should return a vector"),
        }
    }
    
    /// Tests version_num() function returns YYYYMMDD number
    #[test]
    fn test_eval_version_num() {
        let ctx = EvaluationContext::new();
        let expr = Expression::FunctionCall {
            name: "version_num".to_string(),
            arguments: vec![]
        };
        let result = eval_expr(&expr, &ctx);
        match result {
            Some(Value::Number(n)) => {
                assert!(n >= 20000000.0, "version_num should be >= 20000000");
                assert!(n < 30000000.0, "version_num should be < 30000000");
            }
            _ => panic!("version_num() should return a number"),
        }
    }
    
    /// Tests str() function converts values to strings
    #[test]
    fn test_eval_str() {
        let ctx = EvaluationContext::new();
        // str(123) -> "123"
        let expr = Expression::FunctionCall {
            name: "str".to_string(),
            arguments: vec![
                openscad_ast::Argument { name: None, value: Expression::Number(123.0) }
            ]
        };
        assert_eq!(eval_expr(&expr, &ctx), Some(Value::String("123".to_string())));
    }
    
    /// Tests concat() function concatenates vectors
    #[test]
    fn test_eval_concat() {
        let ctx = EvaluationContext::new();
        // concat([1,2], [3,4]) -> [1,2,3,4]
        let expr = Expression::FunctionCall {
            name: "concat".to_string(),
            arguments: vec![
                openscad_ast::Argument { 
                    name: None, 
                    value: Expression::Vector(vec![Expression::Number(1.0), Expression::Number(2.0)]) 
                },
                openscad_ast::Argument { 
                    name: None, 
                    value: Expression::Vector(vec![Expression::Number(3.0), Expression::Number(4.0)]) 
                },
            ]
        };
        let result = eval_expr(&expr, &ctx);
        match result {
            Some(Value::Vector(v)) => {
                assert_eq!(v.len(), 4);
                assert_eq!(v[0], Value::Number(1.0));
                assert_eq!(v[3], Value::Number(4.0));
            }
            _ => panic!("concat() should return a vector"),
        }
    }
}


fn eval_unary_op(op: UnaryOp, value: Value) -> Option<Value> {
    match op {
        UnaryOp::Negate => value.as_f64().map(|v| Value::Number(-v)),
        UnaryOp::Not => Some(Value::Boolean(!value.is_truthy())),
    }
}

fn eval_function_call(name: &str, arguments: &[openscad_ast::Argument], ctx: &EvaluationContext) -> Option<Value> {
    // Evaluate arguments
    let args: Vec<Value> = arguments.iter()
        .filter_map(|a| eval_expr(&a.value, ctx))
        .collect();
    
    // Check for user-defined function first
    if let Some(func_def) = ctx.get_function(name).cloned() {
        let mut func_ctx = ctx.clone();
        func_ctx.push_scope();
        
        for (i, (param_name, default_expr)) in func_def.parameters.iter().enumerate() {
            let value = args.get(i).cloned()
                .or_else(|| default_expr.as_ref().and_then(|e| eval_expr(e, ctx)))
                .unwrap_or(Value::Undef);
            func_ctx.set_variable(param_name, value);
        }
        
        let result = eval_expr(&func_def.body, &func_ctx);
        // func_ctx.pop_scope(); // Dropped automatically
        return result;
    }
    
    // Built-in functions (simplified for now, mostly numeric)
    // Map Value args to f64 for math functions
    let args_f64: Vec<f64> = args.iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();

    match name {
        "sin" => args_f64.first().map(|x| Value::Number(x.to_radians().sin())),
        "cos" => args_f64.first().map(|x| Value::Number(x.to_radians().cos())),
        "tan" => args_f64.first().map(|x| Value::Number(x.to_radians().tan())),
        "asin" => args_f64.first().map(|x| Value::Number(x.asin().to_degrees())),
        "acos" => args_f64.first().map(|x| Value::Number(x.acos().to_degrees())),
        "atan" => args_f64.first().map(|x| Value::Number(x.atan().to_degrees())),
        "atan2" => args_f64.get(0).and_then(|y| args_f64.get(1).map(|x| Value::Number(y.atan2(*x).to_degrees()))),
        
        "abs" => args_f64.first().map(|x| Value::Number(x.abs())),
        "ceil" => args_f64.first().map(|x| Value::Number(x.ceil())),
        "floor" => args_f64.first().map(|x| Value::Number(x.floor())),
        "round" => args_f64.first().map(|x| Value::Number(x.round())),
        "sqrt" => args_f64.first().map(|x| Value::Number(x.sqrt())),
        "exp" => args_f64.first().map(|x| Value::Number(x.exp())),
        "ln" => args_f64.first().map(|x| Value::Number(x.ln())),
        "log" => args_f64.first().map(|x| Value::Number(x.log10())),
        "pow" => args_f64.get(0).and_then(|base| args_f64.get(1).map(|exp| Value::Number(base.powf(*exp)))),
        "sign" => args_f64.first().map(|x| Value::Number(if *x > 0.0 { 1.0 } else if *x < 0.0 { -1.0 } else { 0.0 })),
        "min" => args_f64.iter().copied().reduce(f64::min).map(Value::Number),
        "max" => args_f64.iter().copied().reduce(f64::max).map(Value::Number),
        
        "len" => {
            if let Some(arg) = args.first() {
                match arg {
                    Value::String(s) => Some(Value::Number(s.len() as f64)),
                    Value::Vector(v) => Some(Value::Number(v.len() as f64)),
                    _ => Some(Value::Undef),
                }
            } else {
                 Some(Value::Undef)
            }
        },
        
        // version() - Returns OpenSCAD version as [year, month, day] vector
        // Example: version() -> [2024, 1, 1]
        // Compatible with OpenSCAD's version() function
        "version" => Some(Value::Vector(vec![
            Value::Number(2024.0),
            Value::Number(1.0),
            Value::Number(1.0),
        ])),
        
        // version_num() - Returns OpenSCAD version as single number YYYYMMDD
        // Example: version_num() -> 20240101
        // Compatible with OpenSCAD's version_num() function
        "version_num" => Some(Value::Number(20240101.0)),
        
        // str() - Converts value to string representation
        // Example: str(123) -> "123", str([1,2,3]) -> "[1, 2, 3]"
        "str" => {
            let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
            Some(Value::String(parts.join("")))
        },
        
        // concat() - Concatenates vectors or strings
        // Example: concat([1,2], [3,4]) -> [1,2,3,4]
        "concat" => {
            let mut result = Vec::new();
            for arg in &args {
                match arg {
                    Value::Vector(v) => result.extend(v.clone()),
                    other => result.push(other.clone()),
                }
            }
            Some(Value::Vector(result))
        },
        
        // lookup() - Linear interpolation lookup in table
        // Example: lookup(1.5, [[0,0], [1,10], [2,20]]) -> 15
        "lookup" => {
            if args.len() >= 2 {
                let key = args[0].as_f64()?;
                if let Value::Vector(table) = &args[1] {
                    // Table is [[key, value], [key, value], ...]
                    let mut pairs: Vec<(f64, f64)> = table.iter()
                        .filter_map(|row| {
                            if let Value::Vector(pair) = row {
                                if pair.len() >= 2 {
                                    Some((pair[0].as_f64()?, pair[1].as_f64()?))
                                } else { None }
                            } else { None }
                        })
                        .collect();
                    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                    
                    if pairs.is_empty() {
                        return Some(Value::Undef);
                    }
                    if key <= pairs[0].0 {
                        return Some(Value::Number(pairs[0].1));
                    }
                    if key >= pairs.last().unwrap().0 {
                        return Some(Value::Number(pairs.last().unwrap().1));
                    }
                    // Linear interpolation
                    for i in 0..pairs.len() - 1 {
                        if key >= pairs[i].0 && key <= pairs[i + 1].0 {
                            let t = (key - pairs[i].0) / (pairs[i + 1].0 - pairs[i].0);
                            let val = pairs[i].1 + t * (pairs[i + 1].1 - pairs[i].1);
                            return Some(Value::Number(val));
                        }
                    }
                }
            }
            Some(Value::Undef)
        },
        
        _ => None,
    }
}

/// Evaluates an expression to a DVec3.
pub fn eval_vec3(expr: &Expression, ctx: &EvaluationContext) -> DVec3 {
    let val = eval_expr(expr, ctx).unwrap_or(Value::Undef);
    val.as_vec3().unwrap_or(DVec3::ZERO)
}

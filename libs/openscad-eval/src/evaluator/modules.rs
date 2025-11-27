use crate::context::{EvaluationContext, FunctionDefinition, ModuleDefinition};
use crate::error::EvalError;
use crate::ir::{BooleanOperation, GeometryNode};
use crate::value::Value;
use openscad_ast::{Argument, Expression, Parameter, Span, Statement};
use super::expressions::eval_expr;
use super::evaluate_statements;

pub fn register_module(
    name: &str,
    parameters: &[Parameter],
    body: &[Statement],
    ctx: &mut EvaluationContext
) -> Result<Option<GeometryNode>, EvalError> {
    let params: Vec<(String, Option<Expression>)> = parameters.iter()
        .map(|p| {
            (p.name.clone(), p.default.clone())
        })
        .collect();
        
    ctx.register_module(name.to_string(), ModuleDefinition {
        parameters: params,
        body: body.to_vec(),
    });
    Ok(None)
}

pub fn register_function(
    name: &str,
    parameters: &[Parameter],
    body: &Expression,
    ctx: &mut EvaluationContext
) -> Result<Option<GeometryNode>, EvalError> {
     let params: Vec<(String, Option<Expression>)> = parameters.iter()
        .map(|p| {
            (p.name.clone(), p.default.clone())
        })
        .collect();
        
    ctx.register_function(name.to_string(), FunctionDefinition {
        parameters: params,
        body: body.clone(),
    });
    Ok(None)
}

pub fn eval_module_call(
    name: &str,
    arguments: &[Argument],
    children: &[Statement],
    span: Span,
    ctx: &mut EvaluationContext
) -> Result<Option<GeometryNode>, EvalError> {
    // Handle special "children" module
    if name == "children" {
        // Use get_current_children_index to find the children associated with the current scope context
        // This respects the current scope stack (which may have been temporarily reduced)
        if let Some(idx) = ctx.get_current_children_index() {
            if let Some((available_children, definition_scope_depth)) = ctx.children_stack.get(idx).cloned() {
                let selected_indices: Vec<usize> = if arguments.is_empty() {
                    (0..available_children.len()).collect()
                } else {
                    // Evaluate first argument to get index or range
                    let arg_val = eval_expr(&arguments[0].value, ctx).unwrap_or(Value::Undef);
                    match arg_val {
                        Value::Number(n) => {
                            if n >= 0.0 {
                                let idx = n as usize;
                                if idx < available_children.len() {
                                    vec![idx]
                                } else {
                                    vec![]
                                }
                            } else {
                                vec![]
                            }
                        },
                        Value::Vector(v) => {
                             let mut indices = Vec::new();
                             for item in v {
                                 if let Value::Number(n) = item {
                                     if n >= 0.0 {
                                         let idx = n as usize;
                                         if idx < available_children.len() {
                                             indices.push(idx);
                                         }
                                     }
                                 }
                             }
                             indices
                        },
                        _ => vec![]
                    }
                };

                if selected_indices.is_empty() {
                    return Ok(None);
                }

                // Select statements
                let mut statements_to_eval = Vec::new();
                for idx in selected_indices {
                    if let Some(stmt) = available_children.get(idx) {
                        statements_to_eval.push(stmt.clone());
                    }
                }

                // Evaluate in the definition scope
                // We temporarily pop scopes to match the definition scope depth
                let popped_scopes = ctx.temporary_pop_scopes(definition_scope_depth);
                
                let result = evaluate_statements(&statements_to_eval, ctx);
                
                ctx.restore_scopes(popped_scopes);
                
                let nodes = result?;
                if nodes.is_empty() {
                    Ok(None)
                } else if nodes.len() == 1 {
                    Ok(Some(nodes.into_iter().next().unwrap()))
                } else {
                    Ok(Some(GeometryNode::Boolean {
                        operation: BooleanOperation::Union,
                        children: nodes,
                        span,
                    }))
                }
            } else {
                Ok(None)
            }
        } else {
            // No children available
            Ok(None)
        }
    } else if let Some(module_def) = ctx.get_module(name).cloned() {
        if ctx.recursion_depth >= ctx.max_recursion_depth {
            return Err(EvalError::RecursionLimit { span });
        }

        ctx.recursion_depth += 1;
        ctx.call_stack.push(name.to_string());
        
        // Push children for this module call
        let children_index = ctx.children_stack.len();
        ctx.children_stack.push((children.to_vec(), ctx.scope_depth()));
        
        // Evaluate arguments in the current scope (before pushing new module scope)
        let mut evaluated_args = Vec::new();
        for (i, (param_name, default_expr)) in module_def.parameters.iter().enumerate() {
            let value = if let Some(arg) = arguments.get(i) {
                // Positional or named argument?
                // For now assuming positional if argument name is None, or matching name if present
                // But simplified logic: match by position for now (OpenSCAD supports named args too, but let's stick to positional for MVP unless logic handles both)
                // Actually `arguments` in AST has `name` field.
                
                // Better argument matching logic:
                // 1. If arg has name, match param by name.
                // 2. If arg has no name, match by position (skipping already matched).
                
                // For now, let's assume positional for simplicity as `arguments` is a list.
                // TODO: Implement proper named argument handling.
                eval_expr(&arg.value, ctx).unwrap_or(Value::Undef)
            } else if let Some(expr) = default_expr {
                eval_expr(expr, ctx).unwrap_or(Value::Undef)
            } else {
                Value::Undef
            };
            evaluated_args.push((param_name.clone(), value));
        }

        ctx.push_module_scope(Some(children_index));
        
        // Bind arguments to new scope
        for (name, value) in evaluated_args {
            ctx.set_variable(&name, value);
        }
        
        let result = evaluate_statements(&module_def.body, ctx);
        
        ctx.pop_scope();
        
        // Pop children
        ctx.children_stack.pop();
        
        ctx.call_stack.pop();
        ctx.recursion_depth -= 1;
        
        let nodes = result?;
        if nodes.is_empty() {
            Ok(None)
        } else if nodes.len() == 1 {
            Ok(Some(nodes.into_iter().next().unwrap()))
        } else {
            Ok(Some(GeometryNode::Boolean {
                operation: BooleanOperation::Union,
                children: nodes,
                span,
            }))
        }
    } else {
        Err(EvalError::UnknownModule {
            name: name.to_string(),
            span,
        })
    }
}

pub fn eval_assignment(name: &str, value: &Expression, ctx: &mut EvaluationContext) -> Result<Option<GeometryNode>, EvalError> {
    if let Some(val) = eval_expr(value, ctx) {
        ctx.set_variable(name, val);
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::EvaluationContext;
    use openscad_ast::{Span, Statement, BinaryOp};

    #[test]
    fn test_recursion_limit() {
        let mut ctx = EvaluationContext::new();
        ctx.max_recursion_depth = 5; // Low limit for testing
        
        // Register a module that calls itself
        // module recursive() { recursive(); }
        let body = vec![
            Statement::ModuleCall {
                name: "recursive".to_string(),
                arguments: vec![],
                children: vec![],
                span: Span::default(),
            }
        ];
        
        ctx.register_module("recursive".to_string(), ModuleDefinition {
            parameters: vec![],
            body,
        });
        
        let result = eval_module_call(
            "recursive",
            &[],
            &[],
            Span::default(),
            &mut ctx
        );
        
        match result {
            Err(EvalError::RecursionLimit { .. }) => (), // Expected
            _ => panic!("Expected RecursionLimit error, got {:?}", result),
        }
    }

    #[test]
    fn test_children_passing() {
        let mut ctx = EvaluationContext::new();
        
        // module child_user() { children(); }
        // child_user() { cube(10); }
        
        let child_stmt = Statement::Cube {
            size: Expression::Number(10.0),
            center: false,
            span: Span::default(),
        };
        
        let body = vec![
            Statement::ModuleCall {
                name: "children".to_string(),
                arguments: vec![],
                children: vec![],
                span: Span::default(),
            }
        ];
        
        ctx.register_module("child_user".to_string(), ModuleDefinition {
            parameters: vec![],
            body,
        });
        
        let result = eval_module_call(
            "child_user",
            &[],
            &[child_stmt],
            Span::default(),
            &mut ctx
        );
        
        assert!(result.is_ok());
        let node = result.unwrap();
        assert!(node.is_some());
        
        match node.unwrap() {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0].into());
            }
            GeometryNode::Boolean { operation, children, .. } => {
                assert_eq!(operation, BooleanOperation::Union);
                assert!(!children.is_empty());
                // Should contain the cube
            }
            _ => panic!("Expected Cube or Union"),
        }
    }

    #[test]
    fn test_nested_children_passing() {
        let mut ctx = EvaluationContext::new();
        
        // module inner() { children(); }
        // module outer() { inner() { children(); } }
        // outer() { cube(10); }
        
        let child_stmt = Statement::Cube {
            size: Expression::Number(10.0),
            center: false,
            span: Span::default(),
        };
        
        // inner definition
        let inner_body = vec![
            Statement::ModuleCall {
                name: "children".to_string(),
                arguments: vec![],
                children: vec![],
                span: Span::default(),
            }
        ];
        ctx.register_module("inner".to_string(), ModuleDefinition {
            parameters: vec![],
            body: inner_body,
        });
        
        // outer definition
        let outer_body = vec![
            Statement::ModuleCall {
                name: "inner".to_string(),
                arguments: vec![],
                children: vec![
                    Statement::ModuleCall {
                        name: "children".to_string(),
                        arguments: vec![],
                        children: vec![],
                        span: Span::default(),
                    }
                ],
                span: Span::default(),
            }
        ];
        ctx.register_module("outer".to_string(), ModuleDefinition {
            parameters: vec![],
            body: outer_body,
        });
        
        let result = eval_module_call(
            "outer",
            &[],
            &[child_stmt],
            Span::default(),
            &mut ctx
        );
        
        assert!(result.is_ok());
        let node = result.unwrap();
        assert!(node.is_some());
        
        // Should eventually resolve to the cube
        match node.unwrap() {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0].into());
            }
            GeometryNode::Boolean { operation, children, .. } => {
                assert_eq!(operation, BooleanOperation::Union);
                assert!(!children.is_empty());
            }
            _ => panic!("Expected Cube or Union"),
        }
    }

    #[test]
    fn test_children_count_variable() {
        let mut ctx = EvaluationContext::new();
        
        // module check_count() { if ($children == 2) { cube(10); } }
        // check_count() { cube(1); cube(2); }
        
        let check_body = vec![
            Statement::If {
                condition: Expression::Binary {
                    left: Box::new(Expression::Variable("$children".to_string())),
                    operator: BinaryOp::Equal,
                    right: Box::new(Expression::Number(2.0))
                },
                then_branch: vec![Statement::Cube {
                    size: Expression::Number(10.0),
                    center: false,
                    span: Span::default(),
                }],
                else_branch: None,
                span: Span::default(),
            }
        ];
        
        ctx.register_module("check_count".to_string(), ModuleDefinition {
            parameters: vec![],
            body: check_body,
        });
        
        let children = vec![
            Statement::Cube { size: Expression::Number(1.0), center: false, span: Span::default() },
            Statement::Cube { size: Expression::Number(2.0), center: false, span: Span::default() },
        ];
        
        let result = eval_module_call(
            "check_count",
            &[],
            &children,
            Span::default(),
            &mut ctx
        );
        
        assert!(result.is_ok());
        let node = result.unwrap();
        assert!(node.is_some()); // Should produce cube(10)
        
         match node.unwrap() {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0].into());
            }
            GeometryNode::Boolean { children, .. } => {
                // If it returns a union (common for block statements), check children
                 assert!(!children.is_empty());
            }
            _ => panic!("Expected Cube or Union"),
        }
    }

    #[test]
    fn test_module_scope_isolation() {
        let mut ctx = EvaluationContext::new();
        
        // x = 10;
        // module set_x() { x = 20; }
        // set_x();
        // check x == 10
        
        ctx.set_variable("x", Value::Number(10.0));
        
        let module_body = vec![
            Statement::Assignment {
                name: "x".to_string(),
                value: Expression::Number(20.0),
                span: Span::default(),
            }
        ];
        
        ctx.register_module("set_x".to_string(), ModuleDefinition {
            parameters: vec![],
            body: module_body,
        });
        
        let result = eval_module_call(
            "set_x",
            &[],
            &[],
            Span::default(),
            &mut ctx
        );
        
        assert!(result.is_ok());
        
        // Check outer variable is unchanged
        assert_eq!(ctx.get_variable("x"), Some(Value::Number(10.0)));
    }

    #[test]
    fn test_default_param_scoping() {
        let mut ctx = EvaluationContext::new();
        
        // a = 10;
        // module test(val = a) { cube(val); }
        // {
        //   a = 20;
        //   test(); 
        // }
        // Should use a=20 if dynamic scoping works for default params
        
        ctx.set_variable("a", Value::Number(10.0));
        
        let body = vec![
            Statement::Cube {
                size: Expression::Variable("val".to_string()),
                center: false,
                span: Span::default(),
            }
        ];
        
        ctx.register_module("test_module".to_string(), ModuleDefinition {
            parameters: vec![
                ("val".to_string(), Some(Expression::Variable("a".to_string())))
            ],
            body,
        });
        
        // Call in a new scope where a = 20
        ctx.push_scope();
        ctx.set_variable("a", Value::Number(20.0));
        
        let result = eval_module_call(
            "test_module",
            &[],
            &[],
            Span::default(),
            &mut ctx
        );
        
        ctx.pop_scope();
        
        assert!(result.is_ok());
        let node = result.unwrap().unwrap();
        
        match node {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [20.0, 20.0, 20.0].into());
            }
            _ => panic!("Expected cube"),
        }
    }
}

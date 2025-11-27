use crate::context::EvaluationContext;
use crate::error::EvalError;
use crate::ir::{BooleanOperation, GeometryNode};
use crate::value::Value;
use openscad_ast::{Expression, Span, Statement};
use super::expressions::eval_expr;
use super::evaluate_statements;

pub fn eval_for_loop(
    variable: &str,
    range: &Expression,
    body: &[Statement],
    span: Span,
    ctx: &mut EvaluationContext
) -> Result<Option<GeometryNode>, EvalError> {
    let mut all_nodes = Vec::new();
    
    // Evaluate range logic
    match range {
        Expression::Range { start, step, end } => {
            let s = eval_expr(start, ctx).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let st = step.as_ref()
                .map(|e| eval_expr(e, ctx).and_then(|v| v.as_f64()).unwrap_or(1.0))
                .unwrap_or(1.0);
            let e = eval_expr(end, ctx).and_then(|v| v.as_f64()).unwrap_or(0.0);
            
            // Iterate with range [start:step:end]
            let mut i = s;
            let max_iterations = 10000; // Safety limit
            let mut count = 0;
            while (st > 0.0 && i <= e) || (st < 0.0 && i >= e) {
                if count >= max_iterations { break; }
                ctx.push_scope();
                ctx.set_variable(variable, Value::Number(i));
                let nodes = evaluate_statements(body, ctx)?;
                all_nodes.extend(nodes);
                ctx.pop_scope();
                i += st;
                count += 1;
            }
        }
        Expression::Vector(items) => {
            // Iterate over vector items
            // The items themselves are Expressions, need to evaluate them?
            // Or Expression::Vector contains Expressions.
            for item in items {
                if let Some(val) = eval_expr(item, ctx) {
                    ctx.push_scope();
                    ctx.set_variable(variable, val);
                    let nodes = evaluate_statements(body, ctx)?;
                    all_nodes.extend(nodes);
                    ctx.pop_scope();
                }
            }
        }
        _ => {
            // Treat as single value or evaluated vector
            if let Some(val) = eval_expr(range, ctx) {
                match val {
                    Value::Vector(items) => {
                        for item in items {
                            ctx.push_scope();
                            ctx.set_variable(variable, item);
                            let nodes = evaluate_statements(body, ctx)?;
                            all_nodes.extend(nodes);
                            ctx.pop_scope();
                        }
                    }
                    Value::Range { start, step, end } => {
                         let mut i = start;
                        let max_iterations = 10000;
                        let mut count = 0;
                        while (step > 0.0 && i <= end) || (step < 0.0 && i >= end) {
                            if count >= max_iterations { break; }
                            ctx.push_scope();
                            ctx.set_variable(variable, Value::Number(i));
                            let nodes = evaluate_statements(body, ctx)?;
                            all_nodes.extend(nodes);
                            ctx.pop_scope();
                            i += step;
                            count += 1;
                        }
                    }
                    _ => {
                        // Single value iteration?
                        // OpenSCAD: for(i=10) -> i is 10 once?
                        // Yes.
                        ctx.push_scope();
                        ctx.set_variable(variable, val);
                        let nodes = evaluate_statements(body, ctx)?;
                        all_nodes.extend(nodes);
                        ctx.pop_scope();
                    }
                }
            }
        }
    };
    
    if all_nodes.is_empty() {
        Ok(None)
    } else {
        Ok(Some(GeometryNode::Boolean {
            operation: BooleanOperation::Union,
            children: all_nodes,
            span,
        }))
    }
}

pub fn eval_if(
    condition: &Expression,
    then_branch: &[Statement],
    else_branch: &Option<Vec<Statement>>,
    span: Span,
    ctx: &mut EvaluationContext
) -> Result<Option<GeometryNode>, EvalError> {
    let cond_val = eval_expr(condition, ctx).unwrap_or(Value::Undef);
    
    let branch = if cond_val.is_truthy() { 
        then_branch 
    } else { 
        else_branch.as_ref().map(|v| v.as_slice()).unwrap_or(&[])
    };
    
    // We need evaluate_children here too, or just reuse evaluate_statements manually
    // evaluate_statements returns Vec<GeometryNode>
    // GeometryNode::Boolean expects children: Vec<GeometryNode>
    
    ctx.push_scope();
    let child_nodes = evaluate_statements(branch, ctx)?;
    ctx.pop_scope();
    
    if child_nodes.is_empty() {
        Ok(None)
    } else if child_nodes.len() == 1 {
        Ok(Some(child_nodes.into_iter().next().unwrap()))
    } else {
        Ok(Some(GeometryNode::Boolean {
            operation: BooleanOperation::Union, // Implicit union for if block
            children: child_nodes,
            span,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::EvaluationContext;
    use openscad_ast::{Expression, Statement, Span};
    use crate::ir::GeometryNode;

    fn create_ctx() -> EvaluationContext {
        EvaluationContext::new()
    }

    fn dummy_span() -> Span {
        Span::new(0, 0)
    }

    #[test]
    fn test_eval_for_loop_range() {
        let mut ctx = create_ctx();
        // for(i=[0:1:2]) cube(i)
        // i=0, i=1, i=2
        
        let range = Expression::Range {
            start: Box::new(Expression::Number(0.0)),
            step: Some(Box::new(Expression::Number(1.0))),
            end: Box::new(Expression::Number(2.0)),
        };
        
        let body = vec![
            Statement::Cube {
                size: Expression::Variable("i".to_string()),
                center: false,
                span: dummy_span(),
            }
        ];
        
        let result = eval_for_loop("i", &range, &body, dummy_span(), &mut ctx);
        
        assert!(result.is_ok());
        let node = result.unwrap();
        assert!(node.is_some());
        
        if let Some(GeometryNode::Boolean { operation, children, .. }) = node {
            assert_eq!(operation, BooleanOperation::Union);
            assert_eq!(children.len(), 3);
            // i=0 -> cube(0)
            // i=1 -> cube(1)
            // i=2 -> cube(2)
        } else {
            panic!("Expected Boolean Union");
        }
    }
    
    #[test]
    fn test_eval_if_true() {
        let mut ctx = create_ctx();
        // if(1) cube(10)
        
        let condition = Expression::Number(1.0);
        let then_branch = vec![
            Statement::Cube {
                size: Expression::Number(10.0),
                center: false,
                span: dummy_span(),
            }
        ];
        
        let result = eval_if(&condition, &then_branch, &None, dummy_span(), &mut ctx);
        
        assert!(result.is_ok());
        let node = result.unwrap();
        assert!(node.is_some());
        
        // Should return the single node directly if only 1 child
        if let Some(GeometryNode::Cube { size, .. }) = node {
             assert_eq!(size, glam::DVec3::new(10.0, 10.0, 10.0));
        } else {
             panic!("Expected Cube node");
        }
    }

    #[test]
    fn test_eval_if_false_no_else() {
        let mut ctx = create_ctx();
        // if(0) cube(10)
        
        let condition = Expression::Number(0.0);
        let then_branch = vec![
            Statement::Cube {
                size: Expression::Number(10.0),
                center: false,
                span: dummy_span(),
            }
        ];
        
        let result = eval_if(&condition, &then_branch, &None, dummy_span(), &mut ctx);
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_eval_for_loop_vector() {
        let mut ctx = create_ctx();
        // for(i=[10, 20]) cube(i)
        
        let range = Expression::Vector(vec![
            Expression::Number(10.0),
            Expression::Number(20.0),
        ]);
        
        let body = vec![
             Statement::Cube {
                size: Expression::Variable("i".to_string()),
                center: false,
                span: dummy_span(),
            }
        ];
        
        let result = eval_for_loop("i", &range, &body, dummy_span(), &mut ctx);
        
        assert!(result.is_ok());
        if let Some(GeometryNode::Boolean { operation, children, .. }) = result.unwrap() {
            assert_eq!(operation, BooleanOperation::Union);
            assert_eq!(children.len(), 2);
        } else {
            panic!("Expected Union");
        }
    }

    #[test]
    fn test_eval_if_false_else() {
        let mut ctx = create_ctx();
        // if(0) cube(10) else cube(20)
        
        let condition = Expression::Number(0.0);
        let then_branch = vec![
            Statement::Cube {
                size: Expression::Number(10.0),
                center: false,
                span: dummy_span(),
            }
        ];
        let else_branch = vec![
            Statement::Cube {
                size: Expression::Number(20.0),
                center: false,
                span: dummy_span(),
            }
        ];
        
        let result = eval_if(&condition, &then_branch, &Some(else_branch), dummy_span(), &mut ctx);
        
        assert!(result.is_ok());
        let node = result.unwrap();
        assert!(node.is_some());
        
        if let Some(GeometryNode::Cube { size, .. }) = node {
             assert_eq!(size, glam::DVec3::new(20.0, 20.0, 20.0));
        } else {
             panic!("Expected Cube node");
        }
    }

    #[test]
    fn test_eval_for_loop_negative_step() {
        let mut ctx = create_ctx();
        // for(i=[10:-2:6]) cube(i)
        // i=10, i=8, i=6
        
        let range = Expression::Range {
            start: Box::new(Expression::Number(10.0)),
            step: Some(Box::new(Expression::Number(-2.0))),
            end: Box::new(Expression::Number(6.0)),
        };
        
        let body = vec![
            Statement::Cube {
                size: Expression::Variable("i".to_string()),
                center: false,
                span: dummy_span(),
            }
        ];
        
        let result = eval_for_loop("i", &range, &body, dummy_span(), &mut ctx);
        
        assert!(result.is_ok());
        if let Some(GeometryNode::Boolean { operation, children, .. }) = result.unwrap() {
            assert_eq!(operation, BooleanOperation::Union);
            assert_eq!(children.len(), 3);
            
            // Verify order/values if needed, but length is good indicator here
        } else {
            panic!("Expected Union");
        }
    }
    
    #[test]
    fn test_eval_for_loop_empty_range() {
        let mut ctx = create_ctx();
        // for(i=[0:10:-5]) cube(i) -> empty
        
        let range = Expression::Range {
            start: Box::new(Expression::Number(0.0)),
            step: Some(Box::new(Expression::Number(10.0))),
            end: Box::new(Expression::Number(-5.0)),
        };
        
        let body = vec![
            Statement::Cube {
                size: Expression::Variable("i".to_string()),
                center: false,
                span: dummy_span(),
            }
        ];
        
        let result = eval_for_loop("i", &range, &body, dummy_span(), &mut ctx);
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}


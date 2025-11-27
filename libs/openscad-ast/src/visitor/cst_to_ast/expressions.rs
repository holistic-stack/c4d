//! # Expression Transformation
//!
//! Transforms CST expression nodes to AST expressions.

use crate::ast::{Expression, BinaryOp, UnaryOp, Argument};
use crate::error::AstError;
use openscad_parser::{CstNode, NodeKind};

// =============================================================================
// PUBLIC API
// =============================================================================

/// Transform a CST node to an AST expression.
///
/// ## Parameters
///
/// - `node`: CST node to transform
///
/// ## Returns
///
/// AST expression
pub fn transform_expression(node: &CstNode) -> Result<Expression, AstError> {
    match node.kind {
        // Literals
        NodeKind::Number => transform_number(node),
        NodeKind::String => transform_string(node),
        NodeKind::Boolean => transform_boolean(node),
        NodeKind::Undef => Ok(Expression::Undef),
        
        // Identifiers
        NodeKind::Identifier => Ok(Expression::Identifier(node.text_or_empty().to_string())),
        NodeKind::SpecialVariable => Ok(Expression::SpecialVariable(node.text_or_empty().to_string())),
        
        // Compound expressions
        NodeKind::List => transform_list(node),
        NodeKind::Range => transform_range(node),
        NodeKind::BinaryExpression => transform_binary(node),
        NodeKind::UnaryExpression => transform_unary(node),
        NodeKind::TernaryExpression => transform_ternary(node),
        NodeKind::FunctionCall => transform_function_call(node),
        NodeKind::IndexExpression => transform_index(node),
        NodeKind::DotExpression => transform_member(node),
        
        // Argument wraps expression
        NodeKind::Argument => {
            node.children.first()
                .map(transform_expression)
                .transpose()?
                .ok_or_else(|| AstError::InvalidExpression("Empty argument".to_string()))
        }
        
        _ => Err(AstError::UnsupportedNode(format!("{:?}", node.kind))),
    }
}

// =============================================================================
// LITERALS
// =============================================================================

/// Transform number literal.
fn transform_number(node: &CstNode) -> Result<Expression, AstError> {
    let text = node.text_or_empty();
    let value: f64 = text.parse()
        .map_err(|_| AstError::InvalidNumber(text.to_string()))?;
    Ok(Expression::Number(value))
}

/// Transform string literal.
fn transform_string(node: &CstNode) -> Result<Expression, AstError> {
    let text = node.text_or_empty();
    // Remove quotes
    let content = if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        &text[1..text.len()-1]
    } else {
        text
    };
    Ok(Expression::String(content.to_string()))
}

/// Transform boolean literal.
fn transform_boolean(node: &CstNode) -> Result<Expression, AstError> {
    let text = node.text_or_empty();
    let value = text == "true";
    Ok(Expression::Boolean(value))
}

// =============================================================================
// COMPOUND EXPRESSIONS
// =============================================================================

/// Transform list literal.
fn transform_list(node: &CstNode) -> Result<Expression, AstError> {
    let elements: Result<Vec<_>, _> = node.children.iter()
        .map(transform_expression)
        .collect();
    Ok(Expression::List(elements?))
}

/// Transform range expression.
fn transform_range(node: &CstNode) -> Result<Expression, AstError> {
    let mut iter = node.children.iter();
    
    let start = iter.next()
        .map(transform_expression)
        .transpose()?
        .ok_or_else(|| AstError::InvalidExpression("Range missing start".to_string()))?;
    
    let second = iter.next()
        .map(transform_expression)
        .transpose()?
        .ok_or_else(|| AstError::InvalidExpression("Range missing end".to_string()))?;
    
    // Check if there's a third element (step)
    if let Some(third_node) = iter.next() {
        let third = transform_expression(third_node)?;
        // [start : step : end]
        Ok(Expression::Range {
            start: Box::new(start),
            step: Some(Box::new(second)),
            end: Box::new(third),
        })
    } else {
        // [start : end]
        Ok(Expression::Range {
            start: Box::new(start),
            end: Box::new(second),
            step: None,
        })
    }
}

/// Transform binary expression.
fn transform_binary(node: &CstNode) -> Result<Expression, AstError> {
    if node.children.len() < 3 {
        return Err(AstError::InvalidExpression("Binary expression needs 3 children".to_string()));
    }
    
    let left = transform_expression(&node.children[0])?;
    let op_text = node.children[1].text_or_empty();
    let right = transform_expression(&node.children[2])?;
    
    let op = BinaryOp::from_str(op_text)
        .ok_or_else(|| AstError::InvalidExpression(format!("Unknown operator: {}", op_text)))?;
    
    Ok(Expression::BinaryOp {
        op,
        left: Box::new(left),
        right: Box::new(right),
    })
}

/// Transform unary expression.
fn transform_unary(node: &CstNode) -> Result<Expression, AstError> {
    if node.children.len() < 2 {
        return Err(AstError::InvalidExpression("Unary expression needs 2 children".to_string()));
    }
    
    let op_text = node.children[0].text_or_empty();
    let operand = transform_expression(&node.children[1])?;
    
    let op = UnaryOp::from_str(op_text)
        .ok_or_else(|| AstError::InvalidExpression(format!("Unknown operator: {}", op_text)))?;
    
    Ok(Expression::UnaryOp {
        op,
        operand: Box::new(operand),
    })
}

/// Transform ternary expression.
fn transform_ternary(node: &CstNode) -> Result<Expression, AstError> {
    if node.children.len() < 3 {
        return Err(AstError::InvalidExpression("Ternary expression needs 3 children".to_string()));
    }
    
    let condition = transform_expression(&node.children[0])?;
    let then_expr = transform_expression(&node.children[1])?;
    let else_expr = transform_expression(&node.children[2])?;
    
    Ok(Expression::Ternary {
        condition: Box::new(condition),
        then_expr: Box::new(then_expr),
        else_expr: Box::new(else_expr),
    })
}

/// Transform function call.
fn transform_function_call(node: &CstNode) -> Result<Expression, AstError> {
    // First child is the function (identifier or expression)
    let name = node.children.first()
        .map(|n| {
            if n.kind == NodeKind::Identifier {
                n.text_or_empty().to_string()
            } else {
                // For chained calls, get the text representation
                n.text.clone().unwrap_or_default()
            }
        })
        .ok_or_else(|| AstError::InvalidExpression("Function call missing name".to_string()))?;
    
    // Arguments are in Arguments node
    let args = node.find_child(NodeKind::Arguments)
        .map(|a| transform_arguments(&a.children))
        .transpose()?
        .unwrap_or_default();
    
    Ok(Expression::FunctionCall { name, args })
}

/// Transform arguments for function call.
fn transform_arguments(nodes: &[CstNode]) -> Result<Vec<Argument>, AstError> {
    let mut args = Vec::new();
    
    for node in nodes {
        match node.kind {
            NodeKind::Argument => {
                if let Some(expr_node) = node.children.first() {
                    let expr = transform_expression(expr_node)?;
                    args.push(Argument::Positional(expr));
                }
            }
            NodeKind::NamedArgument => {
                let name = node.find_child(NodeKind::Identifier)
                    .map(|n| n.text_or_empty().to_string())
                    .ok_or_else(|| AstError::InvalidExpression("Named argument missing name".to_string()))?;
                
                let value = node.children.iter()
                    .find(|c| c.kind != NodeKind::Identifier)
                    .map(transform_expression)
                    .transpose()?
                    .ok_or_else(|| AstError::InvalidExpression("Named argument missing value".to_string()))?;
                
                args.push(Argument::Named { name, value });
            }
            _ => {
                if node.kind.is_expression() {
                    let expr = transform_expression(node)?;
                    args.push(Argument::Positional(expr));
                }
            }
        }
    }
    
    Ok(args)
}

/// Transform index expression.
fn transform_index(node: &CstNode) -> Result<Expression, AstError> {
    if node.children.len() < 2 {
        return Err(AstError::InvalidExpression("Index expression needs 2 children".to_string()));
    }
    
    let object = transform_expression(&node.children[0])?;
    let index = transform_expression(&node.children[1])?;
    
    Ok(Expression::Index {
        object: Box::new(object),
        index: Box::new(index),
    })
}

/// Transform member access.
fn transform_member(node: &CstNode) -> Result<Expression, AstError> {
    if node.children.len() < 2 {
        return Err(AstError::InvalidExpression("Member expression needs 2 children".to_string()));
    }
    
    let object = transform_expression(&node.children[0])?;
    let member = node.children[1].text_or_empty().to_string();
    
    Ok(Expression::Member {
        object: Box::new(object),
        member,
    })
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use openscad_parser::parse as parse_cst;

    fn parse_expr(source: &str) -> Expression {
        let cst = parse_cst(&format!("x = {};", source));
        let assign = &cst.root.children[0];
        let value_node = assign.children.iter()
            .find(|c| c.kind != NodeKind::Identifier)
            .unwrap();
        transform_expression(value_node).unwrap()
    }

    #[test]
    fn test_transform_number() {
        let expr = parse_expr("42");
        match expr {
            Expression::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_transform_float() {
        let expr = parse_expr("3.14");
        match expr {
            Expression::Number(n) => assert!((n - 3.14).abs() < 0.001),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_transform_boolean() {
        let expr = parse_expr("true");
        match expr {
            Expression::Boolean(b) => assert!(b),
            _ => panic!("Expected Boolean"),
        }
    }

    #[test]
    fn test_transform_list() {
        let expr = parse_expr("[1, 2, 3]");
        match expr {
            Expression::List(items) => assert_eq!(items.len(), 3),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_transform_range() {
        let expr = parse_expr("[0:10]");
        match expr {
            Expression::Range { step, .. } => {
                assert!(step.is_none());
            }
            _ => panic!("Expected Range"),
        }
    }

    #[test]
    fn test_transform_binary() {
        let expr = parse_expr("1 + 2");
        match expr {
            Expression::BinaryOp { op, .. } => {
                assert_eq!(op, BinaryOp::Add);
            }
            _ => panic!("Expected BinaryOp"),
        }
    }
}

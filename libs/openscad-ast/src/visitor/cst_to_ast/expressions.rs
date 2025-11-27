//! # Expression Transformation
//!
//! Facade module for transforming CST expression nodes to AST expressions.
//!
//! ## Module Structure (SRP)
//!
//! - `literals` - Number, string, boolean transformations
//! - `operators` - Binary, unary, ternary operators
//! - `arguments` - Function call arguments (shared)
//!
//! ## Example
//!
//! ```rust,ignore
//! let expr = transform_expression(node)?;
//! ```

use crate::ast::Expression;
use crate::error::AstError;
use openscad_parser::{CstNode, NodeKind};

use super::literals::{transform_number, transform_string, transform_boolean, transform_undef};
use super::operators::{transform_binary, transform_unary, transform_ternary};
use super::arguments::transform_arguments;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Transform a CST node to an AST expression.
///
/// Dispatches to the appropriate transformation function based on node type.
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
        NodeKind::Undef => Ok(transform_undef()),
        
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
///
/// ## CST Structure
///
/// ```text
/// Range
/// ├── Expression (start)
/// ├── Expression (end or step)
/// └── Expression (optional end)
/// ```
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

/// Transform function call.
///
/// ## CST Structure
///
/// ```text
/// FunctionCall
/// ├── Identifier (function name)
/// └── Arguments
/// ```
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
    
    // Arguments are in Arguments node (use shared transformer)
    let args = node.find_child(NodeKind::Arguments)
        .map(|a| transform_arguments(&a.children))
        .transpose()?
        .unwrap_or_default();
    
    Ok(Expression::FunctionCall { name, args })
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
    use crate::ast::BinaryOp;
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

//! # Operator Transformation
//!
//! Transforms CST operator nodes to AST expressions.
//!
//! ## Supported Operators
//!
//! - Binary: +, -, *, /, %, ^, <, >, <=, >=, ==, !=, &&, ||
//! - Unary: -, +, !
//! - Ternary: condition ? then : else
//!
//! ## Example
//!
//! ```rust,ignore
//! let expr = transform_binary(node)?;
//! ```

use crate::ast::{Expression, BinaryOp, UnaryOp};
use crate::error::AstError;
use openscad_parser::CstNode;

use super::expressions::transform_expression;

// =============================================================================
// BINARY OPERATORS
// =============================================================================

/// Transform binary expression.
///
/// Binary expressions have 3 children: left, operator, right.
///
/// ## CST Structure
///
/// ```text
/// BinaryExpression
/// ├── Expression (left)
/// ├── Operator
/// └── Expression (right)
/// ```
///
/// ## Example
///
/// ```text
/// 1 + 2   -> BinaryOp { op: Add, left: 1, right: 2 }
/// a && b  -> BinaryOp { op: And, left: a, right: b }
/// ```
pub fn transform_binary(node: &CstNode) -> Result<Expression, AstError> {
    if node.children.len() < 3 {
        return Err(AstError::InvalidExpression(
            "Binary expression needs 3 children".to_string()
        ));
    }
    
    let left = transform_expression(&node.children[0])?;
    let op_text = node.children[1].text_or_empty();
    let right = transform_expression(&node.children[2])?;
    
    let op = BinaryOp::from_str(op_text)
        .ok_or_else(|| AstError::InvalidExpression(
            format!("Unknown binary operator: {}", op_text)
        ))?;
    
    Ok(Expression::BinaryOp {
        op,
        left: Box::new(left),
        right: Box::new(right),
    })
}

// =============================================================================
// UNARY OPERATORS
// =============================================================================

/// Transform unary expression.
///
/// Unary expressions have 2 children: operator, operand.
///
/// ## CST Structure
///
/// ```text
/// UnaryExpression
/// ├── Operator
/// └── Expression (operand)
/// ```
///
/// ## Example
///
/// ```text
/// -x   -> UnaryOp { op: Neg, operand: x }
/// !b   -> UnaryOp { op: Not, operand: b }
/// ```
pub fn transform_unary(node: &CstNode) -> Result<Expression, AstError> {
    if node.children.len() < 2 {
        return Err(AstError::InvalidExpression(
            "Unary expression needs 2 children".to_string()
        ));
    }
    
    let op_text = node.children[0].text_or_empty();
    let operand = transform_expression(&node.children[1])?;
    
    let op = UnaryOp::from_str(op_text)
        .ok_or_else(|| AstError::InvalidExpression(
            format!("Unknown unary operator: {}", op_text)
        ))?;
    
    Ok(Expression::UnaryOp {
        op,
        operand: Box::new(operand),
    })
}

// =============================================================================
// TERNARY OPERATOR
// =============================================================================

/// Transform ternary expression.
///
/// Ternary expressions have 3 children: condition, then, else.
///
/// ## CST Structure
///
/// ```text
/// TernaryExpression
/// ├── Expression (condition)
/// ├── Expression (then)
/// └── Expression (else)
/// ```
///
/// ## Example
///
/// ```text
/// a ? b : c  -> Ternary { condition: a, then_expr: b, else_expr: c }
/// ```
pub fn transform_ternary(node: &CstNode) -> Result<Expression, AstError> {
    if node.children.len() < 3 {
        return Err(AstError::InvalidExpression(
            "Ternary expression needs 3 children".to_string()
        ));
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

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use openscad_parser::{parse as parse_cst, NodeKind};

    fn parse_expr(source: &str) -> Expression {
        let cst = parse_cst(&format!("x = {};", source));
        let assign = &cst.root.children[0];
        let value_node = assign.children.iter()
            .find(|c| c.kind != NodeKind::Identifier)
            .unwrap();
        transform_expression(value_node).unwrap()
    }

    #[test]
    fn test_transform_add() {
        let expr = parse_expr("1 + 2");
        match expr {
            Expression::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Add),
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_transform_sub() {
        let expr = parse_expr("5 - 3");
        match expr {
            Expression::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Sub),
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_transform_mul() {
        let expr = parse_expr("2 * 3");
        match expr {
            Expression::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Mul),
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_transform_div() {
        let expr = parse_expr("10 / 2");
        match expr {
            Expression::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Div),
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_transform_and() {
        let expr = parse_expr("true && false");
        match expr {
            Expression::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::And),
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_transform_or() {
        let expr = parse_expr("true || false");
        match expr {
            Expression::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Or),
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_transform_comparison() {
        let expr = parse_expr("1 < 2");
        match expr {
            Expression::BinaryOp { op, .. } => assert_eq!(op, BinaryOp::Lt),
            _ => panic!("Expected BinaryOp"),
        }
    }

    #[test]
    fn test_transform_unary_neg() {
        let expr = parse_expr("-5");
        match expr {
            Expression::UnaryOp { op, .. } => assert_eq!(op, UnaryOp::Neg),
            _ => panic!("Expected UnaryOp"),
        }
    }

    #[test]
    fn test_transform_unary_not() {
        let expr = parse_expr("!true");
        match expr {
            Expression::UnaryOp { op, .. } => assert_eq!(op, UnaryOp::Not),
            _ => panic!("Expected UnaryOp"),
        }
    }

    #[test]
    fn test_transform_ternary() {
        let expr = parse_expr("true ? 1 : 2");
        match expr {
            Expression::Ternary { .. } => {}
            _ => panic!("Expected Ternary"),
        }
    }
}

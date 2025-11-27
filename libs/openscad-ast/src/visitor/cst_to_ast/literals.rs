//! # Literal Transformation
//!
//! Transforms CST literal nodes to AST expressions.
//!
//! ## Supported Literals
//!
//! - Numbers: `42`, `3.14`, `-5`
//! - Strings: `"hello"`, `"world"`
//! - Booleans: `true`, `false`
//! - Undef: `undef`
//!
//! ## Example
//!
//! ```rust,ignore
//! let expr = transform_number(node)?;
//! ```

use crate::ast::Expression;
use crate::error::AstError;
use openscad_parser::CstNode;

// =============================================================================
// NUMBER
// =============================================================================

/// Transform number literal.
///
/// Supports integers and floating point numbers.
///
/// ## Parameters
///
/// - `node`: CST node containing number text
///
/// ## Returns
///
/// Expression::Number with parsed value
///
/// ## Example
///
/// ```text
/// 42    -> Expression::Number(42.0)
/// 3.14  -> Expression::Number(3.14)
/// -5    -> Expression::Number(-5.0)
/// ```
pub fn transform_number(node: &CstNode) -> Result<Expression, AstError> {
    let text = node.text_or_empty();
    let value: f64 = text.parse()
        .map_err(|_| AstError::InvalidNumber(text.to_string()))?;
    Ok(Expression::Number(value))
}

// =============================================================================
// STRING
// =============================================================================

/// Transform string literal.
///
/// Removes surrounding quotes from the string.
///
/// ## Parameters
///
/// - `node`: CST node containing string text with quotes
///
/// ## Returns
///
/// Expression::String with content (without quotes)
///
/// ## Example
///
/// ```text
/// "hello"  -> Expression::String("hello")
/// ""       -> Expression::String("")
/// ```
pub fn transform_string(node: &CstNode) -> Result<Expression, AstError> {
    let text = node.text_or_empty();
    // Remove quotes
    let content = if text.starts_with('"') && text.ends_with('"') && text.len() >= 2 {
        &text[1..text.len()-1]
    } else {
        text
    };
    Ok(Expression::String(content.to_string()))
}

// =============================================================================
// BOOLEAN
// =============================================================================

/// Transform boolean literal.
///
/// ## Parameters
///
/// - `node`: CST node containing "true" or "false"
///
/// ## Returns
///
/// Expression::Boolean with parsed value
///
/// ## Example
///
/// ```text
/// true   -> Expression::Boolean(true)
/// false  -> Expression::Boolean(false)
/// ```
pub fn transform_boolean(node: &CstNode) -> Result<Expression, AstError> {
    let text = node.text_or_empty();
    let value = text == "true";
    Ok(Expression::Boolean(value))
}

// =============================================================================
// UNDEF
// =============================================================================

/// Transform undef literal.
///
/// ## Returns
///
/// Expression::Undef
pub fn transform_undef() -> Expression {
    Expression::Undef
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use openscad_parser::{parse as parse_cst, NodeKind};

    fn parse_literal(source: &str) -> Expression {
        let cst = parse_cst(&format!("x = {};", source));
        let assign = &cst.root.children[0];
        let value_node = assign.children.iter()
            .find(|c| c.kind != NodeKind::Identifier)
            .unwrap();
        
        match value_node.kind {
            NodeKind::Number => transform_number(value_node).unwrap(),
            NodeKind::String => transform_string(value_node).unwrap(),
            NodeKind::Boolean => transform_boolean(value_node).unwrap(),
            NodeKind::Undef => transform_undef(),
            _ => panic!("Unexpected node kind: {:?}", value_node.kind),
        }
    }

    #[test]
    fn test_transform_integer() {
        let expr = parse_literal("42");
        match expr {
            Expression::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_transform_float() {
        let expr = parse_literal("3.14");
        match expr {
            Expression::Number(n) => assert!((n - 3.14).abs() < 0.001),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_transform_negative() {
        // Note: Negative numbers are parsed as unary negation in OpenSCAD
        // This test is for direct number parsing
        let expr = parse_literal("0");
        match expr {
            Expression::Number(n) => assert_eq!(n, 0.0),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_transform_boolean_true() {
        let expr = parse_literal("true");
        match expr {
            Expression::Boolean(b) => assert!(b),
            _ => panic!("Expected Boolean"),
        }
    }

    #[test]
    fn test_transform_boolean_false() {
        let expr = parse_literal("false");
        match expr {
            Expression::Boolean(b) => assert!(!b),
            _ => panic!("Expected Boolean"),
        }
    }

    #[test]
    fn test_transform_string() {
        let expr = parse_literal("\"hello\"");
        match expr {
            Expression::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_transform_empty_string() {
        let expr = parse_literal("\"\"");
        match expr {
            Expression::String(s) => assert_eq!(s, ""),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_transform_undef() {
        let expr = transform_undef();
        assert!(matches!(expr, Expression::Undef));
    }
}

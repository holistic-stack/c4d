//! # Argument Transformation
//!
//! Transforms CST argument nodes to AST arguments.
//! Shared between statements (module calls) and expressions (function calls).
//!
//! ## Example
//!
//! ```rust,ignore
//! let args = transform_arguments(&nodes)?;
//! ```

use crate::ast::{Argument, Expression};
use crate::error::AstError;
use openscad_parser::{CstNode, NodeKind};

use super::expressions::transform_expression;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Transform a list of CST argument nodes to AST arguments.
///
/// Handles both positional and named arguments.
/// Named arguments can use identifiers or special variables ($fn, $fa, $fs).
///
/// ## Parameters
///
/// - `nodes`: CST argument nodes
///
/// ## Returns
///
/// Vector of AST arguments
///
/// ## Example
///
/// ```text
/// cube(10, center=true);
/// // Args: [Positional(10), Named("center", true)]
///
/// sphere(5, $fn=32);
/// // Args: [Positional(5), Named("$fn", 32)]
/// ```
pub fn transform_arguments(nodes: &[CstNode]) -> Result<Vec<Argument>, AstError> {
    let mut args = Vec::new();
    
    for node in nodes {
        if let Some(arg) = transform_argument(node)? {
            args.push(arg);
        }
    }
    
    Ok(args)
}

/// Transform a single CST argument node to an AST argument.
///
/// ## Parameters
///
/// - `node`: CST node to transform
///
/// ## Returns
///
/// Optional argument (None for non-argument nodes like commas)
fn transform_argument(node: &CstNode) -> Result<Option<Argument>, AstError> {
    match node.kind {
        NodeKind::Argument => transform_positional(node),
        NodeKind::NamedArgument => transform_named(node),
        _ => {
            // Try to parse as expression (positional)
            if node.kind.is_expression() {
                let expr = transform_expression(node)?;
                Ok(Some(Argument::Positional(expr)))
            } else {
                Ok(None)
            }
        }
    }
}

// =============================================================================
// POSITIONAL ARGUMENTS
// =============================================================================

/// Transform a positional argument.
///
/// ## CST Structure
///
/// ```text
/// Argument
/// └── Expression
/// ```
fn transform_positional(node: &CstNode) -> Result<Option<Argument>, AstError> {
    if let Some(expr_node) = node.children.first() {
        let expr = transform_expression(expr_node)?;
        Ok(Some(Argument::Positional(expr)))
    } else {
        Ok(None)
    }
}

// =============================================================================
// NAMED ARGUMENTS
// =============================================================================

/// Transform a named argument.
///
/// Named arguments can use identifiers or special variables ($fn, $fa, $fs).
///
/// ## CST Structure
///
/// ```text
/// NamedArgument
/// ├── Identifier | SpecialVariable (name)
/// └── Expression (value)
/// ```
///
/// ## Example
///
/// ```text
/// center=true   -> Named { name: "center", value: Boolean(true) }
/// $fn=32        -> Named { name: "$fn", value: Number(32) }
/// ```
fn transform_named(node: &CstNode) -> Result<Option<Argument>, AstError> {
    // Name can be Identifier or SpecialVariable
    let name = find_argument_name(node)?;
    
    // Value is the non-identifier/non-special-variable child
    let value = find_argument_value(node)?;
    
    Ok(Some(Argument::Named { name, value }))
}

/// Find the name of a named argument.
fn find_argument_name(node: &CstNode) -> Result<String, AstError> {
    node.find_child(NodeKind::Identifier)
        .or_else(|| node.find_child(NodeKind::SpecialVariable))
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst("Named argument missing name".to_string()))
}

/// Find the value of a named argument.
fn find_argument_value(node: &CstNode) -> Result<Expression, AstError> {
    node.children.iter()
        .find(|c| c.kind != NodeKind::Identifier && c.kind != NodeKind::SpecialVariable)
        .map(|c| transform_expression(c))
        .transpose()?
        .ok_or_else(|| AstError::InvalidCst("Named argument missing value".to_string()))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use openscad_parser::parse as parse_cst;

    fn get_args(source: &str) -> Vec<Argument> {
        let cst = parse_cst(source);
        let module_call = &cst.root.children[0];
        let args_node = module_call.find_child(NodeKind::Arguments).unwrap();
        transform_arguments(&args_node.children).unwrap()
    }

    #[test]
    fn test_transform_positional() {
        let args = get_args("cube(10);");
        assert_eq!(args.len(), 1);
        assert!(matches!(&args[0], Argument::Positional(_)));
    }

    #[test]
    fn test_transform_named() {
        let args = get_args("cube(10, center=true);");
        assert_eq!(args.len(), 2);
        match &args[1] {
            Argument::Named { name, .. } => assert_eq!(name, "center"),
            _ => panic!("Expected Named argument"),
        }
    }

    #[test]
    fn test_transform_special_variable() {
        let args = get_args("sphere(5, $fn=32);");
        assert_eq!(args.len(), 2);
        match &args[1] {
            Argument::Named { name, .. } => assert_eq!(name, "$fn"),
            _ => panic!("Expected Named argument"),
        }
    }

    #[test]
    fn test_transform_multiple_positional() {
        let args = get_args("cylinder(10, 5, 3);");
        assert_eq!(args.len(), 3);
        assert!(args.iter().all(|a| matches!(a, Argument::Positional(_))));
    }
}

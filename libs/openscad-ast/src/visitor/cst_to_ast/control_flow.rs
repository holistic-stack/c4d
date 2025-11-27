//! # Control Flow Transformation
//!
//! Transforms CST control flow nodes to AST statements.
//!
//! ## Supported Control Flow
//!
//! - Blocks: `{ ... }`
//! - For loops: `for (i = [0:10]) { ... }`
//! - If/else: `if (condition) { ... } else { ... }`
//!
//! ## Example
//!
//! ```rust,ignore
//! let stmt = transform_for_block(node)?;
//! ```

use crate::ast::{Statement, Expression};
use crate::error::AstError;
use openscad_parser::{CstNode, NodeKind};

use super::statements::{transform_statements, transform_statement};
use super::expressions::transform_expression;

// =============================================================================
// BLOCK
// =============================================================================

/// Transform block node.
///
/// A block contains a list of statements enclosed in braces.
///
/// ## CST Structure
///
/// ```text
/// Block
/// ├── Statement1
/// ├── Statement2
/// └── ...
/// ```
///
/// ## Example
///
/// ```text
/// { cube(10); sphere(5); }
/// ```
pub fn transform_block(node: &CstNode) -> Result<Statement, AstError> {
    let statements = transform_statements(&node.children)?;
    
    Ok(Statement::Block {
        statements,
        span: node.span,
    })
}

// =============================================================================
// FOR LOOP
// =============================================================================

/// Transform for loop node.
///
/// For loops have assignments and a body statement.
///
/// ## CST Structure
///
/// ```text
/// ForBlock
/// ├── ForAssignments
/// │   └── ForAssignment
/// │       ├── Identifier (variable name)
/// │       └── Expression (range or list)
/// └── Statement (body)
/// ```
///
/// ## Example
///
/// ```text
/// for (i = [0:10]) cube(i);
/// for (i = [0:10], j = [0:5]) translate([i, j, 0]) cube(1);
/// ```
pub fn transform_for_block(node: &CstNode) -> Result<Statement, AstError> {
    let mut assignments = Vec::new();
    let mut body = Vec::new();

    for child in &node.children {
        match child.kind {
            NodeKind::ForAssignments => {
                // Parse each ForAssignment
                for assign in &child.children {
                    if assign.kind == NodeKind::ForAssignment {
                        if let Some(assignment) = transform_for_assignment(assign)? {
                            assignments.push(assignment);
                        }
                    }
                }
            }
            _ => {
                // Body statement
                if let Some(stmt) = transform_statement(child)? {
                    body.push(stmt);
                }
            }
        }
    }
    
    Ok(Statement::ForLoop {
        assignments,
        body,
        span: node.span,
    })
}

/// Transform a single for assignment (i = [0:10]).
///
/// ## CST Structure
///
/// ```text
/// ForAssignment
/// ├── Identifier (variable name)
/// └── Expression (range or list)
/// ```
fn transform_for_assignment(node: &CstNode) -> Result<Option<(String, Expression)>, AstError> {
    let name = node.find_child(NodeKind::Identifier)
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst(
            "For assignment missing variable name".to_string()
        ))?;
    
    // Value is the non-identifier child
    let value = node.children.iter()
        .find(|c| c.kind != NodeKind::Identifier)
        .map(|c| transform_expression(c))
        .transpose()?
        .ok_or_else(|| AstError::InvalidCst(
            "For assignment missing value".to_string()
        ))?;
    
    Ok(Some((name, value)))
}

// =============================================================================
// IF/ELSE
// =============================================================================

/// Transform if/else node.
///
/// If/else has a condition, then body, and optional else body.
///
/// ## CST Structure
///
/// ```text
/// IfBlock
/// ├── Expression (condition)
/// ├── Statement (then body)
/// └── Statement (optional else body)
/// ```
///
/// ## Example
///
/// ```text
/// if (x > 0) cube(x);
/// if (x > 0) cube(x); else sphere(5);
/// ```
pub fn transform_if_block(node: &CstNode) -> Result<Statement, AstError> {
    let mut children = node.children.iter();
    
    // First child is condition
    let condition = children.next()
        .map(|c| transform_expression(c))
        .transpose()?
        .unwrap_or(Expression::Boolean(true));
    
    // Second child is then body
    let then_body = children.next()
        .map(|c| transform_statement(c))
        .transpose()?
        .flatten()
        .map(|s| vec![s])
        .unwrap_or_default();
    
    // Optional else body
    let else_body = children.next()
        .map(|c| transform_statement(c))
        .transpose()?
        .flatten()
        .map(|s| vec![s]);
    
    Ok(Statement::IfElse {
        condition,
        then_body,
        else_body,
        span: node.span,
    })
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use openscad_parser::parse as parse_cst;

    #[test]
    fn test_transform_block() {
        let cst = parse_cst("{ cube(10); sphere(5); }");
        let block = &cst.root.children[0];
        let stmt = transform_block(block).unwrap();
        
        match stmt {
            Statement::Block { statements, .. } => {
                assert_eq!(statements.len(), 2);
            }
            _ => panic!("Expected Block"),
        }
    }

    #[test]
    fn test_transform_for_loop() {
        let cst = parse_cst("for (i = [0:10]) cube(i);");
        let for_block = &cst.root.children[0];
        let stmt = transform_for_block(for_block).unwrap();
        
        match stmt {
            Statement::ForLoop { assignments, body, .. } => {
                assert_eq!(assignments.len(), 1);
                assert_eq!(assignments[0].0, "i");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected ForLoop"),
        }
    }

    #[test]
    fn test_transform_if() {
        let cst = parse_cst("if (true) cube(10);");
        let if_block = &cst.root.children[0];
        let stmt = transform_if_block(if_block).unwrap();
        
        match stmt {
            Statement::IfElse { then_body, else_body, .. } => {
                assert_eq!(then_body.len(), 1);
                assert!(else_body.is_none());
            }
            _ => panic!("Expected IfElse"),
        }
    }

    #[test]
    fn test_transform_if_else() {
        let cst = parse_cst("if (false) cube(10); else sphere(5);");
        let if_block = &cst.root.children[0];
        let stmt = transform_if_block(if_block).unwrap();
        
        match stmt {
            Statement::IfElse { then_body, else_body, .. } => {
                assert_eq!(then_body.len(), 1);
                assert!(else_body.is_some());
                assert_eq!(else_body.unwrap().len(), 1);
            }
            _ => panic!("Expected IfElse"),
        }
    }
}

//! # Statement Transformation
//!
//! Transforms CST statement nodes to AST statements.

use crate::ast::{Statement, Argument, Expression};
use crate::error::AstError;
use openscad_parser::{CstNode, NodeKind};
use super::expressions;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Transform a list of CST nodes to AST statements.
///
/// ## Parameters
///
/// - `nodes`: CST nodes to transform
///
/// ## Returns
///
/// Vector of AST statements
pub fn transform_statements(nodes: &[CstNode]) -> Result<Vec<Statement>, AstError> {
    let mut statements = Vec::new();
    
    for node in nodes {
        if let Some(stmt) = transform_statement(node)? {
            statements.push(stmt);
        }
    }
    
    Ok(statements)
}

/// Transform a single CST node to an AST statement.
///
/// ## Returns
///
/// `Some(Statement)` if the node is a statement, `None` for non-statement nodes
pub fn transform_statement(node: &CstNode) -> Result<Option<Statement>, AstError> {
    match node.kind {
        NodeKind::ModuleCall => {
            Ok(Some(transform_module_call(node)?))
        }
        NodeKind::Assignment => {
            Ok(Some(transform_assignment(node)?))
        }
        NodeKind::Block => {
            Ok(Some(transform_block(node)?))
        }
        NodeKind::ForBlock => {
            Ok(Some(transform_for_block(node)?))
        }
        NodeKind::IfBlock => {
            Ok(Some(transform_if_block(node)?))
        }
        NodeKind::ModuleDeclaration => {
            Ok(Some(transform_module_declaration(node)?))
        }
        NodeKind::FunctionDeclaration => {
            Ok(Some(transform_function_declaration(node)?))
        }
        // Skip non-statement nodes
        NodeKind::Semicolon | NodeKind::Comment => Ok(None),
        // Modifier wraps another statement
        NodeKind::Modifier => {
            // Get the wrapped statement (last child)
            if let Some(child) = node.children.last() {
                transform_statement(child)
            } else {
                Ok(None)
            }
        }
        _ => {
            // Unknown node type - skip with warning in debug
            #[cfg(debug_assertions)]
            eprintln!("Unknown statement node: {:?}", node.kind);
            Ok(None)
        }
    }
}

// =============================================================================
// MODULE CALL
// =============================================================================

/// Transform module call node.
///
/// ## Example CST
///
/// ```text
/// ModuleCall
/// ├── Identifier "cube"
/// ├── Arguments
/// │   └── Argument
/// │       └── Number "10"
/// └── (optional child statement)
/// ```
fn transform_module_call(node: &CstNode) -> Result<Statement, AstError> {
    // Get module name
    let name = node.find_child(NodeKind::Identifier)
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst("Module call missing name".to_string()))?;
    
    // Get arguments
    let args = if let Some(args_node) = node.find_child(NodeKind::Arguments) {
        transform_arguments(&args_node.children)?
    } else {
        Vec::new()
    };
    
    // Get child statements (for transforms like translate)
    let children: Vec<Statement> = node.children.iter()
        .filter(|c| c.kind != NodeKind::Identifier && 
                    c.kind != NodeKind::Arguments &&
                    c.kind.is_statement())
        .filter_map(|c| transform_statement(c).ok().flatten())
        .collect();
    
    Ok(Statement::ModuleCall {
        name,
        args,
        children,
        span: node.span,
    })
}

/// Transform arguments list.
fn transform_arguments(nodes: &[CstNode]) -> Result<Vec<Argument>, AstError> {
    let mut args = Vec::new();
    
    for node in nodes {
        match node.kind {
            NodeKind::Argument => {
                // Positional argument
                if let Some(expr_node) = node.children.first() {
                    let expr = expressions::transform_expression(expr_node)?;
                    args.push(Argument::Positional(expr));
                }
            }
            NodeKind::NamedArgument => {
                // Named argument: name=value
                let name = node.find_child(NodeKind::Identifier)
                    .map(|n| n.text_or_empty().to_string())
                    .ok_or_else(|| AstError::InvalidCst("Named argument missing name".to_string()))?;
                
                // Value is the non-identifier child
                let value = node.children.iter()
                    .find(|c| c.kind != NodeKind::Identifier)
                    .map(|c| expressions::transform_expression(c))
                    .transpose()?
                    .ok_or_else(|| AstError::InvalidCst("Named argument missing value".to_string()))?;
                
                args.push(Argument::Named { name, value });
            }
            _ => {
                // Try to parse as expression (positional)
                if node.kind.is_expression() {
                    let expr = expressions::transform_expression(node)?;
                    args.push(Argument::Positional(expr));
                }
            }
        }
    }
    
    Ok(args)
}

// =============================================================================
// ASSIGNMENT
// =============================================================================

/// Transform assignment node.
fn transform_assignment(node: &CstNode) -> Result<Statement, AstError> {
    let name = node.find_child(NodeKind::Identifier)
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst("Assignment missing name".to_string()))?;
    
    // Value is the non-identifier child
    let value = node.children.iter()
        .find(|c| c.kind != NodeKind::Identifier)
        .map(|c| expressions::transform_expression(c))
        .transpose()?
        .ok_or_else(|| AstError::InvalidCst("Assignment missing value".to_string()))?;
    
    Ok(Statement::Assignment {
        name,
        value,
        span: node.span,
    })
}

// =============================================================================
// BLOCK
// =============================================================================

/// Transform block node.
fn transform_block(node: &CstNode) -> Result<Statement, AstError> {
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
fn transform_for_block(node: &CstNode) -> Result<Statement, AstError> {
    // For now, just get the body
    let body = node.children.iter()
        .filter_map(|c| transform_statement(c).ok().flatten())
        .collect();
    
    Ok(Statement::ForLoop {
        assignments: Vec::new(), // TODO: Parse loop assignments
        body,
        span: node.span,
    })
}

// =============================================================================
// IF/ELSE
// =============================================================================

/// Transform if/else node.
fn transform_if_block(node: &CstNode) -> Result<Statement, AstError> {
    let mut children = node.children.iter();
    
    // First child is condition
    let condition = children.next()
        .map(|c| expressions::transform_expression(c))
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
// DECLARATIONS (simplified)
// =============================================================================

fn transform_module_declaration(node: &CstNode) -> Result<Statement, AstError> {
    let name = node.find_child(NodeKind::Identifier)
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst("Module declaration missing name".to_string()))?;
    
    let body = node.find_child(NodeKind::Block)
        .map(|b| transform_statements(&b.children))
        .transpose()?
        .unwrap_or_default();
    
    Ok(Statement::ModuleDeclaration {
        name,
        params: Vec::new(), // TODO: Parse parameters
        body,
        span: node.span,
    })
}

fn transform_function_declaration(node: &CstNode) -> Result<Statement, AstError> {
    let name = node.find_child(NodeKind::Identifier)
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst("Function declaration missing name".to_string()))?;
    
    // Body is the last non-identifier child
    let body = node.children.iter()
        .filter(|c| c.kind != NodeKind::Identifier)
        .last()
        .map(|c| expressions::transform_expression(c))
        .transpose()?
        .unwrap_or(Expression::Undef);
    
    Ok(Statement::FunctionDeclaration {
        name,
        params: Vec::new(), // TODO: Parse parameters
        body,
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
    fn test_transform_module_call() {
        let cst = parse_cst("cube(10);");
        let stmts = transform_statements(&cst.root.children).unwrap();
        
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Statement::ModuleCall { name, args, .. } => {
                assert_eq!(name, "cube");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected ModuleCall"),
        }
    }

    #[test]
    fn test_transform_named_argument() {
        let cst = parse_cst("cube(10, center=true);");
        let stmts = transform_statements(&cst.root.children).unwrap();
        
        match &stmts[0] {
            Statement::ModuleCall { args, .. } => {
                assert_eq!(args.len(), 2);
                match &args[1] {
                    Argument::Named { name, .. } => {
                        assert_eq!(name, "center");
                    }
                    _ => panic!("Expected Named argument"),
                }
            }
            _ => panic!("Expected ModuleCall"),
        }
    }

    #[test]
    fn test_transform_with_child() {
        let cst = parse_cst("translate([1,2,3]) cube(10);");
        let stmts = transform_statements(&cst.root.children).unwrap();
        
        match &stmts[0] {
            Statement::ModuleCall { name, children, .. } => {
                assert_eq!(name, "translate");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected ModuleCall"),
        }
    }
}

//! # Statement Transformation
//!
//! Facade module for transforming CST statement nodes to AST statements.
//!
//! ## Module Structure (SRP)
//!
//! - `declarations` - Variable assignment, module/function declarations
//! - `control_flow` - Blocks, for loops, if/else
//! - `arguments` - Argument handling (shared)
//!
//! ## Example
//!
//! ```rust,ignore
//! let statements = transform_statements(&cst.root.children)?;
//! ```

use crate::ast::Statement;
use crate::error::AstError;
use openscad_parser::{CstNode, NodeKind};

use super::arguments::transform_arguments;
use super::control_flow::{transform_block, transform_for_block, transform_if_block};
use super::declarations::{transform_assignment, transform_module_declaration, transform_function_declaration};

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
/// Dispatches to the appropriate transformation function based on node type.
///
/// ## Returns
///
/// `Some(Statement)` if the node is a statement, `None` for non-statement nodes
pub fn transform_statement(node: &CstNode) -> Result<Option<Statement>, AstError> {
    match node.kind {
        // Module calls (cube, translate, etc.)
        NodeKind::ModuleCall => {
            Ok(Some(transform_module_call(node)?))
        }
        
        // Declarations
        NodeKind::Assignment => {
            Ok(Some(transform_assignment(node)?))
        }
        NodeKind::ModuleDeclaration => {
            Ok(Some(transform_module_declaration(node)?))
        }
        NodeKind::FunctionDeclaration => {
            Ok(Some(transform_function_declaration(node)?))
        }
        
        // Control flow
        NodeKind::Block => {
            Ok(Some(transform_block(node)?))
        }
        NodeKind::ForBlock => {
            Ok(Some(transform_for_block(node)?))
        }
        NodeKind::IfBlock => {
            Ok(Some(transform_if_block(node)?))
        }
        
        // Skip non-statement nodes
        NodeKind::Semicolon | NodeKind::Comment => Ok(None),
        
        // Modifier wraps another statement
        NodeKind::Modifier => {
            if let Some(child) = node.children.last() {
                transform_statement(child)
            } else {
                Ok(None)
            }
        }
        
        _ => {
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
/// Module calls include primitives (cube, sphere) and transforms (translate, rotate).
///
/// ## CST Structure
///
/// ```text
/// ModuleCall
/// ├── Identifier "cube"
/// ├── Arguments
/// │   └── Argument
/// │       └── Number "10"
/// └── (optional child statement)
/// ```
///
/// ## Example
///
/// ```text
/// cube(10);
/// translate([1, 2, 3]) cube(10);
/// ```
fn transform_module_call(node: &CstNode) -> Result<Statement, AstError> {
    // Get module name
    let name = node.find_child(NodeKind::Identifier)
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst("Module call missing name".to_string()))?;
    
    // Get arguments using shared argument transformer
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

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Argument;
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

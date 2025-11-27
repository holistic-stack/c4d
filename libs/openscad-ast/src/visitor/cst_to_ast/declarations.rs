//! # Declaration Transformation
//!
//! Transforms CST declaration nodes to AST statements.
//!
//! ## Supported Declarations
//!
//! - Module declarations: `module foo() { ... }`
//! - Function declarations: `function foo() = ...;`
//! - Variable assignments: `x = 10;`
//!
//! ## Example
//!
//! ```rust,ignore
//! let stmt = transform_module_declaration(node)?;
//! ```

use crate::ast::{Statement, Expression, Parameter};
use crate::error::AstError;
use openscad_parser::{CstNode, NodeKind};

use super::statements::transform_statements;
use super::expressions::transform_expression;

// =============================================================================
// VARIABLE ASSIGNMENT
// =============================================================================

/// Transform assignment node.
///
/// ## CST Structure
///
/// ```text
/// Assignment
/// ├── Identifier | SpecialVariable (variable name)
/// └── Expression (value)
/// ```
///
/// ## Example
///
/// ```text
/// x = 10;
/// size = [10, 20, 30];
/// $fn = 32;
/// ```
pub fn transform_assignment(node: &CstNode) -> Result<Statement, AstError> {
    // Name can be Identifier or SpecialVariable
    let name = node.find_child(NodeKind::Identifier)
        .or_else(|| node.find_child(NodeKind::SpecialVariable))
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst(
            "Assignment missing name".to_string()
        ))?;
    
    // Value is the non-identifier/non-special-variable child
    let value = node.children.iter()
        .find(|c| c.kind != NodeKind::Identifier && c.kind != NodeKind::SpecialVariable)
        .map(|c| transform_expression(c))
        .transpose()?
        .ok_or_else(|| AstError::InvalidCst(
            "Assignment missing value".to_string()
        ))?;
    
    Ok(Statement::Assignment {
        name,
        value,
        span: node.span,
    })
}

// =============================================================================
// MODULE DECLARATION
// =============================================================================

/// Transform module declaration node.
///
/// ## CST Structure
///
/// ```text
/// ModuleDeclaration
/// ├── Identifier (module name)
/// ├── Parameters (optional)
/// └── Block (body)
/// ```
///
/// ## Example
///
/// ```text
/// module foo() { cube(10); }
/// module bar(size=10) { cube(size); }
/// ```
pub fn transform_module_declaration(node: &CstNode) -> Result<Statement, AstError> {
    let name = node.find_child(NodeKind::Identifier)
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst(
            "Module declaration missing name".to_string()
        ))?;
    
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

// =============================================================================
// FUNCTION DECLARATION
// =============================================================================

/// Transform function declaration node.
///
/// ## CST Structure
///
/// ```text
/// FunctionDeclaration
/// ├── Identifier (function name)
/// ├── Parameters (optional)
/// └── Expression (body)
/// ```
///
/// ## Example
///
/// ```text
/// function foo() = 10;
/// function bar(x) = x * 2;
/// function add(a, b) = a + b;
/// ```
pub fn transform_function_declaration(node: &CstNode) -> Result<Statement, AstError> {
    let name = node.find_child(NodeKind::Identifier)
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst(
            "Function declaration missing name".to_string()
        ))?;
    
    // Parse parameters
    let params = node.find_child(NodeKind::Parameters)
        .map(|p| transform_parameters(p))
        .transpose()?
        .unwrap_or_default();
    
    // Body is the expression child (not identifier, not parameters)
    let body = node.children.iter()
        .filter(|c| c.kind != NodeKind::Identifier && c.kind != NodeKind::Parameters)
        .last()
        .map(|c| transform_expression(c))
        .transpose()?
        .unwrap_or(Expression::Undef);
    
    Ok(Statement::FunctionDeclaration {
        name,
        params,
        body,
        span: node.span,
    })
}

/// Transform parameters node.
///
/// ## CST Structure
///
/// ```text
/// Parameters
/// ├── Parameter (name only)
/// └── Parameter (name + default)
/// ```
fn transform_parameters(node: &CstNode) -> Result<Vec<Parameter>, AstError> {
    node.children.iter()
        .map(|p| transform_parameter(p))
        .collect()
}

/// Transform single parameter.
fn transform_parameter(node: &CstNode) -> Result<Parameter, AstError> {
    let name = node.find_child(NodeKind::Identifier)
        .map(|n| n.text_or_empty().to_string())
        .ok_or_else(|| AstError::InvalidCst(
            "Parameter missing name".to_string()
        ))?;
    
    // Check for default value (second child after identifier)
    let default = node.children.iter()
        .skip(1)
        .next()
        .map(|c| transform_expression(c))
        .transpose()?;
    
    Ok(Parameter { name, default })
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use openscad_parser::parse as parse_cst;

    #[test]
    fn test_transform_assignment() {
        let cst = parse_cst("x = 10;");
        let assign = &cst.root.children[0];
        let stmt = transform_assignment(assign).unwrap();
        
        match stmt {
            Statement::Assignment { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected Assignment"),
        }
    }

    #[test]
    fn test_transform_assignment_list() {
        let cst = parse_cst("size = [10, 20, 30];");
        let assign = &cst.root.children[0];
        let stmt = transform_assignment(assign).unwrap();
        
        match stmt {
            Statement::Assignment { name, value, .. } => {
                assert_eq!(name, "size");
                assert!(matches!(value, Expression::List(_)));
            }
            _ => panic!("Expected Assignment"),
        }
    }

    #[test]
    fn test_transform_module_declaration() {
        let cst = parse_cst("module foo() { cube(10); }");
        let module_decl = &cst.root.children[0];
        let stmt = transform_module_declaration(module_decl).unwrap();
        
        match stmt {
            Statement::ModuleDeclaration { name, body, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected ModuleDeclaration"),
        }
    }

    #[test]
    fn test_transform_function_declaration() {
        let cst = parse_cst("function foo() = 10;");
        let func_decl = &cst.root.children[0];
        let stmt = transform_function_declaration(func_decl).unwrap();
        
        match stmt {
            Statement::FunctionDeclaration { name, .. } => {
                assert_eq!(name, "foo");
            }
            _ => panic!("Expected FunctionDeclaration"),
        }
    }
}

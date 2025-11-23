//! Variable assignment and declaration parsing.
//!
//! This module handles parsing of OpenSCAD variable assignments, which are
//! used to set special variables (like `$fn`, `$fa`, `$fs`) and user-defined
//! variables.
//!
//! # Current Limitations
//!
//! This implementation currently supports only simple numeric assignments:
//! - `$fn = 50;` ✓
//! - `width = 10.5;` ✓
//! - `points = [[0,0], [1,1]];` ✗ (complex values not yet supported)
//!
//! Complex assignments (vectors, expressions, etc.) are ignored and return `None`,
//! allowing the parser to continue without errors.
//!
//! # Special Variables
//!
//! OpenSCAD uses special variables prefixed with `$` for rendering control:
//! - `$fn`: Number of fragments for circles/spheres
//! - `$fa`: Minimum angle for fragments
//! - `$fs`: Minimum size for fragments
//!
//! # Examples
//!
//! ```
//! use openscad_ast::parse_to_ast;
//! use openscad_ast::ast_types::Statement;
//!
//! // Parse a special variable assignment
//! let ast = parse_to_ast("$fn = 50;").expect("parse succeeds");
//! match &ast[0] {
//!     Statement::Assignment { name, value, .. } => {
//!         assert_eq!(name, "$fn");
//!         assert_eq!(*value, 50.0);
//!     }
//!     _ => panic!("Expected assignment"),
//! }
//! ```

use crate::{ast_types::Statement, Diagnostic, Span};
use tree_sitter::Node;

/// Parses a variable declaration node from the CST.
///
/// This function extracts variable assignments from the syntax tree. Currently,
/// only simple numeric assignments are supported. Complex assignments (vectors,
/// expressions, etc.) are silently ignored by returning `None`.
///
/// # Arguments
///
/// * `node` - The tree-sitter CST node representing the var_declaration
/// * `source` - The original source code for text extraction
///
/// # Returns
///
/// * `Ok(Some(Statement::Assignment))` - Successfully parsed numeric assignment
/// * `Ok(None)` - Complex assignment (not yet supported, silently ignored)
/// * `Err(Vec<Diagnostic>)` - Parse errors (e.g., invalid number format)
///
/// # Supported Syntax
///
/// ```text
/// var_declaration = identifier "=" value ";"
/// ```
///
/// Where `value` is currently limited to:
/// - `number`: Integer or float literals
/// - `integer`: Integer literals
/// - `float`: Floating-point literals
///
/// # Examples
///
/// ```ignore
/// use openscad_ast::parser::assignments::parse_var_declaration;
///
/// // Typically called by parse_statement, not directly
/// let stmt = parse_var_declaration(&node, source)?;
/// ```
pub fn parse_var_declaration(
    node: &Node,
    source: &str,
) -> Result<Option<Statement>, Vec<Diagnostic>> {
    let mut cursor = node.walk();
    let assignment = node.children(&mut cursor).find(|n| n.kind() == "assignment");

    if let Some(assign) = assignment {
        let name_node = assign.child_by_field_name("name");
        let value_node = assign.child_by_field_name("value");

        if let (Some(name_n), Some(value_n)) = (name_node, value_node) {
            let name = source[name_n.byte_range()].to_string();

            // Parse value - only simple numbers for now
            match value_n.kind() {
                "number" | "integer" | "float" => {
                    let text = &source[value_n.byte_range()];
                    let value = text.parse::<f64>().map_err(|_| {
                        vec![Diagnostic::error(
                            format!("Invalid number in assignment: {}", text),
                            Span::new(value_n.start_byte(), value_n.end_byte()).unwrap(),
                        )]
                    })?;

                    return Ok(Some(Statement::Assignment {
                        name,
                        value,
                        span: Span::new(node.start_byte(), node.end_byte()).unwrap(),
                    }));
                }
                _ => {
                    // Ignore complex assignments for now (vectors, expressions, etc.)
                    // This allows the parser to continue without errors
                    return Ok(None);
                }
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_to_ast;

    /// Tests parsing a simple numeric assignment to a special variable.
    ///
    /// OpenSCAD uses `$fn` to control the number of fragments in circles.
    #[test]
    fn test_parse_assignment() {
        let ast = parse_to_ast("$fn = 50;").expect("parse succeeds");
        assert_eq!(ast.len(), 1);

        match &ast[0] {
            Statement::Assignment { name, value, .. } => {
                assert_eq!(name, "$fn");
                assert_eq!(*value, 50.0);
            }
            _ => panic!("expected Assignment"),
        }
    }
}

//! Transform chain parsing for spatial transformations.
//!
//! This module handles parsing of OpenSCAD's transform operations that modify
//! the position, rotation, or scale of child objects. Transform chains can be
//! nested to create complex spatial arrangements.
//!
//! # Supported Transforms
//!
//! - **translate([x, y, z])**: Moves objects in 3D space
//! - **rotate([x, y, z])**: Rotates objects around axes (degrees)
//! - **scale([x, y, z])**: Scales objects along axes
//!
//! # Grammar Structure
//!
//! In the OpenSCAD grammar, a transform chain has the structure:
//! ```text
//! transform_chain = modifier* module_call statement
//! ```
//!
//! This means:
//! - A `cube(10);` is actually `cube(10) ;` (module_call + semicolon statement)
//! - A `translate([1,0,0]) cube(5);` is a transform_chain with nested statement
//!
//! # Recursive Nature
//!
//! Transform chains are inherently recursive - the child statement can itself
//! be another transform_chain, allowing arbitrary nesting:
//! ```text
//! translate([1,0,0]) rotate([0,90,0]) scale([2,1,1]) cube(5);
//! ```
//!
//! # Examples
//!
//! ```
//! use openscad_ast::parse_to_ast;
//! use openscad_ast::ast_types::Statement;
//!
//! // Simple translation
//! let ast = parse_to_ast("translate([10, 0, 0]) cube(5);").expect("parse succeeds");
//!
//! // Nested transforms
//! let ast = parse_to_ast("translate([1,0,0]) rotate([0,90,0]) cube(10);")
//!     .expect("parse succeeds");
//! ```

use crate::{ast_types::{Statement, CubeSize}, Diagnostic, Span};
use tree_sitter::Node;
use super::module_call::parse_module_call;
use super::arguments::shared::parse_vector;
use super::statement::parse_statement;

/// Parses a transform chain node from the CST.
///
/// A transform chain consists of a transformation module (translate/rotate/scale)
/// followed by a child statement. The child can be a primitive or another transform,
/// enabling nested transformations.
///
/// # Arguments
///
/// * `node` - The tree-sitter CST node representing the transform_chain
/// * `source` - The original source code for text extraction
///
/// # Returns
///
/// * `Ok(Some(Statement))` - Successfully parsed transform or primitive
/// * `Ok(None)` - Unknown transform type
/// * `Err(Vec<Diagnostic>)` - Parse errors with source locations
///
/// # Grammar Handling
///
/// The function handles two cases:
/// 1. **Primitive leaf**: `cube(10);` where module_call is a primitive
/// 2. **Transform node**: `translate([1,0,0]) ...` where module_call is a transform
///
/// # Examples
///
/// ```ignore
/// use openscad_ast::parser::transform_chain::parse_transform_chain;
///
/// // Typically called by parse_statement, not directly
/// let stmt = parse_transform_chain(&node, source)?;
/// ```
pub fn parse_transform_chain(
    node: &Node,
    source: &str,
) -> Result<Option<Statement>, Vec<Diagnostic>> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    // Extract the module_call and the child statement
    let transform_node = children.iter().find(|n| n.kind() == "module_call");
    let body_node = children.last();

    if let (Some(transform), Some(body)) = (transform_node, body_node) {
        // Parse the child statement recursively
        let inner_stmt = if body.kind() == "statement" {
            if let Some(child) = body.named_child(0) {
                parse_statement(&child, source)?
            } else {
                // Semicolon case - no child statement
                if body.child_count() > 0 && body.child(0).unwrap().kind() == ";" {
                    None
                } else {
                    None
                }
            }
        } else {
            // Direct child node
            if body.kind() == ";" { None } else { parse_statement(body, source)? }
        };

        // Check if the module_call is a primitive (cube/sphere/cylinder)
        if let Some(primitive) = parse_module_call(transform, source)? {
            return Ok(Some(primitive));
        }

        // Not a primitive, so it must be a transform operation
        let transform_name_node = transform.child_by_field_name("name");
        let args_node = transform.child_by_field_name("arguments");

        if let (Some(name_node), Some(args), Some(child_stmt)) = (transform_name_node, args_node, inner_stmt) {
            let name = &source[name_node.byte_range()];
            let vector = parse_vector_arg(&args, source)?;
            let span = Span::new(node.start_byte(), node.end_byte()).unwrap();

            // Dispatch to appropriate transform type
            match name {
                "translate" => return Ok(Some(Statement::Translate { vector, child: Box::new(child_stmt), span })),
                "rotate" => return Ok(Some(Statement::Rotate { vector, child: Box::new(child_stmt), span })),
                "scale" => return Ok(Some(Statement::Scale { vector, child: Box::new(child_stmt), span })),
                _ => return Ok(None), // Unknown transform
            }
        }
    }

    Ok(None)
}

/// Parses a 3D vector argument from transform arguments.
///
/// Transforms require a single vector argument `[x, y, z]` specifying the
/// transformation parameters.
///
/// # Arguments
///
/// * `args_node` - The CST node containing the arguments
/// * `source` - The original source code
///
/// # Returns
///
/// * `Ok([f64; 3])` - Successfully parsed 3D vector
/// * `Err(Vec<Diagnostic>)` - Parse errors if vector is missing or invalid
///
/// # Examples
///
/// ```ignore
/// // Parses: translate([10, 20, 30])
/// let vector = parse_vector_arg(&args_node, source)?;
/// assert_eq!(vector, [10.0, 20.0, 30.0]);
/// ```
fn parse_vector_arg(args_node: &Node, source: &str) -> Result<[f64; 3], Vec<Diagnostic>> {
    let mut cursor = args_node.walk();
    for child in args_node.children(&mut cursor) {
        if child.kind() == "list" {
             match parse_vector(&child, source) {
                 Ok(CubeSize::Vector(v)) => return Ok(v),
                 Ok(_) => return Err(vec![Diagnostic::error("Expected vector", Span::new(child.start_byte(), child.end_byte()).unwrap())]),
                 Err(e) => return Err(e),
             }
        }
    }
    Err(vec![Diagnostic::error("Transform requires a vector argument", Span::new(args_node.start_byte(), args_node.end_byte()).unwrap())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_to_ast;

    /// Tests parsing a simple translate transform with a cube child.
    #[test]
    fn test_parse_translate() {
        let ast = parse_to_ast("translate([10, 20, 30]) cube(10);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);

        match &ast[0] {
            Statement::Translate { vector, child, .. } => {
                assert_eq!(*vector, [10.0, 20.0, 30.0]);
                match &**child {
                    Statement::Cube { size, .. } => {
                        assert_eq!(*size, CubeSize::Scalar(10.0));
                    }
                    _ => panic!("Expected Cube child"),
                }
            }
            _ => panic!("Expected Translate"),
        }
    }

    /// Tests parsing nested transforms (translate → rotate → cube).
    /// This validates the recursive nature of transform chains.
    #[test]
    fn test_parse_nested_transforms() {
        let ast = parse_to_ast("translate([1,0,0]) rotate([0,90,0]) cube(10);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);

        match &ast[0] {
            Statement::Translate { vector: v1, child: c1, .. } => {
                assert_eq!(*v1, [1.0, 0.0, 0.0]);
                match &**c1 {
                    Statement::Rotate { vector: v2, child: c2, .. } => {
                        assert_eq!(*v2, [0.0, 90.0, 0.0]);
                        match &**c2 {
                            Statement::Cube { .. } => {},
                            _ => panic!("Expected Cube"),
                        }
                    }
                    _ => panic!("Expected Rotate"),
                }
            }
            _ => panic!("Expected Translate"),
        }
    }
}

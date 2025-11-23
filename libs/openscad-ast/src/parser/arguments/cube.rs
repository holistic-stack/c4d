//! Cube argument parsing.

use crate::{ast_types::CubeSize, Diagnostic, Span};
use tree_sitter::Node;
use super::shared::{parse_f64, parse_bool, parse_vector};

/// Parses cube arguments from the CST.
///
/// Handles both scalar and vector forms, plus named arguments.
/// Returns (size, center) where center is None if not specified.
///
/// # Supported Forms
/// - `cube(10)` → (Scalar(10), None)
/// - `cube([1,2,3])` → (Vector([1,2,3]), None)
/// - `cube(10, true)` → (Scalar(10), Some(true))
/// - `cube(size=10, center=true)` → (Scalar(10), Some(true))
/// - `cube([1,2,3], center=false)` → (Vector([1,2,3]), Some(false))
pub fn parse_cube_arguments(
    args_node: &Node,
    source: &str,
) -> Result<(CubeSize, Option<bool>), Vec<Diagnostic>> {
    let mut cursor = args_node.walk();
    let mut size: Option<CubeSize> = None;
    let mut center: Option<bool> = None;
    let mut positional_index = 0;
    
    for child in args_node.children(&mut cursor) {
        let kind = child.kind();
        
        if kind == "assignment" {
            // Named argument: name=value
            let name_node = child.child_by_field_name("name");
            let value_node = child.child_by_field_name("value");
            
            if let (Some(name_n), Some(value_n)) = (name_node, value_node) {
                let param_name = &source[name_n.byte_range()];
                
                match param_name {
                    "size" => {
                        size = Some(parse_size_value(&value_n, source)?);
                    }
                    "center" => {
                        if value_n.kind() == "boolean" {
                            center = Some(parse_bool(&value_n, source)?);
                        } else {
                            return Err(vec![Diagnostic::error(
                                "center parameter must be a boolean",
                                Span::new(value_n.start_byte(), value_n.end_byte()).unwrap(),
                            )]);
                        }
                    }
                    _ => {
                        return Err(vec![Diagnostic::error(
                            format!("Unknown parameter: {}", param_name),
                            Span::new(child.start_byte(), child.end_byte()).unwrap(),
                        )]);
                    }
                }
            }
        } else if kind == "number" || kind == "integer" || kind == "float" {
            // Positional scalar argument
            if positional_index == 0 {
                let value = parse_f64(&child, source)?;
                size = Some(CubeSize::Scalar(value));
            } else {
                return Err(vec![Diagnostic::error(
                    "Unexpected positional argument",
                    Span::new(child.start_byte(), child.end_byte()).unwrap(),
                )]);
            }
            positional_index += 1;
        } else if kind == "list" {
            // Positional vector argument
            if positional_index == 0 {
                size = Some(parse_vector(&child, source)?);
            } else {
                return Err(vec![Diagnostic::error(
                    "Unexpected positional argument",
                    Span::new(child.start_byte(), child.end_byte()).unwrap(),
                )]);
            }
            positional_index += 1;
        } else if kind == "boolean" {
            // Positional boolean argument (center)
            if positional_index == 1 {
                center = Some(parse_bool(&child, source)?);
            } else {
                return Err(vec![Diagnostic::error(
                    "Unexpected boolean argument",
                    Span::new(child.start_byte(), child.end_byte()).unwrap(),
                )]);
            }
            positional_index += 1;
        }
    }

    if size.is_none() {
        return Err(vec![Diagnostic::error(
            "cube() requires a size argument",
            Span::new(args_node.start_byte(), args_node.end_byte()).unwrap(),
        )
        .with_hint("Try: cube(10); or cube([1, 2, 3]);")]);
    }

    Ok((size.unwrap(), center))
}

/// Parses a size value (number or vector) from a CST node.
fn parse_size_value(
    value_node: &Node,
    source: &str,
) -> Result<CubeSize, Vec<Diagnostic>> {
    match value_node.kind() {
        "number" | "integer" | "float" => {
            let num = parse_f64(value_node, source)?;
            Ok(CubeSize::Scalar(num))
        }
        "list" => {
            parse_vector(value_node, source)
        }
        _ => {
            Err(vec![Diagnostic::error(
                format!("size parameter must be a number or vector, got {}", value_node.kind()),
                Span::new(value_node.start_byte(), value_node.end_byte()).unwrap(),
            )])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_to_ast;
    use crate::ast_types::Statement;

    #[test]
    fn test_parse_cube_scalar() {
        let ast = parse_to_ast("cube(10);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, .. } => {
                assert_eq!(*size, CubeSize::Scalar(10.0));
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_parse_cube_vector() {
        let ast = parse_to_ast("cube([1, 2, 3]);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, .. } => {
                assert_eq!(*size, CubeSize::Vector([1.0, 2.0, 3.0]));
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_parse_multiple_cubes() {
        let ast = parse_to_ast("cube(10);\ncube([1, 2, 3]);").expect("parse succeeds");
        assert_eq!(ast.len(), 2);
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let result = parse_to_ast("cube(");
        assert!(result.is_err());
        
        let diagnostics = result.unwrap_err();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity(), crate::Severity::Error);
    }

    #[test]
    fn test_parse_invalid_vector_size() {
        let result = parse_to_ast("cube([1, 2]);");
        assert!(result.is_err());
        
        let diagnostics = result.unwrap_err();
        assert!(diagnostics[0].message().contains("3 elements"));
    }

    #[test]
    fn test_parse_cube_with_center_named() {
        let ast = parse_to_ast("cube(size=[10,10,10], center=true);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, center, .. } => {
                assert_eq!(*size, CubeSize::Vector([10.0, 10.0, 10.0]));
                assert_eq!(*center, Some(true));
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_parse_cube_positional_with_center() {
        let ast = parse_to_ast("cube([10,10,10], true);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, center, .. } => {
                assert_eq!(*size, CubeSize::Vector([10.0, 10.0, 10.0]));
                assert_eq!(*center, Some(true));
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_parse_cube_mixed_args() {
        let ast = parse_to_ast("cube([10,10,10], center=false);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, center, .. } => {
                assert_eq!(*size, CubeSize::Vector([10.0, 10.0, 10.0]));
                assert_eq!(*center, Some(false));
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_parse_cube_center_only_named() {
        let ast = parse_to_ast("cube(size=10, center=true);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, center, .. } => {
                assert_eq!(*size, CubeSize::Scalar(10.0));
                assert_eq!(*center, Some(true));
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_parse_cube_center_default() {
        let ast = parse_to_ast("cube(10);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, center, .. } => {
                assert_eq!(*size, CubeSize::Scalar(10.0));
                assert_eq!(*center, None);
            }
            _ => panic!("Expected Cube"),
        }
    }
}

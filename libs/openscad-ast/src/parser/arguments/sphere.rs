//! Sphere argument parsing.

use crate::{Diagnostic, Span};
use tree_sitter::Node;
use super::shared::{parse_f64, parse_u32};

/// Parses sphere arguments from the CST.
///
/// # Supported Forms
/// - `sphere(r=10)`
/// - `sphere(d=20)`
/// - `sphere(10)`
pub fn parse_sphere_arguments(
    args_node: &Node,
    source: &str,
) -> Result<(f64, Option<f64>, Option<f64>, Option<u32>), Vec<Diagnostic>> {
    let mut cursor = args_node.walk();
    let mut radius: Option<f64> = None;
    let mut fa: Option<f64> = None;
    let mut fs: Option<f64> = None;
    let mut fn_: Option<u32> = None;
    let mut positional_index = 0;
    
    for child in args_node.children(&mut cursor) {
        let kind = child.kind();
        
        if kind == "assignment" {
            let name_node = child.child_by_field_name("name");
            let value_node = child.child_by_field_name("value");

            if let (Some(name_n), Some(value_n)) = (name_node, value_node) {
                let param_name = &source[name_n.byte_range()];

                match param_name {
                    "r" | "d" => { // 'd' is diameter, r = d/2. For now treat as radius or handle d.
                        // OpenSCAD supports r=radius or d=diameter.
                        // If d is used, radius = d/2.
                        let val = parse_f64(&value_n, source)?;
                        if param_name == "d" {
                            radius = Some(val / 2.0);
                        } else {
                            radius = Some(val);
                        }
                    }
                    "$fn" => {
                        fn_ = Some(parse_u32(&value_n, source)?);
                    }
                    "$fa" => {
                        fa = Some(parse_f64(&value_n, source)?);
                    }
                    "$fs" => {
                        fs = Some(parse_f64(&value_n, source)?);
                    }
                    _ => {
                        // Ignore unknown parameters or error?
                        // For sphere, only these matter for resolution.
                    }
                }
            }
        } else if kind == "number" || kind == "integer" || kind == "float" {
            if positional_index == 0 {
                radius = Some(parse_f64(&child, source)?);
            }
            positional_index += 1;
        }
    }

    if radius.is_none() {
        return Err(vec![Diagnostic::error(
            "sphere() requires a radius argument (r or d)",
            Span::new(args_node.start_byte(), args_node.end_byte()).unwrap(),
        )]);
    }

    Ok((radius.unwrap(), fa, fs, fn_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_to_ast;
    use crate::ast_types::Statement;

    #[test]
    fn test_parse_sphere_radius() {
        let ast = parse_to_ast("sphere(10);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Sphere { radius, .. } => {
                assert_eq!(*radius, 10.0);
            }
            _ => panic!("Expected Sphere"),
        }
    }

    #[test]
    fn test_parse_sphere_named_radius() {
        let ast = parse_to_ast("sphere(r=10);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Sphere { radius, .. } => {
                assert_eq!(*radius, 10.0);
            }
            _ => panic!("Expected Sphere"),
        }
    }

    #[test]
    fn test_parse_sphere_diameter() {
        let ast = parse_to_ast("sphere(d=20);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Sphere { radius, .. } => {
                assert_eq!(*radius, 10.0);
            }
            _ => panic!("Expected Sphere"),
        }
    }

    #[test]
    fn test_parse_sphere_fn() {
        let ast = parse_to_ast("sphere(10, $fn=100);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Sphere { radius, fn_, .. } => {
                assert_eq!(*radius, 10.0);
                assert_eq!(*fn_, Some(100));
            }
            _ => panic!("Expected Sphere"),
        }
    }

    #[test]
    fn test_parse_sphere_missing_args() {
        let result = parse_to_ast("sphere();");
        assert!(result.is_err());
    }
}

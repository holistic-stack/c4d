//! Parser integration tests.

use crate::{parse_to_ast, ast_types::{Statement, CubeSize}};

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

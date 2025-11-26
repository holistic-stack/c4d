//! # AST Tests
//!
//! Integration tests for the OpenSCAD AST parser.
//! Note: Primitives now use Expression for deferred evaluation.

use crate::parse_to_ast;
use crate::ast::{Expression, Statement};

/// Tests simple cube parsing - size is now stored as Expression
#[test]
fn test_parse_cube_simple() {
    let source = "cube(10);";
    let result = parse_to_ast(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 1);
    
    if let Statement::Cube { size, center, .. } = &stmts[0] {
        // Size is now stored as Expression::Number for scalar
        assert!(matches!(size, Expression::Number(n) if (*n - 10.0).abs() < 0.001));
        assert!(!center);
    } else {
        panic!("Expected Cube statement");
    }
}

#[test]
fn test_parse_cube_with_center() {
    let source = "cube(10, center=true);";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Cube { center, .. } = &stmts[0] {
        assert!(*center);
    } else {
        panic!("Expected Cube statement");
    }
}

/// Tests cube parsing with vector size - size is now stored as Expression
#[test]
fn test_parse_cube_vector_size() {
    let source = "cube([10, 20, 30]);";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Cube { size, .. } = &stmts[0] {
        // Size is now stored as Expression::Vector
        if let Expression::Vector(v) = size {
            assert_eq!(v.len(), 3);
            assert!(matches!(&v[0], Expression::Number(n) if (*n - 10.0).abs() < 0.001));
            assert!(matches!(&v[1], Expression::Number(n) if (*n - 20.0).abs() < 0.001));
            assert!(matches!(&v[2], Expression::Number(n) if (*n - 30.0).abs() < 0.001));
        } else {
            panic!("Expected Expression::Vector for size");
        }
    } else {
        panic!("Expected Cube statement");
    }
}

/// Tests sphere parsing - radius is now stored as Expression
#[test]
fn test_parse_sphere() {
    let source = "sphere(5);";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Sphere { radius, fn_override, .. } = &stmts[0] {
        // Radius is now stored as Expression::Number
        assert!(matches!(radius, Expression::Number(n) if (*n - 5.0).abs() < 0.001));
        // No $fn override
        assert!(fn_override.is_none());
    } else {
        panic!("Expected Sphere statement");
    }
}

/// Tests sphere parsing with $fn parameter
#[test]
fn test_parse_sphere_with_fn() {
    let source = "sphere(r=10, $fn=8);";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Sphere { radius, fn_override, .. } = &stmts[0] {
        assert!(matches!(radius, Expression::Number(n) if (*n - 10.0).abs() < 0.001));
        // $fn=8 should be stored as override
        assert_eq!(*fn_override, Some(8.0));
    } else {
        panic!("Expected Sphere statement");
    }
}

/// Tests sphere parsing with diameter - creates Expression::Binary for r = d/2
#[test]
fn test_parse_sphere_diameter() {
    let source = "sphere(d=10);";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Sphere { radius, .. } = &stmts[0] {
        // For d=10, radius should be Expression::Binary (10 / 2)
        if let Expression::Binary { left, operator, right } = radius {
            assert!(matches!(operator, crate::ast::BinaryOp::Divide));
            assert!(matches!(left.as_ref(), Expression::Number(n) if (*n - 10.0).abs() < 0.001));
            assert!(matches!(right.as_ref(), Expression::Number(n) if (*n - 2.0).abs() < 0.001));
        } else {
            panic!("Expected Expression::Binary for diameter to radius conversion");
        }
    } else {
        panic!("Expected Sphere statement");
    }
}

#[test]
fn test_parse_translate() {
    let source = "translate([10, 20, 30]) cube(5);";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Translate { vector, children, .. } = &stmts[0] {
        // Vector is now an Expression, check it's a Vector with correct values
        if let Expression::Vector(items) = vector {
            assert_eq!(items.len(), 3);
            if let (Expression::Number(x), Expression::Number(y), Expression::Number(z)) = 
                (&items[0], &items[1], &items[2]) {
                assert_eq!(*x, 10.0);
                assert_eq!(*y, 20.0);
                assert_eq!(*z, 30.0);
            } else {
                panic!("Expected Number expressions in vector");
            }
        } else {
            panic!("Expected Vector expression");
        }
        assert_eq!(children.len(), 1);
    } else {
        panic!("Expected Translate statement");
    }
}

#[test]
fn test_parse_union() {
    let source = "union() { cube(10); sphere(5); }";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Union { children, .. } = &stmts[0] {
        assert_eq!(children.len(), 2);
    } else {
        panic!("Expected Union statement");
    }
}

#[test]
fn test_parse_difference() {
    let source = "difference() { cube(15, center=true); sphere(10); }";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Difference { children, .. } = &stmts[0] {
        assert_eq!(children.len(), 2);
    } else {
        panic!("Expected Difference statement");
    }
}

#[test]
fn test_parse_intersection() {
    let source = "intersection() { cube(15, center=true); sphere(10); }";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Intersection { children, .. } = &stmts[0] {
        assert_eq!(children.len(), 2);
    } else {
        panic!("Expected Intersection statement");
    }
}

#[test]
fn test_parse_assignment() {
    let source = "$fn = 32;";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Assignment { name, .. } = &stmts[0] {
        assert_eq!(name, "$fn");
    } else {
        panic!("Expected Assignment statement");
    }
}

#[test]
fn test_parse_multiple_statements() {
    let source = "cube(10); sphere(5);";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 2);
}

#[test]
fn test_parse_nested_transforms() {
    let source = "translate([10,0,0]) rotate([0,90,0]) cube(5);";
    let result = parse_to_ast(source);
    assert!(result.is_ok());
    let stmts = result.unwrap();
    
    if let Statement::Translate { children, .. } = &stmts[0] {
        assert_eq!(children.len(), 1);
        if let Statement::Rotate { children: inner, .. } = &children[0] {
            assert_eq!(inner.len(), 1);
        } else {
            panic!("Expected nested Rotate");
        }
    } else {
        panic!("Expected Translate statement");
    }
}

#[test]
fn test_parse_target_validation_case() {
    // The target validation test case from overview-plan.md
    let source = r#"
translate([-24,0,0]) {
    union() {
        cube(15, center=true);
        sphere(10);
    }
}

intersection() {
    cube(15, center=true);
    sphere(10);
}

translate([24,0,0]) {
    difference() {
        cube(15, center=true);
        sphere(10);
    }
}
"#;
    let result = parse_to_ast(source);
    assert!(result.is_ok(), "Failed to parse target validation case: {:?}", result.err());
    let stmts = result.unwrap();
    assert_eq!(stmts.len(), 3, "Expected 3 top-level statements");
}

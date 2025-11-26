//! # Evaluator Tests

use crate::{evaluate, EvaluationContext, GeometryNode};
use crate::ir::BooleanOperation;
use glam::DVec3;

#[test]
fn test_evaluate_cube() {
    let source = "cube(10);";
    let result = evaluate(source);
    assert!(result.is_ok());
    let nodes = result.unwrap();
    assert_eq!(nodes.len(), 1);
    
    if let GeometryNode::Cube { size, center, .. } = &nodes[0] {
        assert_eq!(*size, DVec3::splat(10.0));
        assert!(!center);
    } else {
        panic!("Expected Cube node");
    }
}

#[test]
fn test_evaluate_sphere() {
    let source = "sphere(5);";
    let result = evaluate(source);
    assert!(result.is_ok());
    let nodes = result.unwrap();
    
    if let GeometryNode::Sphere { radius, segments, .. } = &nodes[0] {
        assert_eq!(*radius, 5.0);
        assert!(*segments >= 5); // At least MIN_FRAGMENTS
    } else {
        panic!("Expected Sphere node");
    }
}

#[test]
fn test_evaluate_translate() {
    let source = "translate([10, 20, 30]) cube(5);";
    let result = evaluate(source);
    assert!(result.is_ok());
    let nodes = result.unwrap();
    
    if let GeometryNode::Transform { matrix, children, .. } = &nodes[0] {
        // Check translation is in the matrix
        let translation = matrix.w_axis;
        assert_eq!(translation.x, 10.0);
        assert_eq!(translation.y, 20.0);
        assert_eq!(translation.z, 30.0);
        assert_eq!(children.len(), 1);
    } else {
        panic!("Expected Transform node");
    }
}

#[test]
fn test_evaluate_union() {
    let source = "union() { cube(10); sphere(5); }";
    let result = evaluate(source);
    assert!(result.is_ok());
    let nodes = result.unwrap();
    
    if let GeometryNode::Boolean { operation, children, .. } = &nodes[0] {
        assert_eq!(*operation, BooleanOperation::Union);
        assert_eq!(children.len(), 2);
    } else {
        panic!("Expected Boolean node");
    }
}

#[test]
fn test_evaluate_difference() {
    let source = "difference() { cube(15, center=true); sphere(10); }";
    let result = evaluate(source);
    assert!(result.is_ok());
    let nodes = result.unwrap();
    
    if let GeometryNode::Boolean { operation, children, .. } = &nodes[0] {
        assert_eq!(*operation, BooleanOperation::Difference);
        assert_eq!(children.len(), 2);
    } else {
        panic!("Expected Boolean node");
    }
}

#[test]
fn test_evaluate_intersection() {
    let source = "intersection() { cube(15, center=true); sphere(10); }";
    let result = evaluate(source);
    assert!(result.is_ok());
    let nodes = result.unwrap();
    
    if let GeometryNode::Boolean { operation, children, .. } = &nodes[0] {
        assert_eq!(*operation, BooleanOperation::Intersection);
        assert_eq!(children.len(), 2);
    } else {
        panic!("Expected Boolean node");
    }
}

#[test]
fn test_evaluate_fn_assignment() {
    let source = "$fn = 32; sphere(10);";
    let result = evaluate(source);
    assert!(result.is_ok());
    let nodes = result.unwrap();
    
    // Should have one sphere (assignment doesn't produce geometry)
    assert_eq!(nodes.len(), 1);
    
    if let GeometryNode::Sphere { segments, .. } = &nodes[0] {
        assert_eq!(*segments, 32);
    } else {
        panic!("Expected Sphere node");
    }
}

/// Tests that local $fn override on sphere takes precedence over global $fn
#[test]
fn test_evaluate_sphere_local_fn_override() {
    let source = "$fn = 32; sphere(r=10, $fn=8);";
    let result = evaluate(source);
    assert!(result.is_ok());
    let nodes = result.unwrap();
    
    assert_eq!(nodes.len(), 1);
    
    if let GeometryNode::Sphere { segments, .. } = &nodes[0] {
        // Local $fn=8 should override global $fn=32
        assert_eq!(*segments, 8);
    } else {
        panic!("Expected Sphere node");
    }
}

#[test]
fn test_evaluate_nested_transforms() {
    let source = "translate([10,0,0]) rotate([0,90,0]) cube(5);";
    let result = evaluate(source);
    assert!(result.is_ok());
    let nodes = result.unwrap();
    
    if let GeometryNode::Transform { children, .. } = &nodes[0] {
        assert_eq!(children.len(), 1);
        if let GeometryNode::Transform { children: inner, .. } = &children[0] {
            assert_eq!(inner.len(), 1);
            assert!(matches!(inner[0], GeometryNode::Cube { .. }));
        } else {
            panic!("Expected nested Transform");
        }
    } else {
        panic!("Expected Transform node");
    }
}

#[test]
fn test_evaluate_target_validation_case() {
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
    let result = evaluate(source);
    assert!(result.is_ok(), "Failed to evaluate: {:?}", result.err());
    let nodes = result.unwrap();
    assert_eq!(nodes.len(), 3, "Expected 3 top-level geometry nodes");
    
    // First: translate with union
    assert!(matches!(&nodes[0], GeometryNode::Transform { .. }));
    
    // Second: intersection
    assert!(matches!(&nodes[1], GeometryNode::Boolean { operation: BooleanOperation::Intersection, .. }));
    
    // Third: translate with difference
    assert!(matches!(&nodes[2], GeometryNode::Transform { .. }));
}

#[test]
fn test_context_fn_affects_sphere() {
    let mut ctx = EvaluationContext::new();
    ctx.set_fn(64.0);
    
    let source = "sphere(10);";
    let statements = openscad_ast::parse_to_ast(source).unwrap();
    let nodes = crate::evaluator::evaluate_statements(&statements, &mut ctx).unwrap();
    
    if let GeometryNode::Sphere { segments, .. } = &nodes[0] {
        assert_eq!(*segments, 64);
    } else {
        panic!("Expected Sphere node");
    }
}

/// Test with exact browser CST JSON for module_call
#[test]
fn test_browser_cst_module_call() {
    // This is the exact JSON from the browser for: sphere(r=10, $fn=8);
    let module_call_json = r#"{"type":"module_call","text":"sphere(r=10, $fn=8)","startIndex":0,"endIndex":19,"startPosition":{"row":0,"column":0},"endPosition":{"row":0,"column":19},"children":[{"type":"identifier","text":"sphere","startIndex":0,"endIndex":6,"startPosition":{"row":0,"column":0},"endPosition":{"row":0,"column":6},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"name"},{"type":"arguments","text":"(r=10, $fn=8)","startIndex":6,"endIndex":19,"startPosition":{"row":0,"column":6},"endPosition":{"row":0,"column":19},"children":[{"type":"(","text":"(","startIndex":6,"endIndex":7,"startPosition":{"row":0,"column":6},"endPosition":{"row":0,"column":7},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null},{"type":"assignment","text":"r=10","startIndex":7,"endIndex":11,"startPosition":{"row":0,"column":7},"endPosition":{"row":0,"column":11},"children":[{"type":"identifier","text":"r","startIndex":7,"endIndex":8,"startPosition":{"row":0,"column":7},"endPosition":{"row":0,"column":8},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"name"},{"type":"=","text":"=","startIndex":8,"endIndex":9,"startPosition":{"row":0,"column":8},"endPosition":{"row":0,"column":9},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null},{"type":"integer","text":"10","startIndex":9,"endIndex":11,"startPosition":{"row":0,"column":9},"endPosition":{"row":0,"column":11},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"value"}],"namedChildren":[{"type":"identifier","text":"r","startIndex":7,"endIndex":8,"startPosition":{"row":0,"column":7},"endPosition":{"row":0,"column":8},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"name"},{"type":"integer","text":"10","startIndex":9,"endIndex":11,"startPosition":{"row":0,"column":9},"endPosition":{"row":0,"column":11},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"value"}],"isNamed":true,"fieldName":null},{"type":",","text":",","startIndex":11,"endIndex":12,"startPosition":{"row":0,"column":11},"endPosition":{"row":0,"column":12},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null},{"type":"assignment","text":"$fn=8","startIndex":13,"endIndex":18,"startPosition":{"row":0,"column":13},"endPosition":{"row":0,"column":18},"children":[{"type":"special_variable","text":"$fn","startIndex":13,"endIndex":16,"startPosition":{"row":0,"column":13},"endPosition":{"row":0,"column":16},"children":[{"type":"$","text":"$","startIndex":13,"endIndex":14,"startPosition":{"row":0,"column":13},"endPosition":{"row":0,"column":14},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null},{"type":"identifier","text":"fn","startIndex":14,"endIndex":16,"startPosition":{"row":0,"column":14},"endPosition":{"row":0,"column":16},"children":[],"namedChildren":[],"isNamed":true,"fieldName":null}],"namedChildren":[{"type":"identifier","text":"fn","startIndex":14,"endIndex":16,"startPosition":{"row":0,"column":14},"endPosition":{"row":0,"column":16},"children":[],"namedChildren":[],"isNamed":true,"fieldName":null}],"isNamed":true,"fieldName":"name"},{"type":"=","text":"=","startIndex":16,"endIndex":17,"startPosition":{"row":0,"column":16},"endPosition":{"row":0,"column":17},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null},{"type":"integer","text":"8","startIndex":17,"endIndex":18,"startPosition":{"row":0,"column":17},"endPosition":{"row":0,"column":18},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"value"}],"namedChildren":[{"type":"special_variable","text":"$fn","startIndex":13,"endIndex":16,"startPosition":{"row":0,"column":13},"endPosition":{"row":0,"column":16},"children":[{"type":"$","text":"$","startIndex":13,"endIndex":14,"startPosition":{"row":0,"column":13},"endPosition":{"row":0,"column":14},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null},{"type":"identifier","text":"fn","startIndex":14,"endIndex":16,"startPosition":{"row":0,"column":14},"endPosition":{"row":0,"column":16},"children":[],"namedChildren":[],"isNamed":true,"fieldName":null}],"namedChildren":[{"type":"identifier","text":"fn","startIndex":14,"endIndex":16,"startPosition":{"row":0,"column":14},"endPosition":{"row":0,"column":16},"children":[],"namedChildren":[],"isNamed":true,"fieldName":null}],"isNamed":true,"fieldName":"name"},{"type":"integer","text":"8","startIndex":17,"endIndex":18,"startPosition":{"row":0,"column":17},"endPosition":{"row":0,"column":18},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"value"}],"isNamed":true,"fieldName":null},{"type":")","text":")","startIndex":18,"endIndex":19,"startPosition":{"row":0,"column":18},"endPosition":{"row":0,"column":19},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null}],"namedChildren":[{"type":"assignment","text":"r=10","startIndex":7,"endIndex":11,"startPosition":{"row":0,"column":7},"endPosition":{"row":0,"column":11},"children":[{"type":"identifier","text":"r","startIndex":7,"endIndex":8,"startPosition":{"row":0,"column":7},"endPosition":{"row":0,"column":8},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"name"},{"type":"=","text":"=","startIndex":8,"endIndex":9,"startPosition":{"row":0,"column":8},"endPosition":{"row":0,"column":9},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null},{"type":"integer","text":"10","startIndex":9,"endIndex":11,"startPosition":{"row":0,"column":9},"endPosition":{"row":0,"column":11},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"value"}],"namedChildren":[{"type":"identifier","text":"r","startIndex":7,"endIndex":8,"startPosition":{"row":0,"column":7},"endPosition":{"row":0,"column":8},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"name"},{"type":"integer","text":"10","startIndex":9,"endIndex":11,"startPosition":{"row":0,"column":9},"endPosition":{"row":0,"column":11},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"value"}],"isNamed":true,"fieldName":null},{"type":"assignment","text":"$fn=8","startIndex":13,"endIndex":18,"startPosition":{"row":0,"column":13},"endPosition":{"row":0,"column":18},"children":[{"type":"special_variable","text":"$fn","startIndex":13,"endIndex":16,"startPosition":{"row":0,"column":13},"endPosition":{"row":0,"column":16},"children":[{"type":"$","text":"$","startIndex":13,"endIndex":14,"startPosition":{"row":0,"column":13},"endPosition":{"row":0,"column":14},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null},{"type":"identifier","text":"fn","startIndex":14,"endIndex":16,"startPosition":{"row":0,"column":14},"endPosition":{"row":0,"column":16},"children":[],"namedChildren":[],"isNamed":true,"fieldName":null}],"namedChildren":[{"type":"identifier","text":"fn","startIndex":14,"endIndex":16,"startPosition":{"row":0,"column":14},"endPosition":{"row":0,"column":16},"children":[],"namedChildren":[],"isNamed":true,"fieldName":null}],"isNamed":true,"fieldName":"name"},{"type":"=","text":"=","startIndex":16,"endIndex":17,"startPosition":{"row":0,"column":16},"endPosition":{"row":0,"column":17},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null},{"type":"integer","text":"8","startIndex":17,"endIndex":18,"startPosition":{"row":0,"column":17},"endPosition":{"row":0,"column":18},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"value"}],"namedChildren":[{"type":"special_variable","text":"$fn","startIndex":13,"endIndex":16,"startPosition":{"row":0,"column":13},"endPosition":{"row":0,"column":16},"children":[{"type":"$","text":"$","startIndex":13,"endIndex":14,"startPosition":{"row":0,"column":13},"endPosition":{"row":0,"column":14},"children":[],"namedChildren":[],"isNamed":false,"fieldName":null},{"type":"identifier","text":"fn","startIndex":14,"endIndex":16,"startPosition":{"row":0,"column":14},"endPosition":{"row":0,"column":16},"children":[],"namedChildren":[],"isNamed":true,"fieldName":null}],"namedChildren":[{"type":"identifier","text":"fn","startIndex":14,"endIndex":16,"startPosition":{"row":0,"column":14},"endPosition":{"row":0,"column":16},"children":[],"namedChildren":[],"isNamed":true,"fieldName":null}],"isNamed":true,"fieldName":"name"},{"type":"integer","text":"8","startIndex":17,"endIndex":18,"startPosition":{"row":0,"column":17},"endPosition":{"row":0,"column":18},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"value"}],"isNamed":true,"fieldName":null}],"isNamed":true,"fieldName":"arguments"}],"namedChildren":[{"type":"identifier","text":"sphere","startIndex":0,"endIndex":6,"startPosition":{"row":0,"column":0},"endPosition":{"row":0,"column":6},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"name"},{"type":"arguments","text":"(r=10, $fn=8)","startIndex":6,"endIndex":19,"startPosition":{"row":0,"column":6},"endPosition":{"row":0,"column":19},"children":[],"namedChildren":[],"isNamed":true,"fieldName":"arguments"}],"isNamed":true,"fieldName":null}"#;
    
    let node: openscad_ast::SerializedNode = serde_json::from_str(module_call_json).unwrap();
    
    // Find the arguments node in children
    let args_node = node.children.iter().find(|c| c.node_type == "arguments").unwrap();
    
    // Check that namedChildren has the assignment nodes
    assert_eq!(args_node.named_children.len(), 2, "Should have 2 named children (assignments)");
    
    // Check the $fn assignment
    let fn_assignment = args_node.named_children.iter().find(|c| c.text == "$fn=8").unwrap();
    assert_eq!(fn_assignment.children.len(), 3, "Assignment should have 3 children");
    
    // Check that child_by_field finds the name
    let name_node = fn_assignment.child_by_field("name");
    assert!(name_node.is_some(), "Should find name field");
    assert_eq!(name_node.unwrap().text, "$fn", "Name should be $fn");
}

/// Test get_named_arg function
#[test]
fn test_get_named_arg() {
    use openscad_ast::{Argument, Expression};
    
    let args = vec![
        Argument::named("r".to_string(), Expression::Number(10.0)),
        Argument::named("$fn".to_string(), Expression::Number(8.0)),
    ];
    
    // Test finding $fn
    let fn_arg = args.iter().find(|a| a.name.as_deref() == Some("$fn"));
    assert!(fn_arg.is_some(), "Should find $fn argument");
    
    if let Expression::Number(n) = &fn_arg.unwrap().value {
        assert_eq!(*n, 8.0);
    } else {
        panic!("Expected Number expression");
    }
}

/// Test CST child_by_field function
#[test]
fn test_child_by_field() {
    let json = r#"{
        "type": "assignment",
        "text": "$fn=8",
        "startIndex": 0,
        "endIndex": 5,
        "startPosition": {"row": 0, "column": 0},
        "endPosition": {"row": 0, "column": 5},
        "children": [
            {"type": "special_variable", "text": "$fn", "startIndex": 0, "endIndex": 3, "startPosition": {"row": 0, "column": 0}, "endPosition": {"row": 0, "column": 3}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "name"},
            {"type": "=", "text": "=", "startIndex": 3, "endIndex": 4, "startPosition": {"row": 0, "column": 3}, "endPosition": {"row": 0, "column": 4}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null},
            {"type": "integer", "text": "8", "startIndex": 4, "endIndex": 5, "startPosition": {"row": 0, "column": 4}, "endPosition": {"row": 0, "column": 5}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "value"}
        ],
        "namedChildren": [
            {"type": "special_variable", "text": "$fn", "startIndex": 0, "endIndex": 3, "startPosition": {"row": 0, "column": 0}, "endPosition": {"row": 0, "column": 3}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "name"},
            {"type": "integer", "text": "8", "startIndex": 4, "endIndex": 5, "startPosition": {"row": 0, "column": 4}, "endPosition": {"row": 0, "column": 5}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "value"}
        ],
        "isNamed": true,
        "fieldName": null
    }"#;
    
    let node: openscad_ast::SerializedNode = serde_json::from_str(json).unwrap();
    
    // Test child_by_field
    let name_node = node.child_by_field("name");
    assert!(name_node.is_some(), "Should find name field");
    assert_eq!(name_node.unwrap().text, "$fn");
    
    let value_node = node.child_by_field("value");
    assert!(value_node.is_some(), "Should find value field");
    assert_eq!(value_node.unwrap().text, "8");
}

/// Test CST parsing of sphere with $fn parameter
#[test]
fn test_cst_sphere_with_fn() {
    // Simulate the CST structure from web-tree-sitter
    let cst_json = r#"{
        "type": "source_file",
        "text": "sphere(r=10, $fn=8);",
        "startIndex": 0,
        "endIndex": 20,
        "startPosition": {"row": 0, "column": 0},
        "endPosition": {"row": 0, "column": 20},
        "children": [{
            "type": "transform_chain",
            "text": "sphere(r=10, $fn=8);",
            "startIndex": 0,
            "endIndex": 20,
            "startPosition": {"row": 0, "column": 0},
            "endPosition": {"row": 0, "column": 20},
            "children": [{
                "type": "module_call",
                "text": "sphere(r=10, $fn=8)",
                "startIndex": 0,
                "endIndex": 19,
                "startPosition": {"row": 0, "column": 0},
                "endPosition": {"row": 0, "column": 19},
                "children": [
                    {
                        "type": "identifier",
                        "text": "sphere",
                        "startIndex": 0,
                        "endIndex": 6,
                        "startPosition": {"row": 0, "column": 0},
                        "endPosition": {"row": 0, "column": 6},
                        "children": [],
                        "namedChildren": [],
                        "isNamed": true,
                        "fieldName": "name"
                    },
                    {
                        "type": "arguments",
                        "text": "(r=10, $fn=8)",
                        "startIndex": 6,
                        "endIndex": 19,
                        "startPosition": {"row": 0, "column": 6},
                        "endPosition": {"row": 0, "column": 19},
                        "children": [
                            {"type": "(", "text": "(", "startIndex": 6, "endIndex": 7, "startPosition": {"row": 0, "column": 6}, "endPosition": {"row": 0, "column": 7}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null},
                            {
                                "type": "assignment",
                                "text": "r=10",
                                "startIndex": 7,
                                "endIndex": 11,
                                "startPosition": {"row": 0, "column": 7},
                                "endPosition": {"row": 0, "column": 11},
                                "children": [
                                    {"type": "identifier", "text": "r", "startIndex": 7, "endIndex": 8, "startPosition": {"row": 0, "column": 7}, "endPosition": {"row": 0, "column": 8}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "name"},
                                    {"type": "=", "text": "=", "startIndex": 8, "endIndex": 9, "startPosition": {"row": 0, "column": 8}, "endPosition": {"row": 0, "column": 9}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null},
                                    {"type": "integer", "text": "10", "startIndex": 9, "endIndex": 11, "startPosition": {"row": 0, "column": 9}, "endPosition": {"row": 0, "column": 11}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "value"}
                                ],
                                "namedChildren": [
                                    {"type": "identifier", "text": "r", "startIndex": 7, "endIndex": 8, "startPosition": {"row": 0, "column": 7}, "endPosition": {"row": 0, "column": 8}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "name"},
                                    {"type": "integer", "text": "10", "startIndex": 9, "endIndex": 11, "startPosition": {"row": 0, "column": 9}, "endPosition": {"row": 0, "column": 11}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "value"}
                                ],
                                "isNamed": true,
                                "fieldName": null
                            },
                            {"type": ",", "text": ",", "startIndex": 11, "endIndex": 12, "startPosition": {"row": 0, "column": 11}, "endPosition": {"row": 0, "column": 12}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null},
                            {
                                "type": "assignment",
                                "text": "$fn=8",
                                "startIndex": 13,
                                "endIndex": 18,
                                "startPosition": {"row": 0, "column": 13},
                                "endPosition": {"row": 0, "column": 18},
                                "children": [
                                    {"type": "special_variable", "text": "$fn", "startIndex": 13, "endIndex": 16, "startPosition": {"row": 0, "column": 13}, "endPosition": {"row": 0, "column": 16}, "children": [{"type": "$", "text": "$", "startIndex": 13, "endIndex": 14, "startPosition": {"row": 0, "column": 13}, "endPosition": {"row": 0, "column": 14}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null}, {"type": "identifier", "text": "fn", "startIndex": 14, "endIndex": 16, "startPosition": {"row": 0, "column": 14}, "endPosition": {"row": 0, "column": 16}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": null}], "namedChildren": [{"type": "identifier", "text": "fn", "startIndex": 14, "endIndex": 16, "startPosition": {"row": 0, "column": 14}, "endPosition": {"row": 0, "column": 16}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": null}], "isNamed": true, "fieldName": "name"},
                                    {"type": "=", "text": "=", "startIndex": 16, "endIndex": 17, "startPosition": {"row": 0, "column": 16}, "endPosition": {"row": 0, "column": 17}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null},
                                    {"type": "integer", "text": "8", "startIndex": 17, "endIndex": 18, "startPosition": {"row": 0, "column": 17}, "endPosition": {"row": 0, "column": 18}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "value"}
                                ],
                                "namedChildren": [
                                    {"type": "special_variable", "text": "$fn", "startIndex": 13, "endIndex": 16, "startPosition": {"row": 0, "column": 13}, "endPosition": {"row": 0, "column": 16}, "children": [{"type": "$", "text": "$", "startIndex": 13, "endIndex": 14, "startPosition": {"row": 0, "column": 13}, "endPosition": {"row": 0, "column": 14}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null}, {"type": "identifier", "text": "fn", "startIndex": 14, "endIndex": 16, "startPosition": {"row": 0, "column": 14}, "endPosition": {"row": 0, "column": 16}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": null}], "namedChildren": [{"type": "identifier", "text": "fn", "startIndex": 14, "endIndex": 16, "startPosition": {"row": 0, "column": 14}, "endPosition": {"row": 0, "column": 16}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": null}], "isNamed": true, "fieldName": "name"},
                                    {"type": "integer", "text": "8", "startIndex": 17, "endIndex": 18, "startPosition": {"row": 0, "column": 17}, "endPosition": {"row": 0, "column": 18}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "value"}
                                ],
                                "isNamed": true,
                                "fieldName": null
                            },
                            {"type": ")", "text": ")", "startIndex": 18, "endIndex": 19, "startPosition": {"row": 0, "column": 18}, "endPosition": {"row": 0, "column": 19}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null}
                        ],
                        "namedChildren": [
                            {
                                "type": "assignment",
                                "text": "r=10",
                                "startIndex": 7,
                                "endIndex": 11,
                                "startPosition": {"row": 0, "column": 7},
                                "endPosition": {"row": 0, "column": 11},
                                "children": [
                                    {"type": "identifier", "text": "r", "startIndex": 7, "endIndex": 8, "startPosition": {"row": 0, "column": 7}, "endPosition": {"row": 0, "column": 8}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "name"},
                                    {"type": "=", "text": "=", "startIndex": 8, "endIndex": 9, "startPosition": {"row": 0, "column": 8}, "endPosition": {"row": 0, "column": 9}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null},
                                    {"type": "integer", "text": "10", "startIndex": 9, "endIndex": 11, "startPosition": {"row": 0, "column": 9}, "endPosition": {"row": 0, "column": 11}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "value"}
                                ],
                                "namedChildren": [
                                    {"type": "identifier", "text": "r", "startIndex": 7, "endIndex": 8, "startPosition": {"row": 0, "column": 7}, "endPosition": {"row": 0, "column": 8}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "name"},
                                    {"type": "integer", "text": "10", "startIndex": 9, "endIndex": 11, "startPosition": {"row": 0, "column": 9}, "endPosition": {"row": 0, "column": 11}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "value"}
                                ],
                                "isNamed": true,
                                "fieldName": null
                            },
                            {
                                "type": "assignment",
                                "text": "$fn=8",
                                "startIndex": 13,
                                "endIndex": 18,
                                "startPosition": {"row": 0, "column": 13},
                                "endPosition": {"row": 0, "column": 18},
                                "children": [
                                    {"type": "special_variable", "text": "$fn", "startIndex": 13, "endIndex": 16, "startPosition": {"row": 0, "column": 13}, "endPosition": {"row": 0, "column": 16}, "children": [{"type": "$", "text": "$", "startIndex": 13, "endIndex": 14, "startPosition": {"row": 0, "column": 13}, "endPosition": {"row": 0, "column": 14}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null}, {"type": "identifier", "text": "fn", "startIndex": 14, "endIndex": 16, "startPosition": {"row": 0, "column": 14}, "endPosition": {"row": 0, "column": 16}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": null}], "namedChildren": [{"type": "identifier", "text": "fn", "startIndex": 14, "endIndex": 16, "startPosition": {"row": 0, "column": 14}, "endPosition": {"row": 0, "column": 16}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": null}], "isNamed": true, "fieldName": "name"},
                                    {"type": "=", "text": "=", "startIndex": 16, "endIndex": 17, "startPosition": {"row": 0, "column": 16}, "endPosition": {"row": 0, "column": 17}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null},
                                    {"type": "integer", "text": "8", "startIndex": 17, "endIndex": 18, "startPosition": {"row": 0, "column": 17}, "endPosition": {"row": 0, "column": 18}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "value"}
                                ],
                                "namedChildren": [
                                    {"type": "special_variable", "text": "$fn", "startIndex": 13, "endIndex": 16, "startPosition": {"row": 0, "column": 13}, "endPosition": {"row": 0, "column": 16}, "children": [{"type": "$", "text": "$", "startIndex": 13, "endIndex": 14, "startPosition": {"row": 0, "column": 13}, "endPosition": {"row": 0, "column": 14}, "children": [], "namedChildren": [], "isNamed": false, "fieldName": null}, {"type": "identifier", "text": "fn", "startIndex": 14, "endIndex": 16, "startPosition": {"row": 0, "column": 14}, "endPosition": {"row": 0, "column": 16}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": null}], "namedChildren": [{"type": "identifier", "text": "fn", "startIndex": 14, "endIndex": 16, "startPosition": {"row": 0, "column": 14}, "endPosition": {"row": 0, "column": 16}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": null}], "isNamed": true, "fieldName": "name"},
                                    {"type": "integer", "text": "8", "startIndex": 17, "endIndex": 18, "startPosition": {"row": 0, "column": 17}, "endPosition": {"row": 0, "column": 18}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "value"}
                                ],
                                "isNamed": true,
                                "fieldName": null
                            }
                        ],
                        "isNamed": true,
                        "fieldName": "arguments"
                    }
                ],
                "namedChildren": [
                    {"type": "identifier", "text": "sphere", "startIndex": 0, "endIndex": 6, "startPosition": {"row": 0, "column": 0}, "endPosition": {"row": 0, "column": 6}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "name"},
                    {"type": "arguments", "text": "(r=10, $fn=8)", "startIndex": 6, "endIndex": 19, "startPosition": {"row": 0, "column": 6}, "endPosition": {"row": 0, "column": 19}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": "arguments"}
                ],
                "isNamed": true,
                "fieldName": null
            }],
            "namedChildren": [{"type": "module_call", "text": "sphere(r=10, $fn=8)", "startIndex": 0, "endIndex": 19, "startPosition": {"row": 0, "column": 0}, "endPosition": {"row": 0, "column": 19}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": null}],
            "isNamed": true,
            "fieldName": null
        }],
        "namedChildren": [{"type": "transform_chain", "text": "sphere(r=10, $fn=8);", "startIndex": 0, "endIndex": 20, "startPosition": {"row": 0, "column": 0}, "endPosition": {"row": 0, "column": 20}, "children": [], "namedChildren": [], "isNamed": true, "fieldName": null}],
        "isNamed": true,
        "fieldName": null
    }"#;
    
    let cst: openscad_ast::SerializedNode = serde_json::from_str(cst_json).unwrap();
    let statements = openscad_ast::parse_from_cst(&cst).unwrap();
    
    // Should have one sphere statement
    assert_eq!(statements.len(), 1);
    
    if let openscad_ast::Statement::Sphere { fn_override, .. } = &statements[0] {
        // $fn=8 should be parsed as override
        assert_eq!(*fn_override, Some(8.0), "Expected fn_override to be Some(8.0), got {:?}", fn_override);
    } else {
        panic!("Expected Sphere statement, got {:?}", statements[0]);
    }
}

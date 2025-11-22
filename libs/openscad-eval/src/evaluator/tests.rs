//! Evaluator tests guaranteeing deterministic behavior.

use super::*;
use crate::filesystem::InMemoryFilesystem;
use glam::DVec3;

/// Confirms `evaluate_source` parses cube primitives.
///
/// # Examples
/// ```
/// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem};
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// assert_eq!(evaluator.evaluate_source("cube(1);").unwrap().len(), 1);
/// ```
#[test]
fn evaluate_cube_literal() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator.evaluate_source("cube(2);").expect("cube parsed");
    assert_eq!(nodes.len(), 1);
}

/// Confirms `evaluate_source` correctly handles vector cube arguments.
///
/// # Examples
/// ```rust
/// use glam::DVec3;
/// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem};
///
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// let nodes = evaluator.evaluate_source("cube([1, 2, 3]);").unwrap();
/// assert_eq!(nodes[0].size(), DVec3::new(1.0, 2.0, 3.0));
/// ```
#[test]
fn evaluate_cube_vector_literal() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("cube([1, 2, 3]);")
        .expect("cube parsed");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].size(), DVec3::new(1.0, 2.0, 3.0));
}

/// Ensures unsupported snippets raise explicit errors.
///
/// # Examples
/// ```
/// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem};
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// assert!(evaluator.evaluate_source("cylinder(1);").is_err());
/// ```
#[test]
fn evaluate_unsupported_source() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    // sphere is now supported, so we use cylinder
    let err = evaluator.evaluate_source("cylinder(1);").unwrap_err();
    assert!(matches!(err, EvaluationError::UnsupportedSource { .. }));
}

#[test]
fn evaluate_sphere_literal() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator.evaluate_source("sphere(1);").expect("sphere parsed");
    assert_eq!(nodes.len(), 1);
}

/// Ensures assignments to $fn affect subsequent sphere resolution.
#[test]
fn evaluate_sphere_respects_global_fn() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("$fn = 24; sphere(r=10);")
        .expect("sphere parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Sphere { segments, .. } => assert_eq!(*segments, 24),
        _ => panic!("expected sphere"),
    }
}

/// Verifies rotate nodes wrap children inside a transform matrix that matches OpenSCAD's
/// column-vector semantics (rotation occurs about the origin prior to subsequent translations).
///
/// # Examples
/// ```
/// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem};
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// let nodes = evaluator.evaluate_source("rotate([0,0,90]) cube(1);").unwrap();
/// assert!(matches!(nodes[0], openscad_eval::ir::GeometryNode::Transform { .. }));
/// ```
#[test]
fn evaluate_rotate_wraps_child_transform() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("rotate([0,0,90]) cube(1);")
        .expect("rotate parsed");

    match &nodes[0] {
        GeometryNode::Transform { matrix, child, .. } => {
            // Rotation of 90 degrees around Z should swap X/Y axes (cos=0, sin=1).
            let cols = matrix.to_cols_array_2d();
            assert!((cols[0][0] - 0.0).abs() < 1e-12);
            assert!((cols[0][1] - 1.0).abs() < 1e-12);
            assert!((cols[1][0] + 1.0).abs() < 1e-12);
            assert!((cols[1][1] - 0.0).abs() < 1e-12);

            match &**child {
                GeometryNode::Cube { .. } => {}
                _ => panic!("expected cube child after rotation"),
            }
        }
        _ => panic!("expected transform node for rotate"),
    }
}

/// Ensures nested transforms respect OpenSCAD's inside-out evaluation order.
/// The rotate happens before the translate even though translate is written first.
#[test]
fn evaluate_translate_then_rotate_preserves_inside_out_order() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("translate([5,0,0]) rotate([0,0,90]) cube(1);")
        .expect("nested transform parsed");

    match &nodes[0] {
        GeometryNode::Transform { child, .. } => {
            match &**child {
                GeometryNode::Transform { child: rotate_child, .. } => {
                    match &**rotate_child {
                        GeometryNode::Cube { .. } => {}
                        _ => panic!("innermost node must be cube"),
                    }
                }
                _ => panic!("translate should wrap rotate transform"),
            }
        }
        _ => panic!("outer node must be translate transform"),
    }
}

/// Scaling occurs about the origin; verify the matrix diagonal reflects the requested factors
/// and no translation sneaks in when the child is non-centered.
#[test]
fn evaluate_scale_matrix_matches_requested_factors() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("scale([2,3,4]) cube(1);")
        .expect("scale parsed");

    match &nodes[0] {
        GeometryNode::Transform { matrix, .. } => {
            let cols = matrix.to_cols_array_2d();
            assert_eq!(cols[0][0], 2.0);
            assert_eq!(cols[1][1], 3.0);
            assert_eq!(cols[2][2], 4.0);
            // Translation column should remain zero because scale anchors at origin.
            assert_eq!(cols[3][0], 0.0);
            assert_eq!(cols[3][1], 0.0);
            assert_eq!(cols[3][2], 0.0);
        }
        _ => panic!("scale should yield transform node"),
    }
}

/// Confirms per-call $fn overrides the global context value.
#[test]
fn evaluate_sphere_local_fn_overrides_context() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("$fn = 48; sphere(r=5, $fn=12);")
        .expect("sphere parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Sphere { segments, .. } => assert_eq!(*segments, 12),
        _ => panic!("expected sphere"),
    }
}

/// Ensures $fa/$fs defaults drive segment calculation when $fn == 0.
#[test]
fn evaluate_sphere_uses_fa_fs_when_fn_zero() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("$fn = 0; $fa = 90; $fs = 100; sphere(r=1);")
        .expect("sphere parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Sphere { segments, .. } => assert_eq!(*segments, 5),
        _ => panic!("expected sphere"),
    }
}

#[test]
fn evaluate_translate() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator.evaluate_source("translate([10,0,0]) cube(1);").expect("translate parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Transform { matrix, child, .. } => {
            // Translation matrix should be:
            // 1 0 0 10
            // 0 1 0 0
            // 0 0 1 0
            // 0 0 0 1
            let cols = matrix.to_cols_array_2d();
            // glam stores column-major.
            // col 3 is translation vector.
            assert_eq!(cols[3][0], 10.0);
            assert_eq!(cols[3][1], 0.0);
            assert_eq!(cols[3][2], 0.0);

            match &**child {
                GeometryNode::Cube { size, .. } => {
                    assert_eq!(size.x, 1.0);
                }
                _ => panic!("Expected Cube child"),
            }
        }
        _ => panic!("Expected Transform node"),
    }
}

/// Ensures AST diagnostics are surfaced as explicit evaluator errors.
///
/// # Examples
/// ```rust
/// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem, EvaluationError};
///
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// let result = evaluator.evaluate_source("cube([1, 2]);");
/// assert!(matches!(result, Err(EvaluationError::AstDiagnostics(_))));
/// ```
#[test]
fn evaluate_invalid_cube_vector_reports_ast_error() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let err = evaluator
        .evaluate_source("cube([1, 2]);")
        .expect_err("evaluation should fail");

    if let EvaluationError::AstDiagnostics(diagnostics) = err {
        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message().contains("3 elements"));
    } else {
        panic!("expected AST diagnostics error");
    }
}

#[test]
fn evaluate_assignment_does_not_error() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator.evaluate_source("$fn = 50;").expect("assignment parsed");
    assert!(nodes.is_empty());
}

#[test]
fn evaluate_multiple_assignments() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator.evaluate_source("$fn = 50; $fa = 1;").expect("assignments parsed");
    assert!(nodes.is_empty());
}

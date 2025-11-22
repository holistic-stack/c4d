
use super::*;
use crate::filesystem::InMemoryFilesystem;

/// Ensures cube named arguments map directly to evaluator geometry nodes.
///
/// # Examples
/// ```
/// use openscad_eval::{filesystem::InMemoryFilesystem, evaluator::Evaluator};
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// let nodes = evaluator.evaluate_source("cube(size=10, center=true);").unwrap();
/// assert_eq!(nodes.len(), 1);
/// ```
#[test]
fn evaluate_cube_named_center() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("cube(size=10, center=true);")
        .expect("cube parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Cube { size, center, .. } => {
            assert_eq!(*center, true);
            assert_eq!(size.x, 10.0);
        }
        _ => panic!("Expected Cube"),
    }
}

/// Confirms mixed positional + named args remain supported.
///
/// # Examples
/// ```
/// use openscad_eval::{filesystem::InMemoryFilesystem, evaluator::Evaluator};
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// let nodes = evaluator.evaluate_source("cube([1,2,3], center=false);").unwrap();
/// assert_eq!(nodes[0].span().start(), 0);
/// ```
#[test]
fn evaluate_cube_mixed_args() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("cube([1,2,3], center=false);")
        .expect("cube parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Cube { size, center, .. } => {
            assert_eq!(size.x, 1.0);
            assert_eq!(size.y, 2.0);
            assert_eq!(size.z, 3.0);
            assert!(!*center);
        }
        _ => panic!("Expected Cube"),
    }
}

/// Validates that sphere accepts radius as a named argument.
#[test]
fn evaluate_sphere_named_radius() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("sphere(r=10);")
        .expect("sphere parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Sphere { radius, .. } => {
            assert_eq!(*radius, 10.0);
        }
        _ => panic!("Expected Sphere"),
    }
}

/// Validates that diameter input is normalized into radius.
#[test]
fn evaluate_sphere_diameter_arg() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("sphere(d=20);")
        .expect("sphere parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Sphere { radius, .. } => {
            assert_eq!(*radius, 10.0);
        }
        _ => panic!("Expected Sphere"),
    }
}


use super::*;
use crate::filesystem::InMemoryFilesystem;

#[test]
fn evaluate_cube_named_center() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator.evaluate_source("cube(size=10, center=true);").expect("cube parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Cube { size, center, .. } => {
            assert_eq!(size.x, 10.0);
            assert_eq!(*center, true);
        }
        _ => panic!("Expected Cube"),
    }
}

#[test]
fn evaluate_cube_mixed_args() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator.evaluate_source("cube([1,2,3], center=false);").expect("cube parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Cube { size, center, .. } => {
            assert_eq!(size.x, 1.0);
            assert_eq!(*center, false);
        }
        _ => panic!("Expected Cube"),
    }
}

#[test]
fn evaluate_sphere_named_args() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator.evaluate_source("sphere(r=10);").expect("sphere parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Sphere { radius, .. } => {
            assert_eq!(*radius, 10.0);
        }
        _ => panic!("Expected Sphere"),
    }
}

#[test]
fn evaluate_sphere_diameter_arg() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator.evaluate_source("sphere(d=20);").expect("sphere parsed");
    assert_eq!(nodes.len(), 1);
    match &nodes[0] {
        GeometryNode::Sphere { radius, .. } => {
            assert_eq!(*radius, 10.0);
        }
        _ => panic!("Expected Sphere"),
    }
}

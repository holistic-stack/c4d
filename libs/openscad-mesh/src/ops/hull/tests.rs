//! # Hull Integration Tests
//!
//! Tests for convex hull operations.

use super::*;
use crate::primitives::cube::create_cube;
use crate::primitives::sphere::create_sphere;
use glam::DVec3;

#[test]
fn test_hull_single_cube() {
    let cube = create_cube(DVec3::splat(10.0), true).unwrap();
    let result = hull(&[&cube]).unwrap();
    
    // Hull of a cube is the cube itself
    assert_eq!(result.vertex_count(), 8);
    assert_eq!(result.triangle_count(), 12);
}

#[test]
fn test_hull_two_cubes() {
    let cube1 = create_cube(DVec3::splat(5.0), false).unwrap();
    let mut cube2 = create_cube(DVec3::splat(5.0), false).unwrap();
    cube2.translate(DVec3::new(10.0, 0.0, 0.0));
    
    let result = hull(&[&cube1, &cube2]).unwrap();
    
    // Hull should encompass both cubes
    let (min, max) = result.bounding_box();
    assert!(min.x <= 0.0);
    assert!(max.x >= 15.0);
}

#[test]
fn test_hull_sphere() {
    let sphere = create_sphere(5.0, 16).unwrap();
    let result = hull(&[&sphere]).unwrap();
    
    // Hull of a sphere approximation should have same vertices
    // (sphere vertices are already on convex hull)
    assert!(result.vertex_count() > 0);
    assert!(result.triangle_count() > 0);
}

#[test]
fn test_hull_empty_mesh() {
    let empty = Mesh::new();
    let result = hull(&[&empty]).unwrap();
    
    assert!(result.is_empty());
}

#[test]
fn test_hull_preserves_bounding_box() {
    let cube = create_cube(DVec3::new(10.0, 20.0, 30.0), false).unwrap();
    let (orig_min, orig_max) = cube.bounding_box();
    
    let result = hull(&[&cube]).unwrap();
    let (hull_min, hull_max) = result.bounding_box();
    
    // Bounding box should be approximately the same
    assert!((orig_min - hull_min).length() < 0.1);
    assert!((orig_max - hull_max).length() < 0.1);
}

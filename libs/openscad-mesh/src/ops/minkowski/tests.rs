//! # Minkowski Sum Tests
//!
//! Tests for Minkowski sum operations.

use super::*;
use crate::primitives::cube::create_cube;
use crate::primitives::sphere::create_sphere;
use glam::DVec3;

#[test]
fn test_minkowski_cube_cube() {
    let cube1 = create_cube(DVec3::splat(2.0), true).unwrap();
    let cube2 = create_cube(DVec3::splat(1.0), true).unwrap();
    
    let result = minkowski_sum(&cube1, &cube2).unwrap();
    
    // Minkowski sum of two cubes is a larger cube
    // Size should be 2 + 1 = 3 (centered, so -1.5 to 1.5)
    let (min, max) = result.bounding_box();
    assert!((min.x - (-1.5)).abs() < 0.1);
    assert!((max.x - 1.5).abs() < 0.1);
}

#[test]
fn test_minkowski_cube_sphere() {
    let cube = create_cube(DVec3::splat(4.0), true).unwrap();
    let sphere = create_sphere(1.0, 8).unwrap();
    
    let result = minkowski_sum(&cube, &sphere).unwrap();
    
    // Result should be larger than the cube
    let (min, max) = result.bounding_box();
    assert!(min.x < -2.0);
    assert!(max.x > 2.0);
}

#[test]
fn test_minkowski_empty_a() {
    let empty = Mesh::new();
    let cube = create_cube(DVec3::splat(2.0), true).unwrap();
    
    let result = minkowski_sum(&empty, &cube).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_minkowski_empty_b() {
    let cube = create_cube(DVec3::splat(2.0), true).unwrap();
    let empty = Mesh::new();
    
    let result = minkowski_sum(&cube, &empty).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_minkowski_multiple() {
    let cube1 = create_cube(DVec3::splat(2.0), true).unwrap();
    let cube2 = create_cube(DVec3::splat(1.0), true).unwrap();
    
    let result = minkowski(&[&cube1, &cube2]).unwrap();
    
    assert!(result.vertex_count() > 0);
    assert!(result.triangle_count() > 0);
}

#[test]
fn test_minkowski_single() {
    let cube = create_cube(DVec3::splat(2.0), true).unwrap();
    
    let result = minkowski(&[&cube]).unwrap();
    
    // Single mesh should return itself
    assert_eq!(result.vertex_count(), cube.vertex_count());
}

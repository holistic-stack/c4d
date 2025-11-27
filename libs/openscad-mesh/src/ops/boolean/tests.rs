//! # Boolean Operation Tests
//!
//! Comprehensive tests for CSG boolean operations.

use super::*;
use crate::primitives::cube::create_cube;
use glam::DVec3;

/// Creates a unit cube at origin for testing.
fn test_cube() -> Mesh {
    create_cube(DVec3::ONE, false).unwrap()
}

/// Creates a unit cube centered at origin for testing.
fn test_cube_centered() -> Mesh {
    create_cube(DVec3::ONE, true).unwrap()
}

/// Creates a cube at a specific position.
fn cube_at(pos: DVec3, size: f64) -> Mesh {
    let mut mesh = create_cube(DVec3::splat(size), true).unwrap();
    mesh.translate(pos);
    mesh
}

// =============================================================================
// UNION TESTS
// =============================================================================

#[test]
fn test_union_empty_a() {
    let a = Mesh::new();
    let b = test_cube();
    
    let result = union(&a, &b).unwrap();
    
    // Result should be B
    assert_eq!(result.vertex_count(), b.vertex_count());
}

#[test]
fn test_union_empty_b() {
    let a = test_cube();
    let b = Mesh::new();
    
    let result = union(&a, &b).unwrap();
    
    // Result should be A
    assert_eq!(result.vertex_count(), a.vertex_count());
}

#[test]
fn test_union_non_overlapping() {
    // Two cubes far apart
    let a = cube_at(DVec3::new(-5.0, 0.0, 0.0), 1.0);
    let b = cube_at(DVec3::new(5.0, 0.0, 0.0), 1.0);
    
    let result = union(&a, &b).unwrap();
    
    // Should have vertices from both cubes
    assert!(result.vertex_count() >= 16); // At least 8 + 8
    assert!(result.triangle_count() >= 24); // At least 12 + 12
}

#[test]
fn test_union_overlapping() {
    // Two overlapping cubes
    let a = cube_at(DVec3::ZERO, 2.0);
    let b = cube_at(DVec3::new(1.0, 0.0, 0.0), 2.0);
    
    let result = union(&a, &b).unwrap();
    
    // Result should have geometry
    assert!(result.vertex_count() > 0);
    assert!(result.triangle_count() > 0);
}

#[test]
fn test_union_identical() {
    // Two identical cubes
    let a = test_cube_centered();
    let b = test_cube_centered();
    
    let result = union(&a, &b).unwrap();
    
    // Result should have geometry (may have some artifacts)
    assert!(result.vertex_count() > 0);
}

// =============================================================================
// DIFFERENCE TESTS
// =============================================================================

#[test]
fn test_difference_empty_a() {
    let a = Mesh::new();
    let b = test_cube();
    
    let result = difference(&a, &b).unwrap();
    
    // Result should be empty
    assert_eq!(result.vertex_count(), 0);
}

#[test]
fn test_difference_empty_b() {
    let a = test_cube();
    let b = Mesh::new();
    
    let result = difference(&a, &b).unwrap();
    
    // Result should be A
    assert_eq!(result.vertex_count(), a.vertex_count());
}

#[test]
fn test_difference_non_overlapping() {
    // Two cubes far apart
    let a = cube_at(DVec3::new(-5.0, 0.0, 0.0), 1.0);
    let b = cube_at(DVec3::new(5.0, 0.0, 0.0), 1.0);
    
    let result = difference(&a, &b).unwrap();
    
    // Result should be A unchanged
    assert_eq!(result.vertex_count(), a.vertex_count());
}

#[test]
fn test_difference_overlapping() {
    // Larger cube minus smaller cube
    let a = cube_at(DVec3::ZERO, 4.0);
    let b = cube_at(DVec3::ZERO, 2.0);
    
    let result = difference(&a, &b).unwrap();
    
    // Result should have geometry (hollow cube)
    assert!(result.vertex_count() > 0);
    assert!(result.triangle_count() > 0);
}

// =============================================================================
// INTERSECTION TESTS
// =============================================================================

#[test]
fn test_intersection_empty_a() {
    let a = Mesh::new();
    let b = test_cube();
    
    let result = intersection(&a, &b).unwrap();
    
    // Result should be empty
    assert_eq!(result.vertex_count(), 0);
}

#[test]
fn test_intersection_empty_b() {
    let a = test_cube();
    let b = Mesh::new();
    
    let result = intersection(&a, &b).unwrap();
    
    // Result should be empty
    assert_eq!(result.vertex_count(), 0);
}

#[test]
fn test_intersection_non_overlapping() {
    // Two cubes far apart
    let a = cube_at(DVec3::new(-5.0, 0.0, 0.0), 1.0);
    let b = cube_at(DVec3::new(5.0, 0.0, 0.0), 1.0);
    
    let result = intersection(&a, &b).unwrap();
    
    // Result should be empty
    assert_eq!(result.vertex_count(), 0);
}

#[test]
fn test_intersection_overlapping() {
    // Two overlapping cubes
    let a = cube_at(DVec3::ZERO, 2.0);
    let b = cube_at(DVec3::new(0.5, 0.0, 0.0), 2.0);
    
    let result = intersection(&a, &b).unwrap();
    
    // Result should have geometry (the overlap region)
    assert!(result.vertex_count() > 0);
    assert!(result.triangle_count() > 0);
}

#[test]
fn test_intersection_contained() {
    // Small cube inside large cube
    let a = cube_at(DVec3::ZERO, 4.0);
    let b = cube_at(DVec3::ZERO, 2.0);
    
    let result = intersection(&a, &b).unwrap();
    
    // Result should be approximately the smaller cube
    assert!(result.vertex_count() > 0);
}

// =============================================================================
// BOUNDING BOX TESTS
// =============================================================================

#[test]
fn test_bounding_boxes_overlap_true() {
    let a = cube_at(DVec3::ZERO, 2.0);
    let b = cube_at(DVec3::new(1.0, 0.0, 0.0), 2.0);
    
    assert!(bounding_boxes_overlap(&a, &b));
}

#[test]
fn test_bounding_boxes_overlap_false() {
    let a = cube_at(DVec3::new(-5.0, 0.0, 0.0), 1.0);
    let b = cube_at(DVec3::new(5.0, 0.0, 0.0), 1.0);
    
    assert!(!bounding_boxes_overlap(&a, &b));
}

#[test]
fn test_bounding_boxes_overlap_touching() {
    // Cubes just touching at edge
    let a = cube_at(DVec3::ZERO, 2.0);
    let b = cube_at(DVec3::new(2.0, 0.0, 0.0), 2.0);
    
    // Should be considered overlapping (or just touching)
    assert!(bounding_boxes_overlap(&a, &b));
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[test]
fn test_boolean_chain() {
    // (A ∪ B) - C
    let a = cube_at(DVec3::ZERO, 2.0);
    let b = cube_at(DVec3::new(1.0, 0.0, 0.0), 2.0);
    let c = cube_at(DVec3::new(0.5, 0.0, 0.0), 1.0);
    
    let ab = union(&a, &b).unwrap();
    let result = difference(&ab, &c).unwrap();
    
    assert!(result.vertex_count() > 0);
}

#[test]
fn test_mesh_to_polygons_roundtrip() {
    let mesh = test_cube();
    let polys = mesh_to_polygons(&mesh);
    let result = polygons_to_mesh(&polys).unwrap();
    
    // Should have same triangle count
    assert_eq!(result.triangle_count(), mesh.triangle_count());
}

// =============================================================================
// HIGH-RESOLUTION STRESS TESTS
// =============================================================================

/// Tests that high-resolution boolean operations don't cause stack overflow.
/// This simulates the user's OpenSCAD code with $fs=0.1, $fa=5.
#[test]
fn test_high_resolution_boolean_no_stack_overflow() {
    use crate::primitives::sphere::create_sphere;
    use crate::primitives::cylinder::create_cylinder;
    
    // High resolution (72 segments simulating $fa=5)
    let segments = 72;
    
    // Create sphere and cube (intersection)
    let sphere = create_sphere(10.0, segments).unwrap();
    let cube = create_cube(DVec3::splat(15.0), true).unwrap();
    
    // Create cylinders for holes
    let cyl1 = create_cylinder(20.0, 5.0, 5.0, true, segments).unwrap();
    let mut cyl2 = create_cylinder(20.0, 5.0, 5.0, true, segments).unwrap();
    let mut cyl3 = create_cylinder(20.0, 5.0, 5.0, true, segments).unwrap();
    
    // Rotate cylinders (simplified - just translate for test)
    cyl2.translate(DVec3::new(0.1, 0.0, 0.0)); // Slight offset to simulate rotation
    cyl3.translate(DVec3::new(0.0, 0.1, 0.0)); // Slight offset to simulate rotation
    
    // intersection(sphere, cube)
    let intersected = intersection(&sphere, &cube).unwrap();
    assert!(intersected.vertex_count() > 0, "Intersection should produce geometry");
    
    // union(cyl1, cyl2, cyl3)
    let union1 = union(&cyl1, &cyl2).unwrap();
    let holes = union(&union1, &cyl3).unwrap();
    assert!(holes.vertex_count() > 0, "Union of cylinders should produce geometry");
    
    // difference(intersected, holes)
    let result = difference(&intersected, &holes).unwrap();
    
    // The result should have geometry (not empty)
    assert!(result.vertex_count() > 0, "Final difference should produce geometry");
    assert!(result.triangle_count() > 0, "Final difference should have triangles");
    
    println!("High-res test: {} vertices, {} triangles", 
             result.vertex_count(), result.triangle_count());
}

/// Tests deeply nested boolean operations with high-resolution meshes.
#[test]
fn test_deeply_nested_booleans() {
    // Create 5 cubes in a row
    let cubes: Vec<Mesh> = (0..5)
        .map(|i| cube_at(DVec3::new(i as f64 * 0.8, 0.0, 0.0), 1.0))
        .collect();
    
    // Union all cubes sequentially
    let mut result = cubes[0].clone();
    for cube in &cubes[1..] {
        result = union(&result, cube).unwrap();
    }
    
    // Should have combined geometry
    assert!(result.vertex_count() > 0);
    assert!(result.triangle_count() > 0);
}

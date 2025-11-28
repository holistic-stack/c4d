//! # Boolean Operations Tests
//!
//! Comprehensive test suite for CSG boolean operations.
//!
//! ## Test Categories
//!
//! - **Unit tests**: Individual function tests (in respective modules)
//! - **Integration tests**: Full boolean operations (this file)
//! - **Regression tests**: Specific bug fixes and performance bounds
//!
//! ## Test Naming Convention
//!
//! - `test_<operation>_<geometry>`: Basic operation tests
//! - `test_<operation>_<geometry>_<condition>`: Edge case tests
//! - `test_regression_<issue>`: Regression tests for specific issues

use super::*;
use crate::manifold::constructors::{build_cube, build_sphere};
use crate::mesh::Mesh;

// =============================================================================
// UNION TESTS
// =============================================================================

/// Test union of two overlapping cubes.
///
/// ```text
///   +---+
///   |   +---+
///   +---|   |
///       +---+
/// ```
#[test]
fn test_union_cubes() {
    let mut mesh1 = Mesh::new();
    build_cube(&mut mesh1, [10.0, 10.0, 10.0], true);
    
    let mut mesh2 = Mesh::new();
    build_cube(&mut mesh2, [10.0, 10.0, 10.0], true);
    mesh2.translate(5.0, 0.0, 0.0);
    
    let result = union_all(&[mesh1, mesh2]).unwrap();
    
    assert!(!result.is_empty(), "Union should produce non-empty mesh");
    assert!(result.triangle_count() >= 12, "Union should have at least cube faces");
}

/// Test union of non-overlapping cubes.
#[test]
fn test_union_disjoint_cubes() {
    let mut mesh1 = Mesh::new();
    build_cube(&mut mesh1, [5.0, 5.0, 5.0], true);
    mesh1.translate(-10.0, 0.0, 0.0);
    
    let mut mesh2 = Mesh::new();
    build_cube(&mut mesh2, [5.0, 5.0, 5.0], true);
    mesh2.translate(10.0, 0.0, 0.0);
    
    let result = union_all(&[mesh1, mesh2]).unwrap();
    
    assert!(!result.is_empty());
    // Disjoint cubes should have 24 triangles (12 each)
    assert!(result.triangle_count() >= 24);
}

// =============================================================================
// DIFFERENCE TESTS
// =============================================================================

/// Test difference of two cubes (carving).
///
/// ```text
///   +-------+
///   |  +-+  |  →  +-------+
///   |  +-+  |     |  [ ]  |
///   +-------+     +-------+
/// ```
#[test]
fn test_difference_cubes() {
    let mut mesh1 = Mesh::new();
    build_cube(&mut mesh1, [10.0, 10.0, 10.0], true);
    
    let mut mesh2 = Mesh::new();
    build_cube(&mut mesh2, [5.0, 5.0, 5.0], true);
    
    let result = difference_all(&[mesh1, mesh2]).unwrap();
    
    assert!(!result.is_empty(), "Difference should produce non-empty mesh");
    assert!(result.triangle_count() > 12, "Carved cube should have more faces");
}

/// Test difference where subtracted mesh is outside (no effect).
#[test]
fn test_difference_no_overlap() {
    let mut mesh1 = Mesh::new();
    build_cube(&mut mesh1, [5.0, 5.0, 5.0], true);
    
    let mut mesh2 = Mesh::new();
    build_cube(&mut mesh2, [5.0, 5.0, 5.0], true);
    mesh2.translate(100.0, 0.0, 0.0); // Far away
    
    let result = difference_all(&[mesh1, mesh2]).unwrap();
    
    assert!(!result.is_empty());
    // Should be approximately original cube
    assert!(result.triangle_count() >= 12);
}

// =============================================================================
// INTERSECTION TESTS
// =============================================================================

/// Test intersection of two overlapping cubes.
///
/// ```text
///   +---+           
///   |   +---+  →  +-+
///   +---|   |     +-+
///       +---+
/// ```
#[test]
fn test_intersection_cubes() {
    let mut mesh1 = Mesh::new();
    build_cube(&mut mesh1, [10.0, 10.0, 10.0], true);
    
    let mut mesh2 = Mesh::new();
    build_cube(&mut mesh2, [10.0, 10.0, 10.0], true);
    mesh2.translate(5.0, 0.0, 0.0);
    
    let result = intersection_all(&[mesh1, mesh2]).unwrap();
    
    assert!(!result.is_empty(), "Intersection should produce non-empty mesh");
}

/// Test intersection of non-overlapping cubes (empty result).
#[test]
fn test_intersection_no_overlap() {
    let mut mesh1 = Mesh::new();
    build_cube(&mut mesh1, [5.0, 5.0, 5.0], true);
    mesh1.translate(-10.0, 0.0, 0.0);
    
    let mut mesh2 = Mesh::new();
    build_cube(&mut mesh2, [5.0, 5.0, 5.0], true);
    mesh2.translate(10.0, 0.0, 0.0);
    
    let result = intersection_all(&[mesh1, mesh2]).unwrap();
    
    // Disjoint meshes should have empty intersection
    assert!(result.is_empty() || result.triangle_count() == 0);
}

// =============================================================================
// CUBE + SPHERE TESTS
// =============================================================================

/// Test intersection of cube and sphere.
///
/// Creates a "spherical cube" - sphere with flat faces.
#[test]
fn test_intersection_cube_sphere() {
    let mut cube_mesh = Mesh::new();
    build_cube(&mut cube_mesh, [15.0, 15.0, 15.0], true);
    
    let mut sphere_mesh = Mesh::new();
    build_sphere(&mut sphere_mesh, 10.0, 16);
    
    let result = intersection_all(&[cube_mesh, sphere_mesh]).unwrap();
    
    assert!(!result.is_empty(), "Intersection should produce non-empty mesh");
    assert!(result.triangle_count() > 12, "Expected detailed intersection");
}

/// Test intersection where sphere is entirely inside cube.
#[test]
fn test_intersection_cube_contains_sphere() {
    let mut cube_mesh = Mesh::new();
    build_cube(&mut cube_mesh, [20.0, 20.0, 20.0], true);
    
    let mut sphere_mesh = Mesh::new();
    build_sphere(&mut sphere_mesh, 5.0, 16);
    
    let result = intersection_all(&[cube_mesh, sphere_mesh]).unwrap();
    
    assert!(!result.is_empty());
    assert!(result.triangle_count() > 50, "Expected full sphere triangles");
}

/// Test difference of cube minus sphere (carving a spherical hole).
#[test]
fn test_difference_cube_sphere() {
    let mut cube_mesh = Mesh::new();
    build_cube(&mut cube_mesh, [15.0, 15.0, 15.0], true);
    
    let mut sphere_mesh = Mesh::new();
    build_sphere(&mut sphere_mesh, 10.0, 16);
    
    let result = difference_all(&[cube_mesh, sphere_mesh]).unwrap();
    
    assert!(!result.is_empty(), "Difference should produce non-empty mesh");
    assert!(result.triangle_count() > 12, "Expected detailed difference");
    
    // Verify cube corners remain (distance > sphere radius)
    let has_far_vertices = result.vertices.chunks(6).any(|v| {
        let dist = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
        dist > 10.1
    });
    assert!(has_far_vertices, "Should keep cube corners");
}

// =============================================================================
// EDGE CASES
// =============================================================================

/// Test empty input.
#[test]
fn test_empty_input() {
    let result = union_all(&[]).unwrap();
    assert!(result.is_empty());
    
    let result = difference_all(&[]).unwrap();
    assert!(result.is_empty());
    
    let result = intersection_all(&[]).unwrap();
    assert!(result.is_empty());
}

/// Test single mesh input (identity operation).
#[test]
fn test_single_mesh() {
    let mut mesh = Mesh::new();
    build_cube(&mut mesh, [10.0, 10.0, 10.0], true);
    let original_count = mesh.triangle_count();
    
    let result = union_all(&[mesh.clone()]).unwrap();
    assert_eq!(result.triangle_count(), original_count);
    
    let result = difference_all(&[mesh.clone()]).unwrap();
    assert_eq!(result.triangle_count(), original_count);
    
    let result = intersection_all(&[mesh]).unwrap();
    assert_eq!(result.triangle_count(), original_count);
}

// =============================================================================
// REGRESSION TESTS
// =============================================================================

/// Regression test for union mesh optimization.
///
/// ## OpenSCAD Reference
/// 
/// `union() { cube(15, center=true); sphere(10); }`
/// OpenSCAD with Manifold backend: 506 vertices, 1008 facets
///
/// ## BSP Limitations
///
/// Our BSP-based implementation produces more triangles than Manifold's
/// edge-intersection algorithm because BSP splits triangles along arbitrary
/// planes rather than computing exact intersection curves.
///
/// ## Performance Bounds
///
/// - BSP: ~600-650 vertices, ~1400-1500 triangles (~44% overhead vs Manifold)
/// - With polygon merging: reduced from 2000+ triangles
///
/// ## Future Work
///
/// Implement Manifold-style edge-edge intersection algorithm for 100% compatibility.
#[test]
fn test_regression_union_cube_sphere_optimization() {
    use crate::openscad::SegmentParams;
    
    let mut cube_mesh = Mesh::new();
    build_cube(&mut cube_mesh, [15.0, 15.0, 15.0], true);
    
    // OpenSCAD default for sphere(10): ceil(2*pi*10/2) = 32 segments
    let params = SegmentParams::default();
    let segments = params.calculate_segments(10.0);
    
    let mut sphere_mesh = Mesh::new();
    build_sphere(&mut sphere_mesh, 10.0, segments);
    
    let result = union_all(&[cube_mesh, sphere_mesh]).unwrap();
    
    let vertex_count = result.vertices.len() / 6;
    let triangle_count = result.triangle_count();
    
    assert!(!result.is_empty(), "Union should produce non-empty mesh");
    
    // BSP produces more geometry than Manifold (~44% overhead)
    // OpenSCAD target: 506 vertices, 1008 triangles
    // BSP limit: ~1.5x overhead allowed
    assert!(
        vertex_count < 800,
        "Vertex count {} exceeds BSP limit (OpenSCAD: 506, BSP max: 800)",
        vertex_count
    );
    
    assert!(
        triangle_count < 1600,
        "Triangle count {} exceeds BSP limit (OpenSCAD: 1008, BSP max: 1600)",
        triangle_count
    );
    
    // Verify geometry correctness
    
    // Sphere bulge: vertices beyond cube face (7.5) up to sphere radius (10)
    let has_bulge = result.vertices.chunks(6).any(|v| {
        v[0].abs() > 7.6 || v[1].abs() > 7.6 || v[2].abs() > 7.6
    });
    assert!(has_bulge, "Should have sphere bulge vertices");
    
    // Cube corners: vertices at ~13 distance (beyond sphere radius)
    let has_corners = result.vertices.chunks(6).any(|v| {
        let dist = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
        dist > 12.0
    });
    assert!(has_corners, "Should have cube corner vertices");
}

/// Test that multiple sequential operations work correctly.
#[test]
fn test_sequential_operations() {
    let mut cube1 = Mesh::new();
    build_cube(&mut cube1, [10.0, 10.0, 10.0], true);
    
    let mut cube2 = Mesh::new();
    build_cube(&mut cube2, [10.0, 10.0, 10.0], true);
    cube2.translate(5.0, 0.0, 0.0);
    
    let mut cube3 = Mesh::new();
    build_cube(&mut cube3, [5.0, 5.0, 5.0], true);
    
    // Union then difference: (cube1 ∪ cube2) - cube3
    let union_result = union_all(&[cube1.clone(), cube2.clone()]).unwrap();
    let final_result = difference_all(&[union_result, cube3]).unwrap();
    
    assert!(!final_result.is_empty());
    assert!(final_result.triangle_count() > 12);
}

/// Test three-way operations.
#[test]
fn test_three_way_union() {
    let mut cube1 = Mesh::new();
    build_cube(&mut cube1, [5.0, 5.0, 5.0], true);
    cube1.translate(-5.0, 0.0, 0.0);
    
    let mut cube2 = Mesh::new();
    build_cube(&mut cube2, [5.0, 5.0, 5.0], true);
    
    let mut cube3 = Mesh::new();
    build_cube(&mut cube3, [5.0, 5.0, 5.0], true);
    cube3.translate(5.0, 0.0, 0.0);
    
    let result = union_all(&[cube1, cube2, cube3]).unwrap();
    
    assert!(!result.is_empty());
    assert!(result.triangle_count() >= 12);
}

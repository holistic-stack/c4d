//! # Minkowski Sum
//!
//! Computes the Minkowski sum of multiple meshes using vertex-sum + hull.
//!
//! ## Algorithm
//!
//! For convex shapes A and B:
//! `A ⊕ B = { a + b : a ∈ A, b ∈ B }`
//!
//! Implementation:
//! 1. Collect all vertices from both meshes
//! 2. Compute pairwise sums
//! 3. Take convex hull of result
//!
//! Note: This is exact for convex inputs, approximate for non-convex.

use crate::error::ManifoldResult;
use crate::mesh::Mesh;
use super::hull::compute_hull;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Compute Minkowski sum of multiple meshes.
///
/// Takes all vertices and computes pairwise sums, then convex hull.
///
/// ## Parameters
///
/// - `meshes`: Slice of meshes (at least 2 for meaningful result)
///
/// ## Returns
///
/// Minkowski sum mesh
///
/// ## Example
///
/// ```rust
/// use manifold_rs::mesh::Mesh;
/// use manifold_rs::manifold::minkowski::compute_minkowski;
/// use manifold_rs::manifold::constructors::build_cube;
///
/// let mut cube1 = Mesh::new();
/// build_cube(&mut cube1, [10.0, 10.0, 10.0], true);
///
/// let mut cube2 = Mesh::new();
/// build_cube(&mut cube2, [2.0, 2.0, 2.0], true);
///
/// let result = compute_minkowski(&[cube1, cube2]).unwrap();
/// // Minkowski of two centered cubes produces a larger centered cube
/// assert!(!result.is_empty());
/// ```
pub fn compute_minkowski(meshes: &[Mesh]) -> ManifoldResult<Mesh> {
    if meshes.is_empty() {
        return Ok(Mesh::new());
    }
    
    if meshes.len() == 1 {
        return Ok(meshes[0].clone());
    }
    
    // Start with first mesh vertices
    let mut current_points: Vec<[f32; 3]> = Vec::new();
    for i in (0..meshes[0].vertices.len()).step_by(3) {
        current_points.push([
            meshes[0].vertices[i],
            meshes[0].vertices[i + 1],
            meshes[0].vertices[i + 2],
        ]);
    }
    
    // Add each subsequent mesh via pairwise sums
    for mesh in &meshes[1..] {
        let mut next_points: Vec<[f32; 3]> = Vec::new();
        
        // Collect vertices from current mesh
        let mut mesh_points: Vec<[f32; 3]> = Vec::new();
        for i in (0..mesh.vertices.len()).step_by(3) {
            mesh_points.push([
                mesh.vertices[i],
                mesh.vertices[i + 1],
                mesh.vertices[i + 2],
            ]);
        }
        
        // Compute pairwise sums
        for p1 in &current_points {
            for p2 in &mesh_points {
                next_points.push([
                    p1[0] + p2[0],
                    p1[1] + p2[1],
                    p1[2] + p2[2],
                ]);
            }
        }
        
        current_points = next_points;
    }
    
    // Convert points to mesh for hull computation
    let mut point_mesh = Mesh::new();
    for p in &current_points {
        point_mesh.add_vertex(p[0], p[1], p[2], 0.0, 0.0, 1.0);
    }
    
    // Take convex hull
    compute_hull(&[point_mesh])
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifold::constructors::build_cube;

    /// Test Minkowski sum of two cubes.
    ///
    /// Minkowski of cube(10, center=true) + cube(2, center=true) should produce
    /// a cube of size 12 centered at origin.
    #[test]
    fn test_minkowski_cubes() {
        let mut cube1 = Mesh::new();
        build_cube(&mut cube1, [10.0, 10.0, 10.0], true);
        
        let mut cube2 = Mesh::new();
        build_cube(&mut cube2, [2.0, 2.0, 2.0], true);
        
        let result = compute_minkowski(&[cube1, cube2]).unwrap();
        assert!(!result.is_empty(), "Minkowski result should not be empty");
        // Result should be cube-like (8 vertices for convex hull of combined corners)
        assert!(result.triangle_count() >= 4, "Should have at least 4 triangles");
    }

    /// Test Minkowski with single mesh.
    #[test]
    fn test_minkowski_single() {
        let mut cube = Mesh::new();
        build_cube(&mut cube, [10.0, 10.0, 10.0], true);
        
        let result = compute_minkowski(&[cube]).unwrap();
        assert!(!result.is_empty());
    }

    /// Test Minkowski with empty input.
    #[test]
    fn test_minkowski_empty() {
        let result = compute_minkowski(&[]).unwrap();
        assert!(result.is_empty());
    }
}

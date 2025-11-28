//! # Minkowski Sum
//!
//! Computes the Minkowski sum of two meshes.
//!
//! ## Algorithm Overview
//!
//! The Minkowski sum of shapes A and B is the set of all points A + B,
//! where A is a point from shape A and B is a point from shape B.
//!
//! For convex shapes, this is computed by:
//! 1. Collecting all vertices from both meshes
//! 2. Computing all pairwise vertex sums
//! 3. Computing the convex hull of the resulting points
//!
//! For non-convex shapes, this provides an approximation that works well
//! for the common OpenSCAD use case of "rounding" shapes.
//!
//! ## Example
//!
//! ```rust,ignore
//! use openscad_mesh::ops::minkowski::minkowski_sum;
//! use openscad_mesh::Mesh;
//!
//! let cube = create_cube_mesh();
//! let sphere = create_sphere_mesh();
//! let result = minkowski_sum(&cube, &sphere);
//! // Result is a cube with rounded edges
//! ```

use crate::Mesh;
use super::hull::convex_hull;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Compute the Minkowski sum of two meshes.
///
/// This approximates the Minkowski sum by computing all pairwise vertex sums
/// and taking their convex hull. This is exact for convex inputs and provides
/// a reasonable approximation for non-convex shapes.
///
/// ## Parameters
///
/// - `a`: First mesh (base shape)
/// - `b`: Second mesh (inflate shape)
///
/// ## Returns
///
/// A mesh representing the Minkowski sum.
///
/// ## Example
///
/// ```rust,ignore
/// // Create a rounded cube
/// let cube = create_cube();
/// let sphere = create_small_sphere();
/// let rounded_cube = minkowski_sum(&cube, &sphere);
/// ```
pub fn minkowski_sum(a: &Mesh, b: &Mesh) -> Mesh {
    if a.is_empty() {
        return b.clone();
    }
    if b.is_empty() {
        return a.clone();
    }

    // Extract vertices from both meshes
    let vertices_a = extract_vertices(a);
    let vertices_b = extract_vertices(b);

    // Compute all pairwise sums
    let mut sum_points: Vec<[f32; 3]> = Vec::with_capacity(vertices_a.len() * vertices_b.len());
    
    for va in &vertices_a {
        for vb in &vertices_b {
            sum_points.push([
                va[0] + vb[0],
                va[1] + vb[1],
                va[2] + vb[2],
            ]);
        }
    }

    // Deduplicate points (important for performance)
    let unique_points = deduplicate_points(&sum_points, 1e-6);

    // Compute convex hull of all sum points
    convex_hull(&unique_points)
}

// =============================================================================
// HELPERS
// =============================================================================

/// Extract unique vertices from a mesh.
fn extract_vertices(mesh: &Mesh) -> Vec<[f32; 3]> {
    let mut vertices = Vec::new();
    
    for i in (0..mesh.vertices.len()).step_by(3) {
        vertices.push([
            mesh.vertices[i],
            mesh.vertices[i + 1],
            mesh.vertices[i + 2],
        ]);
    }
    
    // Deduplicate vertices
    deduplicate_points(&vertices, 1e-6)
}

/// Deduplicate points within a tolerance.
fn deduplicate_points(points: &[[f32; 3]], tolerance: f32) -> Vec<[f32; 3]> {
    use std::collections::HashMap;
    
    // Quantize points to grid cells
    let inv_tol = 1.0 / tolerance;
    let mut seen: HashMap<(i64, i64, i64), usize> = HashMap::new();
    let mut result = Vec::new();
    
    for point in points {
        let key = (
            (point[0] * inv_tol).round() as i64,
            (point[1] * inv_tol).round() as i64,
            (point[2] * inv_tol).round() as i64,
        );
        
        if !seen.contains_key(&key) {
            seen.insert(key, result.len());
            result.push(*point);
        }
    }
    
    result
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a simple cube mesh for testing.
    fn create_test_cube(size: f32) -> Mesh {
        let mut mesh = Mesh::new();
        let s = size / 2.0;
        
        // Just add the 8 corner vertices for testing
        // Front face
        let v0 = mesh.add_vertex(-s, -s, -s, 0.0, 0.0, -1.0);
        let v1 = mesh.add_vertex(s, -s, -s, 0.0, 0.0, -1.0);
        let v2 = mesh.add_vertex(s, s, -s, 0.0, 0.0, -1.0);
        let v3 = mesh.add_vertex(-s, s, -s, 0.0, 0.0, -1.0);
        mesh.add_triangle(v0, v1, v2);
        mesh.add_triangle(v0, v2, v3);
        
        // Back face
        let v4 = mesh.add_vertex(-s, -s, s, 0.0, 0.0, 1.0);
        let v5 = mesh.add_vertex(s, -s, s, 0.0, 0.0, 1.0);
        let v6 = mesh.add_vertex(s, s, s, 0.0, 0.0, 1.0);
        let v7 = mesh.add_vertex(-s, s, s, 0.0, 0.0, 1.0);
        mesh.add_triangle(v4, v6, v5);
        mesh.add_triangle(v4, v7, v6);
        
        mesh
    }

    /// Test Minkowski sum of two cubes.
    #[test]
    fn test_minkowski_two_cubes() {
        let cube1 = create_test_cube(2.0);
        let cube2 = create_test_cube(1.0);
        
        let result = minkowski_sum(&cube1, &cube2);
        
        // Result should not be empty
        assert!(!result.is_empty());
        // Result should have triangles
        assert!(result.triangle_count() > 0);
    }

    /// Test Minkowski with empty mesh.
    #[test]
    fn test_minkowski_empty() {
        let cube = create_test_cube(2.0);
        let empty = Mesh::new();
        
        let result = minkowski_sum(&cube, &empty);
        assert_eq!(result.vertex_count(), cube.vertex_count());
        
        let result2 = minkowski_sum(&empty, &cube);
        assert_eq!(result2.vertex_count(), cube.vertex_count());
    }

    /// Test point deduplication.
    #[test]
    fn test_deduplicate_points() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],  // Duplicate
            [1.0, 0.0, 0.0],
            [1.0, 0.00001, 0.0],  // Near duplicate
            [2.0, 0.0, 0.0],
        ];
        
        let unique = deduplicate_points(&points, 0.001);
        assert_eq!(unique.len(), 3);  // Should have 3 unique points
    }
}

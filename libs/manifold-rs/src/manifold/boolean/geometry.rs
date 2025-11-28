//! # Geometry Utilities
//!
//! Low-level geometric operations for boolean operations.
//!
//! ## Contents
//!
//! - **Vector math**: `dot`, `cross`, `normalize`
//! - **Ray casting**: `ray_triangle_intersect`, `point_inside_mesh`
//! - **Distance**: `point_to_triangle_distance`, `polygon_centroid`
//!
//! ## Design Principles
//!
//! - **KISS**: Simple, focused functions with clear inputs/outputs
//! - **DRY**: Reusable primitives used by multiple modules
//! - **SRP**: Only geometry calculations, no mesh/BSP logic

use crate::mesh::Mesh;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Tolerance for floating-point comparisons.
///
/// Used in ray-triangle intersection to avoid numerical issues.
pub const EPSILON: f32 = 1e-5;

/// Stricter tolerance for ray intersection.
const RAY_EPSILON: f32 = 1e-7;

// =============================================================================
// VECTOR MATH
// =============================================================================

/// Compute dot product of two 3D vectors.
///
/// ## Example
///
/// ```ignore
/// let a = [1.0, 0.0, 0.0];
/// let b = [0.0, 1.0, 0.0];
/// assert_eq!(dot(&a, &b), 0.0); // Perpendicular
/// ```
#[inline]
pub fn dot(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Compute cross product of two 3D vectors.
///
/// Returns vector perpendicular to both inputs (right-hand rule).
///
/// ## Example
///
/// ```ignore
/// let x = [1.0, 0.0, 0.0];
/// let y = [0.0, 1.0, 0.0];
/// let z = cross(&x, &y);
/// assert_eq!(z, [0.0, 0.0, 1.0]);
/// ```
#[inline]
pub fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Compute length of a 3D vector.
#[inline]
pub fn length(v: &[f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Normalize a 3D vector to unit length.
///
/// Returns `[0, 0, 1]` for zero-length vectors (safe fallback).
#[inline]
pub fn normalize(v: &[f32; 3]) -> [f32; 3] {
    let len = length(v);
    if len > 1e-9 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 1.0] // Safe default for degenerate cases
    }
}

// =============================================================================
// RAY CASTING
// =============================================================================

/// Möller–Trumbore ray-triangle intersection algorithm.
///
/// Tests if a ray from `origin` in direction `dir` intersects the triangle
/// defined by vertices `v0`, `v1`, `v2`.
///
/// ## Algorithm
///
/// Uses barycentric coordinates to efficiently test intersection without
/// computing the intersection point explicitly.
///
/// ## Parameters
///
/// - `origin`: Ray starting point
/// - `dir`: Ray direction (should be normalized for consistent results)
/// - `v0`, `v1`, `v2`: Triangle vertices in counter-clockwise order
///
/// ## Returns
///
/// `true` if ray intersects triangle in positive direction
///
/// ## Reference
///
/// Möller, T., & Trumbore, B. (1997). Fast, minimum storage ray-triangle intersection.
pub fn ray_triangle_intersect(
    origin: &[f32; 3],
    dir: &[f32; 3],
    v0: &[f32; 3],
    v1: &[f32; 3],
    v2: &[f32; 3],
) -> bool {
    let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    
    let h = cross(dir, &edge2);
    let a = dot(&edge1, &h);
    
    // Ray parallel to triangle
    if a.abs() < RAY_EPSILON {
        return false;
    }
    
    let f = 1.0 / a;
    let s = [origin[0] - v0[0], origin[1] - v0[1], origin[2] - v0[2]];
    let u = f * dot(&s, &h);
    
    // Outside triangle (u coordinate)
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    
    let q = cross(&s, &edge1);
    let v = f * dot(dir, &q);
    
    // Outside triangle (v coordinate or u+v > 1)
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    
    let t = f * dot(&edge2, &q);
    
    // Intersection in positive direction only
    t > RAY_EPSILON
}

/// Test if point is inside closed mesh using robust ray casting.
///
/// ## Algorithm
///
/// Casts rays in 6 cardinal directions (±X, ±Y, ±Z) and counts intersections.
/// Uses voting to handle edge cases where a ray might graze an edge/vertex.
///
/// A point is inside if the majority of rays (≥3 of 6) indicate odd intersection count.
///
/// ## Parameters
///
/// - `point`: Point to test
/// - `mesh`: Closed mesh to test against
///
/// ## Returns
///
/// `true` if point is inside the mesh
///
/// ## Example
///
/// ```ignore
/// let center = [0.0, 0.0, 0.0];
/// let inside = point_inside_mesh(&center, &unit_cube);
/// ```
pub fn point_inside_mesh(point: &[f32; 3], mesh: &Mesh) -> bool {
    // Cast rays in 6 cardinal directions for robustness
    const DIRS: [[f32; 3]; 6] = [
        [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0], [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0], [0.0, 0.0, -1.0],
    ];
    
    let mut inside_votes = 0;
    
    for dir in &DIRS {
        let count = count_ray_intersections(point, dir, mesh);
        
        // Odd count = inside
        if count % 2 == 1 {
            inside_votes += 1;
        }
    }
    
    // Majority voting (3+ of 6)
    inside_votes >= 3
}

/// Count ray-mesh intersections for a single ray direction.
fn count_ray_intersections(origin: &[f32; 3], dir: &[f32; 3], mesh: &Mesh) -> usize {
    let mut count = 0;
    
    for i in (0..mesh.indices.len()).step_by(3) {
        let (v0, v1, v2) = get_triangle_vertices(mesh, i);
        
        if ray_triangle_intersect(origin, dir, &v0, &v1, &v2) {
            count += 1;
        }
    }
    
    count
}

// =============================================================================
// TRIANGLE/MESH HELPERS
// =============================================================================

/// Extract triangle vertices from mesh by index offset.
///
/// ## Parameters
///
/// - `mesh`: Source mesh
/// - `idx_offset`: Starting index in `mesh.indices` (must be multiple of 3)
///
/// ## Returns
///
/// Tuple of three vertex positions `(v0, v1, v2)`
#[inline]
pub fn get_triangle_vertices(mesh: &Mesh, idx_offset: usize) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let i0 = mesh.indices[idx_offset] as usize * 3;
    let i1 = mesh.indices[idx_offset + 1] as usize * 3;
    let i2 = mesh.indices[idx_offset + 2] as usize * 3;
    
    (
        [mesh.vertices[i0], mesh.vertices[i0 + 1], mesh.vertices[i0 + 2]],
        [mesh.vertices[i1], mesh.vertices[i1 + 1], mesh.vertices[i1 + 2]],
        [mesh.vertices[i2], mesh.vertices[i2 + 1], mesh.vertices[i2 + 2]],
    )
}

/// Compute triangle normal from three vertices.
///
/// Returns normalized cross product of edges (v1-v0) × (v2-v0).
pub fn compute_triangle_normal(v0: &[f32; 3], v1: &[f32; 3], v2: &[f32; 3]) -> [f32; 3] {
    let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    normalize(&cross(&edge1, &edge2))
}

// =============================================================================
// DISTANCE CALCULATIONS
// =============================================================================

/// Compute approximate distance from point to triangle.
///
/// Projects point onto triangle's plane and returns the plane distance.
/// This is an approximation that ignores triangle boundaries (for performance).
///
/// ## Use Case
///
/// Used for boundary tolerance checks, not exact distance computation.
#[allow(dead_code)]
pub fn point_to_triangle_distance(
    point: &[f32; 3],
    v0: &[f32; 3],
    v1: &[f32; 3],
    v2: &[f32; 3],
) -> f32 {
    let normal = compute_triangle_normal(v0, v1, v2);
    let to_point = [point[0] - v0[0], point[1] - v0[1], point[2] - v0[2]];
    dot(&to_point, &normal).abs()
}

/// Compute minimum distance from point to mesh surface.
///
/// Iterates all triangles and returns minimum plane distance.
#[allow(dead_code)]
pub fn point_to_mesh_distance(point: &[f32; 3], mesh: &Mesh) -> f32 {
    let mut min_dist = f32::MAX;
    
    for i in (0..mesh.indices.len()).step_by(3) {
        let (v0, v1, v2) = get_triangle_vertices(mesh, i);
        let dist = point_to_triangle_distance(point, &v0, &v1, &v2);
        min_dist = min_dist.min(dist);
    }
    
    min_dist
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product() {
        assert_eq!(dot(&[1.0, 0.0, 0.0], &[1.0, 0.0, 0.0]), 1.0);
        assert_eq!(dot(&[1.0, 0.0, 0.0], &[0.0, 1.0, 0.0]), 0.0);
        assert_eq!(dot(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]), 32.0);
    }

    #[test]
    fn test_cross_product() {
        let x = [1.0, 0.0, 0.0];
        let y = [0.0, 1.0, 0.0];
        let z = cross(&x, &y);
        assert!((z[0]).abs() < 1e-6);
        assert!((z[1]).abs() < 1e-6);
        assert!((z[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize() {
        let v = normalize(&[3.0, 4.0, 0.0]);
        assert!((v[0] - 0.6).abs() < 1e-6);
        assert!((v[1] - 0.8).abs() < 1e-6);
        assert!((v[2]).abs() < 1e-6);
    }
}

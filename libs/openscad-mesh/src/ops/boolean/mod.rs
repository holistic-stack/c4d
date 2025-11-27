//! # Boolean Operations (CSG)
//!
//! Constructive Solid Geometry operations using BSP trees.
//!
//! ## Algorithm
//!
//! Based on the csg.js algorithm by Evan Wallace:
//! - Union: A.clipTo(B); B.clipTo(A); B.invert(); B.clipTo(A); B.invert(); combine
//! - Difference: A.invert(); A.clipTo(B); B.clipTo(A); B.invert(); B.clipTo(A); B.invert(); combine; result.invert()
//! - Intersection: A.invert(); B.clipTo(A); B.invert(); A.clipTo(B); B.clipTo(A); combine; result.invert()
//!
//! ## Example
//!
//! ```rust
//! use openscad_mesh::ops::boolean::{union, difference, intersection};
//! use openscad_mesh::Mesh;
//!
//! let a = Mesh::new(); // First mesh
//! let b = Mesh::new(); // Second mesh
//! let result = difference(&a, &b);
//! ```

mod bsp;
mod plane;
mod polygon;
mod vertex;

use crate::Mesh;
use bsp::BspNode;
use polygon::Polygon;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Compute union of two meshes.
///
/// Returns a mesh containing all geometry from both inputs.
///
/// ## Parameters
///
/// - `a`: First mesh
/// - `b`: Second mesh
///
/// ## Returns
///
/// New mesh containing the union.
pub fn union(a: &Mesh, b: &Mesh) -> Mesh {
    // Convert meshes to polygons
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    if polys_a.is_empty() {
        return b.clone();
    }
    if polys_b.is_empty() {
        return a.clone();
    }

    // Build BSP trees
    let mut bsp_a = BspNode::new(polys_a);
    let mut bsp_b = BspNode::new(polys_b);

    // Union algorithm:
    // a.clipTo(b) - remove parts of A inside B
    // b.clipTo(a) - remove parts of B inside A
    // b.invert(); b.clipTo(a); b.invert() - remove coplanar faces from B
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();

    // Combine polygons
    let mut result_polys = bsp_a.all_polygons();
    result_polys.extend(bsp_b.all_polygons());

    polygons_to_mesh(&result_polys)
}

/// Compute difference of two meshes (A - B).
///
/// Returns a mesh containing geometry from A that is not in B.
///
/// ## Parameters
///
/// - `a`: First mesh (base)
/// - `b`: Second mesh (to subtract)
///
/// ## Returns
///
/// New mesh containing the difference.
pub fn difference(a: &Mesh, b: &Mesh) -> Mesh {
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    if polys_a.is_empty() {
        return Mesh::new();
    }
    if polys_b.is_empty() {
        return a.clone();
    }

    let mut bsp_a = BspNode::new(polys_a);
    let mut bsp_b = BspNode::new(polys_b);

    // Difference algorithm: A - B = ~(~A | B)
    // a.invert()
    // a.clipTo(b)
    // b.clipTo(a)
    // b.invert(); b.clipTo(a); b.invert()
    // combine and invert result
    bsp_a.invert();
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();

    let mut result_polys = bsp_a.all_polygons();
    result_polys.extend(bsp_b.all_polygons());

    let mut result = BspNode::new(result_polys);
    result.invert();

    polygons_to_mesh(&result.all_polygons())
}

/// Compute intersection of two meshes.
///
/// Returns a mesh containing only geometry that is in both inputs.
///
/// ## Parameters
///
/// - `a`: First mesh
/// - `b`: Second mesh
///
/// ## Returns
///
/// New mesh containing the intersection.
pub fn intersection(a: &Mesh, b: &Mesh) -> Mesh {
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    if polys_a.is_empty() || polys_b.is_empty() {
        return Mesh::new();
    }

    let mut bsp_a = BspNode::new(polys_a);
    let mut bsp_b = BspNode::new(polys_b);

    // Intersection algorithm: A & B = ~(~A | ~B)
    // a.invert()
    // b.clipTo(a)
    // b.invert()
    // a.clipTo(b)
    // b.clipTo(a)
    // combine and invert result
    bsp_a.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);

    let mut result_polys = bsp_a.all_polygons();
    result_polys.extend(bsp_b.all_polygons());

    let mut result = BspNode::new(result_polys);
    result.invert();

    polygons_to_mesh(&result.all_polygons())
}

// =============================================================================
// CONVERSION HELPERS
// =============================================================================

/// Convert mesh to list of polygons.
fn mesh_to_polygons(mesh: &Mesh) -> Vec<Polygon> {
    let mut polygons = Vec::new();

    // Each triangle becomes a polygon
    for i in (0..mesh.indices.len()).step_by(3) {
        let i0 = mesh.indices[i] as usize;
        let i1 = mesh.indices[i + 1] as usize;
        let i2 = mesh.indices[i + 2] as usize;

        let v0 = vertex::Vertex::new(
            mesh.vertices[i0 * 3] as f64,
            mesh.vertices[i0 * 3 + 1] as f64,
            mesh.vertices[i0 * 3 + 2] as f64,
        );
        let v1 = vertex::Vertex::new(
            mesh.vertices[i1 * 3] as f64,
            mesh.vertices[i1 * 3 + 1] as f64,
            mesh.vertices[i1 * 3 + 2] as f64,
        );
        let v2 = vertex::Vertex::new(
            mesh.vertices[i2 * 3] as f64,
            mesh.vertices[i2 * 3 + 1] as f64,
            mesh.vertices[i2 * 3 + 2] as f64,
        );

        if let Some(poly) = Polygon::from_vertices(vec![v0, v1, v2]) {
            polygons.push(poly);
        }
    }

    polygons
}

/// Convert list of polygons back to mesh.
fn polygons_to_mesh(polygons: &[Polygon]) -> Mesh {
    let mut mesh = Mesh::new();

    for poly in polygons {
        let vertices = poly.vertices();
        if vertices.len() < 3 {
            continue;
        }

        // Triangulate polygon (fan triangulation)
        let normal = poly.plane().normal();
        let base_idx = mesh.vertices.len() as u32 / 3;

        // Add all vertices
        for v in vertices {
            mesh.vertices.push(v.x as f32);
            mesh.vertices.push(v.y as f32);
            mesh.vertices.push(v.z as f32);
            mesh.normals.push(normal.x as f32);
            mesh.normals.push(normal.y as f32);
            mesh.normals.push(normal.z as f32);
        }

        // Fan triangulation
        for i in 1..(vertices.len() - 1) {
            mesh.indices.push(base_idx);
            mesh.indices.push(base_idx + i as u32);
            mesh.indices.push(base_idx + i as u32 + 1);
        }
    }

    mesh
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a simple cube mesh for testing.
    fn create_cube(size: f32, offset: [f32; 3]) -> Mesh {
        let mut mesh = Mesh::new();
        let s = size / 2.0;
        let [ox, oy, oz] = offset;

        // 8 vertices of cube
        let verts = [
            [-s + ox, -s + oy, -s + oz], // 0: front-bottom-left
            [s + ox, -s + oy, -s + oz],  // 1: front-bottom-right
            [s + ox, s + oy, -s + oz],   // 2: front-top-right
            [-s + ox, s + oy, -s + oz],  // 3: front-top-left
            [-s + ox, -s + oy, s + oz],  // 4: back-bottom-left
            [s + ox, -s + oy, s + oz],   // 5: back-bottom-right
            [s + ox, s + oy, s + oz],    // 6: back-top-right
            [-s + ox, s + oy, s + oz],   // 7: back-top-left
        ];

        // 6 faces, each with 2 triangles (CCW winding when viewed from outside)
        let faces: [([usize; 3], [usize; 3], [f32; 3]); 6] = [
            ([0, 2, 1], [0, 3, 2], [0.0, 0.0, -1.0]), // Front (z-)
            ([4, 5, 6], [4, 6, 7], [0.0, 0.0, 1.0]),  // Back (z+)
            ([0, 4, 7], [0, 7, 3], [-1.0, 0.0, 0.0]), // Left (x-)
            ([1, 2, 6], [1, 6, 5], [1.0, 0.0, 0.0]),  // Right (x+)
            ([0, 1, 5], [0, 5, 4], [0.0, -1.0, 0.0]), // Bottom (y-)
            ([3, 7, 6], [3, 6, 2], [0.0, 1.0, 0.0]),  // Top (y+)
        ];

        for (tri1, tri2, normal) in &faces {
            // First triangle
            let base = (mesh.vertices.len() / 3) as u32;
            for &idx in tri1 {
                let v = verts[idx];
                mesh.vertices.extend_from_slice(&v);
                mesh.normals.extend_from_slice(normal);
            }
            mesh.indices.extend_from_slice(&[base, base + 1, base + 2]);

            // Second triangle
            let base = (mesh.vertices.len() / 3) as u32;
            for &idx in tri2 {
                let v = verts[idx];
                mesh.vertices.extend_from_slice(&v);
                mesh.normals.extend_from_slice(normal);
            }
            mesh.indices.extend_from_slice(&[base, base + 1, base + 2]);
        }

        mesh
    }

    #[test]
    fn test_union_non_overlapping() {
        let a = create_cube(2.0, [-2.0, 0.0, 0.0]);
        let b = create_cube(2.0, [2.0, 0.0, 0.0]);

        let result = union(&a, &b);

        // Should have vertices from both cubes
        assert!(!result.vertices.is_empty());
        assert!(!result.indices.is_empty());
    }

    #[test]
    fn test_union_overlapping() {
        let a = create_cube(2.0, [0.0, 0.0, 0.0]);
        let b = create_cube(2.0, [1.0, 0.0, 0.0]);

        let result = union(&a, &b);

        // Should produce a valid mesh
        assert!(!result.vertices.is_empty());
        assert!(!result.indices.is_empty());
    }

    #[test]
    fn test_difference_overlapping() {
        let a = create_cube(4.0, [0.0, 0.0, 0.0]);
        let b = create_cube(2.0, [0.0, 0.0, 0.0]);

        let result = difference(&a, &b);

        // Should produce a valid mesh (hollow cube)
        assert!(!result.vertices.is_empty());
        assert!(!result.indices.is_empty());
    }

    #[test]
    fn test_intersection_overlapping() {
        let a = create_cube(2.0, [0.0, 0.0, 0.0]);
        let b = create_cube(2.0, [1.0, 0.0, 0.0]);

        let result = intersection(&a, &b);

        // Should produce a valid mesh (overlapping region)
        assert!(!result.vertices.is_empty());
        assert!(!result.indices.is_empty());
    }

    #[test]
    fn test_difference_non_overlapping() {
        let a = create_cube(2.0, [-2.0, 0.0, 0.0]);
        let b = create_cube(2.0, [2.0, 0.0, 0.0]);

        let result = difference(&a, &b);

        // Should return A unchanged (approximately)
        assert!(!result.vertices.is_empty());
    }

    #[test]
    fn test_intersection_non_overlapping() {
        let a = create_cube(2.0, [-2.0, 0.0, 0.0]);
        let b = create_cube(2.0, [2.0, 0.0, 0.0]);

        let result = intersection(&a, &b);

        // Should return empty mesh
        assert!(result.vertices.is_empty() || result.indices.is_empty());
    }
}

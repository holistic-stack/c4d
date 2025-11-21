/// Cube primitive implementation.
///
/// This module provides a function to create a cube manifold using the
/// index-based half-edge representation.

mod tests;

use crate::core::ds::{Face, HalfEdge, Vertex};
use crate::core::vec3::Vec3;
use crate::error::ManifoldError;
use crate::Manifold;

/// Creates a cube manifold.
///
/// # Arguments
/// * `size` - The dimensions of the cube (width, height, depth)
/// * `center` - If true, center the cube at the origin. If false, place one corner at the origin.
///
/// # Returns
/// A `Result` containing the cube `Manifold` or an error if the size is invalid.
///
/// # Examples
/// ```
/// use manifold_rs::primitives::cube::cube;
/// use manifold_rs::Vec3;
///
/// // Create a unit cube at the origin
/// let c = cube(Vec3::new(1.0, 1.0, 1.0), false).unwrap();
/// assert_eq!(c.vertex_count(), 8);
///
/// // Create a centered cube
/// let c = cube(Vec3::new(2.0, 2.0, 2.0), true).unwrap();
/// let (min, max) = c.bounding_box();
/// assert_eq!(min, Vec3::new(-1.0, -1.0, -1.0));
/// ```
pub fn cube(size: Vec3, center: bool) -> Result<Manifold, ManifoldError> {
    // Validate size
    if size.x <= 0.0 || size.y <= 0.0 || size.z <= 0.0 {
        return Err(ManifoldError::InvalidTopology(
            "Cube size must be positive in all dimensions".to_string(),
        ));
    }

    let mut manifold = Manifold::new();

    // Calculate offset based on centering
    let offset = if center {
        Vec3::new(-size.x / 2.0, -size.y / 2.0, -size.z / 2.0)
    } else {
        Vec3::ZERO
    };

    // Create 8 vertices for the cube
    // Vertex ordering:
    // 0: (0, 0, 0) - bottom-front-left
    // 1: (x, 0, 0) - bottom-front-right
    // 2: (x, y, 0) - bottom-back-right
    // 3: (0, y, 0) - bottom-back-left
    // 4: (0, 0, z) - top-front-left
    // 5: (x, 0, z) - top-front-right
    // 6: (x, y, z) - top-back-right
    // 7: (0, y, z) - top-back-left
    
    let vertices = vec![
        Vec3::new(0.0, 0.0, 0.0) + offset,      // 0
        Vec3::new(size.x, 0.0, 0.0) + offset,   // 1
        Vec3::new(size.x, size.y, 0.0) + offset, // 2
        Vec3::new(0.0, size.y, 0.0) + offset,   // 3
        Vec3::new(0.0, 0.0, size.z) + offset,   // 4
        Vec3::new(size.x, 0.0, size.z) + offset, // 5
        Vec3::new(size.x, size.y, size.z) + offset, // 6
        Vec3::new(0.0, size.y, size.z) + offset, // 7
    ];

    // Add vertices to manifold (we'll set first_edge later)
    for pos in vertices {
        manifold.vertices.push(Vertex::new(pos, 0));
    }

    // Define the 12 triangular faces (2 per cube face)
    // Each face is defined by 3 vertex indices
    // Faces are oriented with outward-facing normals (counter-clockwise winding)
    
    let face_triangles = vec![
        // Bottom face (z = 0)
        ([0, 2, 1], Vec3::new(0.0, 0.0, -1.0)),
        ([0, 3, 2], Vec3::new(0.0, 0.0, -1.0)),
        
        // Top face (z = size.z)
        ([4, 5, 6], Vec3::new(0.0, 0.0, 1.0)),
        ([4, 6, 7], Vec3::new(0.0, 0.0, 1.0)),
        
        // Front face (y = 0)
        ([0, 1, 5], Vec3::new(0.0, -1.0, 0.0)),
        ([0, 5, 4], Vec3::new(0.0, -1.0, 0.0)),
        
        // Back face (y = size.y)
        ([2, 3, 7], Vec3::new(0.0, 1.0, 0.0)),
        ([2, 7, 6], Vec3::new(0.0, 1.0, 0.0)),
        
        // Left face (x = 0)
        ([3, 0, 4], Vec3::new(-1.0, 0.0, 0.0)),
        ([3, 4, 7], Vec3::new(-1.0, 0.0, 0.0)),
        
        // Right face (x = size.x)
        ([1, 2, 6], Vec3::new(1.0, 0.0, 0.0)),
        ([1, 6, 5], Vec3::new(1.0, 0.0, 0.0)),
    ];

    // Build half-edge structure
    for (face_idx, (triangle, normal)) in face_triangles.iter().enumerate() {
        let [v0, v1, v2] = *triangle;
        
        // Create 3 half-edges for this face
        let edge_base = manifold.half_edges.len() as u32;
        
        // Edge 0: v0 -> v1
        manifold.half_edges.push(HalfEdge::new(
            v0 as u32,
            v1 as u32,
            edge_base + 1, // next
            0,             // pair (will be set later)
            face_idx as u32,
        ));
        
        // Edge 1: v1 -> v2
        manifold.half_edges.push(HalfEdge::new(
            v1 as u32,
            v2 as u32,
            edge_base + 2, // next
            0,             // pair (will be set later)
            face_idx as u32,
        ));
        
        // Edge 2: v2 -> v0
        manifold.half_edges.push(HalfEdge::new(
            v2 as u32,
            v0 as u32,
            edge_base,     // next (back to first edge)
            0,             // pair (will be set later)
            face_idx as u32,
        ));
        
        // Add face
        manifold.faces.push(Face::new(edge_base, *normal));
    }

    // Set vertex first_edge pointers
    for (edge_idx, edge) in manifold.half_edges.iter().enumerate() {
        let vert_idx = edge.start_vert as usize;
        if manifold.vertices[vert_idx].first_edge == 0 || edge_idx < manifold.vertices[vert_idx].first_edge as usize {
            manifold.vertices[vert_idx].first_edge = edge_idx as u32;
        }
    }

    // Set up edge pairing
    // For each edge, find its pair (the edge going in the opposite direction)
    for i in 0..manifold.half_edges.len() {
        if manifold.half_edges[i].pair_edge != 0 {
            continue; // Already paired
        }
        
        let edge_i = manifold.half_edges[i];
        
        // Find the matching edge
        for j in (i + 1)..manifold.half_edges.len() {
            let edge_j = manifold.half_edges[j];
            
            if edge_i.start_vert == edge_j.end_vert && edge_i.end_vert == edge_j.start_vert {
                // Found the pair
                manifold.half_edges[i].pair_edge = j as u32;
                manifold.half_edges[j].pair_edge = i as u32;
                break;
            }
        }
    }

    Ok(manifold)
}

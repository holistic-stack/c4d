/// Mesh buffer export for GPU rendering.
///
/// This module provides functionality to export manifold geometry to
/// GPU-friendly mesh buffers using `f32` precision.

use crate::Manifold;

/// Mesh buffers suitable for GPU rendering.
///
/// Contains vertex positions and triangle indices in formats
/// compatible with WebGL/WebGPU.
///
/// # Examples
/// ```
/// use manifold_rs::primitives::cube::cube;
/// use manifold_rs::Vec3;
///
/// let c = cube(Vec3::new(1.0, 1.0, 1.0), false).unwrap();
/// let buffers = c.to_mesh_buffers();
///
/// assert_eq!(buffers.vertices.len(), 8 * 3); // 8 vertices * 3 components
/// assert_eq!(buffers.indices.len(), 12 * 3); // 12 triangles * 3 indices
/// ```
#[derive(Debug, Clone)]
pub struct MeshBuffers {
    /// Vertex positions as flat array [x, y, z, x, y, z, ...].
    /// Uses `f32` for GPU compatibility.
    pub vertices: Vec<f32>,
    
    /// Triangle indices as flat array [i0, i1, i2, i0, i1, i2, ...].
    pub indices: Vec<u32>,
}

impl MeshBuffers {
    /// Creates empty mesh buffers.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Returns the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    /// Returns the number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

impl Default for MeshBuffers {
    fn default() -> Self {
        Self::new()
    }
}

impl Manifold {
    /// Exports the manifold to GPU-friendly mesh buffers.
    ///
    /// Converts internal `f64` precision to `f32` for GPU rendering.
    /// Vertices are deduplicated and indexed.
    ///
    /// # Examples
    /// ```
    /// use manifold_rs::primitives::cube::cube;
    /// use manifold_rs::Vec3;
    ///
    /// let c = cube(Vec3::new(2.0, 2.0, 2.0), true).unwrap();
    /// let buffers = c.to_mesh_buffers();
    ///
    /// assert!(buffers.vertex_count() > 0);
    /// assert!(buffers.triangle_count() > 0);
    /// ```
    pub fn to_mesh_buffers(&self) -> MeshBuffers {
        let mut buffers = MeshBuffers::new();

        // Export vertices
        for vertex in &self.vertices {
            buffers.vertices.push(vertex.position.x as f32);
            buffers.vertices.push(vertex.position.y as f32);
            buffers.vertices.push(vertex.position.z as f32);
        }

        // Export triangle indices
        // Each face is a triangle, so we can directly use the half-edge structure
        for face in &self.faces {
            let first_edge_idx = face.first_edge as usize;
            let edge0 = &self.half_edges[first_edge_idx];
            let edge1 = &self.half_edges[edge0.next_edge as usize];
            let edge2 = &self.half_edges[edge1.next_edge as usize];

            buffers.indices.push(edge0.start_vert);
            buffers.indices.push(edge1.start_vert);
            buffers.indices.push(edge2.start_vert);
        }

        buffers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::cube::cube;
    use crate::Vec3;

    #[test]
    fn test_mesh_buffers_creation() {
        let buffers = MeshBuffers::new();
        assert_eq!(buffers.vertex_count(), 0);
        assert_eq!(buffers.triangle_count(), 0);
    }

    #[test]
    fn test_cube_export() {
        let c = cube(Vec3::new(1.0, 1.0, 1.0), false).unwrap();
        let buffers = c.to_mesh_buffers();

        // 8 vertices * 3 components
        assert_eq!(buffers.vertices.len(), 24);
        assert_eq!(buffers.vertex_count(), 8);

        // 12 triangles * 3 indices
        assert_eq!(buffers.indices.len(), 36);
        assert_eq!(buffers.triangle_count(), 12);
    }

    #[test]
    fn test_centered_cube_export() {
        let c = cube(Vec3::new(2.0, 2.0, 2.0), true).unwrap();
        let buffers = c.to_mesh_buffers();

        assert_eq!(buffers.vertex_count(), 8);
        assert_eq!(buffers.triangle_count(), 12);

        // Verify vertices are in f32 range
        for &v in &buffers.vertices {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn test_indices_in_range() {
        let c = cube(Vec3::new(1.0, 1.0, 1.0), false).unwrap();
        let buffers = c.to_mesh_buffers();

        let vertex_count = buffers.vertex_count() as u32;
        for &idx in &buffers.indices {
            assert!(idx < vertex_count, "Index {} out of range", idx);
        }
    }
}

//! # Mesh Types
//!
//! Triangle mesh representation.

use serde::{Deserialize, Serialize};

// =============================================================================
// MESH
// =============================================================================

/// Triangle mesh output.
///
/// Flat arrays for efficient transfer to WebGL.
///
/// ## Example
///
/// ```rust
/// let mesh = Mesh::new();
/// assert!(mesh.vertices.is_empty());
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Mesh {
    /// Vertex positions as flat array [x, y, z, x, y, z, ...].
    pub vertices: Vec<f32>,
    /// Triangle indices as flat array [i0, i1, i2, i0, i1, i2, ...].
    pub indices: Vec<u32>,
    /// Per-vertex normals as flat array [nx, ny, nz, nx, ny, nz, ...].
    pub normals: Vec<f32>,
    /// Optional per-vertex colors [r, g, b, a, r, g, b, a, ...].
    pub colors: Option<Vec<f32>>,
}

impl Mesh {
    /// Create empty mesh.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get vertex count.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    /// Get triangle count.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if mesh is empty.
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Add a vertex and return its index.
    ///
    /// ## Parameters
    ///
    /// - `x`, `y`, `z`: Vertex position
    /// - `nx`, `ny`, `nz`: Vertex normal
    ///
    /// ## Returns
    ///
    /// Index of the added vertex
    pub fn add_vertex(&mut self, x: f32, y: f32, z: f32, nx: f32, ny: f32, nz: f32) -> u32 {
        let index = self.vertex_count() as u32;
        self.vertices.extend_from_slice(&[x, y, z]);
        self.normals.extend_from_slice(&[nx, ny, nz]);
        index
    }

    /// Add a triangle from vertex indices.
    pub fn add_triangle(&mut self, i0: u32, i1: u32, i2: u32) {
        self.indices.extend_from_slice(&[i0, i1, i2]);
    }

    /// Merge another mesh into this one.
    ///
    /// Adjusts indices of the merged mesh.
    pub fn merge(&mut self, other: &Mesh) {
        let offset = self.vertex_count() as u32;
        
        // Add vertices and normals
        self.vertices.extend_from_slice(&other.vertices);
        self.normals.extend_from_slice(&other.normals);
        
        // Add indices with offset
        for &idx in &other.indices {
            self.indices.push(idx + offset);
        }
        
        // Merge colors if both have them
        if let (Some(ref mut our_colors), Some(ref their_colors)) = 
            (&mut self.colors, &other.colors) {
            our_colors.extend_from_slice(their_colors);
        }
    }

    /// Apply a 4x4 transformation matrix to all vertices.
    ///
    /// Also transforms normals (with inverse transpose for correct lighting).
    pub fn transform(&mut self, matrix: &[[f32; 4]; 4]) {
        use glam::{Mat4, Vec3, Vec4};
        
        let mat = Mat4::from_cols_array_2d(matrix);
        let normal_mat = mat.inverse().transpose();
        
        // Transform vertices
        for i in (0..self.vertices.len()).step_by(3) {
            let v = Vec4::new(
                self.vertices[i],
                self.vertices[i + 1],
                self.vertices[i + 2],
                1.0,
            );
            let transformed = mat * v;
            self.vertices[i] = transformed.x;
            self.vertices[i + 1] = transformed.y;
            self.vertices[i + 2] = transformed.z;
        }
        
        // Transform normals
        for i in (0..self.normals.len()).step_by(3) {
            let n = Vec4::new(
                self.normals[i],
                self.normals[i + 1],
                self.normals[i + 2],
                0.0,
            );
            let transformed = (normal_mat * n).truncate().normalize();
            self.normals[i] = transformed.x;
            self.normals[i + 1] = transformed.y;
            self.normals[i + 2] = transformed.z;
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_new() {
        let mesh = Mesh::new();
        assert!(mesh.is_empty());
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
    }

    #[test]
    fn test_mesh_add_vertex() {
        let mut mesh = Mesh::new();
        let idx = mesh.add_vertex(1.0, 2.0, 3.0, 0.0, 1.0, 0.0);
        assert_eq!(idx, 0);
        assert_eq!(mesh.vertex_count(), 1);
    }

    #[test]
    fn test_mesh_add_triangle() {
        let mut mesh = Mesh::new();
        mesh.add_vertex(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        mesh.add_vertex(1.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        mesh.add_vertex(0.0, 1.0, 0.0, 0.0, 1.0, 0.0);
        mesh.add_triangle(0, 1, 2);
        
        assert_eq!(mesh.triangle_count(), 1);
    }

    #[test]
    fn test_mesh_merge() {
        let mut mesh1 = Mesh::new();
        mesh1.add_vertex(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        mesh1.add_triangle(0, 0, 0);

        let mut mesh2 = Mesh::new();
        mesh2.add_vertex(1.0, 1.0, 1.0, 0.0, 1.0, 0.0);
        mesh2.add_triangle(0, 0, 0);

        mesh1.merge(&mesh2);
        
        assert_eq!(mesh1.vertex_count(), 2);
        // Second triangle indices should be offset by 1
        assert_eq!(mesh1.indices[3], 1);
    }
}

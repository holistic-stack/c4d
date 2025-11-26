//! # Mesh Data Structure
//!
//! Core mesh representation with vertices, triangles, and optional colors.

use config::constants::{DEFAULT_COLOR, VERTEX_MERGE_EPSILON};
use glam::DVec3;

/// A triangle mesh with vertices and indices.
///
/// All geometry calculations use f64 internally. Export to f32 only
/// happens at the WASM boundary for GPU rendering.
///
/// # Example
///
/// ```rust
/// use openscad_mesh::Mesh;
/// use glam::DVec3;
///
/// let mut mesh = Mesh::new();
/// mesh.add_vertex(DVec3::new(0.0, 0.0, 0.0));
/// mesh.add_vertex(DVec3::new(1.0, 0.0, 0.0));
/// mesh.add_vertex(DVec3::new(0.0, 1.0, 0.0));
/// mesh.add_triangle(0, 1, 2);
/// ```
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Vertex positions (f64 for precision)
    vertices: Vec<DVec3>,
    /// Triangle indices (3 indices per triangle)
    triangles: Vec<[u32; 3]>,
    /// Optional vertex colors (RGBA, f32 for GPU)
    colors: Option<Vec<[f32; 4]>>,
    /// Optional vertex normals
    normals: Option<Vec<DVec3>>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesh {
    /// Creates an empty mesh.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
            colors: None,
            normals: None,
        }
    }

    /// Creates a mesh with pre-allocated capacity.
    pub fn with_capacity(vertex_count: usize, triangle_count: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_count),
            triangles: Vec::with_capacity(triangle_count),
            colors: None,
            normals: None,
        }
    }

    /// Returns the number of vertices.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of triangles.
    #[inline]
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Returns true if the mesh is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Adds a vertex and returns its index.
    pub fn add_vertex(&mut self, position: DVec3) -> u32 {
        let index = self.vertices.len() as u32;
        self.vertices.push(position);
        index
    }

    /// Adds a triangle by vertex indices.
    pub fn add_triangle(&mut self, v0: u32, v1: u32, v2: u32) {
        self.triangles.push([v0, v1, v2]);
    }

    /// Returns a reference to the vertices.
    #[inline]
    pub fn vertices(&self) -> &[DVec3] {
        &self.vertices
    }

    /// Returns a reference to the triangles.
    #[inline]
    pub fn triangles(&self) -> &[[u32; 3]] {
        &self.triangles
    }

    /// Returns the vertex at the given index.
    #[inline]
    pub fn vertex(&self, index: u32) -> DVec3 {
        self.vertices[index as usize]
    }

    /// Returns the triangle at the given index.
    #[inline]
    pub fn triangle(&self, index: usize) -> [u32; 3] {
        self.triangles[index]
    }

    /// Sets vertex colors.
    pub fn set_colors(&mut self, colors: Vec<[f32; 4]>) {
        self.colors = Some(colors);
    }

    /// Sets a uniform color for all vertices.
    pub fn set_uniform_color(&mut self, color: [f32; 4]) {
        let colors = vec![color; self.vertices.len()];
        self.colors = Some(colors);
    }

    /// Returns the vertex colors.
    pub fn colors(&self) -> Option<&[[f32; 4]]> {
        self.colors.as_deref()
    }

    /// Sets vertex normals.
    pub fn set_normals(&mut self, normals: Vec<DVec3>) {
        self.normals = Some(normals);
    }

    /// Returns the vertex normals.
    pub fn normals(&self) -> Option<&[DVec3]> {
        self.normals.as_deref()
    }

    /// Computes and sets face normals for each vertex.
    pub fn compute_normals(&mut self) {
        let mut normals = vec![DVec3::ZERO; self.vertices.len()];

        for tri in &self.triangles {
            let v0 = self.vertices[tri[0] as usize];
            let v1 = self.vertices[tri[1] as usize];
            let v2 = self.vertices[tri[2] as usize];

            let edge1 = v1 - v0;
            let edge2 = v2 - v0;
            let normal = edge1.cross(edge2);

            normals[tri[0] as usize] += normal;
            normals[tri[1] as usize] += normal;
            normals[tri[2] as usize] += normal;
        }

        // Normalize
        for normal in &mut normals {
            let len = normal.length();
            if len > 0.0 {
                *normal /= len;
            }
        }

        self.normals = Some(normals);
    }

    /// Computes the axis-aligned bounding box.
    ///
    /// Returns (min, max) corners of the bounding box.
    pub fn bounding_box(&self) -> (DVec3, DVec3) {
        if self.vertices.is_empty() {
            return (DVec3::ZERO, DVec3::ZERO);
        }

        let mut min = self.vertices[0];
        let mut max = self.vertices[0];

        for v in &self.vertices[1..] {
            min = min.min(*v);
            max = max.max(*v);
        }

        (min, max)
    }

    /// Transforms all vertices by a 4x4 matrix.
    pub fn transform(&mut self, matrix: &glam::DMat4) {
        for v in &mut self.vertices {
            let transformed = matrix.transform_point3(*v);
            *v = transformed;
        }

        // Transform normals if present (use inverse transpose for normals)
        if let Some(normals) = &mut self.normals {
            let normal_matrix = matrix.inverse().transpose();
            for n in normals {
                let transformed = normal_matrix.transform_vector3(*n);
                *n = transformed.normalize();
            }
        }
    }

    /// Translates the mesh by a vector.
    ///
    /// # Arguments
    ///
    /// * `offset` - Translation vector
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// mesh.translate(DVec3::new(10.0, 0.0, 0.0));
    /// ```
    pub fn translate(&mut self, offset: DVec3) {
        for v in &mut self.vertices {
            *v += offset;
        }
    }

    /// Merges another mesh into this one.
    pub fn merge(&mut self, other: &Mesh) {
        let offset = self.vertices.len() as u32;

        self.vertices.extend_from_slice(&other.vertices);

        for tri in &other.triangles {
            self.triangles.push([
                tri[0] + offset,
                tri[1] + offset,
                tri[2] + offset,
            ]);
        }

        // Merge colors if both have them
        if let (Some(self_colors), Some(other_colors)) = (&mut self.colors, &other.colors) {
            self_colors.extend_from_slice(other_colors);
        } else if other.colors.is_some() {
            // Other has colors but self doesn't - add default colors to self
            let mut colors = vec![DEFAULT_COLOR; self.vertices.len() - other.vertices.len()];
            colors.extend_from_slice(other.colors.as_ref().unwrap());
            self.colors = Some(colors);
        }
    }

    /// Validates the mesh for correctness.
    ///
    /// Checks:
    /// - All triangle indices are valid
    /// - No degenerate triangles (zero area)
    ///
    /// Returns true if valid.
    pub fn validate(&self) -> bool {
        let vertex_count = self.vertices.len() as u32;

        for tri in &self.triangles {
            // Check indices are valid
            if tri[0] >= vertex_count || tri[1] >= vertex_count || tri[2] >= vertex_count {
                return false;
            }

            // Check for degenerate triangles
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
                return false;
            }

            // Check for zero-area triangles
            let v0 = self.vertices[tri[0] as usize];
            let v1 = self.vertices[tri[1] as usize];
            let v2 = self.vertices[tri[2] as usize];
            let area = (v1 - v0).cross(v2 - v0).length();
            if area < VERTEX_MERGE_EPSILON {
                return false;
            }
        }

        true
    }

    /// Exports vertices as f32 array for GPU.
    ///
    /// Returns flattened [x, y, z, x, y, z, ...] array.
    pub fn vertices_f32(&self) -> Vec<f32> {
        let mut result = Vec::with_capacity(self.vertices.len() * 3);
        for v in &self.vertices {
            result.push(v.x as f32);
            result.push(v.y as f32);
            result.push(v.z as f32);
        }
        result
    }

    /// Exports triangle indices as u32 array for GPU.
    ///
    /// Returns flattened [i0, i1, i2, i0, i1, i2, ...] array.
    pub fn indices_u32(&self) -> Vec<u32> {
        let mut result = Vec::with_capacity(self.triangles.len() * 3);
        for tri in &self.triangles {
            result.push(tri[0]);
            result.push(tri[1]);
            result.push(tri[2]);
        }
        result
    }

    /// Exports normals as f32 array for GPU.
    pub fn normals_f32(&self) -> Option<Vec<f32>> {
        self.normals.as_ref().map(|normals| {
            let mut result = Vec::with_capacity(normals.len() * 3);
            for n in normals {
                result.push(n.x as f32);
                result.push(n.y as f32);
                result.push(n.z as f32);
            }
            result
        })
    }
}

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
        let idx = mesh.add_vertex(DVec3::new(1.0, 2.0, 3.0));
        assert_eq!(idx, 0);
        assert_eq!(mesh.vertex_count(), 1);
        assert_eq!(mesh.vertex(0), DVec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_mesh_add_triangle() {
        let mut mesh = Mesh::new();
        mesh.add_vertex(DVec3::ZERO);
        mesh.add_vertex(DVec3::X);
        mesh.add_vertex(DVec3::Y);
        mesh.add_triangle(0, 1, 2);
        assert_eq!(mesh.triangle_count(), 1);
        assert_eq!(mesh.triangle(0), [0, 1, 2]);
    }

    #[test]
    fn test_mesh_bounding_box() {
        let mut mesh = Mesh::new();
        mesh.add_vertex(DVec3::new(-1.0, -2.0, -3.0));
        mesh.add_vertex(DVec3::new(4.0, 5.0, 6.0));
        let (min, max) = mesh.bounding_box();
        assert_eq!(min, DVec3::new(-1.0, -2.0, -3.0));
        assert_eq!(max, DVec3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_mesh_validate_valid() {
        let mut mesh = Mesh::new();
        mesh.add_vertex(DVec3::new(0.0, 0.0, 0.0));
        mesh.add_vertex(DVec3::new(1.0, 0.0, 0.0));
        mesh.add_vertex(DVec3::new(0.0, 1.0, 0.0));
        mesh.add_triangle(0, 1, 2);
        assert!(mesh.validate());
    }

    #[test]
    fn test_mesh_validate_invalid_index() {
        let mut mesh = Mesh::new();
        mesh.add_vertex(DVec3::ZERO);
        mesh.add_triangle(0, 1, 2); // Invalid indices
        assert!(!mesh.validate());
    }

    #[test]
    fn test_mesh_vertices_f32() {
        let mut mesh = Mesh::new();
        mesh.add_vertex(DVec3::new(1.0, 2.0, 3.0));
        let f32_verts = mesh.vertices_f32();
        assert_eq!(f32_verts, vec![1.0f32, 2.0, 3.0]);
    }

    #[test]
    fn test_mesh_merge() {
        let mut mesh1 = Mesh::new();
        mesh1.add_vertex(DVec3::ZERO);
        mesh1.add_vertex(DVec3::X);
        mesh1.add_vertex(DVec3::Y);
        mesh1.add_triangle(0, 1, 2);

        let mut mesh2 = Mesh::new();
        mesh2.add_vertex(DVec3::Z);
        mesh2.add_vertex(DVec3::new(1.0, 0.0, 1.0));
        mesh2.add_vertex(DVec3::new(0.0, 1.0, 1.0));
        mesh2.add_triangle(0, 1, 2);

        mesh1.merge(&mesh2);
        assert_eq!(mesh1.vertex_count(), 6);
        assert_eq!(mesh1.triangle_count(), 2);
        assert_eq!(mesh1.triangle(1), [3, 4, 5]); // Offset by 3
    }
}

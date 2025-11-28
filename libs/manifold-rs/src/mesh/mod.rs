//! # Mesh Module
//!
//! Triangle mesh representation for 3D geometry output.
//!
//! ## Structure
//!
//! - `Mesh` - Main triangle mesh with vertices, indices, normals
//! - `halfedge` - HalfEdge mesh for topology operations
//!
//! ## Example
//!
//! ```rust
//! use manifold_rs::Mesh;
//!
//! let mut mesh = Mesh::new();
//! let v0 = mesh.add_vertex(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
//! let v1 = mesh.add_vertex(1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
//! let v2 = mesh.add_vertex(0.5, 1.0, 0.0, 0.0, 0.0, 1.0);
//! mesh.add_triangle(v0, v1, v2);
//! ```

pub mod halfedge;

// =============================================================================
// MESH STRUCT
// =============================================================================

/// Triangle mesh with vertices, indices, and normals.
///
/// This is the output format for all manifold operations. The mesh uses
/// flat arrays optimized for WebGL rendering via Three.js.
///
/// ## Memory Layout
///
/// - `vertices`: [x0, y0, z0, x1, y1, z1, ...] - 3 floats per vertex
/// - `indices`: [i0, i1, i2, ...] - 3 indices per triangle
/// - `normals`: [nx0, ny0, nz0, ...] - 3 floats per vertex
/// - `colors`: Optional [r, g, b, a, ...] - 4 floats per vertex
///
/// ## Example
///
/// ```rust
/// use manifold_rs::Mesh;
///
/// let mesh = Mesh::new();
/// assert_eq!(mesh.vertex_count(), 0);
/// assert_eq!(mesh.triangle_count(), 0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    /// Vertex positions: [x0, y0, z0, x1, y1, z1, ...]
    ///
    /// Each vertex has 3 components (x, y, z).
    pub vertices: Vec<f32>,
    
    /// Triangle indices: [i0, i1, i2, ...]
    ///
    /// Each triangle has 3 vertex indices.
    pub indices: Vec<u32>,
    
    /// Vertex normals: [nx0, ny0, nz0, ...]
    ///
    /// Each vertex has 3 normal components (nx, ny, nz).
    pub normals: Vec<f32>,
    
    /// Optional vertex colors: [r0, g0, b0, a0, ...]
    ///
    /// Each vertex has 4 color components (r, g, b, a) in range [0.0, 1.0].
    pub colors: Option<Vec<f32>>,
}

impl Mesh {
    // =========================================================================
    // CONSTRUCTORS
    // =========================================================================

    /// Create a new empty mesh.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::Mesh;
    ///
    /// let mesh = Mesh::new();
    /// assert!(mesh.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create mesh with pre-allocated capacity.
    ///
    /// ## Parameters
    ///
    /// - `vertex_capacity`: Expected number of vertices
    /// - `triangle_capacity`: Expected number of triangles
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::Mesh;
    ///
    /// // Pre-allocate for a cube (24 vertices, 12 triangles)
    /// let mesh = Mesh::with_capacity(24, 12);
    /// ```
    #[must_use]
    pub fn with_capacity(vertex_capacity: usize, triangle_capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_capacity * 3),
            indices: Vec::with_capacity(triangle_capacity * 3),
            normals: Vec::with_capacity(vertex_capacity * 3),
            colors: None,
        }
    }

    // =========================================================================
    // VERTEX OPERATIONS
    // =========================================================================

    /// Add a vertex with position and normal.
    ///
    /// Returns the vertex index for use in triangle definitions.
    ///
    /// ## Parameters
    ///
    /// - `x, y, z`: Vertex position
    /// - `nx, ny, nz`: Vertex normal (should be normalized)
    ///
    /// ## Returns
    ///
    /// Vertex index (u32)
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::Mesh;
    ///
    /// let mut mesh = Mesh::new();
    /// let idx = mesh.add_vertex(1.0, 2.0, 3.0, 0.0, 0.0, 1.0);
    /// assert_eq!(idx, 0);
    /// ```
    pub fn add_vertex(&mut self, x: f32, y: f32, z: f32, nx: f32, ny: f32, nz: f32) -> u32 {
        let index = (self.vertices.len() / 3) as u32;
        self.vertices.extend_from_slice(&[x, y, z]);
        self.normals.extend_from_slice(&[nx, ny, nz]);
        index
    }

    /// Add a vertex with position, normal, and color.
    ///
    /// ## Parameters
    ///
    /// - `x, y, z`: Vertex position
    /// - `nx, ny, nz`: Vertex normal
    /// - `r, g, b, a`: Vertex color (range [0.0, 1.0])
    ///
    /// ## Returns
    ///
    /// Vertex index (u32)
    pub fn add_vertex_with_color(
        &mut self,
        x: f32, y: f32, z: f32,
        nx: f32, ny: f32, nz: f32,
        r: f32, g: f32, b: f32, a: f32,
    ) -> u32 {
        let index = self.add_vertex(x, y, z, nx, ny, nz);
        
        // Initialize colors if needed
        if self.colors.is_none() {
            self.colors = Some(Vec::with_capacity(self.vertices.len() / 3 * 4));
        }
        
        if let Some(ref mut colors) = self.colors {
            colors.extend_from_slice(&[r, g, b, a]);
        }
        
        index
    }

    // =========================================================================
    // TRIANGLE OPERATIONS
    // =========================================================================

    /// Add a triangle by vertex indices.
    ///
    /// ## Parameters
    ///
    /// - `v0, v1, v2`: Vertex indices (from `add_vertex`)
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::Mesh;
    ///
    /// let mut mesh = Mesh::new();
    /// let v0 = mesh.add_vertex(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    /// let v1 = mesh.add_vertex(1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    /// let v2 = mesh.add_vertex(0.5, 1.0, 0.0, 0.0, 0.0, 1.0);
    /// mesh.add_triangle(v0, v1, v2);
    /// assert_eq!(mesh.triangle_count(), 1);
    /// ```
    pub fn add_triangle(&mut self, v0: u32, v1: u32, v2: u32) {
        self.indices.extend_from_slice(&[v0, v1, v2]);
    }

    // =========================================================================
    // QUERY METHODS
    // =========================================================================

    /// Get the number of vertices.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::Mesh;
    ///
    /// let mesh = Mesh::new();
    /// assert_eq!(mesh.vertex_count(), 0);
    /// ```
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len() / 3
    }

    /// Get the number of triangles.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::Mesh;
    ///
    /// let mesh = Mesh::new();
    /// assert_eq!(mesh.triangle_count(), 0);
    /// ```
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if mesh is empty.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::Mesh;
    ///
    /// let mesh = Mesh::new();
    /// assert!(mesh.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    // =========================================================================
    // TRANSFORM OPERATIONS
    // =========================================================================

    /// Apply translation to all vertices.
    ///
    /// ## Parameters
    ///
    /// - `dx, dy, dz`: Translation offset
    pub fn translate(&mut self, dx: f32, dy: f32, dz: f32) {
        for i in (0..self.vertices.len()).step_by(3) {
            self.vertices[i] += dx;
            self.vertices[i + 1] += dy;
            self.vertices[i + 2] += dz;
        }
    }

    /// Apply scale to all vertices.
    ///
    /// ## Parameters
    ///
    /// - `sx, sy, sz`: Scale factors
    pub fn scale(&mut self, sx: f32, sy: f32, sz: f32) {
        for i in (0..self.vertices.len()).step_by(3) {
            self.vertices[i] *= sx;
            self.vertices[i + 1] *= sy;
            self.vertices[i + 2] *= sz;
        }
    }

    /// Apply 4x4 transformation matrix to all vertices and normals.
    ///
    /// ## Parameters
    ///
    /// - `matrix`: 4x4 transformation matrix in column-major order
    pub fn transform(&mut self, matrix: &[[f32; 4]; 4]) {
        // Transform vertices
        for i in (0..self.vertices.len()).step_by(3) {
            let x = self.vertices[i];
            let y = self.vertices[i + 1];
            let z = self.vertices[i + 2];
            
            self.vertices[i] = matrix[0][0] * x + matrix[1][0] * y + matrix[2][0] * z + matrix[3][0];
            self.vertices[i + 1] = matrix[0][1] * x + matrix[1][1] * y + matrix[2][1] * z + matrix[3][1];
            self.vertices[i + 2] = matrix[0][2] * x + matrix[1][2] * y + matrix[2][2] * z + matrix[3][2];
        }
        
        // Transform normals (without translation, only rotation)
        for i in (0..self.normals.len()).step_by(3) {
            let nx = self.normals[i];
            let ny = self.normals[i + 1];
            let nz = self.normals[i + 2];
            
            let rnx = matrix[0][0] * nx + matrix[1][0] * ny + matrix[2][0] * nz;
            let rny = matrix[0][1] * nx + matrix[1][1] * ny + matrix[2][1] * nz;
            let rnz = matrix[0][2] * nx + matrix[1][2] * ny + matrix[2][2] * nz;
            
            // Renormalize
            let len = (rnx * rnx + rny * rny + rnz * rnz).sqrt();
            if len > 0.0 {
                self.normals[i] = rnx / len;
                self.normals[i + 1] = rny / len;
                self.normals[i + 2] = rnz / len;
            }
        }
    }

    // =========================================================================
    // MERGE OPERATIONS
    // =========================================================================

    /// Merge another mesh into this one.
    ///
    /// Indices are adjusted to account for existing vertices.
    ///
    /// ## Parameters
    ///
    /// - `other`: Mesh to merge
    pub fn merge(&mut self, other: &Mesh) {
        let vertex_offset = self.vertex_count() as u32;
        
        // Append vertices and normals
        self.vertices.extend_from_slice(&other.vertices);
        self.normals.extend_from_slice(&other.normals);
        
        // Append indices with offset
        for &idx in &other.indices {
            self.indices.push(idx + vertex_offset);
        }
        
        // Merge colors if present
        if let Some(ref other_colors) = other.colors {
            let colors = self.colors.get_or_insert_with(Vec::new);
            colors.extend_from_slice(other_colors);
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test creating empty mesh.
    #[test]
    fn test_mesh_new() {
        let mesh = Mesh::new();
        assert!(mesh.is_empty());
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
    }

    /// Test adding vertices.
    #[test]
    fn test_add_vertex() {
        let mut mesh = Mesh::new();
        let idx = mesh.add_vertex(1.0, 2.0, 3.0, 0.0, 0.0, 1.0);
        assert_eq!(idx, 0);
        assert_eq!(mesh.vertex_count(), 1);
    }

    /// Test adding triangles.
    #[test]
    fn test_add_triangle() {
        let mut mesh = Mesh::new();
        let v0 = mesh.add_vertex(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let v1 = mesh.add_vertex(1.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let v2 = mesh.add_vertex(0.5, 1.0, 0.0, 0.0, 0.0, 1.0);
        mesh.add_triangle(v0, v1, v2);
        assert_eq!(mesh.triangle_count(), 1);
    }

    /// Test mesh translation.
    #[test]
    fn test_translate() {
        let mut mesh = Mesh::new();
        mesh.add_vertex(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        mesh.translate(10.0, 20.0, 30.0);
        
        assert!((mesh.vertices[0] - 10.0).abs() < 0.001);
        assert!((mesh.vertices[1] - 20.0).abs() < 0.001);
        assert!((mesh.vertices[2] - 30.0).abs() < 0.001);
    }

    /// Test mesh merging.
    #[test]
    fn test_merge() {
        let mut mesh1 = Mesh::new();
        mesh1.add_vertex(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        
        let mut mesh2 = Mesh::new();
        mesh2.add_vertex(1.0, 1.0, 1.0, 0.0, 0.0, 1.0);
        
        mesh1.merge(&mesh2);
        assert_eq!(mesh1.vertex_count(), 2);
    }
}

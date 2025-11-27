//! # Halfedge Mesh Data Structure
//!
//! Efficient mesh representation for CSG operations.
//! Uses index-based references instead of pointers for Rust safety.
//!
//! ## Structure
//!
//! - **Vertex**: 3D position with outgoing halfedge reference
//! - **Halfedge**: Connects vertices, references face, twin, next, prev
//! - **Face**: References one halfedge in its boundary loop
//!
//! ## Example
//!
//! ```rust,ignore
//! use openscad_mesh::ops::boolean::halfedge::HalfedgeMesh;
//!
//! let mut mesh = HalfedgeMesh::new();
//! let v0 = mesh.add_vertex([0.0, 0.0, 0.0].into());
//! let v1 = mesh.add_vertex([1.0, 0.0, 0.0].into());
//! let v2 = mesh.add_vertex([0.0, 1.0, 0.0].into());
//! mesh.add_face(&[v0, v1, v2]);
//! ```

use glam::DVec3;
use std::collections::HashMap;

/// Index type for vertices in the mesh.
/// 
/// # Example
/// 
/// ```rust,ignore
/// let v: VertexId = VertexId(0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VertexId(pub u32);

/// Index type for halfedges in the mesh.
/// 
/// # Example
/// 
/// ```rust,ignore
/// let he: HalfedgeId = HalfedgeId(0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct HalfedgeId(pub u32);

/// Index type for faces in the mesh.
/// 
/// # Example
/// 
/// ```rust,ignore
/// let f: FaceId = FaceId(0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FaceId(pub u32);

/// Invalid/null reference constant.
const INVALID: u32 = u32::MAX;

/// Vertex in the halfedge mesh.
/// 
/// Stores 3D position and reference to one outgoing halfedge.
/// 
/// # Fields
/// 
/// - `position`: 3D coordinates of the vertex
/// - `halfedge`: One halfedge originating from this vertex (INVALID if isolated)
#[derive(Debug, Clone)]
pub struct Vertex {
    /// 3D position of the vertex
    pub position: DVec3,
    /// One outgoing halfedge from this vertex (INVALID if isolated)
    pub halfedge: HalfedgeId,
}

impl Vertex {
    /// Creates a new vertex at the given position.
    /// 
    /// # Arguments
    /// 
    /// * `position` - 3D coordinates
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let v = Vertex::new([1.0, 2.0, 3.0].into());
    /// ```
    pub fn new(position: DVec3) -> Self {
        Self {
            position,
            halfedge: HalfedgeId(INVALID),
        }
    }
}

/// Halfedge in the mesh.
/// 
/// Connects two vertices and references adjacent structures.
/// 
/// # Fields
/// 
/// - `vertex`: Vertex this halfedge points TO
/// - `twin`: Opposite halfedge (same edge, opposite direction)
/// - `next`: Next halfedge in face loop (CCW)
/// - `prev`: Previous halfedge in face loop (CCW)
/// - `face`: Face this halfedge belongs to
#[derive(Debug, Clone)]
pub struct Halfedge {
    /// Vertex this halfedge points TO
    pub vertex: VertexId,
    /// Twin (opposite) halfedge
    pub twin: HalfedgeId,
    /// Next halfedge in face loop (counter-clockwise)
    pub next: HalfedgeId,
    /// Previous halfedge in face loop (counter-clockwise)
    pub prev: HalfedgeId,
    /// Face this halfedge belongs to (INVALID for boundary)
    pub face: FaceId,
}

impl Default for Halfedge {
    fn default() -> Self {
        Self {
            vertex: VertexId(INVALID),
            twin: HalfedgeId(INVALID),
            next: HalfedgeId(INVALID),
            prev: HalfedgeId(INVALID),
            face: FaceId(INVALID),
        }
    }
}

/// Face in the halfedge mesh.
/// 
/// References one halfedge in its boundary loop.
/// 
/// # Fields
/// 
/// - `halfedge`: One halfedge on this face's boundary
#[derive(Debug, Clone)]
pub struct Face {
    /// One halfedge on this face's boundary
    pub halfedge: HalfedgeId,
}

impl Default for Face {
    fn default() -> Self {
        Self {
            halfedge: HalfedgeId(INVALID),
        }
    }
}

/// Halfedge mesh data structure.
/// 
/// Efficient representation for mesh operations like CSG.
/// Uses contiguous arrays with index-based references.
/// 
/// # Example
/// 
/// ```rust,ignore
/// let mut mesh = HalfedgeMesh::new();
/// let v0 = mesh.add_vertex([0.0, 0.0, 0.0].into());
/// let v1 = mesh.add_vertex([1.0, 0.0, 0.0].into());
/// let v2 = mesh.add_vertex([0.0, 1.0, 0.0].into());
/// mesh.add_face(&[v0, v1, v2]);
/// ```
#[derive(Debug, Clone)]
pub struct HalfedgeMesh {
    /// All vertices in the mesh
    pub vertices: Vec<Vertex>,
    /// All halfedges in the mesh
    pub halfedges: Vec<Halfedge>,
    /// All faces in the mesh
    pub faces: Vec<Face>,
    /// Edge lookup: (v_from, v_to) -> halfedge_id
    edge_map: HashMap<(u32, u32), HalfedgeId>,
}

impl HalfedgeMesh {
    /// Creates a new empty halfedge mesh.
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let mesh = HalfedgeMesh::new();
    /// assert_eq!(mesh.vertex_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            halfedges: Vec::new(),
            faces: Vec::new(),
            edge_map: HashMap::new(),
        }
    }

    /// Adds a vertex to the mesh.
    /// 
    /// # Arguments
    /// 
    /// * `position` - 3D coordinates of the vertex
    /// 
    /// # Returns
    /// 
    /// The ID of the new vertex.
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let v = mesh.add_vertex([1.0, 0.0, 0.0].into());
    /// ```
    pub fn add_vertex(&mut self, position: DVec3) -> VertexId {
        let id = VertexId(self.vertices.len() as u32);
        self.vertices.push(Vertex::new(position));
        id
    }

    /// Adds a face to the mesh given vertex IDs.
    /// 
    /// Creates halfedges and links them appropriately.
    /// 
    /// # Arguments
    /// 
    /// * `vertex_ids` - Slice of vertex IDs forming the face (CCW order)
    /// 
    /// # Returns
    /// 
    /// The ID of the new face, or None if invalid.
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let face = mesh.add_face(&[v0, v1, v2]).unwrap();
    /// ```
    pub fn add_face(&mut self, vertex_ids: &[VertexId]) -> Option<FaceId> {
        if vertex_ids.len() < 3 {
            return None;
        }

        let face_id = FaceId(self.faces.len() as u32);
        let n = vertex_ids.len();

        // Create halfedges for this face
        let first_he_id = HalfedgeId(self.halfedges.len() as u32);
        
        // Pre-allocate halfedges
        for _ in 0..n {
            self.halfedges.push(Halfedge::default());
        }

        // Set up halfedge connectivity
        for i in 0..n {
            let he_id = HalfedgeId(first_he_id.0 + i as u32);
            let next_id = HalfedgeId(first_he_id.0 + ((i + 1) % n) as u32);
            let prev_id = HalfedgeId(first_he_id.0 + ((i + n - 1) % n) as u32);

            let v_from = vertex_ids[i];
            let v_to = vertex_ids[(i + 1) % n];

            let he = &mut self.halfedges[he_id.0 as usize];
            he.vertex = v_to;
            he.next = next_id;
            he.prev = prev_id;
            he.face = face_id;

            // Set vertex's outgoing halfedge if not set
            if self.vertices[v_from.0 as usize].halfedge.0 == INVALID {
                self.vertices[v_from.0 as usize].halfedge = he_id;
            }

            // Check for existing twin halfedge
            let edge_key = (v_to.0, v_from.0);
            if let Some(&twin_id) = self.edge_map.get(&edge_key) {
                self.halfedges[he_id.0 as usize].twin = twin_id;
                self.halfedges[twin_id.0 as usize].twin = he_id;
            }

            // Register this halfedge in the edge map
            self.edge_map.insert((v_from.0, v_to.0), he_id);
        }

        // Create the face
        self.faces.push(Face { halfedge: first_he_id });

        Some(face_id)
    }

    /// Returns the number of vertices in the mesh.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of faces in the mesh.
    #[inline]
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Returns the number of halfedges in the mesh.
    #[inline]
    pub fn halfedge_count(&self) -> usize {
        self.halfedges.len()
    }

    /// Gets the vertex position by ID.
    /// 
    /// # Arguments
    /// 
    /// * `id` - Vertex ID
    /// 
    /// # Returns
    /// 
    /// The 3D position of the vertex.
    #[inline]
    pub fn vertex_position(&self, id: VertexId) -> DVec3 {
        self.vertices[id.0 as usize].position
    }

    /// Gets vertices of a face.
    /// 
    /// # Arguments
    /// 
    /// * `face_id` - Face ID
    /// 
    /// # Returns
    /// 
    /// Vector of vertex IDs forming the face boundary.
    pub fn face_vertices(&self, face_id: FaceId) -> Vec<VertexId> {
        let mut result = Vec::new();
        let start_he = self.faces[face_id.0 as usize].halfedge;
        
        if start_he.0 == INVALID {
            return result;
        }

        let mut current = start_he;
        loop {
            let he = &self.halfedges[current.0 as usize];
            // Get the source vertex (prev's target)
            let prev_he = &self.halfedges[he.prev.0 as usize];
            result.push(prev_he.vertex);
            
            current = he.next;
            if current == start_he {
                break;
            }
        }

        result
    }

    /// Computes the face normal.
    /// 
    /// # Arguments
    /// 
    /// * `face_id` - Face ID
    /// 
    /// # Returns
    /// 
    /// The normalized face normal vector.
    pub fn face_normal(&self, face_id: FaceId) -> DVec3 {
        let verts = self.face_vertices(face_id);
        if verts.len() < 3 {
            return DVec3::Z;
        }

        let p0 = self.vertex_position(verts[0]);
        let p1 = self.vertex_position(verts[1]);
        let p2 = self.vertex_position(verts[2]);

        let e1 = p1 - p0;
        let e2 = p2 - p0;
        e1.cross(e2).normalize_or_zero()
    }

    /// Checks if the mesh is valid.
    /// 
    /// Validates connectivity and references.
    pub fn is_valid(&self) -> bool {
        // Check all halfedges have valid references
        for (i, he) in self.halfedges.iter().enumerate() {
            if he.vertex.0 == INVALID {
                return false;
            }
            if he.next.0 == INVALID || he.prev.0 == INVALID {
                return false;
            }
            if he.face.0 == INVALID {
                return false;
            }

            // Check next/prev consistency
            let next = &self.halfedges[he.next.0 as usize];
            if next.prev.0 != i as u32 {
                return false;
            }
        }

        true
    }
}

impl Default for HalfedgeMesh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_mesh() {
        let mesh = HalfedgeMesh::new();
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.face_count(), 0);
    }

    #[test]
    fn test_add_vertex() {
        let mut mesh = HalfedgeMesh::new();
        let v0 = mesh.add_vertex(DVec3::new(1.0, 2.0, 3.0));
        assert_eq!(v0.0, 0);
        assert_eq!(mesh.vertex_count(), 1);
        assert_eq!(mesh.vertex_position(v0), DVec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_add_triangle_face() {
        let mut mesh = HalfedgeMesh::new();
        let v0 = mesh.add_vertex(DVec3::new(0.0, 0.0, 0.0));
        let v1 = mesh.add_vertex(DVec3::new(1.0, 0.0, 0.0));
        let v2 = mesh.add_vertex(DVec3::new(0.0, 1.0, 0.0));

        let face = mesh.add_face(&[v0, v1, v2]);
        assert!(face.is_some());
        assert_eq!(mesh.face_count(), 1);
        assert_eq!(mesh.halfedge_count(), 3);
    }

    #[test]
    fn test_face_normal() {
        let mut mesh = HalfedgeMesh::new();
        let v0 = mesh.add_vertex(DVec3::new(0.0, 0.0, 0.0));
        let v1 = mesh.add_vertex(DVec3::new(1.0, 0.0, 0.0));
        let v2 = mesh.add_vertex(DVec3::new(0.0, 1.0, 0.0));

        let face = mesh.add_face(&[v0, v1, v2]).unwrap();
        let normal = mesh.face_normal(face);
        
        // Normal should point in +Z direction
        assert!((normal.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_twin_halfedges() {
        let mut mesh = HalfedgeMesh::new();
        let v0 = mesh.add_vertex(DVec3::new(0.0, 0.0, 0.0));
        let v1 = mesh.add_vertex(DVec3::new(1.0, 0.0, 0.0));
        let v2 = mesh.add_vertex(DVec3::new(0.5, 1.0, 0.0));
        let v3 = mesh.add_vertex(DVec3::new(0.5, -1.0, 0.0));

        // Two triangles sharing edge v0-v1
        mesh.add_face(&[v0, v1, v2]);
        mesh.add_face(&[v1, v0, v3]);

        // Check that twins are properly set
        let he_01 = mesh.edge_map.get(&(v0.0, v1.0)).unwrap();
        let he_10 = mesh.edge_map.get(&(v1.0, v0.0)).unwrap();

        assert_eq!(mesh.halfedges[he_01.0 as usize].twin, *he_10);
        assert_eq!(mesh.halfedges[he_10.0 as usize].twin, *he_01);
    }
}

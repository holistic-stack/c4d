//! # HalfEdge Mesh
//!
//! Compact half-edge mesh representation for topology operations.
//!
//! ## Overview
//!
//! The half-edge data structure represents mesh topology efficiently:
//! - Each edge is split into two "half-edges" with opposite directions
//! - Each half-edge stores: start vertex, paired half-edge, face
//! - Enables O(1) adjacency queries
//!
//! ## Memory Layout
//!
//! ```text
//! HalfEdge: [startVert, endVert, pairedHalfedge, face]
//! Vertex:   [x, y, z, halfedge]
//! Face:     [halfedge]
//! ```

// =============================================================================
// HALFEDGE STRUCT
// =============================================================================

/// Index type for half-edges, vertices, and faces.
///
/// Uses u32 for compact memory layout (4 bytes vs 8 for usize).
pub type HalfEdgeId = u32;
pub type VertexId = u32;
pub type FaceId = u32;

/// Invalid index sentinel value.
///
/// Used to indicate "no connection" in the mesh topology.
pub const INVALID_ID: u32 = u32::MAX;

/// Single half-edge in the mesh.
///
/// A half-edge represents one direction of an edge. Each edge in the mesh
/// has two half-edges pointing in opposite directions.
#[derive(Debug, Clone, Copy, Default)]
pub struct HalfEdge {
    /// Starting vertex of this half-edge.
    pub start_vert: VertexId,
    
    /// Ending vertex of this half-edge.
    pub end_vert: VertexId,
    
    /// Paired half-edge (opposite direction).
    ///
    /// INVALID_ID if this is a boundary edge.
    pub pair: HalfEdgeId,
    
    /// Face this half-edge belongs to.
    ///
    /// INVALID_ID if this is a boundary edge.
    pub face: FaceId,
    
    /// Next half-edge around the face (counter-clockwise).
    pub next: HalfEdgeId,
}

/// Vertex in the half-edge mesh.
#[derive(Debug, Clone, Copy, Default)]
pub struct HalfEdgeVertex {
    /// Position x coordinate.
    pub x: f32,
    /// Position y coordinate.
    pub y: f32,
    /// Position z coordinate.
    pub z: f32,
    
    /// One outgoing half-edge from this vertex.
    ///
    /// Any outgoing half-edge works; used as starting point for traversal.
    pub halfedge: HalfEdgeId,
}

/// Face in the half-edge mesh.
#[derive(Debug, Clone, Copy, Default)]
pub struct HalfEdgeFace {
    /// One half-edge on this face's boundary.
    ///
    /// Follow `next` pointers to traverse all edges.
    pub halfedge: HalfEdgeId,
}

// =============================================================================
// HALFEDGE MESH
// =============================================================================

/// Compact half-edge mesh for topology operations.
///
/// Provides efficient adjacency queries and topology traversal.
///
/// ## Example
///
/// ```rust
/// use manifold_rs::mesh::halfedge::HalfEdgeMesh;
///
/// let he_mesh = HalfEdgeMesh::new();
/// assert!(he_mesh.is_empty());
/// ```
#[derive(Debug, Clone, Default)]
pub struct HalfEdgeMesh {
    /// All half-edges in the mesh.
    pub halfedges: Vec<HalfEdge>,
    
    /// All vertices in the mesh.
    pub vertices: Vec<HalfEdgeVertex>,
    
    /// All faces in the mesh.
    pub faces: Vec<HalfEdgeFace>,
}

impl HalfEdgeMesh {
    /// Create empty half-edge mesh.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use manifold_rs::mesh::halfedge::HalfEdgeMesh;
    ///
    /// let mesh = HalfEdgeMesh::new();
    /// assert!(mesh.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if mesh is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    /// Get number of vertices.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get number of faces.
    #[must_use]
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Get number of half-edges.
    #[must_use]
    pub fn halfedge_count(&self) -> usize {
        self.halfedges.len()
    }

    /// Add a vertex to the mesh.
    ///
    /// ## Parameters
    ///
    /// - `x, y, z`: Vertex position
    ///
    /// ## Returns
    ///
    /// Vertex ID for use in face definitions.
    pub fn add_vertex(&mut self, x: f32, y: f32, z: f32) -> VertexId {
        let id = self.vertices.len() as VertexId;
        self.vertices.push(HalfEdgeVertex {
            x,
            y,
            z,
            halfedge: INVALID_ID,
        });
        id
    }

    /// Iterate over all half-edges leaving a vertex.
    ///
    /// ## Parameters
    ///
    /// - `vertex_id`: Starting vertex
    ///
    /// ## Returns
    ///
    /// Iterator over outgoing half-edge IDs.
    pub fn vertex_halfedges(&self, vertex_id: VertexId) -> impl Iterator<Item = HalfEdgeId> + '_ {
        VertexHalfEdgeIterator {
            mesh: self,
            start: self.vertices[vertex_id as usize].halfedge,
            current: self.vertices[vertex_id as usize].halfedge,
            first: true,
        }
    }

    /// Iterate over all half-edges around a face.
    ///
    /// ## Parameters
    ///
    /// - `face_id`: Face to iterate
    ///
    /// ## Returns
    ///
    /// Iterator over half-edge IDs on the face boundary.
    pub fn face_halfedges(&self, face_id: FaceId) -> impl Iterator<Item = HalfEdgeId> + '_ {
        FaceHalfEdgeIterator {
            mesh: self,
            start: self.faces[face_id as usize].halfedge,
            current: self.faces[face_id as usize].halfedge,
            first: true,
        }
    }
}

// =============================================================================
// ITERATORS
// =============================================================================

/// Iterator over half-edges leaving a vertex.
struct VertexHalfEdgeIterator<'a> {
    mesh: &'a HalfEdgeMesh,
    start: HalfEdgeId,
    current: HalfEdgeId,
    first: bool,
}

impl<'a> Iterator for VertexHalfEdgeIterator<'a> {
    type Item = HalfEdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == INVALID_ID {
            return None;
        }
        
        if !self.first && self.current == self.start {
            return None;
        }
        
        self.first = false;
        let result = self.current;
        
        // Move to next outgoing half-edge: pair -> next
        let pair = self.mesh.halfedges[self.current as usize].pair;
        if pair != INVALID_ID {
            self.current = self.mesh.halfedges[pair as usize].next;
        } else {
            self.current = INVALID_ID;
        }
        
        Some(result)
    }
}

/// Iterator over half-edges around a face.
struct FaceHalfEdgeIterator<'a> {
    mesh: &'a HalfEdgeMesh,
    start: HalfEdgeId,
    current: HalfEdgeId,
    first: bool,
}

impl<'a> Iterator for FaceHalfEdgeIterator<'a> {
    type Item = HalfEdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == INVALID_ID {
            return None;
        }
        
        if !self.first && self.current == self.start {
            return None;
        }
        
        self.first = false;
        let result = self.current;
        self.current = self.mesh.halfedges[self.current as usize].next;
        
        Some(result)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test creating empty half-edge mesh.
    #[test]
    fn test_halfedge_mesh_new() {
        let mesh = HalfEdgeMesh::new();
        assert!(mesh.is_empty());
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.face_count(), 0);
    }

    /// Test adding vertices.
    #[test]
    fn test_add_vertex() {
        let mut mesh = HalfEdgeMesh::new();
        let v0 = mesh.add_vertex(1.0, 2.0, 3.0);
        assert_eq!(v0, 0);
        assert_eq!(mesh.vertex_count(), 1);
    }

    /// Test invalid ID constant.
    #[test]
    fn test_invalid_id() {
        assert_eq!(INVALID_ID, u32::MAX);
    }
}

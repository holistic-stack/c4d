//! Core half-edge data structures for manifold mesh representation
//! 
//! This module implements the fundamental half-edge mesh data structures
//! using index-based references for memory efficiency and cache performance.

use crate::core::vec3::{Vec3, BoundingBox};
use thiserror::Error;

/// Unique identifier for vertices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VertexId(pub u32);

/// Unique identifier for half-edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HalfEdgeId(pub u32);

/// Unique identifier for faces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FaceId(pub u32);

/// Unique identifier for edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeId(pub u32);

/// Vertex data structure
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    /// Position in 3D space
    pub position: Vec3,
    /// Outgoing half-edge (arbitrary if multiple)
    pub halfedge: Option<HalfEdgeId>,
    /// Whether this vertex is on a boundary
    pub is_boundary: bool,
}

impl Vertex {
    /// Creates a new vertex at the given position
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            halfedge: None,
            is_boundary: false,
        }
    }
}

/// Half-edge data structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HalfEdge {
    /// Starting vertex of this half-edge
    pub vertex: VertexId,
    /// Opposite half-edge (twin)
    pub twin: Option<HalfEdgeId>,
    /// Next half-edge in the face
    pub next: Option<HalfEdgeId>,
    /// Face this half-edge belongs to
    pub face: Option<FaceId>,
    /// Edge this half-edge belongs to
    pub edge: EdgeId,
}

impl HalfEdge {
    /// Creates a new half-edge
    pub fn new(vertex: VertexId, edge: EdgeId) -> Self {
        Self {
            vertex,
            twin: None,
            next: None,
            face: None,
            edge,
        }
    }
}

/// Face data structure
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    /// One half-edge bordering this face
    pub halfedge: Option<HalfEdgeId>,
    /// Face normal (cached)
    pub normal: Option<Vec3>,
    /// Face area (cached)
    pub area: Option<f64>,
    /// Whether this face is a boundary face
    pub is_boundary: bool,
}

impl Face {
    /// Creates a new face
    pub fn new() -> Self {
        Self {
            halfedge: None,
            normal: None,
            area: None,
            is_boundary: false,
        }
    }
}

/// Edge data structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    /// Half-edges that make up this edge
    pub halfedges: [HalfEdgeId; 2],
    /// Whether this edge is on a boundary
    pub is_boundary: bool,
}

impl Edge {
    /// Creates a new edge from two half-edges
    pub fn new(h0: HalfEdgeId, h1: HalfEdgeId) -> Self {
        Self {
            halfedges: [h0, h1],
            is_boundary: false,
        }
    }
}

/// Topology errors that can occur in half-edge meshes
#[derive(Debug, Error, PartialEq)]
pub enum TopologyError {
    #[error("Invalid vertex reference: {0:?}")]
    InvalidVertex(VertexId),
    
    #[error("Invalid half-edge reference: {0:?}")]
    InvalidHalfEdge(HalfEdgeId),
    
    #[error("Invalid face reference: {0:?}")]
    InvalidFace(FaceId),
    
    #[error("Invalid edge reference: {0:?}")]
    InvalidEdge(EdgeId),
    
    #[error("Non-manifold vertex detected at {0:?}")]
    NonManifoldVertex(VertexId),
    
    #[error("Non-manifold edge detected at {0:?}")]
    NonManifoldEdge(EdgeId),
    
    #[error("Inconsistent half-edge topology")]
    InconsistentTopology,
    
    #[error("Mesh is not closed (has boundary edges)")]
    OpenMesh,
}

/// Result type for topology operations
pub type TopologyResult<T> = Result<T, TopologyError>;

/// Main half-edge mesh data structure
#[derive(Debug, Clone)]
pub struct HalfEdgeMesh {
    /// Vertex storage
    vertices: Vec<Vertex>,
    /// Half-edge storage
    half_edges: Vec<HalfEdge>,
    /// Face storage
    faces: Vec<Face>,
    /// Edge storage
    edges: Vec<Edge>,
}

impl HalfEdgeMesh {
    /// Creates a new empty mesh
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            half_edges: Vec::new(),
            faces: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Creates a new mesh with specified capacity
    pub fn with_capacity(vertices: usize, half_edges: usize, faces: usize, edges: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(vertices),
            half_edges: Vec::with_capacity(half_edges),
            faces: Vec::with_capacity(faces),
            edges: Vec::with_capacity(edges),
        }
    }

    /// Adds a vertex to the mesh
    pub fn add_vertex(&mut self, position: Vec3) -> VertexId {
        let id = VertexId(self.vertices.len() as u32);
        self.vertices.push(Vertex::new(position));
        id
    }

    /// Adds a half-edge to the mesh
    pub fn add_half_edge(&mut self, vertex: VertexId, edge: EdgeId) -> HalfEdgeId {
        let id = HalfEdgeId(self.half_edges.len() as u32);
        self.half_edges.push(HalfEdge::new(vertex, edge));
        id
    }

    /// Adds a face to the mesh
    pub fn add_face(&mut self) -> FaceId {
        let id = FaceId(self.faces.len() as u32);
        self.faces.push(Face::new());
        id
    }

    /// Adds an edge to the mesh
    pub fn add_edge(&mut self, h0: HalfEdgeId, h1: HalfEdgeId) -> EdgeId {
        let id = EdgeId(self.edges.len() as u32);
        self.edges.push(Edge::new(h0, h1));
        id
    }

    /// Gets a vertex by ID
    pub fn vertex(&self, id: VertexId) -> TopologyResult<&Vertex> {
        self.vertices
            .get(id.0 as usize)
            .ok_or(TopologyError::InvalidVertex(id))
    }

    /// Gets a mutable vertex by ID
    pub fn vertex_mut(&mut self, id: VertexId) -> TopologyResult<&mut Vertex> {
        self.vertices
            .get_mut(id.0 as usize)
            .ok_or(TopologyError::InvalidVertex(id))
    }

    /// Gets a half-edge by ID
    pub fn half_edge(&self, id: HalfEdgeId) -> TopologyResult<&HalfEdge> {
        self.half_edges
            .get(id.0 as usize)
            .ok_or(TopologyError::InvalidHalfEdge(id))
    }

    /// Gets a mutable half-edge by ID
    pub fn half_edge_mut(&mut self, id: HalfEdgeId) -> TopologyResult<&mut HalfEdge> {
        self.half_edges
            .get_mut(id.0 as usize)
            .ok_or(TopologyError::InvalidHalfEdge(id))
    }

    /// Gets a face by ID
    pub fn face(&self, id: FaceId) -> TopologyResult<&Face> {
        self.faces
            .get(id.0 as usize)
            .ok_or(TopologyError::InvalidFace(id))
    }

    /// Gets a mutable face by ID
    pub fn face_mut(&mut self, id: FaceId) -> TopologyResult<&mut Face> {
        self.faces
            .get_mut(id.0 as usize)
            .ok_or(TopologyError::InvalidFace(id))
    }

    /// Gets an edge by ID
    pub fn edge(&self, id: EdgeId) -> TopologyResult<&Edge> {
        self.edges
            .get(id.0 as usize)
            .ok_or(TopologyError::InvalidEdge(id))
    }

    /// Gets a mutable edge by ID
    pub fn edge_mut(&mut self, id: EdgeId) -> TopologyResult<&mut Edge> {
        self.edges
            .get_mut(id.0 as usize)
            .ok_or(TopologyError::InvalidEdge(id))
    }

    /// Returns the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of half-edges
    pub fn half_edge_count(&self) -> usize {
        self.half_edges.len()
    }

    /// Returns the number of faces
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Returns the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Validates basic mesh topology
    pub fn validate_topology(&self) -> TopologyResult<()> {
        // Check that all half-edge references are valid
        for (i, he) in self.half_edges.iter().enumerate() {
            let he_id = HalfEdgeId(i as u32);
            
            // Check vertex reference
            if he.vertex.0 as usize >= self.vertices.len() {
                return Err(TopologyError::InvalidVertex(he.vertex));
            }
            
            // Check twin reference
            if let Some(twin) = he.twin {
                if twin.0 as usize >= self.half_edges.len() {
                    return Err(TopologyError::InvalidHalfEdge(twin));
                }
                // Check that twin relationship is symmetric
                let twin_he = self.half_edge(twin)?;
                if twin_he.twin != Some(he_id) {
                    return Err(TopologyError::InconsistentTopology);
                }
            }
            
            // Check next reference
            if let Some(next) = he.next {
                if next.0 as usize >= self.half_edges.len() {
                    return Err(TopologyError::InvalidHalfEdge(next));
                }
            }
            
            // Check face reference
            if let Some(face) = he.face {
                if face.0 as usize >= self.faces.len() {
                    return Err(TopologyError::InvalidFace(face));
                }
            }
            
            // Check edge reference
            if he.edge.0 as usize >= self.edges.len() {
                return Err(TopologyError::InvalidEdge(he.edge));
            }
        }
        
        // Check that all face references are valid
        for (i, face) in self.faces.iter().enumerate() {
            if let Some(he) = face.halfedge {
                if he.0 as usize >= self.half_edges.len() {
                    return Err(TopologyError::InvalidHalfEdge(he));
                }
                // Check that the half-edge points back to this face
                let he_face = self.half_edge(he)?.face;
                if he_face != Some(FaceId(i as u32)) {
                    return Err(TopologyError::InconsistentTopology);
                }
            }
        }
        
        // Check that all edge references are valid
        for (i, edge) in self.edges.iter().enumerate() {
            let edge_id = EdgeId(i as u32);
            for &he_id in &edge.halfedges {
                if he_id.0 as usize >= self.half_edges.len() {
                    return Err(TopologyError::InvalidHalfEdge(he_id));
                }
                // Check that the half-edge points back to this edge
                let he_edge = self.half_edge(he_id)?.edge;
                if he_edge != edge_id {
                    return Err(TopologyError::InconsistentTopology);
                }
            }
        }
        
        Ok(())
    }

    /// Checks if the mesh is closed (no boundary edges)
    pub fn is_closed(&self) -> bool {
        self.half_edges.iter().all(|he| he.twin.is_some())
    }

    /// Returns the bounding box of all vertices
    pub fn bounding_box(&self) -> BoundingBox {
        if self.vertices.is_empty() {
            BoundingBox::empty()
        } else {
            let mut bbox = BoundingBox::new(self.vertices[0].position, self.vertices[0].position);
            for vertex in &self.vertices[1..] {
                bbox.expand(vertex.position);
            }
            bbox
        }
    }

    /// Triangulates the mesh and returns a flat vector of vertex coordinates.
    /// Each triangle is represented by 9 f64 values (3 vertices * 3 coordinates).
    pub fn triangulate(&self) -> Vec<f64> {
        let mut vertices = Vec::new();
        
        for i in 0..self.face_count() {
            let face_id = FaceId(i as u32);
            if let Ok(face) = self.face(face_id) {
                if let Some(start_he_id) = face.halfedge {
                    let mut current_he_id = start_he_id;
                    let mut face_vertices = Vec::new();
                    
                    // Collect vertices for this face
                    loop {
                        if let Ok(he) = self.half_edge(current_he_id) {
                            if let Ok(vertex) = self.vertex(he.vertex) {
                                face_vertices.push(vertex.position);
                            }
                            
                            if let Some(next_he_id) = he.next {
                                current_he_id = next_he_id;
                                if current_he_id == start_he_id {
                                    break;
                                }
                            } else {
                                break; // Should not happen in valid mesh
                            }
                        } else {
                            break;
                        }
                    }
                    
                    // Triangulate face (simple fan for convex faces)
                    if face_vertices.len() >= 3 {
                        // Triangle fan from first vertex
                        let v0 = face_vertices[0];
                        for i in 1..face_vertices.len() - 1 {
                            let v1 = face_vertices[i];
                            let v2 = face_vertices[i + 1];
                            
                            vertices.push(v0.x); vertices.push(v0.y); vertices.push(v0.z);
                            vertices.push(v1.x); vertices.push(v1.y); vertices.push(v1.z);
                            vertices.push(v2.x); vertices.push(v2.y); vertices.push(v2.z);
                        }
                    }
                }
            }
        }
        vertices
    }
}

impl Default for HalfEdgeMesh {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let vertex = Vertex::new(pos);
        
        assert_eq!(vertex.position, pos);
        assert_eq!(vertex.halfedge, None);
        assert_eq!(vertex.is_boundary, false);
    }

    #[test]
    fn test_half_edge_creation() {
        let v0 = VertexId(0);
        let e0 = EdgeId(0);
        let he = HalfEdge::new(v0, e0);
        
        assert_eq!(he.vertex, v0);
        assert_eq!(he.edge, e0);
        assert_eq!(he.twin, None);
        assert_eq!(he.next, None);
        assert_eq!(he.face, None);
    }

    #[test]
    fn test_face_creation() {
        let face = Face::new();
        
        assert_eq!(face.halfedge, None);
        assert_eq!(face.normal, None);
        assert_eq!(face.area, None);
        assert_eq!(face.is_boundary, false);
    }

    #[test]
    fn test_edge_creation() {
        let h0 = HalfEdgeId(0);
        let h1 = HalfEdgeId(1);
        let edge = Edge::new(h0, h1);
        
        assert_eq!(edge.halfedges[0], h0);
        assert_eq!(edge.halfedges[1], h1);
        assert_eq!(edge.is_boundary, false);
    }

    #[test]
    fn test_topology_error_display() {
        let error = TopologyError::InvalidVertex(VertexId(42));
        assert_eq!(error.to_string(), "Invalid vertex reference: VertexId(42)");
    }

    #[test]
    fn test_half_edge_mesh_creation() {
        let mesh = HalfEdgeMesh::new();
        
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.half_edge_count(), 0);
        assert_eq!(mesh.face_count(), 0);
        assert_eq!(mesh.edge_count(), 0);
    }

    #[test]
    fn test_half_edge_mesh_with_capacity() {
        let mesh = HalfEdgeMesh::with_capacity(10, 20, 5, 15);
        
        assert_eq!(mesh.vertex_count(), 0);
        assert_eq!(mesh.half_edge_count(), 0);
        assert_eq!(mesh.face_count(), 0);
        assert_eq!(mesh.edge_count(), 0);
        
        // Capacity should be set (though we can't easily test this)
        assert!(mesh.vertices.capacity() >= 10);
        assert!(mesh.half_edges.capacity() >= 20);
        assert!(mesh.faces.capacity() >= 5);
        assert!(mesh.edges.capacity() >= 15);
    }

    #[test]
    fn test_add_vertex() {
        let mut mesh = HalfEdgeMesh::new();
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let v_id = mesh.add_vertex(pos);
        
        assert_eq!(v_id, VertexId(0));
        assert_eq!(mesh.vertex_count(), 1);
        
        let vertex = mesh.vertex(v_id).unwrap();
        assert_eq!(vertex.position, pos);
        assert_eq!(vertex.halfedge, None);
    }

    #[test]
    fn test_add_half_edge() {
        let mut mesh = HalfEdgeMesh::new();
        let v_id = mesh.add_vertex(Vec3::new(0.0, 0.0, 0.0));
        let e_id = EdgeId(0);
        let he_id = mesh.add_half_edge(v_id, e_id);
        
        assert_eq!(he_id, HalfEdgeId(0));
        assert_eq!(mesh.half_edge_count(), 1);
        
        let he = mesh.half_edge(he_id).unwrap();
        assert_eq!(he.vertex, v_id);
        assert_eq!(he.edge, e_id);
        assert_eq!(he.twin, None);
    }

    #[test]
    fn test_add_face() {
        let mut mesh = HalfEdgeMesh::new();
        let f_id = mesh.add_face();
        
        assert_eq!(f_id, FaceId(0));
        assert_eq!(mesh.face_count(), 1);
        
        let face = mesh.face(f_id).unwrap();
        assert_eq!(face.halfedge, None);
        assert_eq!(face.normal, None);
    }

    #[test]
    fn test_add_edge() {
        let mut mesh = HalfEdgeMesh::new();
        let v_id = mesh.add_vertex(Vec3::new(0.0, 0.0, 0.0));
        let he0 = mesh.add_half_edge(v_id, EdgeId(0));
        let he1 = mesh.add_half_edge(v_id, EdgeId(0));
        let e_id = mesh.add_edge(he0, he1);
        
        assert_eq!(e_id, EdgeId(0));
        assert_eq!(mesh.edge_count(), 1);
        
        let edge = mesh.edge(e_id).unwrap();
        assert_eq!(edge.halfedges[0], he0);
        assert_eq!(edge.halfedges[1], he1);
    }

    #[test]
    fn test_access_invalid_ids() {
        let mesh = HalfEdgeMesh::new();
        
        assert!(mesh.vertex(VertexId(0)).is_err());
        assert!(mesh.half_edge(HalfEdgeId(0)).is_err());
        assert!(mesh.face(FaceId(0)).is_err());
        assert!(mesh.edge(EdgeId(0)).is_err());
        
        assert_eq!(
            mesh.vertex(VertexId(0)).unwrap_err().to_string(),
            "Invalid vertex reference: VertexId(0)"
        );
    }

    #[test]
    fn test_empty_mesh_validation() {
        let mesh = HalfEdgeMesh::new();
        assert!(mesh.validate_topology().is_ok());
    }

    #[test]
    fn test_simple_mesh_validation() {
        let mut mesh = HalfEdgeMesh::new();
        let v0 = mesh.add_vertex(Vec3::new(0.0, 0.0, 0.0));
        let v1 = mesh.add_vertex(Vec3::new(1.0, 0.0, 0.0));
        let v2 = mesh.add_vertex(Vec3::new(0.0, 1.0, 0.0));
        
        let e0 = mesh.add_edge(HalfEdgeId(0), HalfEdgeId(1));
        let e1 = mesh.add_edge(HalfEdgeId(2), HalfEdgeId(3));
        let e2 = mesh.add_edge(HalfEdgeId(4), HalfEdgeId(5));
        
        let he0 = mesh.add_half_edge(v0, e0);
        let _he1 = mesh.add_half_edge(v1, e0);
        let he2 = mesh.add_half_edge(v1, e1);
        let _he3 = mesh.add_half_edge(v2, e1);
        let he4 = mesh.add_half_edge(v2, e2);
        let _he5 = mesh.add_half_edge(v0, e2);
        
        let f0 = mesh.add_face();
        
        // Set up basic connectivity (this is a simplified triangle)
        mesh.half_edge_mut(he0).unwrap().next = Some(he2);
        mesh.half_edge_mut(he2).unwrap().next = Some(he4);
        mesh.half_edge_mut(he4).unwrap().next = Some(he0);
        
        mesh.half_edge_mut(he0).unwrap().face = Some(f0);
        mesh.half_edge_mut(he2).unwrap().face = Some(f0);
        mesh.half_edge_mut(he4).unwrap().face = Some(f0);
        
        mesh.face_mut(f0).unwrap().halfedge = Some(he0);
        
        // This should pass basic validation
        assert!(mesh.validate_topology().is_ok());
    }

    #[test]
    fn test_invalid_reference_validation() {
        let mut mesh = HalfEdgeMesh::new();
        let _v0 = mesh.add_vertex(Vec3::new(0.0, 0.0, 0.0));
        
        // Add a half-edge with invalid vertex reference
        let _he0 = mesh.add_half_edge(VertexId(999), EdgeId(0));
        
        // Validation should fail due to invalid vertex reference
        let result = mesh.validate_topology();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid vertex reference: VertexId(999)"
        );
    }

    #[test]
    fn test_empty_mesh_is_closed() {
        let mesh = HalfEdgeMesh::new();
        assert!(mesh.is_closed()); // Empty mesh is considered closed
    }

    #[test]
    fn test_mesh_with_boundary_edges() {
        let mut mesh = HalfEdgeMesh::new();
        let v0 = mesh.add_vertex(Vec3::new(0.0, 0.0, 0.0));
        let _v1 = mesh.add_vertex(Vec3::new(1.0, 0.0, 0.0));
        
        // Add a half-edge without a twin (boundary edge)
        let _he0 = mesh.add_half_edge(v0, EdgeId(0));
        
        assert!(!mesh.is_closed());
    }

    #[test]
    fn test_empty_mesh_bounding_box() {
        let mesh = HalfEdgeMesh::new();
        let bbox = mesh.bounding_box();
        // Empty mesh should have an invalid bounding box
        assert!(!bbox.is_valid());
    }

    #[test]
    fn test_single_vertex_bounding_box() {
        let mut mesh = HalfEdgeMesh::new();
        let pos = Vec3::new(1.0, 2.0, 3.0);
        mesh.add_vertex(pos);
        
        let bbox = mesh.bounding_box();
        assert_eq!(bbox.min, pos);
        assert_eq!(bbox.max, pos);
    }

    #[test]
    fn test_multiple_vertices_bounding_box() {
        let mut mesh = HalfEdgeMesh::new();
        let _v0 = mesh.add_vertex(Vec3::new(0.0, 0.0, 0.0));
        let _v1 = mesh.add_vertex(Vec3::new(1.0, 2.0, 3.0));
        let _v2 = mesh.add_vertex(Vec3::new(-1.0, -2.0, -3.0));
        
        let bbox = mesh.bounding_box();
        assert_eq!(bbox.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(bbox.max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_default_impl() {
        let mesh1 = HalfEdgeMesh::default();
        let mesh2 = HalfEdgeMesh::new();
        
        assert_eq!(mesh1.vertex_count(), mesh2.vertex_count());
        assert_eq!(mesh1.half_edge_count(), mesh2.half_edge_count());
        assert_eq!(mesh1.face_count(), mesh2.face_count());
        assert_eq!(mesh1.edge_count(), mesh2.edge_count());
    }
}
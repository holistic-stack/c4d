use crate::core::vec3::Vec3;

/// Invalid index sentinel value.
pub const NO_INDEX: u32 = u32::MAX;

/// A vertex in the half-edge data structure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// The 3D position of the vertex.
    pub position: Vec3,
    /// Index of the first half-edge starting from this vertex.
    pub first_edge: u32,
}

impl Vertex {
    /// Creates a new vertex.
    pub fn new(position: Vec3, first_edge: u32) -> Self {
        Self {
            position,
            first_edge,
        }
    }
}

/// A half-edge in the half-edge data structure.
///
/// A half-edge is a directed edge. Each edge in the mesh is represented by two
/// half-edges, one for each face sharing the edge (or one for the boundary if open).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HalfEdge {
    /// Index of the vertex at the start of this half-edge.
    pub start_vert: u32,
    /// Index of the vertex at the end of this half-edge.
    pub end_vert: u32,
    /// Index of the next half-edge in the loop around the face.
    pub next_edge: u32,
    /// Index of the paired half-edge (going in the opposite direction).
    pub pair_edge: u32,
    /// Index of the face this half-edge belongs to.
    pub face: u32,
}

impl HalfEdge {
    /// Creates a new half-edge.
    pub fn new(
        start_vert: u32,
        end_vert: u32,
        next_edge: u32,
        pair_edge: u32,
        face: u32,
    ) -> Self {
        Self {
            start_vert,
            end_vert,
            next_edge,
            pair_edge,
            face,
        }
    }
}

/// A face in the half-edge data structure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Face {
    /// Index of the first half-edge in the face loop.
    pub first_edge: u32,
    /// The normal vector of the face.
    pub normal: Vec3,
}

impl Face {
    /// Creates a new face.
    pub fn new(first_edge: u32, normal: Vec3) -> Self {
        Self { first_edge, normal }
    }
}

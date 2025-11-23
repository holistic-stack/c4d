use crate::core::ds::{Face, HalfEdge, Vertex};
use crate::error::Error;
use glam::DVec4;

/// A manifold mesh represented by a half-edge data structure.
#[derive(Debug, Default, Clone)]
pub struct Manifold {
    pub vertices: Vec<Vertex>,
    pub half_edges: Vec<HalfEdge>,
    pub faces: Vec<Face>,
    /// Optional color for the entire mesh (RGBA).
    pub color: Option<DVec4>,
}

/// Helper enum for boolean operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BooleanOp {
    Union,
    Difference,
    Intersection,
}

impl Manifold {
    /// Creates a new empty manifold.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the color of the manifold.
    pub fn with_color(mut self, color: DVec4) -> Self {
        self.color = Some(color);
        self
    }

    /// Performs a boolean operation with another manifold.
    pub fn boolean(&self, other: &Manifold, op: BooleanOp) -> Result<Manifold, Error> {
        match op {
            BooleanOp::Union => {
                // Trivial union (append) logic for now.
                let mut result = self.clone();

                let vert_offset = result.vertices.len() as u32;
                let edge_offset = result.half_edges.len() as u32;

                // Append vertices
                for v in &other.vertices {
                    let mut new_v = *v;
                    new_v.first_edge += edge_offset;
                    result.vertices.push(new_v);
                }

                // Append faces
                for f in &other.faces {
                    let mut new_f = *f;
                    new_f.first_edge += edge_offset;
                    result.faces.push(new_f);
                }

                // Append half-edges
                for e in &other.half_edges {
                    let mut new_e = *e;
                    new_e.start_vert += vert_offset;
                    new_e.end_vert += vert_offset;
                    new_e.next_edge += edge_offset;
                    new_e.pair_edge += edge_offset;
                    new_e.face += self.faces.len() as u32;
                    result.half_edges.push(new_e);
                }

                // Color logic
                if result.color.is_none() {
                    result.color = other.color;
                }

                Ok(result)
            }
            _ => Err(Error::InvalidGeometry {
                message: "Boolean difference/intersection not yet implemented".to_string(),
            }),
        }
    }

    /// Validates the integrity of the half-edge data structure.
    pub fn validate(&self) -> Result<(), Error> {
        // Check vertex indices
        for (i, vert) in self.vertices.iter().enumerate() {
            if vert.first_edge >= self.half_edges.len() as u32 {
                return Err(Error::IndexOutOfBounds(format!(
                    "Vertex {} points to invalid edge {}",
                    i, vert.first_edge
                )));
            }
            // Verify the edge actually starts at this vertex
            if self.half_edges[vert.first_edge as usize].start_vert != i as u32 {
                return Err(Error::InvalidTopology(format!(
                    "Vertex {} points to edge {} which starts at {}",
                    i, vert.first_edge, self.half_edges[vert.first_edge as usize].start_vert
                )));
            }
        }

        // Check half-edge indices
        for (i, edge) in self.half_edges.iter().enumerate() {
            if edge.start_vert >= self.vertices.len() as u32 {
                return Err(Error::IndexOutOfBounds(format!(
                    "Edge {} start_vert {} out of bounds",
                    i, edge.start_vert
                )));
            }
            if edge.end_vert >= self.vertices.len() as u32 {
                return Err(Error::IndexOutOfBounds(format!(
                    "Edge {} end_vert {} out of bounds",
                    i, edge.end_vert
                )));
            }
            if edge.next_edge >= self.half_edges.len() as u32 {
                return Err(Error::IndexOutOfBounds(format!(
                    "Edge {} next_edge {} out of bounds",
                    i, edge.next_edge
                )));
            }
            if edge.pair_edge >= self.half_edges.len() as u32 {
                return Err(Error::IndexOutOfBounds(format!(
                    "Edge {} pair_edge {} out of bounds",
                    i, edge.pair_edge
                )));
            }
            if edge.face >= self.faces.len() as u32 {
                return Err(Error::IndexOutOfBounds(format!(
                    "Edge {} face {} out of bounds",
                    i, edge.face
                )));
            }

            // Verify pairing
            let pair = &self.half_edges[edge.pair_edge as usize];
            if pair.pair_edge != i as u32 {
                return Err(Error::InvalidTopology(format!(
                    "Edge {} is paired with {}, but that edge is paired with {}",
                    i, edge.pair_edge, pair.pair_edge
                )));
            }
            if pair.start_vert != edge.end_vert || pair.end_vert != edge.start_vert {
                return Err(Error::InvalidTopology(format!(
                    "Edge {} and its pair {} do not match vertices",
                    i, edge.pair_edge
                )));
            }
        }

        // Check face indices
        for (i, face) in self.faces.iter().enumerate() {
            if face.first_edge >= self.half_edges.len() as u32 {
                return Err(Error::IndexOutOfBounds(format!(
                    "Face {} points to invalid edge {}",
                    i, face.first_edge
                )));
            }
            // Verify the edge actually belongs to this face
            if self.half_edges[face.first_edge as usize].face != i as u32 {
                return Err(Error::InvalidTopology(format!(
                    "Face {} points to edge {} which belongs to face {}",
                    i, face.first_edge, self.half_edges[face.first_edge as usize].face
                )));
            }
        }

        Ok(())
    }

    /// Returns the number of vertices in the manifold.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of faces in the manifold.
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Returns the axis-aligned bounding box of the manifold.
    pub fn bounding_box(&self) -> (crate::Vec3, crate::Vec3) {
        use crate::Vec3;
        
        if self.vertices.is_empty() {
            return (Vec3::ZERO, Vec3::ZERO);
        }

        let first = self.vertices[0].position;
        let mut min = first;
        let mut max = first;

        for vertex in &self.vertices {
            let pos = vertex.position;
            min = Vec3::new(min.x.min(pos.x), min.y.min(pos.y), min.z.min(pos.z));
            max = Vec3::new(max.x.max(pos.x), max.y.max(pos.y), max.z.max(pos.z));
        }

        (min, max)
    }

    /// Applies a transformation matrix to all vertices in the manifold.
    pub fn transform(&mut self, matrix: glam::DMat4) {
        // Transform vertices
        for vertex in &mut self.vertices {
            vertex.position = matrix.transform_point3(vertex.position);
        }

        // Transform normals
        let normal_matrix = matrix.inverse().transpose();
        for face in &mut self.faces {
            let n = face.normal;
            let n4 = normal_matrix * glam::DVec4::new(n.x, n.y, n.z, 0.0);
            face.normal = glam::DVec3::new(n4.x, n4.y, n4.z).normalize_or_zero();
        }

        // Flip winding if negative determinant (reflection/scale -1)
        if matrix.determinant() < 0.0 {
            self.flip_faces();
        }
    }

    fn flip_faces(&mut self) {
        // Reverse the winding order of all faces
        for face in &self.faces {
            let mut edges = Vec::new();
            let mut curr = face.first_edge;
            loop {
                edges.push(curr);
                curr = self.half_edges[curr as usize].next_edge;
                if curr == face.first_edge { break; }
                if edges.len() > self.half_edges.len() { break; }
            }

            // Re-link next pointers in reverse order
            for i in 0..edges.len() {
                let curr = edges[i];
                let next_in_reverse = edges[(i + edges.len() - 1) % edges.len()];
                self.half_edges[curr as usize].next_edge = next_in_reverse;
            }
        }

        // Swap vertices for all edges
        for edge in &mut self.half_edges {
            std::mem::swap(&mut edge.start_vert, &mut edge.end_vert);
        }

        // Update vertex first_edge to ensure validity
        for (i, edge) in self.half_edges.iter().enumerate() {
            self.vertices[edge.start_vert as usize].first_edge = i as u32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vec3::Vec3;

    #[test]
    fn test_empty_manifold_is_valid() {
        let m = Manifold::new();
        assert!(m.validate().is_ok());
    }

    #[test]
    fn test_color_setter() {
        let m = Manifold::new().with_color(DVec4::ONE);
        assert_eq!(m.color, Some(DVec4::ONE));
    }

    #[test]
    fn test_invalid_vertex_index() {
        let mut m = Manifold::new();
        m.vertices.push(Vertex::new(Vec3::ZERO, 100)); // Invalid edge index
        assert!(matches!(
            m.validate(),
            Err(Error::IndexOutOfBounds(_))
        ));
    }

    #[test]
    fn test_invalid_edge_pairing() {
        let mut m = Manifold::new();
        // Create two vertices
        m.vertices.push(Vertex::new(Vec3::ZERO, 0));
        m.vertices.push(Vertex::new(Vec3::new(1.0, 0.0, 0.0), 1));

        // Create a face
        m.faces.push(Face::new(0, Vec3::Z));

        // Create two edges that should be pairs but aren't linked correctly
        m.half_edges.push(HalfEdge::new(0, 1, 0, 1, 0)); // Edge 0
        m.half_edges.push(HalfEdge::new(1, 0, 1, 0, 0)); // Edge 1

        // Break pairing
        m.half_edges[1].pair_edge = 1; // Point to itself instead of 0

        match m.validate() {
            Err(Error::InvalidTopology(msg)) => {
                if !msg.contains("is paired with") {
                    panic!("Unexpected error message: {}", msg);
                }
            }
            Err(e) => panic!("Unexpected error type: {:?}", e),
            Ok(_) => panic!("Validation should have failed"),
        }
    }
}

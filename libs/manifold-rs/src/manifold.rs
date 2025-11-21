use crate::core::ds::{Face, HalfEdge, Vertex};
use crate::error::ManifoldError;

/// A manifold mesh represented by a half-edge data structure.
#[derive(Debug, Default, Clone)]
pub struct Manifold {
    pub vertices: Vec<Vertex>,
    pub half_edges: Vec<HalfEdge>,
    pub faces: Vec<Face>,
}

impl Manifold {
    /// Creates a new empty manifold.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates the integrity of the half-edge data structure.
    ///
    /// Checks for:
    /// - Index bounds
    /// - Half-edge pairing
    /// - Face loops
    pub fn validate(&self) -> Result<(), ManifoldError> {
        // Check vertex indices
        for (i, vert) in self.vertices.iter().enumerate() {
            if vert.first_edge >= self.half_edges.len() as u32 {
                return Err(ManifoldError::IndexOutOfBounds(format!(
                    "Vertex {} points to invalid edge {}",
                    i, vert.first_edge
                )));
            }
            // Verify the edge actually starts at this vertex
            if self.half_edges[vert.first_edge as usize].start_vert != i as u32 {
                return Err(ManifoldError::InvalidTopology(format!(
                    "Vertex {} points to edge {} which starts at {}",
                    i, vert.first_edge, self.half_edges[vert.first_edge as usize].start_vert
                )));
            }
        }

        // Check half-edge indices
        for (i, edge) in self.half_edges.iter().enumerate() {
            if edge.start_vert >= self.vertices.len() as u32 {
                return Err(ManifoldError::IndexOutOfBounds(format!(
                    "Edge {} start_vert {} out of bounds",
                    i, edge.start_vert
                )));
            }
            if edge.end_vert >= self.vertices.len() as u32 {
                return Err(ManifoldError::IndexOutOfBounds(format!(
                    "Edge {} end_vert {} out of bounds",
                    i, edge.end_vert
                )));
            }
            if edge.next_edge >= self.half_edges.len() as u32 {
                return Err(ManifoldError::IndexOutOfBounds(format!(
                    "Edge {} next_edge {} out of bounds",
                    i, edge.next_edge
                )));
            }
            if edge.pair_edge >= self.half_edges.len() as u32 {
                return Err(ManifoldError::IndexOutOfBounds(format!(
                    "Edge {} pair_edge {} out of bounds",
                    i, edge.pair_edge
                )));
            }
            if edge.face >= self.faces.len() as u32 {
                return Err(ManifoldError::IndexOutOfBounds(format!(
                    "Edge {} face {} out of bounds",
                    i, edge.face
                )));
            }

            // Verify pairing
            let pair = &self.half_edges[edge.pair_edge as usize];
            if pair.pair_edge != i as u32 {
                return Err(ManifoldError::InvalidTopology(format!(
                    "Edge {} is paired with {}, but that edge is paired with {}",
                    i, edge.pair_edge, pair.pair_edge
                )));
            }
            if pair.start_vert != edge.end_vert || pair.end_vert != edge.start_vert {
                return Err(ManifoldError::InvalidTopology(format!(
                    "Edge {} and its pair {} do not match vertices",
                    i, edge.pair_edge
                )));
            }
        }

        // Check face indices
        for (i, face) in self.faces.iter().enumerate() {
            if face.first_edge >= self.half_edges.len() as u32 {
                return Err(ManifoldError::IndexOutOfBounds(format!(
                    "Face {} points to invalid edge {}",
                    i, face.first_edge
                )));
            }
            // Verify the edge actually belongs to this face
            if self.half_edges[face.first_edge as usize].face != i as u32 {
                return Err(ManifoldError::InvalidTopology(format!(
                    "Face {} points to edge {} which belongs to face {}",
                    i, face.first_edge, self.half_edges[face.first_edge as usize].face
                )));
            }
        }

        Ok(())
    }

    /// Returns the number of vertices in the manifold.
    ///
    /// # Examples
    /// ```
    /// use manifold_rs::Manifold;
    ///
    /// let m = Manifold::new();
    /// assert_eq!(m.vertex_count(), 0);
    /// ```
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the number of faces in the manifold.
    ///
    /// # Examples
    /// ```
    /// use manifold_rs::Manifold;
    ///
    /// let m = Manifold::new();
    /// assert_eq!(m.face_count(), 0);
    /// ```
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    /// Returns the axis-aligned bounding box of the manifold.
    ///
    /// Returns (min_corner, max_corner) where each is a Vec3.
    /// For an empty manifold, returns (Vec3::ZERO, Vec3::ZERO).
    ///
    /// # Examples
    /// ```
    /// use manifold_rs::primitives::cube::cube;
    /// use manifold_rs::Vec3;
    ///
    /// let c = cube(Vec3::new(2.0, 3.0, 4.0), false).unwrap();
    /// let (min, max) = c.bounding_box();
    /// assert_eq!(min, Vec3::new(0.0, 0.0, 0.0));
    /// assert_eq!(max, Vec3::new(2.0, 3.0, 4.0));
    /// ```
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
    fn test_invalid_vertex_index() {
        let mut m = Manifold::new();
        m.vertices.push(Vertex::new(Vec3::ZERO, 100)); // Invalid edge index
        assert!(matches!(
            m.validate(),
            Err(ManifoldError::IndexOutOfBounds(_))
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
            Err(ManifoldError::InvalidTopology(msg)) => {
                if !msg.contains("is paired with") {
                    panic!("Unexpected error message: {}", msg);
                }
            }
            Err(e) => panic!("Unexpected error type: {:?}", e),
            Ok(_) => panic!("Validation should have failed"),
        }
    }
}

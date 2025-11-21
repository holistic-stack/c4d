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

    /// Applies a transformation matrix to all vertices in the manifold.
    pub fn transform(&mut self, matrix: glam::DMat4) {
        // For now, we assume DMat4 is compatible or we convert.
        // libs/manifold-rs/src/core/vec3/mod.rs aliases Vec3 to DVec3.
        // So we can use DMat4::transform_point3.

        // matrix: DMat4 is 4x4.
        // vertices: DVec3.

        for vertex in &mut self.vertices {
            // glam DVec3
            vertex.position = matrix.transform_point3(vertex.position);
        }

        // Normals in faces might need update if we stored them?
        // Yes, Face has normal: Vec3.
        // Transform normal: use inverse transpose of upper 3x3.
        // If orthogonal (rotation/translation/uniform scale), it's just the rotation part.
        // If non-uniform scale, it's more complex.
        // For now, recompute normals or transform them properly.
        // Given we just have `Face::new(..., normal)`, we should probably update it.
        // Since this is a half-edge structure, we can recompute normals from geometry if needed.
        // Or transform current normals.

        // Inverse transpose for normals.
        let normal_matrix = matrix.inverse().transpose();
        // We only need the 3x3 part for directions, and renormalization.

        for face in &mut self.faces {
            // Transform normal as a vector (direction), ignoring translation.
            // Use transform_vector3 for 3x3 part of affine transform?
            // No, for normals under non-uniform scale, we need inverse transpose.
            // transform_point3 is for points.
            // transform_vector3 is for vectors (ignoring translation).

            // Correct normal transformation: N' = (M^-1)^T * N
            let n = face.normal;
            let n4 = normal_matrix * glam::DVec4::new(n.x, n.y, n.z, 0.0);
            face.normal = glam::DVec3::new(n4.x, n4.y, n4.z).normalize_or_zero();
        }

        // Check for negative determinant (reflection/scale -1) which flips winding order?
        // If det < 0, we need to flip loops.
        if matrix.determinant() < 0.0 {
            self.flip_faces();
        }
    }

    fn flip_faces(&mut self) {
        // Reverse the winding order of all faces.
        // For each half-edge, swap next/prev?
        // In half-edge, `next` pointers define the loop.
        // To reverse loop: A->B->C becomes A->C->B?
        // Actually, we just need to reverse the linked list of edges for each face.
        // But half-edges are stored in a Vec.
        // Simpler: Swap start/end vertices of every half-edge.
        // And update `next` pointers to go backwards.
        // This is non-trivial without `prev` pointers or efficient traversal.
        // With only `next`, traversing backwards is O(N) per face.

        // Implementation of flip is required for negative scale.
        // For Task 5.1, maybe we assume positive determinant or defer this?
        // "scale" can be negative.
        // Let's try to implement it.

        // 1. Swap start_vert and end_vert for all half-edges.
        // 2. Re-link `next` pointers.
        //    Currently: E1.next -> E2.next -> E3.next -> E1
        //    New: E1.next -> E3, E3.next -> E2, E2.next -> E1
        //    Effectively reversing the linked list.

        // To do this efficiently:
        // For each face, collect edges in order.
        // Reverse the order.
        // Update `next` pointers.

        for face in &self.faces {
            let mut edges = Vec::new();
            let mut curr = face.first_edge;
            loop {
                edges.push(curr);
                curr = self.half_edges[curr as usize].next_edge;
                if curr == face.first_edge { break; }
                // Safety check for infinite loops?
                if edges.len() > self.half_edges.len() { break; }
            }

            // Reverse edges
            // e.g. [E1, E2, E3] -> [E3, E2, E1] ??
            // Old loop: E1->E2->E3->E1
            // We want new loop to traverse vertices in reverse.
            // Old: V1 -(E1)-> V2 -(E2)-> V3 -(E3)-> V1
            // New: V1 -(?)-> V3 -(?)-> V2 -(?)-> V1
            // The half-edges themselves are directed V_start -> V_end.
            // If we swap V_start/V_end:
            // E1': V2 -> V1
            // E2': V3 -> V2
            // E3': V1 -> V3
            // We want loop V1 -> V3 -> V2 -> V1
            // That corresponds to E3' -> E2' -> E1' -> E3'
            // So the `next` sequence is E3' -> E2', E2' -> E1', E1' -> E3'.
            // Which implies reversing the list of edges, and linking them.

            // Step 1: Swap vertices (done globally later or here)
            // Step 2: Re-link next.

            // Let's do re-linking locally for this face.
            // edges = [E1, E2, E3]
            // We want E3.next = E2, E2.next = E1, E1.next = E3.

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

        // Fix vertex `first_edge`?
        // Vertex V points to an outgoing edge.
        // E1 was V1->V2. Now E1 is V2->V1.
        // So E1 is outgoing from V2.
        // V1 needs a new outgoing edge. E3 (was V3->V1) is now V1->V3.
        // So we need to update vertices to point to valid outgoing edges.
        // We can iterate all edges and set `vertices[edge.start_vert].first_edge = edge_index`.
        // Since we just need *any* outgoing edge.

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

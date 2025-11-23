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

    /// Constructs a Manifold from raw mesh buffers (f32).
    /// Assumes the mesh is a valid triangle mesh.
    /// This rebuilds the half-edge structure from shared vertices and indices.
    pub fn from_mesh_buffers(buffers: crate::MeshBuffers) -> Result<Self, Error> {
        let mut m = Manifold::new();

        // 1. Convert vertices (f32 -> f64)
        for chunk in buffers.vertices.chunks(3) {
             let pos = crate::Vec3::new(chunk[0] as f64, chunk[1] as f64, chunk[2] as f64);
             m.vertices.push(Vertex::new(pos, u32::MAX));
        }

        // 2. Build faces and half-edges
        use std::collections::HashMap;
        let mut edge_map: HashMap<(u32, u32), u32> = HashMap::new();

        for chunk in buffers.indices.chunks(3) {
            if chunk.len() < 3 { break; }
            let idx0 = chunk[0];
            let idx1 = chunk[1];
            let idx2 = chunk[2];

            let len = m.vertices.len() as u32;
            if idx0 >= len || idx1 >= len || idx2 >= len {
                 return Err(Error::IndexOutOfBounds(format!("Index out of bounds: {}/{}/{}/{}", idx0, idx1, idx2, len)));
            }

            let face_idx = m.faces.len() as u32;
            let start_he_idx = m.half_edges.len() as u32;

            // Create face
            let v0 = m.vertices[idx0 as usize].position;
            let v1 = m.vertices[idx1 as usize].position;
            let v2 = m.vertices[idx2 as usize].position;
            let normal = (v1 - v0).cross(v2 - v0).normalize_or_zero();
            m.faces.push(Face::new(start_he_idx, normal));

            // Create half-edges
            let he0_idx = start_he_idx;
            let he1_idx = start_he_idx + 1;
            let he2_idx = start_he_idx + 2;

            m.half_edges.push(HalfEdge::new(idx0, idx1, he1_idx, u32::MAX, face_idx));
            m.half_edges.push(HalfEdge::new(idx1, idx2, he2_idx, u32::MAX, face_idx));
            m.half_edges.push(HalfEdge::new(idx2, idx0, he0_idx, u32::MAX, face_idx));

            if m.vertices[idx0 as usize].first_edge == u32::MAX { m.vertices[idx0 as usize].first_edge = he0_idx; }
            if m.vertices[idx1 as usize].first_edge == u32::MAX { m.vertices[idx1 as usize].first_edge = he1_idx; }
            if m.vertices[idx2 as usize].first_edge == u32::MAX { m.vertices[idx2 as usize].first_edge = he2_idx; }

            edge_map.insert((idx0, idx1), he0_idx);
            edge_map.insert((idx1, idx2), he1_idx);
            edge_map.insert((idx2, idx0), he2_idx);
        }

        // 3. Pair half-edges
        for he_idx in 0..m.half_edges.len() {
            let he = &m.half_edges[he_idx];
            let start = he.start_vert;
            let end = he.end_vert;

            if let Some(&pair_idx) = edge_map.get(&(end, start)) {
                m.half_edges[he_idx].pair_edge = pair_idx;
            }
        }

        Ok(m)
    }

    /// Tries to extract a 2D cross-section from the manifold.
    ///
    /// This only works if the manifold represents a flat 2D shape on the Z=0 plane.
    /// It returns `None` if the shape is not 2D or validation fails.
    pub fn to_cross_section(&self) -> Option<crate::core::cross_section::CrossSection> {
        let mut contours = Vec::new();
        let mut visited_edges = vec![false; self.half_edges.len()];

        for (i, edge) in self.half_edges.iter().enumerate() {
            if visited_edges[i] { continue; }

            let face = &self.faces[edge.face as usize];
            if face.normal.z <= 0.0 { continue; } // Only trace Front faces

            let pair_idx = edge.pair_edge;
            let is_boundary = if pair_idx == u32::MAX {
                true
            } else {
                let pair_face = &self.faces[self.half_edges[pair_idx as usize].face as usize];
                pair_face.normal.z < 0.0
            };

            if is_boundary {
                 let mut contour = Vec::new();
                 let start_edge_idx = i as u32; // Not mutable
                 let mut curr_edge_idx = start_edge_idx;

                 let mut steps = 0;
                 let max_steps = self.half_edges.len() * 2;

                 loop {
                    if steps > max_steps { return None; } // Infinite loop protection
                    steps += 1;

                    if visited_edges[curr_edge_idx as usize] {
                        break;
                    }
                    visited_edges[curr_edge_idx as usize] = true;

                    let e = &self.half_edges[curr_edge_idx as usize];
                    let v = &self.vertices[e.start_vert as usize];

                    if v.position.z.abs() > 0.1 { return None; }

                    contour.push(glam::DVec2::new(v.position.x, v.position.y));

                    // Navigate to next boundary edge
                    let mut walker = e.next_edge;
                    let mut found_next = false;

                    // Circulate around vertex e.end_vert until we find a boundary edge
                    for _k in 0..100 { // Max valence check
                        // Check if walker is boundary
                        let w_e = &self.half_edges[walker as usize];
                        let w_pair_idx = w_e.pair_edge;
                        let w_is_boundary = if w_pair_idx == u32::MAX {
                            true
                        } else {
                             let w_face = &self.faces[w_e.face as usize];
                             // Ensure we stay on Front faces!
                             if w_face.normal.z <= 0.0 {
                                 false
                             } else {
                                 let w_pair_face = &self.faces[self.half_edges[w_pair_idx as usize].face as usize];
                                 w_pair_face.normal.z < 0.0
                             }
                        };

                        if w_is_boundary {
                            curr_edge_idx = walker;
                            found_next = true;
                            break;
                        }

                        // Pivot to next edge around vertex
                        if w_pair_idx == u32::MAX {
                            break;
                        }

                        // Correct logic for pivoting
                        let next_candidate = self.half_edges[w_pair_idx as usize].next_edge;
                        walker = next_candidate;
                    }

                    if !found_next { break; } // Loop broken?
                    if curr_edge_idx == start_edge_idx { break; }
                 }

                 if !contour.is_empty() {
                     // Debug print
                     // println!("Extracted contour with {} points", contour.len());
                     contours.push(contour);
                 }
            }
        }

        if contours.is_empty() {
            None
        } else {
            Some(crate::core::cross_section::CrossSection::from_contours(contours))
        }
    }

    /// Validates the integrity of the half-edge data structure.
    pub fn validate(&self) -> Result<(), Error> {
        for (i, vert) in self.vertices.iter().enumerate() {
            if vert.first_edge >= self.half_edges.len() as u32 {
                return Err(Error::IndexOutOfBounds(format!(
                    "Vertex {} points to invalid edge {}",
                    i, vert.first_edge
                )));
            }
            if self.half_edges[vert.first_edge as usize].start_vert != i as u32 {
                return Err(Error::InvalidTopology(format!(
                    "Vertex {} points to edge {} which starts at {}",
                    i, vert.first_edge, self.half_edges[vert.first_edge as usize].start_vert
                )));
            }
        }

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
                 if edge.pair_edge != u32::MAX {
                    return Err(Error::IndexOutOfBounds(format!(
                        "Edge {} pair_edge {} out of bounds",
                        i, edge.pair_edge
                    )));
                 }
            }
            if edge.face >= self.faces.len() as u32 {
                return Err(Error::IndexOutOfBounds(format!(
                    "Edge {} face {} out of bounds",
                    i, edge.face
                )));
            }

            if edge.pair_edge != u32::MAX {
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
        }

        for (i, face) in self.faces.iter().enumerate() {
            if face.first_edge >= self.half_edges.len() as u32 {
                return Err(Error::IndexOutOfBounds(format!(
                    "Face {} points to invalid edge {}",
                    i, face.first_edge
                )));
            }
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
        for vertex in &mut self.vertices {
            vertex.position = matrix.transform_point3(vertex.position);
        }

        let normal_matrix = matrix.inverse().transpose();
        for face in &mut self.faces {
            let n = face.normal;
            let n4 = normal_matrix * glam::DVec4::new(n.x, n.y, n.z, 0.0);
            face.normal = glam::DVec3::new(n4.x, n4.y, n4.z).normalize_or_zero();
        }

        if matrix.determinant() < 0.0 {
            self.flip_faces();
        }
    }

    fn flip_faces(&mut self) {
        for face in &self.faces {
            let mut edges = Vec::new();
            let mut curr = face.first_edge;
            loop {
                edges.push(curr);
                curr = self.half_edges[curr as usize].next_edge;
                if curr == face.first_edge { break; }
                if edges.len() > self.half_edges.len() { break; }
            }

            for i in 0..edges.len() {
                let curr = edges[i];
                let next_in_reverse = edges[(i + edges.len() - 1) % edges.len()];
                self.half_edges[curr as usize].next_edge = next_in_reverse;
            }
        }

        for edge in &mut self.half_edges {
            std::mem::swap(&mut edge.start_vert, &mut edge.end_vert);
        }

        for (i, edge) in self.half_edges.iter().enumerate() {
            self.vertices[edge.start_vert as usize].first_edge = i as u32;
        }
    }
}

//! Triangulation helper for 2D primitives.

use crate::{
    core::{
        half_edge::{Face, HalfEdge, Vertex},
        vec3::Vec3,
    },
    error::ManifoldError,
    Manifold,
};
use glam::DVec2;
use std::collections::HashMap;

/// Creates a double-sided manifold from a set of 2D contours (polygons).
///
/// # Arguments
///
/// * `contours` - List of contours. Each contour is a list of 2D points.
///                The first contour is the outer boundary. Subsequent contours are holes.
///                Points should be ordered CCW for outer, CW for holes (standard winding).
///
/// # Returns
///
/// * `Ok(Manifold)` - A valid double-sided manifold.
pub fn manifold_from_contours(contours: Vec<Vec<DVec2>>) -> Result<Manifold, ManifoldError> {
    if contours.is_empty() {
        return Err(ManifoldError::InvalidGeometry {
            message: "No contours provided".to_string(),
        });
    }

    // Flatten points for earcutr and track start indices of holes
    let mut flat_coords = Vec::new();
    let mut hole_indices = Vec::new();
    let mut current_index = 0;

    // For reconstructing the Manifold, we need to map earcut indices back to our vertices.
    // Since we flatten, the indices map 1:1 if we store all vertices.
    // Earcut takes a flat Vec<f64> [x0, y0, x1, y1, ...].

    // We also need to keep track of which vertices belong to which contour to handle edges?
    // Actually, if we just triangulate, we get a list of triangles.
    // Then we can build the mesh from triangles.
    // Building a Manifold from a Triangle Soup is easier if we know it's a closed surface.
    // For double-sided:
    // 1. Generate Front triangles using earcut.
    // 2. Generate Back triangles (reversed indices of Front).
    // 3. Stitch boundaries.

    // Note: earcutr expects hole indices to be the index of the *start* of the hole in the flat array.

    for (i, contour) in contours.iter().enumerate() {
        if i > 0 {
            hole_indices.push(current_index);
        }
        for p in contour {
            flat_coords.push(p.x);
            flat_coords.push(p.y);
            current_index += 1;
        }
    }

    // Run earcut
    // earcutr::earcut(data: &[T], hole_indices: &[usize], dim: usize)
    let triangles = earcutr::earcut(&flat_coords, &hole_indices, 2).map_err(|e| {
        ManifoldError::InvalidGeometry {
            message: format!("Triangulation failed: {:?}", e)
        }
    })?;

    if triangles.len() % 3 != 0 {
        return Err(ManifoldError::InvalidGeometry {
            message: "Triangulation produced invalid index count".to_string(),
        });
    }

    // Vertices for Manifold
    // We have `current_index` vertices.
    let num_vertices = current_index;
    let mut vertices = Vec::with_capacity(num_vertices);
    for i in 0..num_vertices {
        let x = flat_coords[i * 2];
        let y = flat_coords[i * 2 + 1];
        vertices.push(Vertex {
            position: Vec3::new(x, y, 0.0),
            first_edge: 0, // Will be set later
        });
    }

    let mut faces = Vec::new();
    let mut half_edges = Vec::new();

    // Create Front Faces
    for chunk in triangles.chunks(3) {
        let v0 = chunk[0];
        let v1 = chunk[1];
        let v2 = chunk[2];

        let start_edge = half_edges.len() as u32;
        faces.push(Face { first_edge: start_edge, normal: Vec3::Z });

        half_edges.push(HalfEdge {
            start_vert: v0 as u32, end_vert: v1 as u32, next_edge: start_edge + 1, pair_edge: u32::MAX, face: faces.len() as u32 - 1
        });
        half_edges.push(HalfEdge {
            start_vert: v1 as u32, end_vert: v2 as u32, next_edge: start_edge + 2, pair_edge: u32::MAX, face: faces.len() as u32 - 1
        });
        half_edges.push(HalfEdge {
            start_vert: v2 as u32, end_vert: v0 as u32, next_edge: start_edge, pair_edge: u32::MAX, face: faces.len() as u32 - 1
        });
    }

    // Create Back Faces (Same triangles, reversed winding)
    for chunk in triangles.chunks(3) {
        let v0 = chunk[0];
        let v1 = chunk[1];
        let v2 = chunk[2];

        // Back face: v0 -> v2 -> v1
        let start_edge = half_edges.len() as u32;
        faces.push(Face { first_edge: start_edge, normal: -Vec3::Z });

        half_edges.push(HalfEdge {
            start_vert: v0 as u32, end_vert: v2 as u32, next_edge: start_edge + 1, pair_edge: u32::MAX, face: faces.len() as u32 - 1
        });
        half_edges.push(HalfEdge {
            start_vert: v2 as u32, end_vert: v1 as u32, next_edge: start_edge + 2, pair_edge: u32::MAX, face: faces.len() as u32 - 1
        });
        half_edges.push(HalfEdge {
            start_vert: v1 as u32, end_vert: v0 as u32, next_edge: start_edge, pair_edge: u32::MAX, face: faces.len() as u32 - 1
        });
    }

    // Stitching / Pairing
    // Map: (start, end) -> edge_index
    let mut edge_map: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
    for (i, edge) in half_edges.iter().enumerate() {
        edge_map.entry((edge.start_vert, edge.end_vert)).or_default().push(i as u32);
    }

    // Helper to check if face is front
    let get_is_front = |edge: &HalfEdge, faces: &[Face]| -> bool {
        faces[edge.face as usize].normal.z > 0.0
    };

    // 1. Pair Same-Side (Internal) Edges
    // Edge (u->v) on Front pairs with Edge (v->u) on Front.
    for i in 0..half_edges.len() {
        if half_edges[i].pair_edge != u32::MAX { continue; }

        let u = half_edges[i].start_vert;
        let v = half_edges[i].end_vert;
        let is_front = get_is_front(&half_edges[i], &faces);

        if let Some(candidates) = edge_map.get(&(v, u)) {
            for &cand in candidates {
                if cand == i as u32 { continue; }
                if half_edges[cand as usize].pair_edge != u32::MAX { continue; }

                let cand_is_front = get_is_front(&half_edges[cand as usize], &faces);
                if cand_is_front == is_front {
                    half_edges[i].pair_edge = cand;
                    half_edges[cand as usize].pair_edge = i as u32;
                    break;
                }
            }
        }
    }

    // 2. Pair Cross-Side (Boundary) Edges
    // Edge (u->v) on Front pairs with Edge (v->u) on Back.
    // Note: Boundary edge on Back face (v0->v2->v1) corresponding to Front (v0->v1->v2)
    // If Front has boundary u->v.
    // Back should have boundary v->u.
    // So we just look for ANY remaining pair (v, u).

    for i in 0..half_edges.len() {
        if half_edges[i].pair_edge != u32::MAX { continue; }

        let u = half_edges[i].start_vert;
        let v = half_edges[i].end_vert;

        if let Some(candidates) = edge_map.get(&(v, u)) {
            for &cand in candidates {
                if cand == i as u32 { continue; }
                if half_edges[cand as usize].pair_edge != u32::MAX { continue; }

                // At this point, internal edges are paired.
                // So this must be the cross-side pair.
                half_edges[i].pair_edge = cand;
                half_edges[cand as usize].pair_edge = i as u32;
                break;
            }
        }
    }

    // Validation Check: ensure all edges paired
    for (i, edge) in half_edges.iter().enumerate() {
        if edge.pair_edge == u32::MAX {
            return Err(ManifoldError::InvalidTopology(
                format!("Edge {} ({}->{}) could not be paired. Mesh is not watertight.", i, edge.start_vert, edge.end_vert)
            ));
        }
    }

    let mut m = Manifold::default();
    m.vertices = vertices;
    m.half_edges = half_edges;
    m.faces = faces;

    // Update vertices first_edge
    for (i, edge) in m.half_edges.iter().enumerate() {
        m.vertices[edge.start_vert as usize].first_edge = i as u32;
    }

    Ok(m)
}

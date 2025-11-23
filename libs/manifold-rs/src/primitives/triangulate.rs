//! Triangulation helper for 2D primitives.

use crate::{
    core::{
        ds::{Face, HalfEdge, Vertex},
        vec3::Vec3,
    },
    error::Error,
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
pub fn manifold_from_contours(contours: Vec<Vec<DVec2>>) -> Result<Manifold, Error> {
    if contours.is_empty() {
        return Err(Error::InvalidGeometry {
            message: "No contours provided".to_string(),
        });
    }

    // Flatten points for earcutr and track start indices of holes
    let mut flat_coords = Vec::new();
    let mut hole_indices = Vec::new();
    let mut current_index = 0;

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
    let triangles = earcutr::earcut(&flat_coords, &hole_indices, 2).map_err(|e| {
        Error::InvalidGeometry {
            message: format!("Triangulation failed: {:?}", e)
        }
    })?;

    if triangles.len() % 3 != 0 {
        return Err(Error::InvalidGeometry {
            message: "Triangulation produced invalid index count".to_string(),
        });
    }

    // Vertices for Manifold
    let num_vertices = current_index;
    let mut vertices = Vec::with_capacity(num_vertices);
    for i in 0..num_vertices {
        let x = flat_coords[i * 2];
        let y = flat_coords[i * 2 + 1];
        vertices.push(Vertex {
            position: Vec3::new(x, y, 0.0),
            first_edge: 0,
        });
    }

    // Use common logic to build manifold from triangles (doubled for front/back)
    let front_triangles = triangles;
    let mut back_triangles = Vec::with_capacity(front_triangles.len());
    for chunk in front_triangles.chunks(3) {
        back_triangles.push(chunk[0]);
        back_triangles.push(chunk[2]); // Swap for CW
        back_triangles.push(chunk[1]);
    }

    // Build faces
    let mut faces = Vec::new();
    let mut half_edges = Vec::new();

    // Front Faces
    for chunk in front_triangles.chunks(3) {
        add_triangle_face(&mut faces, &mut half_edges, chunk[0] as u32, chunk[1] as u32, chunk[2] as u32, Vec3::Z);
    }
    // Back Faces
    for chunk in back_triangles.chunks(3) {
        add_triangle_face(&mut faces, &mut half_edges, chunk[0] as u32, chunk[1] as u32, chunk[2] as u32, -Vec3::Z);
    }

    stitch_mesh(&mut half_edges, &faces)?;

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

fn add_triangle_face(faces: &mut Vec<Face>, half_edges: &mut Vec<HalfEdge>, v0: u32, v1: u32, v2: u32, normal: Vec3) {
    let start_edge = half_edges.len() as u32;
    faces.push(Face { first_edge: start_edge, normal });

    half_edges.push(HalfEdge {
        start_vert: v0, end_vert: v1, next_edge: start_edge + 1, pair_edge: u32::MAX, face: faces.len() as u32 - 1
    });
    half_edges.push(HalfEdge {
        start_vert: v1, end_vert: v2, next_edge: start_edge + 2, pair_edge: u32::MAX, face: faces.len() as u32 - 1
    });
    half_edges.push(HalfEdge {
        start_vert: v2, end_vert: v0, next_edge: start_edge, pair_edge: u32::MAX, face: faces.len() as u32 - 1
    });
}

fn stitch_mesh(half_edges: &mut Vec<HalfEdge>, faces: &[Face]) -> Result<(), Error> {
    let mut edge_map: HashMap<(u32, u32), Vec<u32>> = HashMap::new();
    for (i, edge) in half_edges.iter().enumerate() {
        edge_map.entry((edge.start_vert, edge.end_vert)).or_default().push(i as u32);
    }

    // Pair Same-Side (Internal)
    for i in 0..half_edges.len() {
        if half_edges[i].pair_edge != u32::MAX { continue; }

        let u = half_edges[i].start_vert;
        let v = half_edges[i].end_vert;
        let is_front = faces[half_edges[i].face as usize].normal.z > 0.0;

        if let Some(candidates) = edge_map.get(&(v, u)) {
            for &cand in candidates {
                if cand == i as u32 { continue; }
                if half_edges[cand as usize].pair_edge != u32::MAX { continue; }

                let cand_is_front = faces[half_edges[cand as usize].face as usize].normal.z > 0.0;
                if cand_is_front == is_front {
                    half_edges[i].pair_edge = cand;
                    half_edges[cand as usize].pair_edge = i as u32;
                    break;
                }
            }
        }
    }

    // Pair Cross-Side (Boundary)
    for i in 0..half_edges.len() {
        if half_edges[i].pair_edge != u32::MAX { continue; }

        let u = half_edges[i].start_vert;
        let v = half_edges[i].end_vert;

        if let Some(candidates) = edge_map.get(&(v, u)) {
            for &cand in candidates {
                if cand == i as u32 { continue; }
                if half_edges[cand as usize].pair_edge != u32::MAX { continue; }

                half_edges[i].pair_edge = cand;
                half_edges[cand as usize].pair_edge = i as u32;
                break;
            }
        }
    }

    // Validate
    for (i, edge) in half_edges.iter().enumerate() {
        if edge.pair_edge == u32::MAX {
            return Err(Error::InvalidTopology(
                format!("Edge {} ({}->{}) could not be paired. Mesh is not watertight.", i, edge.start_vert, edge.end_vert)
            ));
        }
    }
    Ok(())
}

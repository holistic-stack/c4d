//! Shared utilities for geometry operations.

use crate::{
    core::{ds::{Face, HalfEdge}, vec3::Vec3},
    error::{Error, Result},
};
use std::collections::HashMap;

/// Adds a triangle to the mesh construction buffers.
///
/// # Arguments
/// * `faces` - The list of faces to append to.
/// * `half_edges` - The list of half-edges to append to.
/// * `v0` - Index of the first vertex.
/// * `v1` - Index of the second vertex.
/// * `v2` - Index of the third vertex.
pub fn add_triangle(faces: &mut Vec<Face>, half_edges: &mut Vec<HalfEdge>, v0: u32, v1: u32, v2: u32) {
    let start_edge = half_edges.len() as u32;
    // Initialize with zero normal; caller or validation should recompute if needed.
    faces.push(Face { first_edge: start_edge, normal: Vec3::ZERO });

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

/// Stitches half-edges by finding pairs (edges with same vertices in opposite directions).
///
/// # Arguments
/// * `half_edges` - The list of half-edges to modify.
///
/// # Returns
/// * `Ok(())` if all edges are successfully paired.
/// * `Err` if any edge remains unpaired (mesh is not watertight).
pub fn stitch_mesh(half_edges: &mut Vec<HalfEdge>) -> Result<()> {
    let mut edge_map: HashMap<(u32, u32), u32> = HashMap::new();
    for (i, edge) in half_edges.iter().enumerate() {
        edge_map.insert((edge.start_vert, edge.end_vert), i as u32);
    }

    for i in 0..half_edges.len() {
        if half_edges[i].pair_edge != u32::MAX { continue; }

        let u = half_edges[i].start_vert;
        let v = half_edges[i].end_vert;

        if let Some(&pair_idx) = edge_map.get(&(v, u)) {
            half_edges[i].pair_edge = pair_idx;
            half_edges[pair_idx as usize].pair_edge = i as u32;
        } else {
             return Err(Error::InvalidTopology(
                format!("Edge {} ({}->{}) could not be paired.", i, u, v)
            ));
        }
    }
    Ok(())
}

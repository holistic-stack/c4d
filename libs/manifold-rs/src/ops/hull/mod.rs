//! Convex Hull operations.

use crate::{
    core::{ds::Vertex, vec3::Vec3},
    error::{Error, Result},
    Manifold,
    ops::utils::{add_triangle, stitch_mesh},
};
use chull::ConvexHullWrapper;

/// Computes the convex hull of a set of points.
///
/// # Arguments
/// * `points` - The input points.
///
/// # Returns
/// * `Ok(Manifold)` - The convex hull mesh.
pub fn hull(points: &[Vec3]) -> Result<Manifold> {
    if points.len() < 4 {
        return Err(Error::InvalidGeometry {
            message: "Convex hull requires at least 4 points".to_string(),
        });
    }

    let input_points: Vec<Vec<f64>> = points
        .iter()
        .map(|p| vec![p.x, p.y, p.z])
        .collect();

    let wrapper = ConvexHullWrapper::try_new(&input_points, None).map_err(|e| {
        Error::InvalidGeometry {
            message: format!("Convex Hull failed: {:?}", e),
        }
    })?;

    let (hull_points, hull_indices) = wrapper.vertices_indices();

    if hull_indices.len() % 3 != 0 {
        return Err(Error::InvalidGeometry {
            message: "Convex Hull produced invalid index count".to_string(),
        });
    }

    let mut new_vertices = Vec::with_capacity(hull_points.len());
    for p in &hull_points {
        if p.len() < 3 { return Err(Error::InvalidGeometry { message: "Invalid point dimension".into() }); }
        new_vertices.push(Vertex {
            position: Vec3::new(p[0], p[1], p[2]),
            first_edge: 0,
        });
    }

    let mut faces = Vec::new();
    let mut half_edges = Vec::new();

    for chunk in hull_indices.chunks(3) {
        let v0 = chunk[0] as u32;
        let v1 = chunk[1] as u32;
        let v2 = chunk[2] as u32;

        if v0 as usize >= new_vertices.len() || v1 as usize >= new_vertices.len() || v2 as usize >= new_vertices.len() {
             return Err(Error::IndexOutOfBounds("Hull index out of bounds".into()));
        }

        add_triangle(&mut faces, &mut half_edges, v0, v1, v2);
    }

    stitch_mesh(&mut half_edges)?;

    let mut m = Manifold {
        vertices: new_vertices,
        half_edges,
        faces,
        color: None,
    };

    for (i, edge) in m.half_edges.iter().enumerate() {
        m.vertices[edge.start_vert as usize].first_edge = i as u32;
    }

    Ok(m)
}

#[cfg(test)]
mod tests;

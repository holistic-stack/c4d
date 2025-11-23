//! Polyhedron primitive.

use crate::{
    core::{ds::Vertex, vec3::Vec3},
    error::{Error, Result},
    Manifold,
    ops::utils::{add_triangle, stitch_mesh},
};

/// Creates a polyhedron from points and faces.
///
/// # Arguments
/// * `points` - List of 3D points.
/// * `faces` - List of faces, where each face is a list of indices into `points`.
///
/// # Returns
/// * `Ok(Manifold)` - The polyhedron mesh.
pub fn polyhedron(
    points: &[Vec3],
    faces: &[Vec<usize>],
) -> Result<Manifold> {
    if points.len() < 4 {
        return Err(Error::InvalidGeometry {
            message: "Polyhedron requires at least 4 points".to_string(),
        });
    }
    if faces.len() < 4 {
        return Err(Error::InvalidGeometry {
            message: "Polyhedron requires at least 4 faces".to_string(),
        });
    }

    let mut vertices = Vec::with_capacity(points.len());
    for p in points {
        vertices.push(Vertex {
            position: *p,
            first_edge: 0,
        });
    }

    let mut mesh_faces = Vec::new();
    let mut half_edges = Vec::new();

    for (face_idx, face) in faces.iter().enumerate() {
        if face.len() < 3 {
            return Err(Error::InvalidGeometry {
                message: format!("Face {} has fewer than 3 vertices", face_idx),
            });
        }

        // Triangulate face if > 3 vertices (fan from 0)
        // This assumes convex faces.
        // For concave faces, proper triangulation (earcut) is needed.
        // OpenSCAD docs say: "If the faces are not planar or convex, the result is undefined."
        // But robust implementation should handle it.
        // For now, simple fan triangulation.

        let v0 = face[0] as u32;
        for i in 1..face.len() - 1 {
            let v1 = face[i] as u32;
            let v2 = face[i + 1] as u32;

            if v0 as usize >= vertices.len() || v1 as usize >= vertices.len() || v2 as usize >= vertices.len() {
                return Err(Error::IndexOutOfBounds(format!(
                    "Face {} references invalid vertex index",
                    face_idx
                )));
            }

            add_triangle(&mut mesh_faces, &mut half_edges, v0, v1, v2);
        }
    }

    stitch_mesh(&mut half_edges)?;

    let mut m = Manifold {
        vertices,
        half_edges,
        faces: mesh_faces,
        color: None,
    };

    for (i, edge) in m.half_edges.iter().enumerate() {
        m.vertices[edge.start_vert as usize].first_edge = i as u32;
    }

    Ok(m)
}

#[cfg(test)]
mod tests;

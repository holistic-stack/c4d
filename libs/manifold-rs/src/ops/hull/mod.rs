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
    if points.len() < 3 {
        return Err(Error::InvalidGeometry {
            message: "Convex hull requires at least 3 points".to_string(),
        });
    }

    // Check if points are coplanar (2D).
    // Simple check: all Z equal? Or fit plane.
    // For OpenSCAD common case, Z=0.
    let is_flat = points.iter().all(|p| p.z.abs() < 1e-6);

    if is_flat {
        return hull_2d(points);
    }

    if points.len() < 4 {
        return Err(Error::InvalidGeometry {
            message: "3D Convex hull requires at least 4 non-coplanar points".to_string(),
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

/// 2D Convex Hull (Monotone Chain algorithm).
/// Returns a Manifold (flat mesh).
fn hull_2d(points: &[Vec3]) -> Result<Manifold> {
    // Project to 2D (ignore Z)
    let mut pts: Vec<(f64, f64, usize)> = points.iter().enumerate().map(|(i, p)| (p.x, p.y, i)).collect();

    // Sort by x, then y
    pts.sort_by(|a, b| {
        a.0.partial_cmp(&b.0).unwrap().then(a.1.partial_cmp(&b.1).unwrap())
    });

    // Cross product of vectors OA and OB
    // (bx-ax)*(cy-ay) - (by-ay)*(cx-ax)
    fn cross(o: &(f64, f64, usize), a: &(f64, f64, usize), b: &(f64, f64, usize)) -> f64 {
        (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
    }

    let mut lower = Vec::new();
    for p in &pts {
        while lower.len() >= 2 && cross(&lower[lower.len()-2], &lower[lower.len()-1], p) <= 0.0 {
            lower.pop();
        }
        lower.push(p.clone());
    }

    let mut upper = Vec::new();
    for p in pts.iter().rev() {
        while upper.len() >= 2 && cross(&upper[upper.len()-2], &upper[upper.len()-1], p) <= 0.0 {
            upper.pop();
        }
        upper.push(p.clone());
    }

    // Remove duplicate last points (start of upper matches end of lower)
    lower.pop();
    upper.pop();

    let mut hull_pts = lower;
    hull_pts.extend(upper);

    // Triangulate the polygon
    // Since it's convex, we can fan from vertex 0.
    if hull_pts.len() < 3 {
        return Err(Error::InvalidGeometry {
            message: "2D Hull resulted in fewer than 3 points".to_string(),
        });
    }

    let mut new_vertices = Vec::new();
    for p in &hull_pts {
        // Use original Z if constant
        new_vertices.push(Vertex {
            position: points[p.2],
            first_edge: 0,
        });
    }

    let mut faces = Vec::new();
    let mut half_edges = Vec::new();

    // Double-sided triangulation (Fan)
    // Front: 0 -> i -> i+1
    // Back: 0 -> i+1 -> i
    for i in 1..new_vertices.len() - 1 {
        let v0 = 0;
        let v1 = i as u32;
        let v2 = (i + 1) as u32;

        add_triangle(&mut faces, &mut half_edges, v0, v1, v2); // Front
        add_triangle(&mut faces, &mut half_edges, v0, v2, v1); // Back
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

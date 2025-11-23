//! Rotational extrusion of 2D cross-sections.

use crate::{
    core::{cross_section::CrossSection, vec3::Vec3, ds::Vertex},
    error::{Error, Result},
    Manifold,
    ops::utils::{add_triangle, stitch_mesh},
};

/// Extrudes a 2D cross-section by rotating it around the Z axis.
///
/// # Arguments
///
/// * `cross_section` - The 2D shape to revolve.
/// * `angle` - The angle of revolution in degrees (360 for full revolution).
/// * `convexity` - The convexity parameter (unused for mesh generation, but stored).
/// * `segments` - Number of segments for the revolution ($fn).
///
/// # Returns
///
/// * `Ok(Manifold)` - The revolved mesh.
pub fn rotate_extrude(
    cross_section: &CrossSection,
    angle: f64,
    _convexity: u32,
    segments: u32,
) -> Result<Manifold> {
    if segments < 3 {
        return Err(Error::InvalidGeometry {
            message: "rotate_extrude requires at least 3 segments".to_string(),
        });
    }

    if cross_section.contours.is_empty() {
        return Ok(Manifold::new());
    }

    let angle_rad = angle.to_radians();
    let is_closed = (angle - 360.0).abs() < 1e-6;
    let total_angle = if is_closed { std::f64::consts::TAU } else { angle_rad };

    let arc_fraction = if is_closed { 1.0 } else { total_angle / std::f64::consts::TAU };
    let steps = (segments as f64 * arc_fraction).ceil() as u32;
    let steps = steps.max(1);

    let step_angle = total_angle / steps as f64;

    let mut vertices = Vec::new();
    let contours_lens: Vec<usize> = cross_section.contours.iter().map(|c| c.len()).collect();
    let total_points_per_profile: usize = contours_lens.iter().sum();

    let num_profiles = if is_closed { steps } else { steps + 1 };

    for i in 0..num_profiles {
        let theta = i as f64 * step_angle;
        let (sin, cos) = theta.sin_cos();

        for contour in &cross_section.contours {
            for p in contour {
                let r = p.x;
                let z = p.y;

                let x3 = r * cos;
                let y3 = r * sin;
                let z3 = z;

                vertices.push(Vertex {
                    position: Vec3::new(x3, y3, z3),
                    first_edge: 0,
                });
            }
        }
    }

    let mut faces = Vec::new();
    let mut half_edges = Vec::new();

    let mut contour_start_idx = 0;
    for contour_len in &contours_lens {
        let n = *contour_len;

        for i in 0..steps {
            let curr_profile = i;
            let next_profile = if is_closed { (i + 1) % steps } else { i + 1 };

            let curr_base = (curr_profile as usize * total_points_per_profile) + contour_start_idx;
            let next_base = (next_profile as usize * total_points_per_profile) + contour_start_idx;

            for j in 0..n {
                let next_j = (j + 1) % n;

                let v_curr_j = (curr_base + j) as u32;
                let v_curr_next = (curr_base + next_j) as u32;
                let v_next_next = (next_base + next_j) as u32;
                let v_next_j = (next_base + j) as u32;

                add_triangle(&mut faces, &mut half_edges, v_curr_j, v_curr_next, v_next_j);
                add_triangle(&mut faces, &mut half_edges, v_curr_next, v_next_next, v_next_j);
            }
        }
        contour_start_idx += n;
    }

    if !is_closed {
        let mut flat_coords = Vec::new();
        let mut hole_indices = Vec::new();
        let mut current_idx = 0;
        for (i, contour) in cross_section.contours.iter().enumerate() {
            if i > 0 { hole_indices.push(current_idx); }
            for p in contour {
                flat_coords.push(p.x);
                flat_coords.push(p.y);
                current_idx += 1;
            }
        }
        let cap_indices = earcutr::earcut(&flat_coords, &hole_indices, 2).map_err(|e| {
            Error::InvalidGeometry { message: format!("Cap triangulation failed: {:?}", e) }
        })?;

        for chunk in cap_indices.chunks(3) {
            let v0 = chunk[0] as u32;
            let v1 = chunk[1] as u32;
            let v2 = chunk[2] as u32;
            add_triangle(&mut faces, &mut half_edges, v0, v2, v1); // Reversed
        }

        let end_offset = (steps as usize * total_points_per_profile) as u32;
        for chunk in cap_indices.chunks(3) {
            let v0 = chunk[0] as u32 + end_offset;
            let v1 = chunk[1] as u32 + end_offset;
            let v2 = chunk[2] as u32 + end_offset;
            add_triangle(&mut faces, &mut half_edges, v0, v1, v2); // Standard
        }
    }

    stitch_mesh(&mut half_edges)?;

    let mut m = Manifold {
        vertices,
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

//! Linear extrusion of 2D cross-sections.

use crate::{
    core::{cross_section::CrossSection, vec3::Vec3, ds::Vertex},
    error::{Error, Result},
    Manifold,
    ops::utils::{add_triangle, stitch_mesh},
};
use glam::DVec2;

/// Extrudes a 2D cross-section linearly along the Z axis.
///
/// # Arguments
///
/// * `cross_section` - The 2D shape to extrude.
/// * `height` - Total height of the extrusion.
/// * `twist` - Total twist angle in degrees.
/// * `slices` - Number of vertical slices (segments).
/// * `center` - Whether to center the extrusion on Z.
/// * `scale` - Scale factor for the top face (X, Y).
///
/// # Returns
///
/// * `Ok(Manifold)` - The extruded mesh.
pub fn linear_extrude(
    cross_section: &CrossSection,
    height: f64,
    twist: f64,
    slices: u32,
    center: bool,
    scale: DVec2,
) -> Result<Manifold> {
    if height <= 0.0 {
        return Err(Error::InvalidGeometry {
            message: "Extrusion height must be positive".to_string(),
        });
    }

    if cross_section.contours.is_empty() {
        return Ok(Manifold::new());
    }

    let slices = if slices < 1 { 1 } else { slices };
    let z_offset = if center { -height / 2.0 } else { 0.0 };

    let mut vertices = Vec::new();
    let contours_lens: Vec<usize> = cross_section.contours.iter().map(|c| c.len()).collect();
    let total_points_per_slice: usize = contours_lens.iter().sum();

    let twist_rad = twist.to_radians();
    let step_z = height / slices as f64;
    let step_twist = twist_rad / slices as f64;

    for i in 0..=slices {
        let frac = i as f64 / slices as f64;
        let z = z_offset + i as f64 * step_z;
        let current_twist = i as f64 * step_twist;
        let current_scale = DVec2::new(
            1.0 + (scale.x - 1.0) * frac,
            1.0 + (scale.y - 1.0) * frac,
        );

        let (sin_t, cos_t) = current_twist.sin_cos();

        for contour in &cross_section.contours {
            for p in contour {
                let sx = p.x * current_scale.x;
                let sy = p.y * current_scale.y;
                let rx = sx * cos_t - sy * sin_t;
                let ry = sx * sin_t + sy * cos_t;

                vertices.push(Vertex {
                    position: Vec3::new(rx, ry, z),
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

        for i in 0..slices {
            let slice_base = (i as usize * total_points_per_slice) + contour_start_idx;
            let next_slice_base = ((i + 1) as usize * total_points_per_slice) + contour_start_idx;

            for j in 0..n {
                let next_j = (j + 1) % n;
                let v_curr_bot = (slice_base + j) as u32;
                let v_next_bot = (slice_base + next_j) as u32;
                let v_curr_top = (next_slice_base + j) as u32;
                let v_next_top = (next_slice_base + next_j) as u32;

                add_triangle(&mut faces, &mut half_edges, v_curr_bot, v_next_bot, v_curr_top);
                add_triangle(&mut faces, &mut half_edges, v_next_bot, v_next_top, v_curr_top);
            }
        }
        contour_start_idx += n;
    }

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
        add_triangle(&mut faces, &mut half_edges, v0, v2, v1); // Reverse for bottom
    }

    let top_offset = (slices as usize * total_points_per_slice) as u32;
    for chunk in cap_indices.chunks(3) {
        let v0 = chunk[0] as u32 + top_offset;
        let v1 = chunk[1] as u32 + top_offset;
        let v2 = chunk[2] as u32 + top_offset;
        add_triangle(&mut faces, &mut half_edges, v0, v1, v2);
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

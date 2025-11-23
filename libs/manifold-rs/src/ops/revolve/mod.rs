//! Rotational extrusion of 2D cross-sections.

use crate::{
    core::{cross_section::CrossSection, vec3::Vec3, ds::{Face, HalfEdge, Vertex}},
    error::{Error, Result},
    Manifold,
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

    let angle_rad = angle.to_radians();
    let is_closed = (angle - 360.0).abs() < 1e-6;
    let _total_angle = if is_closed { std::f64::consts::TAU } else { angle_rad };

    // Number of steps. If closed, segments steps cover 360.
    // If partial, segments steps cover `angle`.
    let _steps = segments;

    // Total vertices = (steps + 1) * total_points (if partial)
    // If closed, steps * total_points (last slice connects to first).

    let _total_points: usize = cross_section.contours.iter().map(|c| c.len()).sum();
    // For simplicity, generate `steps + 1` slices even for closed, then stitch last to first?
    // Or handle indexing.

    // ... Implementation similar to linear_extrude but with rotation.

    // Stub for now.
    Err(Error::InvalidGeometry { message: "Not implemented yet".to_string() })
}

#[cfg(test)]
mod tests;

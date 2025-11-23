//! Circle primitive implementation.

use crate::{
    error::Error,
    primitives::triangulate::manifold_from_contours,
    Manifold,
};
use glam::DVec2;

/// Creates a circle (2D disk) manifold.
///
/// # Arguments
///
/// * `radius` - The radius of the circle.
/// * `segments` - Number of segments for the circle.
///
/// # Returns
///
/// * `Ok(Manifold)` - A valid circle manifold (as a flat 3D mesh, double-sided).
/// * `Err(Error)` - If the circle construction fails.
pub fn circle(radius: f64, segments: u32) -> Result<Manifold, Error> {
    if radius <= 0.0 {
         return Err(Error::InvalidGeometry {
             message: format!("Circle radius must be positive: {}", radius)
         });
    }
    if segments < 3 {
        return Err(Error::InvalidGeometry {
             message: format!("Circle must have at least 3 segments: {}", segments)
         });
    }

    let mut points = Vec::with_capacity(segments as usize);
    let angle_step = 2.0 * std::f64::consts::PI / segments as f64;

    for i in 0..segments {
        let angle = i as f64 * angle_step;
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        points.push(DVec2::new(x, y));
    }

    manifold_from_contours(vec![points])
}

#[cfg(test)]
mod tests;

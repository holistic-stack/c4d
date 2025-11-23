//! Square primitive implementation.

use crate::{
    error::Error,
    primitives::triangulate::manifold_from_contours,
    Manifold,
};
use glam::{DVec2, DVec3};

/// Creates a square (2D rectangle) manifold.
///
/// # Arguments
///
/// * `size` - The size of the square (x, y).
/// * `center` - Whether the square is centered at the origin.
///
/// # Returns
///
/// * `Ok(Manifold)` - A valid square manifold (as a flat 3D mesh).
/// * `Err(Error)` - If the square construction fails.
pub fn square(size: DVec2, center: bool) -> Result<Manifold, Error> {
    if size.x <= 0.0 || size.y <= 0.0 {
         return Err(Error::InvalidGeometry {
             message: format!("Square size must be positive: {:?}", size)
         });
    }

    let (x, y) = (size.x, size.y);
    let (dx, dy) = if center {
        (-x / 2.0, -y / 2.0)
    } else {
        (0.0, 0.0)
    };

    // Vertices in CCW order
    let points = vec![
        DVec2::new(dx, dy),
        DVec2::new(dx + x, dy),
        DVec2::new(dx + x, dy + y),
        DVec2::new(dx, dy + y),
    ];

    manifold_from_contours(vec![points])
}

#[cfg(test)]
mod tests;

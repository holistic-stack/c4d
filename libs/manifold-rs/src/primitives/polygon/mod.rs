//! Polygon primitive implementation.

use crate::{
    error::ManifoldError,
    primitives::triangulate::manifold_from_contours,
    Manifold,
};
use glam::DVec2;

/// Creates a polygon manifold from 2D points and paths.
///
/// # Arguments
///
/// * `points` - List of 2D vertices.
/// * `paths` - List of paths (indices into points). First path is outline, others are holes.
/// * `convexity` - Convexity parameter (currently unused but stored).
///
/// # Returns
///
/// * `Ok(Manifold)` - A valid polygon manifold (as a flat 3D mesh, double-sided).
/// * `Err(ManifoldError)` - If the polygon construction fails.
pub fn polygon(
    points: Vec<DVec2>,
    paths: Vec<Vec<usize>>,
    _convexity: u32,
) -> Result<Manifold, ManifoldError> {
    if points.len() < 3 {
         return Err(ManifoldError::InvalidGeometry {
             message: format!("Polygon must have at least 3 points: {}", points.len())
         });
    }
    if paths.is_empty() {
        return Err(ManifoldError::InvalidGeometry {
             message: "Polygon must have at least one path".to_string()
         });
    }

    let mut contours = Vec::with_capacity(paths.len());
    for path in paths {
        if path.len() < 3 {
            // Or ignore? OpenSCAD might just ignore invalid paths or warn.
            continue;
        }
        let mut contour = Vec::with_capacity(path.len());
        for &idx in &path {
            if idx >= points.len() {
                return Err(ManifoldError::IndexOutOfBounds(
                    format!("Polygon path index {} out of bounds ({} points)", idx, points.len())
                ));
            }
            contour.push(points[idx]);
        }
        contours.push(contour);
    }

    manifold_from_contours(contours)
}

#[cfg(test)]
mod tests;

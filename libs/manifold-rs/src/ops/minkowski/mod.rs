//! Minkowski Sum operations.

use crate::{Manifold, Vec3};
use crate::ops::hull::hull;

/// Computes the Minkowski sum of two manifolds.
///
/// The Minkowski sum of A and B is the set {a + b | a in A, b in B}.
/// For convex shapes, this is equivalent to the convex hull of the sum of all vertex pairs.
/// For non-convex shapes, it's more complex (union of translated A for every point in B).
///
/// # Arguments
/// * `a` - First manifold.
/// * `b` - Second manifold.
///
/// # Returns
/// * `Ok(Manifold)` - The Minkowski sum mesh.
pub fn minkowski(a: &Manifold, b: &Manifold) -> crate::error::Result<Manifold> {
    // Collect all vertex pairs
    let mut points = Vec::with_capacity(a.vertices.len() * b.vertices.len());
    for va in &a.vertices {
        for vb in &b.vertices {
            points.push(va.position + vb.position);
        }
    }

    // Compute convex hull of points
    // Note: This assumes result is convex or we approximate with hull.
    // OpenSCAD `minkowski` works on general shapes, but for simplicity we start with hull.
    // If `hull` is stubbed, this will fail or return error.
    hull(&points)
}

#[cfg(test)]
mod tests;

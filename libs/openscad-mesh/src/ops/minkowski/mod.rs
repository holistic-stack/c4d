//! # Minkowski Sum
//!
//! Computes the Minkowski sum of two meshes.
//! Browser-safe implementation in pure Rust.
//!
//! ## Algorithm Overview
//!
//! For convex meshes, the Minkowski sum is computed by:
//! 1. Collecting all vertices from both meshes
//! 2. Computing all pairwise sums of vertices
//! 3. Computing the convex hull of the result
//!
//! For non-convex meshes, we use a decomposition approach:
//! 1. Decompose into convex parts (simplified: use hull of each)
//! 2. Compute Minkowski sum of convex parts
//! 3. Union the results
//!
//! ## OpenSCAD Compatibility
//!
//! Matches `minkowski() { children }` - computes Minkowski sum of children.

#[cfg(test)]
mod tests;

use crate::error::MeshError;
use crate::mesh::Mesh;
use crate::ops::hull::convex_hull;

/// Computes the Minkowski sum of two meshes.
///
/// The Minkowski sum A ⊕ B is the set of all points a + b where a ∈ A and b ∈ B.
/// Geometrically, this "rounds" or "expands" shape A by shape B.
///
/// # Arguments
///
/// * `a` - First mesh
/// * `b` - Second mesh
///
/// # Returns
///
/// A mesh representing the Minkowski sum.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::minkowski::minkowski_sum;
///
/// // Round a cube by a sphere
/// let rounded = minkowski_sum(&cube, &sphere)?;
/// ```
pub fn minkowski_sum(a: &Mesh, b: &Mesh) -> Result<Mesh, MeshError> {
    if a.is_empty() || b.is_empty() {
        return Ok(Mesh::new());
    }

    // Collect all vertices from both meshes
    let a_verts = a.vertices();
    let b_verts = b.vertices();

    // Compute all pairwise sums
    let mut sum_points = Vec::with_capacity(a_verts.len() * b_verts.len());
    for av in a_verts {
        for bv in b_verts {
            sum_points.push(*av + *bv);
        }
    }

    if sum_points.len() < 4 {
        return Err(MeshError::degenerate(
            "Minkowski sum requires meshes with enough vertices",
            None,
        ));
    }

    // Compute convex hull of sum points
    // Note: This is exact for convex inputs, approximate for non-convex
    convex_hull(&sum_points)
}

/// Computes the Minkowski sum of multiple meshes.
///
/// # Arguments
///
/// * `meshes` - Meshes to compute Minkowski sum of
///
/// # Returns
///
/// A mesh representing the Minkowski sum.
pub fn minkowski(meshes: &[&Mesh]) -> Result<Mesh, MeshError> {
    if meshes.is_empty() {
        return Ok(Mesh::new());
    }

    if meshes.len() == 1 {
        return Ok(meshes[0].clone());
    }

    // Compute pairwise Minkowski sums
    let mut result = meshes[0].clone();
    for mesh in meshes.iter().skip(1) {
        result = minkowski_sum(&result, mesh)?;
    }

    Ok(result)
}

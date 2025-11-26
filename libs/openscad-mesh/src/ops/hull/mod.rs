//! # Convex Hull
//!
//! QuickHull algorithm for computing 3D convex hulls.
//! Browser-safe implementation in pure Rust.
//!
//! ## Algorithm Overview
//!
//! QuickHull is a divide-and-conquer algorithm:
//! 1. Find extreme points to form initial simplex (tetrahedron)
//! 2. For each face, find the farthest point outside
//! 3. Create new faces from that point to the horizon edges
//! 4. Repeat until no points remain outside
//!
//! ## OpenSCAD Compatibility
//!
//! Matches `hull() { children }` - computes convex hull of all children.

mod quickhull;

#[cfg(test)]
mod tests;

pub use quickhull::convex_hull;

use crate::error::MeshError;
use crate::mesh::Mesh;

/// Computes the convex hull of one or more meshes.
///
/// # Arguments
///
/// * `meshes` - Meshes to compute hull of
///
/// # Returns
///
/// A mesh representing the convex hull.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::hull::hull;
///
/// let result = hull(&[&mesh_a, &mesh_b])?;
/// ```
pub fn hull(meshes: &[&Mesh]) -> Result<Mesh, MeshError> {
    // Collect all vertices from all meshes
    let mut points = Vec::new();
    for mesh in meshes {
        for v in mesh.vertices() {
            points.push(*v);
        }
    }

    if points.is_empty() {
        return Ok(Mesh::new());
    }

    if points.len() < 4 {
        return Err(MeshError::degenerate(
            "Hull requires at least 4 non-coplanar points",
            None,
        ));
    }

    convex_hull(&points)
}

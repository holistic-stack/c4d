//! # Boolean Operations
//!
//! CSG boolean operations for 3D meshes: Union, Difference, Intersection.
//!
//! ## Overview
//!
//! This module provides constructive solid geometry (CSG) operations using a
//! Binary Space Partitioning (BSP) tree algorithm with robust point-in-mesh
//! classification.
//!
//! ## Supported Operations
//!
//! | Operation | Result | Example |
//! |-----------|--------|---------|
//! | Union | Combined volume | `A ∪ B` |
//! | Difference | Subtracted volume | `A - B` |
//! | Intersection | Common volume | `A ∩ B` |
//!
//! ## Algorithm
//!
//! ```text
//! 1. Convert meshes to BSP polygons
//! 2. Build BSP tree from each mesh
//! 3. Clip polygons using robust point-in-mesh tests
//! 4. Merge coplanar polygons to reduce fragmentation
//! 5. Convert back to mesh with vertex welding
//! ```
//!
//! ## Example
//!
//! ```rust
//! use manifold_rs::mesh::Mesh;
//! use manifold_rs::manifold::boolean::{union_all, difference_all, intersection_all};
//! use manifold_rs::manifold::constructors::build_cube;
//!
//! // Create two cubes
//! let mut cube1 = Mesh::new();
//! build_cube(&mut cube1, [10.0, 10.0, 10.0], true);
//!
//! let mut cube2 = Mesh::new();
//! build_cube(&mut cube2, [10.0, 10.0, 10.0], true);
//! cube2.translate(5.0, 0.0, 0.0);
//!
//! // Boolean operations
//! let union = union_all(&[cube1.clone(), cube2.clone()]).unwrap();
//! let diff = difference_all(&[cube1.clone(), cube2.clone()]).unwrap();
//! let inter = intersection_all(&[cube1, cube2]).unwrap();
//! ```
//!
//! ## Performance Notes
//!
//! The BSP algorithm produces ~44% more triangles than Manifold's edge-intersection
//! algorithm. This is a fundamental limitation of plane-based splitting vs.
//! intersection-curve-based splitting.
//!
//! For OpenSCAD's `union() { cube(15, center=true); sphere(10); }`:
//! - **OpenSCAD/Manifold**: 506 vertices, 1008 triangles
//! - **BSP (this impl)**: ~620 vertices, ~1450 triangles
//!
//! ## Module Structure
//!
//! - `mod.rs` - Public API (this file)
//! - `bsp.rs` - BSP tree implementation
//! - `polygon.rs` - Polygon operations (split, merge, convert)
//! - `geometry.rs` - Math utilities (ray casting, point-in-mesh)
//! - `tests.rs` - Integration tests

// =============================================================================
// SUBMODULES
// =============================================================================

mod bsp;
mod geometry;
mod polygon;

#[cfg(test)]
mod tests;

// =============================================================================
// RE-EXPORTS (internal use only)
// =============================================================================

use bsp::BspNode;
use polygon::{mesh_to_polygons, polygons_to_mesh};

// =============================================================================
// PUBLIC API
// =============================================================================

use crate::error::ManifoldResult;
use crate::mesh::Mesh;

/// Compute union of multiple meshes.
///
/// Returns the combined volume of all input meshes. Overlapping regions
/// are merged into a single surface.
///
/// ## Parameters
///
/// - `meshes`: Slice of meshes to union
///
/// ## Returns
///
/// Combined mesh (union of all inputs), or empty mesh if input is empty.
///
/// ## Example
///
/// ```rust
/// use manifold_rs::mesh::Mesh;
/// use manifold_rs::manifold::boolean::union_all;
/// use manifold_rs::manifold::constructors::build_cube;
///
/// let mut cube1 = Mesh::new();
/// build_cube(&mut cube1, [10.0, 10.0, 10.0], true);
///
/// let mut cube2 = Mesh::new();
/// build_cube(&mut cube2, [10.0, 10.0, 10.0], true);
/// cube2.translate(5.0, 0.0, 0.0);
///
/// let result = union_all(&[cube1, cube2]).unwrap();
/// assert!(!result.is_empty());
/// ```
pub fn union_all(meshes: &[Mesh]) -> ManifoldResult<Mesh> {
    match meshes.len() {
        0 => Ok(Mesh::new()),
        1 => Ok(meshes[0].clone()),
        _ => {
            let mut result = meshes[0].clone();
            for mesh in &meshes[1..] {
                result = bsp_union(&result, mesh)?;
            }
            Ok(result)
        }
    }
}

/// Compute difference of meshes (first minus rest).
///
/// Returns the first mesh with all subsequent meshes subtracted (carved out).
///
/// ## Parameters
///
/// - `meshes`: Slice of meshes (first is base, rest are subtracted)
///
/// ## Returns
///
/// Resulting mesh after subtraction.
///
/// ## Example
///
/// ```rust
/// use manifold_rs::mesh::Mesh;
/// use manifold_rs::manifold::boolean::difference_all;
/// use manifold_rs::manifold::constructors::build_cube;
///
/// let mut outer = Mesh::new();
/// build_cube(&mut outer, [10.0, 10.0, 10.0], true);
///
/// let mut inner = Mesh::new();
/// build_cube(&mut inner, [5.0, 5.0, 5.0], true);
///
/// // Carve inner cube from outer cube
/// let result = difference_all(&[outer, inner]).unwrap();
/// assert!(!result.is_empty());
/// ```
pub fn difference_all(meshes: &[Mesh]) -> ManifoldResult<Mesh> {
    match meshes.len() {
        0 => Ok(Mesh::new()),
        1 => Ok(meshes[0].clone()),
        _ => {
            let mut result = meshes[0].clone();
            for mesh in &meshes[1..] {
                result = bsp_difference(&result, mesh)?;
            }
            Ok(result)
        }
    }
}

/// Compute intersection of all meshes.
///
/// Returns the volume common to all input meshes. Only regions inside
/// all meshes are retained.
///
/// ## Parameters
///
/// - `meshes`: Slice of meshes to intersect
///
/// ## Returns
///
/// Resulting mesh (common volume), may be empty if meshes don't overlap.
///
/// ## Example
///
/// ```rust
/// use manifold_rs::mesh::Mesh;
/// use manifold_rs::manifold::boolean::intersection_all;
/// use manifold_rs::manifold::constructors::build_cube;
///
/// let mut cube1 = Mesh::new();
/// build_cube(&mut cube1, [10.0, 10.0, 10.0], true);
///
/// let mut cube2 = Mesh::new();
/// build_cube(&mut cube2, [10.0, 10.0, 10.0], true);
/// cube2.translate(5.0, 0.0, 0.0);
///
/// // Get overlapping region
/// let result = intersection_all(&[cube1, cube2]).unwrap();
/// assert!(!result.is_empty());
/// ```
pub fn intersection_all(meshes: &[Mesh]) -> ManifoldResult<Mesh> {
    match meshes.len() {
        0 => Ok(Mesh::new()),
        1 => Ok(meshes[0].clone()),
        _ => {
            let mut result = meshes[0].clone();
            for mesh in &meshes[1..] {
                result = bsp_intersection(&result, mesh)?;
            }
            Ok(result)
        }
    }
}

// =============================================================================
// INTERNAL IMPLEMENTATION
// =============================================================================

/// BSP-based union: A ∪ B = (A outside B) ∪ (B outside A)
fn bsp_union(a: &Mesh, b: &Mesh) -> ManifoldResult<Mesh> {
    let mut tree_a = BspNode::new();
    tree_a.build(mesh_to_polygons(a));
    
    let mut tree_b = BspNode::new();
    tree_b.build(mesh_to_polygons(b));
    
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);
    
    // Keep A outside B
    let result_a = tree_b.clip_polygons_robust(polys_a, b, false);
    
    // Keep B outside A
    let result_b = tree_a.clip_polygons_robust(polys_b, a, false);
    
    // Merge results
    let mut final_polys = result_a;
    final_polys.extend(result_b);
    
    Ok(polygons_to_mesh(&final_polys))
}

/// BSP-based difference: A - B = (A outside B) ∪ (B inside A, reversed)
fn bsp_difference(a: &Mesh, b: &Mesh) -> ManifoldResult<Mesh> {
    if a.is_empty() {
        return Ok(Mesh::new());
    }
    if b.is_empty() {
        return Ok(a.clone());
    }
    
    let mut tree_a = BspNode::new();
    tree_a.build(mesh_to_polygons(a));
    
    let mut tree_b = BspNode::new();
    tree_b.build(mesh_to_polygons(b));
    
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);
    
    // Keep A outside B
    let result_a = tree_b.clip_polygons_robust(polys_a, b, false);
    
    // Keep B inside A (will be reversed to form hole walls)
    let mut result_b = tree_a.clip_polygons_robust(polys_b, a, true);
    
    // Reverse B polygons (flip normals for inside-out surfaces)
    for poly in &mut result_b {
        poly.flip();
    }
    
    // Merge results
    let mut final_polys = result_a;
    final_polys.extend(result_b);
    
    Ok(polygons_to_mesh(&final_polys))
}

/// BSP-based intersection: A ∩ B = (A inside B) ∪ (B inside A)
fn bsp_intersection(a: &Mesh, b: &Mesh) -> ManifoldResult<Mesh> {
    if a.is_empty() || b.is_empty() {
        return Ok(Mesh::new());
    }
    
    let mut tree_a = BspNode::new();
    tree_a.build(mesh_to_polygons(a));
    
    let mut tree_b = BspNode::new();
    tree_b.build(mesh_to_polygons(b));
    
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);
    
    // Keep A inside B
    let result_a = tree_b.clip_polygons_robust(polys_a, b, true);
    
    // Keep B inside A
    let result_b = tree_a.clip_polygons_robust(polys_b, a, true);
    
    // Merge results
    let mut final_polys = result_a;
    final_polys.extend(result_b);
    
    Ok(polygons_to_mesh(&final_polys))
}

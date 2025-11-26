//! # Boolean Operations
//!
//! CSG (Constructive Solid Geometry) boolean operations using BSP trees.
//! Based on the csg.js algorithm by Evan Wallace.
//!
//! ## Algorithm Overview
//!
//! BSP trees recursively partition space using planes. Each node contains:
//! - A dividing plane
//! - Polygons coplanar with the plane
//! - Front subtree (polygons in front of plane)
//! - Back subtree (polygons behind plane)
//!
//! ## Operations
//!
//! - **Union**: A ∪ B = A.clipTo(B) + B.clipTo(A).invert().clipTo(A).invert()
//! - **Difference**: A - B = A.invert().clipTo(B).invert() + B.clipTo(A).invert()
//! - **Intersection**: A ∩ B = A.invert().clipTo(B.invert()).invert() + B.invert().clipTo(A).invert()
//!
//! ## Browser Safety
//!
//! Pure Rust implementation with no native dependencies.
//! Uses `robust` crate for exact geometric predicates.

mod bsp;
mod plane;
mod polygon;
mod vertex;

#[cfg(test)]
mod tests;

use crate::error::MeshError;
use crate::mesh::Mesh;
use bsp::BspNode;
use polygon::Polygon;

/// Computes the union of two meshes (A ∪ B).
///
/// Returns a mesh containing all volume from both inputs.
///
/// # Arguments
///
/// * `a` - First mesh
/// * `b` - Second mesh
///
/// # Returns
///
/// A new mesh representing A ∪ B.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::boolean::union;
///
/// let result = union(&mesh_a, &mesh_b)?;
/// ```
pub fn union(a: &Mesh, b: &Mesh) -> Result<Mesh, MeshError> {
    // Handle empty meshes
    if a.is_empty() {
        return Ok(b.clone());
    }
    if b.is_empty() {
        return Ok(a.clone());
    }

    // Check if bounding boxes overlap - if not, just merge
    if !bounding_boxes_overlap(a, b) {
        return Ok(merge_meshes(a, b));
    }

    // Convert meshes to polygons
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    // Build BSP trees
    let mut bsp_a = BspNode::new(polys_a);
    let mut bsp_b = BspNode::new(polys_b);

    // Union algorithm: A.clipTo(B); B.clipTo(A); B.invert(); B.clipTo(A); B.invert()
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();

    // Combine polygons
    let mut result_polys = bsp_a.all_polygons();
    result_polys.extend(bsp_b.all_polygons());

    // Convert back to mesh
    polygons_to_mesh(&result_polys)
}

/// Computes the difference of two meshes (A - B).
///
/// Returns a mesh containing volume from A that is not in B.
///
/// # Arguments
///
/// * `a` - Mesh to subtract from
/// * `b` - Mesh to subtract
///
/// # Returns
///
/// A new mesh representing A - B.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::boolean::difference;
///
/// let result = difference(&mesh_a, &mesh_b)?;
/// ```
pub fn difference(a: &Mesh, b: &Mesh) -> Result<Mesh, MeshError> {
    // Handle empty meshes
    if a.is_empty() {
        return Ok(Mesh::new());
    }
    if b.is_empty() {
        return Ok(a.clone());
    }

    // Check if bounding boxes overlap - if not, return A unchanged
    if !bounding_boxes_overlap(a, b) {
        return Ok(a.clone());
    }

    // Convert meshes to polygons
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    // Build BSP trees
    let mut bsp_a = BspNode::new(polys_a);
    let mut bsp_b = BspNode::new(polys_b);

    // Difference algorithm: A.invert(); A.clipTo(B); B.clipTo(A); B.invert(); B.clipTo(A); B.invert(); A.build(B.allPolygons()); A.invert()
    bsp_a.invert();
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();

    // Combine polygons
    let mut result_polys = bsp_a.all_polygons();
    result_polys.extend(bsp_b.all_polygons());

    // Build final tree and invert back
    let mut result = BspNode::new(result_polys);
    result.invert();

    // Convert back to mesh
    polygons_to_mesh(&result.all_polygons())
}

/// Computes the intersection of two meshes (A ∩ B).
///
/// Returns a mesh containing only the volume common to both inputs.
///
/// # Arguments
///
/// * `a` - First mesh
/// * `b` - Second mesh
///
/// # Returns
///
/// A new mesh representing A ∩ B.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::boolean::intersection;
///
/// let result = intersection(&mesh_a, &mesh_b)?;
/// ```
pub fn intersection(a: &Mesh, b: &Mesh) -> Result<Mesh, MeshError> {
    // Handle empty meshes
    if a.is_empty() || b.is_empty() {
        return Ok(Mesh::new());
    }

    // Check if bounding boxes overlap - if not, result is empty
    if !bounding_boxes_overlap(a, b) {
        return Ok(Mesh::new());
    }

    // Convert meshes to polygons
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    // Build BSP trees
    let mut bsp_a = BspNode::new(polys_a);
    let mut bsp_b = BspNode::new(polys_b);

    // Intersection algorithm: A.invert(); B.clipTo(A); B.invert(); A.clipTo(B); B.clipTo(A)
    bsp_a.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);

    // Combine polygons
    let mut result_polys = bsp_a.all_polygons();
    result_polys.extend(bsp_b.all_polygons());

    // Build final tree and invert back
    let mut result = BspNode::new(result_polys);
    result.invert();

    // Convert back to mesh
    polygons_to_mesh(&result.all_polygons())
}

/// Checks if two mesh bounding boxes overlap.
fn bounding_boxes_overlap(a: &Mesh, b: &Mesh) -> bool {
    let (min_a, max_a) = a.bounding_box();
    let (min_b, max_b) = b.bounding_box();

    // Check overlap on all three axes
    min_a.x <= max_b.x && max_a.x >= min_b.x &&
    min_a.y <= max_b.y && max_a.y >= min_b.y &&
    min_a.z <= max_b.z && max_a.z >= min_b.z
}

/// Merges two non-overlapping meshes.
fn merge_meshes(a: &Mesh, b: &Mesh) -> Mesh {
    let mut result = a.clone();
    result.merge(b);
    result
}

/// Converts a mesh to a list of polygons.
fn mesh_to_polygons(mesh: &Mesh) -> Vec<Polygon> {
    mesh.triangles()
        .iter()
        .map(|tri| {
            let v0 = vertex::Vertex::new(mesh.vertex(tri[0]));
            let v1 = vertex::Vertex::new(mesh.vertex(tri[1]));
            let v2 = vertex::Vertex::new(mesh.vertex(tri[2]));
            Polygon::new(vec![v0, v1, v2])
        })
        .collect()
}

/// Converts a list of polygons back to a mesh.
fn polygons_to_mesh(polygons: &[Polygon]) -> Result<Mesh, MeshError> {
    let mut mesh = Mesh::new();

    for poly in polygons {
        if poly.vertices.len() < 3 {
            continue;
        }

        // Add vertices
        let base_idx = mesh.vertex_count() as u32;
        for v in &poly.vertices {
            mesh.add_vertex(v.position);
        }

        // Fan triangulation for polygons with more than 3 vertices
        for i in 1..poly.vertices.len() - 1 {
            mesh.add_triangle(base_idx, base_idx + i as u32, base_idx + (i + 1) as u32);
        }
    }

    Ok(mesh)
}

//! # Boolean Operation Mesh Builders
//!
//! Builds meshes from boolean operations (union, difference, intersection)
//! and convex geometry operations (hull, minkowski).

use crate::error::MeshError;
use crate::mesh::Mesh;
use crate::ops::{convex_hull, difference, intersection, minkowski_sum, union};
use openscad_eval::GeometryNode;

use super::build_mesh_into;

// =============================================================================
// UNION
// =============================================================================

/// Build union of child geometries.
///
/// ## OpenSCAD union
///
/// Combines multiple shapes into one.
///
/// ## Example
///
/// ```text
/// union() {
///     cube(10);
///     translate([5, 0, 0]) sphere(5);
/// }
/// ```
pub fn build_union(mesh: &mut Mesh, children: &[GeometryNode]) -> Result<(), MeshError> {
    if children.is_empty() {
        return Ok(());
    }

    // Build first child
    let mut result = Mesh::new();
    build_mesh_into(&mut result, &children[0])?;

    // Union with remaining children
    for child in &children[1..] {
        let mut child_mesh = Mesh::new();
        build_mesh_into(&mut child_mesh, child)?;
        result = union(&result, &child_mesh);
    }

    mesh.merge(&result);
    Ok(())
}

// =============================================================================
// DIFFERENCE
// =============================================================================

/// Build difference of child geometries.
///
/// ## OpenSCAD difference
///
/// Subtracts subsequent shapes from the first shape.
///
/// ## Example
///
/// ```text
/// difference() {
///     cube(20, center=true);
///     sphere(12);
/// }
/// ```
pub fn build_difference(mesh: &mut Mesh, children: &[GeometryNode]) -> Result<(), MeshError> {
    if children.is_empty() {
        return Ok(());
    }

    // Build first child (the base shape)
    let mut result = Mesh::new();
    build_mesh_into(&mut result, &children[0])?;

    // Subtract remaining children
    for child in &children[1..] {
        let mut child_mesh = Mesh::new();
        build_mesh_into(&mut child_mesh, child)?;
        result = difference(&result, &child_mesh);
    }

    mesh.merge(&result);
    Ok(())
}

// =============================================================================
// INTERSECTION
// =============================================================================

/// Build intersection of child geometries.
///
/// ## OpenSCAD intersection
///
/// Returns only the volume common to all shapes.
///
/// ## Example
///
/// ```text
/// intersection() {
///     cube(20, center=true);
///     sphere(15);
/// }
/// ```
pub fn build_intersection(mesh: &mut Mesh, children: &[GeometryNode]) -> Result<(), MeshError> {
    if children.is_empty() {
        return Ok(());
    }

    // Build first child
    let mut result = Mesh::new();
    build_mesh_into(&mut result, &children[0])?;

    // Intersect with remaining children
    for child in &children[1..] {
        let mut child_mesh = Mesh::new();
        build_mesh_into(&mut child_mesh, child)?;
        result = intersection(&result, &child_mesh);
    }

    mesh.merge(&result);
    Ok(())
}

// =============================================================================
// HULL
// =============================================================================

/// Build convex hull of child geometries.
///
/// ## OpenSCAD hull
///
/// Creates the convex hull (smallest convex shape) containing all children.
///
/// ## Example
///
/// ```text
/// hull() {
///     sphere(5);
///     translate([20, 0, 0]) sphere(5);
/// }
/// ```
pub fn build_hull(mesh: &mut Mesh, children: &[GeometryNode]) -> Result<(), MeshError> {
    // Collect all vertices from all children
    let mut all_vertices: Vec<[f32; 3]> = Vec::new();
    
    for child in children {
        let mut child_mesh = Mesh::new();
        build_mesh_into(&mut child_mesh, child)?;
        
        // Extract vertices
        for i in (0..child_mesh.vertices.len()).step_by(3) {
            all_vertices.push([
                child_mesh.vertices[i],
                child_mesh.vertices[i + 1],
                child_mesh.vertices[i + 2],
            ]);
        }
    }
    
    if all_vertices.len() < 4 {
        return Ok(());
    }
    
    // Compute convex hull
    let hull_mesh = convex_hull(&all_vertices);
    mesh.merge(&hull_mesh);
    Ok(())
}

// =============================================================================
// MINKOWSKI
// =============================================================================

/// Build Minkowski sum of child geometries.
///
/// ## OpenSCAD minkowski
///
/// Computes the Minkowski sum (dilation) of shapes.
///
/// ## Example
///
/// ```text
/// minkowski() {
///     cube(20, center=true);
///     sphere(5);
/// }
/// ```
pub fn build_minkowski(mesh: &mut Mesh, children: &[GeometryNode]) -> Result<(), MeshError> {
    if children.len() < 2 {
        // If only one child, just return it
        if children.len() == 1 {
            build_mesh_into(mesh, &children[0])?;
        }
        return Ok(());
    }

    // Build first two children
    let mut mesh_a = Mesh::new();
    build_mesh_into(&mut mesh_a, &children[0])?;
    
    let mut mesh_b = Mesh::new();
    build_mesh_into(&mut mesh_b, &children[1])?;
    
    // Compute Minkowski sum
    let mut result = minkowski_sum(&mesh_a, &mesh_b);
    
    // Chain additional children
    for child in &children[2..] {
        let mut child_mesh = Mesh::new();
        build_mesh_into(&mut child_mesh, child)?;
        result = minkowski_sum(&result, &child_mesh);
    }
    
    mesh.merge(&result);
    Ok(())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_union() {
        let mut mesh = Mesh::new();
        let children = vec![
            GeometryNode::Cube { size: [10.0, 10.0, 10.0], center: false },
            GeometryNode::Cube { size: [10.0, 10.0, 10.0], center: true },
        ];
        build_union(&mut mesh, &children).unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_build_difference() {
        let mut mesh = Mesh::new();
        let children = vec![
            GeometryNode::Cube { size: [20.0, 20.0, 20.0], center: true },
            GeometryNode::Sphere { radius: 12.0, fn_: 16 },
        ];
        build_difference(&mut mesh, &children).unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_build_intersection() {
        let mut mesh = Mesh::new();
        let children = vec![
            GeometryNode::Cube { size: [20.0, 20.0, 20.0], center: true },
            GeometryNode::Sphere { radius: 15.0, fn_: 16 },
        ];
        build_intersection(&mut mesh, &children).unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_build_hull() {
        let mut mesh = Mesh::new();
        let children = vec![
            GeometryNode::Sphere { radius: 5.0, fn_: 8 },
            GeometryNode::Translate {
                offset: [20.0, 0.0, 0.0],
                child: Box::new(GeometryNode::Sphere { radius: 5.0, fn_: 8 }),
            },
        ];
        build_hull(&mut mesh, &children).unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_build_minkowski() {
        let mut mesh = Mesh::new();
        let children = vec![
            GeometryNode::Cube { size: [10.0, 10.0, 10.0], center: true },
            GeometryNode::Sphere { radius: 2.0, fn_: 8 },
        ];
        build_minkowski(&mut mesh, &children).unwrap();
        assert!(mesh.vertex_count() > 0);
    }
}

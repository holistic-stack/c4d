//! # Transform Mesh Builders
//!
//! Applies geometric transforms (translate, rotate, scale, mirror) to child meshes.

use crate::error::MeshError;
use crate::mesh::Mesh;
use openscad_eval::GeometryNode;

use super::build_mesh_into;

// =============================================================================
// TRANSLATE
// =============================================================================

/// Build translated geometry.
///
/// ## OpenSCAD translate
///
/// Moves geometry by the given offset.
///
/// ## Example
///
/// ```text
/// translate([10, 5, 0]) cube(5);
/// ```
pub fn build_translate(mesh: &mut Mesh, offset: [f64; 3], child: &GeometryNode) -> Result<(), MeshError> {
    // Build child into temporary mesh
    let mut child_mesh = Mesh::new();
    build_mesh_into(&mut child_mesh, child)?;

    // Apply translation directly to vertices
    let ox = offset[0] as f32;
    let oy = offset[1] as f32;
    let oz = offset[2] as f32;
    
    for i in (0..child_mesh.vertices.len()).step_by(3) {
        child_mesh.vertices[i] += ox;
        child_mesh.vertices[i + 1] += oy;
        child_mesh.vertices[i + 2] += oz;
    }

    // Merge into main mesh
    mesh.merge(&child_mesh);
    Ok(())
}

// =============================================================================
// ROTATE
// =============================================================================

/// Build rotated geometry.
///
/// ## OpenSCAD rotate
///
/// Rotates geometry by the given angles (in degrees) around X, Y, Z axes.
///
/// ## Example
///
/// ```text
/// rotate([0, 0, 45]) cube(10);
/// ```
pub fn build_rotate(mesh: &mut Mesh, angles: [f64; 3], child: &GeometryNode) -> Result<(), MeshError> {
    // Build child into temporary mesh
    let mut child_mesh = Mesh::new();
    build_mesh_into(&mut child_mesh, child)?;

    // Convert angles to radians
    let rx = (angles[0] as f32).to_radians();
    let ry = (angles[1] as f32).to_radians();
    let rz = (angles[2] as f32).to_radians();

    // Rotation matrices (applied in order: X, then Y, then Z)
    let (sx, cx) = (rx.sin(), rx.cos());
    let (sy, cy) = (ry.sin(), ry.cos());
    let (sz, cz) = (rz.sin(), rz.cos());

    // Combined rotation matrix
    let m00 = cy * cz;
    let m01 = sx * sy * cz - cx * sz;
    let m02 = cx * sy * cz + sx * sz;
    let m10 = cy * sz;
    let m11 = sx * sy * sz + cx * cz;
    let m12 = cx * sy * sz - sx * cz;
    let m20 = -sy;
    let m21 = sx * cy;
    let m22 = cx * cy;

    // Apply rotation to vertices and normals
    for i in (0..child_mesh.vertices.len()).step_by(3) {
        let x = child_mesh.vertices[i];
        let y = child_mesh.vertices[i + 1];
        let z = child_mesh.vertices[i + 2];

        child_mesh.vertices[i] = m00 * x + m01 * y + m02 * z;
        child_mesh.vertices[i + 1] = m10 * x + m11 * y + m12 * z;
        child_mesh.vertices[i + 2] = m20 * x + m21 * y + m22 * z;
    }

    for i in (0..child_mesh.normals.len()).step_by(3) {
        let nx = child_mesh.normals[i];
        let ny = child_mesh.normals[i + 1];
        let nz = child_mesh.normals[i + 2];

        child_mesh.normals[i] = m00 * nx + m01 * ny + m02 * nz;
        child_mesh.normals[i + 1] = m10 * nx + m11 * ny + m12 * nz;
        child_mesh.normals[i + 2] = m20 * nx + m21 * ny + m22 * nz;
    }

    mesh.merge(&child_mesh);
    Ok(())
}

// =============================================================================
// SCALE
// =============================================================================

/// Build scaled geometry.
///
/// ## OpenSCAD scale
///
/// Scales geometry by the given factors along X, Y, Z axes.
///
/// ## Example
///
/// ```text
/// scale([2, 1, 0.5]) cube(10);
/// ```
pub fn build_scale(mesh: &mut Mesh, factors: [f64; 3], child: &GeometryNode) -> Result<(), MeshError> {
    // Build child into temporary mesh
    let mut child_mesh = Mesh::new();
    build_mesh_into(&mut child_mesh, child)?;

    let sx = factors[0] as f32;
    let sy = factors[1] as f32;
    let sz = factors[2] as f32;

    // Apply scale to vertices
    for i in (0..child_mesh.vertices.len()).step_by(3) {
        child_mesh.vertices[i] *= sx;
        child_mesh.vertices[i + 1] *= sy;
        child_mesh.vertices[i + 2] *= sz;
    }

    // Normals need inverse-transpose scaling for non-uniform scale
    if sx != sy || sy != sz {
        let inv_sx = if sx.abs() > 1e-10 { 1.0 / sx } else { 0.0 };
        let inv_sy = if sy.abs() > 1e-10 { 1.0 / sy } else { 0.0 };
        let inv_sz = if sz.abs() > 1e-10 { 1.0 / sz } else { 0.0 };

        for i in (0..child_mesh.normals.len()).step_by(3) {
            let nx = child_mesh.normals[i] * inv_sx;
            let ny = child_mesh.normals[i + 1] * inv_sy;
            let nz = child_mesh.normals[i + 2] * inv_sz;
            
            // Renormalize
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len > 0.0 {
                child_mesh.normals[i] = nx / len;
                child_mesh.normals[i + 1] = ny / len;
                child_mesh.normals[i + 2] = nz / len;
            }
        }
    }

    mesh.merge(&child_mesh);
    Ok(())
}

// =============================================================================
// MIRROR
// =============================================================================

/// Build mirrored geometry.
///
/// ## OpenSCAD mirror
///
/// Mirrors geometry across a plane defined by the given normal vector.
///
/// ## Example
///
/// ```text
/// mirror([1, 0, 0]) cube(10); // Mirror across YZ plane
/// ```
pub fn build_mirror(mesh: &mut Mesh, normal: [f64; 3], child: &GeometryNode) -> Result<(), MeshError> {
    // Build child into temporary mesh
    let mut child_mesh = Mesh::new();
    build_mesh_into(&mut child_mesh, child)?;

    let nx = normal[0] as f32;
    let ny = normal[1] as f32;
    let nz = normal[2] as f32;
    
    // Normalize
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-10 {
        mesh.merge(&child_mesh);
        return Ok(());
    }
    
    let nx = nx / len;
    let ny = ny / len;
    let nz = nz / len;

    // Householder reflection matrix: I - 2 * n * n^T
    let m00 = 1.0 - 2.0 * nx * nx;
    let m01 = -2.0 * nx * ny;
    let m02 = -2.0 * nx * nz;
    let m10 = -2.0 * ny * nx;
    let m11 = 1.0 - 2.0 * ny * ny;
    let m12 = -2.0 * ny * nz;
    let m20 = -2.0 * nz * nx;
    let m21 = -2.0 * nz * ny;
    let m22 = 1.0 - 2.0 * nz * nz;

    // Apply reflection to vertices
    for i in (0..child_mesh.vertices.len()).step_by(3) {
        let x = child_mesh.vertices[i];
        let y = child_mesh.vertices[i + 1];
        let z = child_mesh.vertices[i + 2];

        child_mesh.vertices[i] = m00 * x + m01 * y + m02 * z;
        child_mesh.vertices[i + 1] = m10 * x + m11 * y + m12 * z;
        child_mesh.vertices[i + 2] = m20 * x + m21 * y + m22 * z;
    }

    // Apply reflection to normals
    for i in (0..child_mesh.normals.len()).step_by(3) {
        let x = child_mesh.normals[i];
        let y = child_mesh.normals[i + 1];
        let z = child_mesh.normals[i + 2];

        child_mesh.normals[i] = m00 * x + m01 * y + m02 * z;
        child_mesh.normals[i + 1] = m10 * x + m11 * y + m12 * z;
        child_mesh.normals[i + 2] = m20 * x + m21 * y + m22 * z;
    }

    // Reverse triangle winding (mirror flips handedness)
    for i in (0..child_mesh.indices.len()).step_by(3) {
        child_mesh.indices.swap(i + 1, i + 2);
    }

    mesh.merge(&child_mesh);
    Ok(())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_translate() {
        let mut mesh = Mesh::new();
        let child = GeometryNode::Cube { size: [1.0, 1.0, 1.0], center: false };
        build_translate(&mut mesh, [10.0, 0.0, 0.0], &child).unwrap();
        assert_eq!(mesh.vertex_count(), 24);
        // First vertex should be translated
        assert!((mesh.vertices[0] - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_build_rotate() {
        let mut mesh = Mesh::new();
        let child = GeometryNode::Cube { size: [1.0, 1.0, 1.0], center: false };
        build_rotate(&mut mesh, [0.0, 0.0, 90.0], &child).unwrap();
        assert_eq!(mesh.vertex_count(), 24);
    }

    #[test]
    fn test_build_scale() {
        let mut mesh = Mesh::new();
        let child = GeometryNode::Cube { size: [1.0, 1.0, 1.0], center: false };
        build_scale(&mut mesh, [2.0, 2.0, 2.0], &child).unwrap();
        assert_eq!(mesh.vertex_count(), 24);
    }

    #[test]
    fn test_build_mirror() {
        let mut mesh = Mesh::new();
        let child = GeometryNode::Cube { size: [1.0, 1.0, 1.0], center: false };
        build_mirror(&mut mesh, [1.0, 0.0, 0.0], &child).unwrap();
        assert_eq!(mesh.vertex_count(), 24);
    }
}

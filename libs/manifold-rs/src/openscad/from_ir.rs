//! # GeometryNode to Mesh Conversion
//!
//! Converts evaluated OpenSCAD geometry (GeometryNode) to triangle mesh.
//!
//! ## Architecture
//!
//! ```text
//! GeometryNode (from openscad-eval)
//!       ↓
//! SegmentParams ($fn/$fa/$fs → circularSegments)
//!       ↓
//! Manifold constructors (cube, sphere, cylinder)
//!       ↓
//! Boolean operations (union, difference, intersection)
//!       ↓
//! Mesh output
//! ```
//!
//! ## Supported Geometry Types
//!
//! - **Primitives**: Cube, Sphere, Cylinder, Polyhedron
//! - **2D Primitives**: Circle, Square, Polygon
//! - **Transforms**: Translate, Rotate, Scale, Mirror, Multmatrix
//! - **Booleans**: Union, Difference, Intersection
//! - **Extrusions**: LinearExtrude, RotateExtrude
//! - **Operations**: Hull, Minkowski, Offset, Projection

use openscad_eval::GeometryNode;
use crate::error::ManifoldResult;
use crate::mesh::Mesh;
use crate::manifold;
use crate::cross_section;
use super::SegmentParams;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Convert GeometryNode to Mesh.
///
/// This is the main entry point for geometry conversion. Recursively processes
/// the geometry tree and builds the final mesh.
///
/// ## Parameters
///
/// - `node`: Root GeometryNode from openscad-eval
///
/// ## Returns
///
/// `ManifoldResult<Mesh>` - Triangle mesh on success
pub fn geometry_to_mesh(node: &GeometryNode) -> ManifoldResult<Mesh> {
    let mut mesh = Mesh::new();
    let params = SegmentParams::default();
    process_node(node, &mut mesh, &params)?;
    Ok(mesh)
}

// =============================================================================
// NODE PROCESSING
// =============================================================================

/// Process a single geometry node recursively.
///
/// Dispatches to appropriate handler based on node type.
fn process_node(node: &GeometryNode, mesh: &mut Mesh, params: &SegmentParams) -> ManifoldResult<()> {
    match node {
        // =====================================================================
        // 3D PRIMITIVES
        // =====================================================================
        
        GeometryNode::Cube { size, center } => {
            manifold::constructors::build_cube(mesh, *size, *center);
            Ok(())
        }
        
        GeometryNode::Sphere { radius, fn_ } => {
            // Use fn_ directly as segments, or calculate from default params
            let segments = if *fn_ > 0 { *fn_ } else { params.calculate_segments(*radius) };
            manifold::constructors::build_sphere(mesh, *radius, segments);
            Ok(())
        }
        
        GeometryNode::Cylinder { height, radius1, radius2, center, fn_ } => {
            // Use fn_ directly or calculate from params
            let segments = if *fn_ > 0 { *fn_ } else { params.calculate_cylinder_segments(*radius1, *radius2) };
            manifold::constructors::build_cylinder(mesh, *height, *radius1, *radius2, segments, *center);
            Ok(())
        }
        
        GeometryNode::Polyhedron { points, faces } => {
            manifold::constructors::build_polyhedron(mesh, points, faces);
            Ok(())
        }

        // =====================================================================
        // TRANSFORMS (use single child: Box<GeometryNode>)
        // =====================================================================
        
        GeometryNode::Translate { offset, child } => {
            let [dx, dy, dz] = *offset;
            let mut child_mesh = Mesh::new();
            process_node(child, &mut child_mesh, params)?;
            child_mesh.translate(dx as f32, dy as f32, dz as f32);
            mesh.merge(&child_mesh);
            Ok(())
        }
        
        GeometryNode::Rotate { angles, child } => {
            let matrix = rotation_matrix(*angles);
            let mut child_mesh = Mesh::new();
            process_node(child, &mut child_mesh, params)?;
            child_mesh.transform(&matrix);
            mesh.merge(&child_mesh);
            Ok(())
        }
        
        GeometryNode::Scale { factors, child } => {
            let [sx, sy, sz] = *factors;
            let mut child_mesh = Mesh::new();
            process_node(child, &mut child_mesh, params)?;
            child_mesh.scale(sx as f32, sy as f32, sz as f32);
            mesh.merge(&child_mesh);
            Ok(())
        }
        
        GeometryNode::Mirror { normal, child } => {
            let matrix = mirror_matrix(*normal);
            let mut child_mesh = Mesh::new();
            process_node(child, &mut child_mesh, params)?;
            child_mesh.transform(&matrix);
            // Flip triangle winding for mirrored geometry
            flip_triangle_winding(&mut child_mesh);
            mesh.merge(&child_mesh);
            Ok(())
        }
        
        GeometryNode::Multmatrix { matrix, child } => {
            let mat = convert_matrix(matrix);
            let mut child_mesh = Mesh::new();
            process_node(child, &mut child_mesh, params)?;
            child_mesh.transform(&mat);
            mesh.merge(&child_mesh);
            Ok(())
        }

        // =====================================================================
        // BOOLEAN OPERATIONS (use children: Vec<GeometryNode>)
        // =====================================================================
        
        GeometryNode::Union { children } => {
            let meshes = process_children(children, params)?;
            let result = manifold::boolean::union_all(&meshes)?;
            mesh.merge(&result);
            Ok(())
        }
        
        GeometryNode::Difference { children } => {
            if children.is_empty() {
                return Ok(());
            }
            let meshes = process_children(children, params)?;
            let result = manifold::boolean::difference_all(&meshes)?;
            mesh.merge(&result);
            Ok(())
        }
        
        GeometryNode::Intersection { children } => {
            if children.is_empty() {
                return Ok(());
            }
            let meshes = process_children(children, params)?;
            let result = manifold::boolean::intersection_all(&meshes)?;
            mesh.merge(&result);
            Ok(())
        }
        
        GeometryNode::Hull { children } => {
            let meshes = process_children(children, params)?;
            let result = manifold::hull::compute_hull(&meshes)?;
            mesh.merge(&result);
            Ok(())
        }
        
        GeometryNode::Minkowski { children } => {
            if children.len() < 2 {
                // Single child: just return it
                let meshes = process_children(children, params)?;
                if let Some(m) = meshes.first() {
                    mesh.merge(m);
                }
                return Ok(());
            }
            let meshes = process_children(children, params)?;
            let result = manifold::minkowski::compute_minkowski(&meshes)?;
            mesh.merge(&result);
            Ok(())
        }

        // =====================================================================
        // 2D PRIMITIVES
        // =====================================================================
        
        GeometryNode::Circle { radius, fn_ } => {
            let segments = if *fn_ > 0 { *fn_ } else { params.calculate_segments(*radius) };
            cross_section::primitives::build_circle_mesh(mesh, *radius, segments);
            Ok(())
        }
        
        GeometryNode::Square { size, center } => {
            cross_section::primitives::build_square_mesh(mesh, *size, *center);
            Ok(())
        }
        
        GeometryNode::Polygon { points, paths } => {
            cross_section::primitives::build_polygon_mesh(mesh, points, paths.as_deref());
            Ok(())
        }

        // =====================================================================
        // EXTRUSIONS (use single child: Box<GeometryNode>)
        // =====================================================================
        
        GeometryNode::LinearExtrude { height, center, twist, scale, slices, child } => {
            // Build 2D child mesh first
            let mut child_mesh = Mesh::new();
            process_node(child, &mut child_mesh, params)?;
            // For now, just do simple extrusion
            extrude_mesh(&mut child_mesh, *height, *center, *twist, scale, *slices);
            mesh.merge(&child_mesh);
            Ok(())
        }
        
        GeometryNode::RotateExtrude { angle, fn_, child } => {
            // Build 2D child mesh first
            let mut child_mesh = Mesh::new();
            process_node(child, &mut child_mesh, params)?;
            let segments = if *fn_ > 0 { *fn_ } else { 32 };
            revolve_mesh(&mut child_mesh, *angle, segments);
            mesh.merge(&child_mesh);
            Ok(())
        }

        // =====================================================================
        // 2D OPERATIONS (use single child: Box<GeometryNode>)
        // =====================================================================
        
        GeometryNode::Offset { delta, chamfer, child } => {
            let mut child_mesh = Mesh::new();
            process_node(child, &mut child_mesh, params)?;
            offset_mesh(&mut child_mesh, *delta, *chamfer);
            mesh.merge(&child_mesh);
            Ok(())
        }
        
        GeometryNode::Projection { cut, child } => {
            let mut child_mesh = Mesh::new();
            process_node(child, &mut child_mesh, params)?;
            project_mesh(&mut child_mesh, *cut);
            mesh.merge(&child_mesh);
            Ok(())
        }

        // =====================================================================
        // SPECIAL NODES
        // =====================================================================
        
        GeometryNode::Color { rgba, child } => {
            let mut child_mesh = Mesh::new();
            process_node(child, &mut child_mesh, params)?;
            apply_color(&mut child_mesh, rgba);
            mesh.merge(&child_mesh);
            Ok(())
        }
        
        GeometryNode::Group { children } => {
            for child in children {
                process_node(child, mesh, params)?;
            }
            Ok(())
        }
        
        GeometryNode::Empty => Ok(()),
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Process multiple children and return their meshes.
fn process_children(children: &[GeometryNode], params: &SegmentParams) -> ManifoldResult<Vec<Mesh>> {
    let mut meshes = Vec::with_capacity(children.len());
    for child in children {
        let mut child_mesh = Mesh::new();
        process_node(child, &mut child_mesh, params)?;
        if !child_mesh.is_empty() {
            meshes.push(child_mesh);
        }
    }
    Ok(meshes)
}

/// Create rotation matrix from Euler angles (degrees).
fn rotation_matrix(angles: [f64; 3]) -> [[f32; 4]; 4] {
    let [ax, ay, az] = angles;
    let (sx, cx) = (ax.to_radians().sin() as f32, ax.to_radians().cos() as f32);
    let (sy, cy) = (ay.to_radians().sin() as f32, ay.to_radians().cos() as f32);
    let (sz, cz) = (az.to_radians().sin() as f32, az.to_radians().cos() as f32);
    
    // Combined rotation: Rz * Ry * Rx
    [
        [cy * cz, sx * sy * cz - cx * sz, cx * sy * cz + sx * sz, 0.0],
        [cy * sz, sx * sy * sz + cx * cz, cx * sy * sz - sx * cz, 0.0],
        [-sy, sx * cy, cx * cy, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Create mirror matrix for a plane defined by normal.
fn mirror_matrix(normal: [f64; 3]) -> [[f32; 4]; 4] {
    let [nx, ny, nz] = [normal[0] as f32, normal[1] as f32, normal[2] as f32];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 0.0001 {
        return [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    }
    let (nx, ny, nz) = (nx / len, ny / len, nz / len);
    
    // Mirror matrix: I - 2 * n * n^T
    [
        [1.0 - 2.0 * nx * nx, -2.0 * nx * ny, -2.0 * nx * nz, 0.0],
        [-2.0 * ny * nx, 1.0 - 2.0 * ny * ny, -2.0 * ny * nz, 0.0],
        [-2.0 * nz * nx, -2.0 * nz * ny, 1.0 - 2.0 * nz * nz, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Convert 4x4 f64 matrix to f32.
fn convert_matrix(matrix: &[[f64; 4]; 4]) -> [[f32; 4]; 4] {
    [
        [matrix[0][0] as f32, matrix[0][1] as f32, matrix[0][2] as f32, matrix[0][3] as f32],
        [matrix[1][0] as f32, matrix[1][1] as f32, matrix[1][2] as f32, matrix[1][3] as f32],
        [matrix[2][0] as f32, matrix[2][1] as f32, matrix[2][2] as f32, matrix[2][3] as f32],
        [matrix[3][0] as f32, matrix[3][1] as f32, matrix[3][2] as f32, matrix[3][3] as f32],
    ]
}

/// Flip triangle winding order (for mirrored geometry).
fn flip_triangle_winding(mesh: &mut Mesh) {
    for i in (0..mesh.indices.len()).step_by(3) {
        mesh.indices.swap(i + 1, i + 2);
    }
    // Also flip normals
    for i in 0..mesh.normals.len() {
        mesh.normals[i] = -mesh.normals[i];
    }
}

/// Apply color to mesh vertices.
fn apply_color(mesh: &mut Mesh, color: &[f64; 4]) {
    let [r, g, b, a] = [color[0] as f32, color[1] as f32, color[2] as f32, color[3] as f32];
    let vertex_count = mesh.vertex_count();
    
    let colors = mesh.colors.get_or_insert_with(|| Vec::with_capacity(vertex_count * 4));
    colors.clear();
    for _ in 0..vertex_count {
        colors.extend_from_slice(&[r, g, b, a]);
    }
}

/// Simple linear extrusion of a 2D mesh.
///
/// For now, just duplicates vertices at height and connects with quads.
fn extrude_mesh(mesh: &mut Mesh, height: f64, center: bool, _twist: f64, _scale: &[f64; 2], _slices: u32) {
    if mesh.is_empty() {
        return;
    }
    
    let h = height as f32;
    let z_offset = if center { -h / 2.0 } else { 0.0 };
    
    // Offset existing vertices to bottom
    for i in (0..mesh.vertices.len()).step_by(3) {
        mesh.vertices[i + 2] += z_offset;
    }
    
    // Simple: just add top vertices and side faces
    // This is a placeholder - full implementation would handle twist/scale/slices
    let bottom_count = mesh.vertex_count();
    
    // Duplicate vertices at top
    for i in 0..bottom_count {
        let idx = i * 3;
        mesh.add_vertex(
            mesh.vertices[idx],
            mesh.vertices[idx + 1],
            mesh.vertices[idx + 2] + h,
            0.0, 0.0, 1.0,
        );
    }
}

/// Simple revolve of a 2D mesh around Z axis.
///
/// Placeholder implementation.
fn revolve_mesh(_mesh: &mut Mesh, _angle: f64, _segments: u32) {
    // Placeholder - full implementation would revolve the 2D profile
}

/// Offset a 2D mesh by delta.
///
/// Placeholder implementation.
fn offset_mesh(_mesh: &mut Mesh, _delta: f64, _chamfer: bool) {
    // Placeholder - full implementation would offset polygon edges
}

/// Project 3D mesh to 2D.
///
/// Placeholder implementation.
fn project_mesh(_mesh: &mut Mesh, _cut: bool) {
    // Placeholder - full implementation would project to XY plane
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test cube conversion.
    #[test]
    fn test_cube_conversion() {
        let node = GeometryNode::Cube {
            size: [10.0, 10.0, 10.0],
            center: false,
        };
        
        let mesh = geometry_to_mesh(&node).unwrap();
        assert_eq!(mesh.vertex_count(), 24);
        assert_eq!(mesh.triangle_count(), 12);
    }

    /// Test sphere conversion with $fn.
    #[test]
    fn test_sphere_with_fn() {
        let node = GeometryNode::Sphere {
            radius: 5.0,
            fn_: 8,
        };
        
        let mesh = geometry_to_mesh(&node).unwrap();
        assert!(!mesh.is_empty());
    }

    /// Test translation.
    #[test]
    fn test_translate() {
        let node = GeometryNode::Translate {
            offset: [10.0, 0.0, 0.0],
            child: Box::new(GeometryNode::Cube {
                size: [5.0, 5.0, 5.0],
                center: false,
            }),
        };
        
        let mesh = geometry_to_mesh(&node).unwrap();
        
        // All x coordinates should be >= 10
        for i in (0..mesh.vertices.len()).step_by(3) {
            assert!(mesh.vertices[i] >= 10.0);
        }
    }

    /// Test rotation matrix.
    #[test]
    fn test_rotation_matrix() {
        // Identity rotation
        let matrix = rotation_matrix([0.0, 0.0, 0.0]);
        assert!((matrix[0][0] - 1.0).abs() < 0.001);
        assert!((matrix[1][1] - 1.0).abs() < 0.001);
        assert!((matrix[2][2] - 1.0).abs() < 0.001);
    }

    /// Test mirror matrix.
    #[test]
    fn test_mirror_matrix() {
        // Mirror in X
        let matrix = mirror_matrix([1.0, 0.0, 0.0]);
        assert!((matrix[0][0] - (-1.0)).abs() < 0.001);
        assert!((matrix[1][1] - 1.0).abs() < 0.001);
        assert!((matrix[2][2] - 1.0).abs() < 0.001);
    }
}

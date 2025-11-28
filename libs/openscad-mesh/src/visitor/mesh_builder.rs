//! # Mesh Builder
//!
//! Main entry point for converting GeometryNode tree to triangle mesh.
//! Delegates to specialized modules for each geometry type (SRP).
//!
//! ## Example
//!
//! ```rust
//! use openscad_eval::GeometryNode;
//! use openscad_mesh::visitor::build_mesh;
//!
//! let node = GeometryNode::Cube { size: [10.0, 10.0, 10.0], center: false };
//! let mesh = build_mesh(&node).unwrap();
//! ```

use crate::error::MeshError;
use crate::mesh::Mesh;
use openscad_eval::GeometryNode;

use super::booleans;
use super::extrusions;
use super::ops_2d;
use super::primitives_2d;
use super::primitives_3d;
use super::transforms;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Build a mesh from a geometry node.
///
/// ## Parameters
///
/// - `node`: Root geometry node
///
/// ## Returns
///
/// Triangle mesh ready for rendering
pub fn build_mesh(node: &GeometryNode) -> Result<Mesh, MeshError> {
    let mut mesh = Mesh::new();
    build_mesh_into(&mut mesh, node)?;
    Ok(mesh)
}

/// Build a mesh from a geometry node into an existing mesh.
///
/// This is used internally for building child meshes.
pub fn build_mesh_into(mesh: &mut Mesh, node: &GeometryNode) -> Result<(), MeshError> {
    match node {
        // =====================================================================
        // 3D PRIMITIVES
        // =====================================================================
        GeometryNode::Cube { size, center } => {
            primitives_3d::build_cube(mesh, *size, *center);
        }
        GeometryNode::Sphere { radius, fn_ } => {
            primitives_3d::build_sphere(mesh, *radius, *fn_);
        }
        GeometryNode::Cylinder { height, radius1, radius2, center, fn_ } => {
            primitives_3d::build_cylinder(mesh, *height, *radius1, *radius2, *fn_, *center);
        }
        GeometryNode::Polyhedron { points, faces, .. } => {
            primitives_3d::build_polyhedron(mesh, points, faces);
        }

        // =====================================================================
        // 2D PRIMITIVES (rendered as flat at Z=0)
        // =====================================================================
        GeometryNode::Circle { radius, fn_ } => {
            primitives_2d::build_circle_2d(mesh, *radius, *fn_);
        }
        GeometryNode::Square { size, center } => {
            primitives_2d::build_square_2d(mesh, *size, *center);
        }
        GeometryNode::Polygon { points, paths } => {
            primitives_2d::build_polygon_2d(mesh, points, paths.as_ref());
        }

        // =====================================================================
        // EXTRUSIONS (2D to 3D)
        // =====================================================================
        GeometryNode::LinearExtrude { height, twist, scale, slices, center, child } => {
            extrusions::build_linear_extrude(mesh, *height, *twist, *scale, *slices, *center, child);
        }
        GeometryNode::RotateExtrude { angle, fn_, child } => {
            extrusions::build_rotate_extrude(mesh, *angle, *fn_, child);
        }

        // =====================================================================
        // 2D OPERATIONS
        // =====================================================================
        GeometryNode::Offset { delta, chamfer, child } => {
            ops_2d::build_offset_2d(mesh, *delta, *chamfer, child);
        }
        GeometryNode::Projection { cut, child } => {
            ops_2d::build_projection(mesh, *cut, child)?;
        }

        // =====================================================================
        // TRANSFORMS
        // =====================================================================
        GeometryNode::Translate { offset, child } => {
            transforms::build_translate(mesh, *offset, child)?;
        }
        GeometryNode::Rotate { angles, child } => {
            transforms::build_rotate(mesh, *angles, child)?;
        }
        GeometryNode::Scale { factors, child } => {
            transforms::build_scale(mesh, *factors, child)?;
        }
        GeometryNode::Mirror { normal, child } => {
            transforms::build_mirror(mesh, *normal, child)?;
        }
        GeometryNode::Color { child, .. } => {
            // Color is for rendering hints only, pass through
            build_mesh_into(mesh, child)?;
        }

        // =====================================================================
        // BOOLEAN OPERATIONS
        // =====================================================================
        GeometryNode::Union { children } => {
            booleans::build_union(mesh, children)?;
        }
        GeometryNode::Difference { children } => {
            booleans::build_difference(mesh, children)?;
        }
        GeometryNode::Intersection { children } => {
            booleans::build_intersection(mesh, children)?;
        }

        // =====================================================================
        // CONVEX OPERATIONS
        // =====================================================================
        GeometryNode::Hull { children } => {
            booleans::build_hull(mesh, children)?;
        }
        GeometryNode::Minkowski { children } => {
            booleans::build_minkowski(mesh, children)?;
        }

        // =====================================================================
        // GROUPS
        // =====================================================================
        GeometryNode::Group { children } => {
            for child in children {
                build_mesh_into(mesh, child)?;
            }
        }

        // =====================================================================
        // EMPTY / UNSUPPORTED
        // =====================================================================
        GeometryNode::Empty => {}

        _ => {
            // Skip unsupported geometries for now
        }
    }

    Ok(())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_mesh_cube() {
        let node = GeometryNode::Cube { size: [1.0, 1.0, 1.0], center: false };
        let mesh = build_mesh(&node).unwrap();
        assert_eq!(mesh.vertex_count(), 24);
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_build_mesh_sphere() {
        let node = GeometryNode::Sphere { radius: 5.0, fn_: 16 };
        let mesh = build_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_build_mesh_cylinder() {
        let node = GeometryNode::Cylinder {
            height: 10.0,
            radius1: 5.0,
            radius2: 5.0,
            center: false,
            fn_: 16,
        };
        let mesh = build_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_build_mesh_circle_2d() {
        let node = GeometryNode::Circle { radius: 10.0, fn_: 16 };
        let mesh = build_mesh(&node).unwrap();
        assert_eq!(mesh.vertex_count(), 17);
        assert_eq!(mesh.triangle_count(), 16);
    }

    #[test]
    fn test_build_mesh_square_2d() {
        let node = GeometryNode::Square { size: [10.0, 5.0], center: false };
        let mesh = build_mesh(&node).unwrap();
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.triangle_count(), 2);
    }

    #[test]
    fn test_build_mesh_linear_extrude() {
        let child = GeometryNode::Circle { radius: 5.0, fn_: 8 };
        let node = GeometryNode::LinearExtrude {
            height: 10.0,
            twist: 0.0,
            scale: [1.0, 1.0],
            slices: 1,
            center: false,
            child: Box::new(child),
        };
        let mesh = build_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_build_mesh_translate() {
        let child = GeometryNode::Cube { size: [1.0, 1.0, 1.0], center: false };
        let node = GeometryNode::Translate {
            offset: [10.0, 0.0, 0.0],
            child: Box::new(child),
        };
        let mesh = build_mesh(&node).unwrap();
        assert_eq!(mesh.vertex_count(), 24);
    }

    #[test]
    fn test_build_mesh_union() {
        let children = vec![
            GeometryNode::Cube { size: [10.0, 10.0, 10.0], center: false },
            GeometryNode::Cube { size: [10.0, 10.0, 10.0], center: true },
        ];
        let node = GeometryNode::Union { children };
        let mesh = build_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_build_mesh_group() {
        let children = vec![
            GeometryNode::Cube { size: [1.0, 1.0, 1.0], center: false },
            GeometryNode::Sphere { radius: 1.0, fn_: 8 },
        ];
        let node = GeometryNode::Group { children };
        let mesh = build_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 24); // More than just cube
    }

    #[test]
    fn test_build_mesh_empty() {
        let node = GeometryNode::Empty;
        let mesh = build_mesh(&node).unwrap();
        assert_eq!(mesh.vertex_count(), 0);
    }
}

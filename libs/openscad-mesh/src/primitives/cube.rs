//! # Cube Primitive
//!
//! Generates mesh for cube and rectangular prism shapes.

use crate::error::MeshError;
use crate::mesh::Mesh;
use glam::DVec3;

/// Creates a cube or rectangular prism mesh.
///
/// # Arguments
///
/// * `size` - Dimensions [x, y, z]
/// * `center` - If true, center at origin; if false, corner at origin
///
/// # Returns
///
/// A mesh with 8 vertices and 12 triangles (2 per face).
///
/// # Example
///
/// ```rust
/// use openscad_mesh::primitives::create_cube;
/// use glam::DVec3;
///
/// let mesh = create_cube(DVec3::splat(10.0), false).unwrap();
/// assert_eq!(mesh.vertex_count(), 8);
/// assert_eq!(mesh.triangle_count(), 12);
/// ```
pub fn create_cube(size: DVec3, center: bool) -> Result<Mesh, MeshError> {
    // Validate size
    if size.x <= 0.0 || size.y <= 0.0 || size.z <= 0.0 {
        return Err(MeshError::degenerate(
            format!("Cube size must be positive: {:?}", size),
            None,
        ));
    }

    let mut mesh = Mesh::with_capacity(8, 12);

    // Calculate min/max corners
    let (min, max) = if center {
        let half = size / 2.0;
        (-half, half)
    } else {
        (DVec3::ZERO, size)
    };

    // Add 8 vertices (corners of the cube)
    // Bottom face (z = min.z)
    let v0 = mesh.add_vertex(DVec3::new(min.x, min.y, min.z)); // 0: left-front-bottom
    let v1 = mesh.add_vertex(DVec3::new(max.x, min.y, min.z)); // 1: right-front-bottom
    let v2 = mesh.add_vertex(DVec3::new(max.x, max.y, min.z)); // 2: right-back-bottom
    let v3 = mesh.add_vertex(DVec3::new(min.x, max.y, min.z)); // 3: left-back-bottom

    // Top face (z = max.z)
    let v4 = mesh.add_vertex(DVec3::new(min.x, min.y, max.z)); // 4: left-front-top
    let v5 = mesh.add_vertex(DVec3::new(max.x, min.y, max.z)); // 5: right-front-top
    let v6 = mesh.add_vertex(DVec3::new(max.x, max.y, max.z)); // 6: right-back-top
    let v7 = mesh.add_vertex(DVec3::new(min.x, max.y, max.z)); // 7: left-back-top

    // Add 12 triangles (2 per face, counter-clockwise winding for outward normals)

    // Bottom face (z = min.z) - looking from below, CCW
    mesh.add_triangle(v0, v2, v1);
    mesh.add_triangle(v0, v3, v2);

    // Top face (z = max.z) - looking from above, CCW
    mesh.add_triangle(v4, v5, v6);
    mesh.add_triangle(v4, v6, v7);

    // Front face (y = min.y) - looking from front, CCW
    mesh.add_triangle(v0, v1, v5);
    mesh.add_triangle(v0, v5, v4);

    // Back face (y = max.y) - looking from back, CCW
    mesh.add_triangle(v2, v3, v7);
    mesh.add_triangle(v2, v7, v6);

    // Left face (x = min.x) - looking from left, CCW
    mesh.add_triangle(v3, v0, v4);
    mesh.add_triangle(v3, v4, v7);

    // Right face (x = max.x) - looking from right, CCW
    mesh.add_triangle(v1, v2, v6);
    mesh.add_triangle(v1, v6, v5);

    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_vertex_count() {
        let mesh = create_cube(DVec3::splat(10.0), false).unwrap();
        assert_eq!(mesh.vertex_count(), 8);
    }

    #[test]
    fn test_cube_triangle_count() {
        let mesh = create_cube(DVec3::splat(10.0), false).unwrap();
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_cube_not_centered() {
        let mesh = create_cube(DVec3::splat(10.0), false).unwrap();
        let (min, max) = mesh.bounding_box();
        assert_eq!(min, DVec3::ZERO);
        assert_eq!(max, DVec3::splat(10.0));
    }

    #[test]
    fn test_cube_centered() {
        let mesh = create_cube(DVec3::splat(10.0), true).unwrap();
        let (min, max) = mesh.bounding_box();
        assert_eq!(min, DVec3::splat(-5.0));
        assert_eq!(max, DVec3::splat(5.0));
    }

    #[test]
    fn test_cube_rectangular() {
        let mesh = create_cube(DVec3::new(10.0, 20.0, 30.0), false).unwrap();
        let (min, max) = mesh.bounding_box();
        assert_eq!(min, DVec3::ZERO);
        assert_eq!(max, DVec3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn test_cube_validates() {
        let mesh = create_cube(DVec3::splat(10.0), false).unwrap();
        assert!(mesh.validate());
    }

    #[test]
    fn test_cube_invalid_size() {
        let result = create_cube(DVec3::new(0.0, 10.0, 10.0), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_cube_negative_size() {
        let result = create_cube(DVec3::new(-5.0, 10.0, 10.0), false);
        assert!(result.is_err());
    }
}

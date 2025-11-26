//! # Cylinder Primitive
//!
//! Generates mesh for cylinder and cone shapes.

use crate::error::MeshError;
use crate::mesh::Mesh;
use glam::DVec3;
use std::f64::consts::PI;

/// Creates a cylinder or cone mesh.
///
/// # Arguments
///
/// * `height` - Height along Z axis
/// * `radius_bottom` - Radius at z=0 (or z=-h/2 if centered)
/// * `radius_top` - Radius at z=h (or z=h/2 if centered)
/// * `center` - If true, center vertically at origin
/// * `segments` - Number of segments around circumference
///
/// # Returns
///
/// A mesh representing the cylinder/cone.
///
/// # Example
///
/// ```rust
/// use openscad_mesh::primitives::create_cylinder;
///
/// // Regular cylinder
/// let mesh = create_cylinder(10.0, 5.0, 5.0, false, 32).unwrap();
///
/// // Cone (r2 = 0)
/// let cone = create_cylinder(10.0, 5.0, 0.0, false, 32).unwrap();
/// ```
pub fn create_cylinder(
    height: f64,
    radius_bottom: f64,
    radius_top: f64,
    center: bool,
    segments: u32,
) -> Result<Mesh, MeshError> {
    if height <= 0.0 {
        return Err(MeshError::degenerate(
            format!("Cylinder height must be positive: {}", height),
            None,
        ));
    }

    if radius_bottom < 0.0 || radius_top < 0.0 {
        return Err(MeshError::degenerate(
            format!(
                "Cylinder radii must be non-negative: r1={}, r2={}",
                radius_bottom, radius_top
            ),
            None,
        ));
    }

    if radius_bottom == 0.0 && radius_top == 0.0 {
        return Err(MeshError::degenerate(
            "Cylinder must have at least one non-zero radius",
            None,
        ));
    }

    if segments < 3 {
        return Err(MeshError::degenerate(
            format!("Cylinder segments must be at least 3: {}", segments),
            None,
        ));
    }

    let mut mesh = Mesh::new();

    // Calculate z positions
    let (z_bottom, z_top) = if center {
        (-height / 2.0, height / 2.0)
    } else {
        (0.0, height)
    };

    // Generate bottom circle vertices (or single apex for inverted cone)
    let bottom_indices: Vec<u32> = if radius_bottom > 0.0 {
        (0..segments)
            .map(|j| {
                let theta = 2.0 * PI * j as f64 / segments as f64;
                let x = radius_bottom * theta.cos();
                let y = radius_bottom * theta.sin();
                mesh.add_vertex(DVec3::new(x, y, z_bottom))
            })
            .collect()
    } else {
        // Single apex vertex for inverted cone
        vec![mesh.add_vertex(DVec3::new(0.0, 0.0, z_bottom))]
    };

    // Generate top circle vertices (or single apex for cone)
    let top_indices: Vec<u32> = if radius_top > 0.0 {
        (0..segments)
            .map(|j| {
                let theta = 2.0 * PI * j as f64 / segments as f64;
                let x = radius_top * theta.cos();
                let y = radius_top * theta.sin();
                mesh.add_vertex(DVec3::new(x, y, z_top))
            })
            .collect()
    } else {
        // Single apex vertex for cone
        vec![mesh.add_vertex(DVec3::new(0.0, 0.0, z_top))]
    };

    // Generate side faces
    if radius_bottom > 0.0 && radius_top > 0.0 {
        // Frustum: quads between two circles
        for j in 0..segments {
            let j_next = (j + 1) % segments;

            let b0 = bottom_indices[j as usize];
            let b1 = bottom_indices[j_next as usize];
            let t0 = top_indices[j as usize];
            let t1 = top_indices[j_next as usize];

            mesh.add_triangle(b0, b1, t1);
            mesh.add_triangle(b0, t1, t0);
        }
    } else if radius_bottom > 0.0 {
        // Cone: triangles from bottom circle to top apex
        let apex = top_indices[0];
        for j in 0..segments {
            let j_next = (j + 1) % segments;
            let b0 = bottom_indices[j as usize];
            let b1 = bottom_indices[j_next as usize];
            mesh.add_triangle(b0, b1, apex);
        }
    } else {
        // Inverted cone: triangles from bottom apex to top circle
        let apex = bottom_indices[0];
        for j in 0..segments {
            let j_next = (j + 1) % segments;
            let t0 = top_indices[j as usize];
            let t1 = top_indices[j_next as usize];
            mesh.add_triangle(apex, t1, t0);
        }
    }

    // Generate bottom cap (if radius_bottom > 0)
    if radius_bottom > 0.0 {
        for j in 1..segments - 1 {
            mesh.add_triangle(
                bottom_indices[0],
                bottom_indices[(j + 1) as usize],
                bottom_indices[j as usize],
            );
        }
    }

    // Generate top cap (if radius_top > 0)
    if radius_top > 0.0 {
        for j in 1..segments - 1 {
            mesh.add_triangle(
                top_indices[0],
                top_indices[j as usize],
                top_indices[(j + 1) as usize],
            );
        }
    }

    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cylinder_basic() {
        let mesh = create_cylinder(10.0, 5.0, 5.0, false, 32).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
        assert!(mesh.validate());
    }

    #[test]
    fn test_cylinder_centered() {
        let mesh = create_cylinder(10.0, 5.0, 5.0, true, 32).unwrap();
        let (min, max) = mesh.bounding_box();
        assert!(min.z < 0.0);
        assert!(max.z > 0.0);
    }

    #[test]
    fn test_cylinder_not_centered() {
        let mesh = create_cylinder(10.0, 5.0, 5.0, false, 32).unwrap();
        let (min, max) = mesh.bounding_box();
        assert!(min.z >= 0.0);
        assert!(max.z <= 10.0);
    }

    #[test]
    fn test_cone() {
        let mesh = create_cylinder(10.0, 5.0, 0.0, false, 32).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.validate());
    }

    #[test]
    fn test_inverted_cone() {
        let mesh = create_cylinder(10.0, 0.0, 5.0, false, 32).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.validate());
    }

    #[test]
    fn test_cylinder_invalid_height() {
        let result = create_cylinder(0.0, 5.0, 5.0, false, 32);
        assert!(result.is_err());
    }

    #[test]
    fn test_cylinder_both_radii_zero() {
        let result = create_cylinder(10.0, 0.0, 0.0, false, 32);
        assert!(result.is_err());
    }

    #[test]
    fn test_cylinder_too_few_segments() {
        let result = create_cylinder(10.0, 5.0, 5.0, false, 2);
        assert!(result.is_err());
    }
}

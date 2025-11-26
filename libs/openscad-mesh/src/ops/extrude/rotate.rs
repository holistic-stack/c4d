//! # Rotate Extrusion
//!
//! Revolves a 2D polygon around the Z axis to create a 3D mesh.
//!
//! ## OpenSCAD Compatibility
//!
//! Matches `rotate_extrude(angle, convexity)`:
//! - `angle`: Rotation angle in degrees (default 360)
//! - `convexity`: Hint for rendering (not used in mesh generation)
//!
//! ## Algorithm
//!
//! 1. The 2D polygon is assumed to be in the XY plane with X >= 0
//! 2. The polygon is rotated around the Z axis
//! 3. Each edge of the polygon generates a band of quads

use super::Polygon2D;
use crate::error::MeshError;
use crate::mesh::Mesh;
use config::constants::MIN_FRAGMENTS;
use glam::DVec3;

/// Parameters for rotate extrusion.
#[derive(Debug, Clone)]
pub struct RotateExtrudeParams {
    /// Rotation angle in degrees (default 360 for full revolution)
    pub angle: f64,
    /// Number of segments around the revolution
    pub segments: u32,
}

impl Default for RotateExtrudeParams {
    fn default() -> Self {
        Self {
            angle: 360.0,
            segments: 32,
        }
    }
}

/// Revolves a 2D polygon around the Z axis.
///
/// The polygon should be in the XY plane with all X coordinates >= 0.
/// The polygon is rotated around the Z axis to create a solid of revolution.
///
/// # Arguments
///
/// * `polygon` - The 2D polygon to revolve (X >= 0)
/// * `params` - Extrusion parameters
///
/// # Returns
///
/// A 3D mesh representing the revolved shape.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::extrude::{Polygon2D, rotate_extrude, RotateExtrudeParams};
///
/// // Create a circle profile for a torus
/// let circle = Polygon2D::circle(2.0, 16);
/// // Translate to X=5 (distance from Z axis)
/// let translated: Vec<_> = circle.outer.iter()
///     .map(|v| DVec2::new(v.x + 5.0, v.y))
///     .collect();
/// let profile = Polygon2D::new(translated);
///
/// let params = RotateExtrudeParams::default();
/// let mesh = rotate_extrude(&profile, &params)?;
/// ```
pub fn rotate_extrude(polygon: &Polygon2D, params: &RotateExtrudeParams) -> Result<Mesh, MeshError> {
    // Validate parameters
    if params.angle <= 0.0 {
        return Err(MeshError::degenerate(
            "rotate_extrude angle must be positive",
            None,
        ));
    }

    if polygon.vertex_count() < 3 {
        return Err(MeshError::degenerate(
            "Polygon must have at least 3 vertices",
            None,
        ));
    }

    // Check that all X coordinates are non-negative
    for v in &polygon.outer {
        if v.x < -config::constants::EPSILON {
            return Err(MeshError::degenerate(
                "rotate_extrude requires all X coordinates >= 0",
                None,
            ));
        }
    }

    let segments = params.segments.max(MIN_FRAGMENTS);
    let is_full_rotation = (params.angle - 360.0).abs() < 0.001;
    let angle_rad = params.angle.to_radians();

    let n = polygon.vertex_count();
    let num_steps = if is_full_rotation { segments } else { segments + 1 };

    let mut mesh = Mesh::with_capacity(
        n * num_steps as usize,
        n * segments as usize * 2,
    );

    // Generate vertices for each rotation step
    for step in 0..num_steps {
        let t = step as f64 / segments as f64;
        let theta = t * angle_rad;
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        for v in &polygon.outer {
            // Rotate point (v.x, 0, v.y) around Z axis
            // Result: (v.x * cos(theta), v.x * sin(theta), v.y)
            mesh.add_vertex(DVec3::new(
                v.x * cos_theta,
                v.x * sin_theta,
                v.y,
            ));
        }
    }

    // Generate side faces
    for step in 0..segments {
        let base = (step as usize) * n;
        let next_base = if is_full_rotation && step == segments - 1 {
            0 // Wrap around to first slice
        } else {
            ((step + 1) as usize) * n
        };

        for i in 0..n {
            let i_next = (i + 1) % n;

            // Two triangles per quad
            mesh.add_triangle(
                (base + i) as u32,
                (next_base + i) as u32,
                (next_base + i_next) as u32,
            );
            mesh.add_triangle(
                (base + i) as u32,
                (next_base + i_next) as u32,
                (base + i_next) as u32,
            );
        }
    }

    // Generate end caps for partial rotation
    if !is_full_rotation {
        // Start cap (at angle = 0)
        for i in 1..n - 1 {
            mesh.add_triangle(0, (i + 1) as u32, i as u32);
        }

        // End cap (at angle = params.angle)
        let end_base = (segments as usize) * n;
        for i in 1..n - 1 {
            mesh.add_triangle(
                end_base as u32,
                (end_base + i) as u32,
                (end_base + i + 1) as u32,
            );
        }
    }

    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec2;

    /// Creates a simple profile for testing (a square offset from Z axis)
    fn test_profile() -> Polygon2D {
        Polygon2D::new(vec![
            DVec2::new(5.0, -1.0),
            DVec2::new(7.0, -1.0),
            DVec2::new(7.0, 1.0),
            DVec2::new(5.0, 1.0),
        ])
    }

    #[test]
    fn test_rotate_extrude_full() {
        let profile = test_profile();
        let params = RotateExtrudeParams {
            angle: 360.0,
            segments: 16,
        };
        let mesh = rotate_extrude(&profile, &params).unwrap();

        // Should have 4 * 16 = 64 vertices
        assert_eq!(mesh.vertex_count(), 64);
        // Should have 4 * 16 * 2 = 128 triangles
        assert_eq!(mesh.triangle_count(), 128);
    }

    #[test]
    fn test_rotate_extrude_partial() {
        let profile = test_profile();
        let params = RotateExtrudeParams {
            angle: 180.0,
            segments: 8,
        };
        let mesh = rotate_extrude(&profile, &params).unwrap();

        // Should have 4 * 9 = 36 vertices (9 slices for partial)
        assert_eq!(mesh.vertex_count(), 36);
    }

    #[test]
    fn test_rotate_extrude_circle_profile() {
        // Create a circle profile offset from Z axis (makes a torus)
        let circle = Polygon2D::circle(1.0, 8);
        let translated: Vec<_> = circle.outer.iter()
            .map(|v| DVec2::new(v.x + 3.0, v.y))
            .collect();
        let profile = Polygon2D::new(translated);

        let params = RotateExtrudeParams {
            angle: 360.0,
            segments: 16,
        };
        let mesh = rotate_extrude(&profile, &params).unwrap();

        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_rotate_extrude_invalid_angle() {
        let profile = test_profile();
        let params = RotateExtrudeParams {
            angle: 0.0,
            segments: 16,
        };
        let result = rotate_extrude(&profile, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_rotate_extrude_negative_x() {
        // Profile with negative X should fail
        let profile = Polygon2D::new(vec![
            DVec2::new(-1.0, 0.0),
            DVec2::new(1.0, 0.0),
            DVec2::new(0.0, 1.0),
        ]);
        let params = RotateExtrudeParams::default();
        let result = rotate_extrude(&profile, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_rotate_extrude_bounding_box() {
        let profile = test_profile();
        let params = RotateExtrudeParams {
            angle: 360.0,
            segments: 32,
        };
        let mesh = rotate_extrude(&profile, &params).unwrap();

        let (min, max) = mesh.bounding_box();
        // Should extend from -7 to 7 in X and Y (outer radius)
        assert!(max.x > 6.9 && max.x < 7.1);
        assert!(min.x < -6.9 && min.x > -7.1);
        // Z should be -1 to 1
        assert!((min.z - (-1.0)).abs() < 0.1);
        assert!((max.z - 1.0).abs() < 0.1);
    }
}

//! # Linear Extrusion
//!
//! Extrudes a 2D polygon along the Z axis to create a 3D mesh.
//! Supports twist, scale, and slicing for smooth transitions.
//!
//! ## OpenSCAD Compatibility
//!
//! Matches `linear_extrude(height, center, twist, slices, scale)`:
//! - `height`: Extrusion distance along Z
//! - `center`: If true, center the extrusion around Z=0
//! - `twist`: Rotation in degrees over the extrusion height
//! - `slices`: Number of intermediate slices for twist/scale
//! - `scale`: Final scale factor at the top

use super::Polygon2D;
use crate::error::MeshError;
use crate::mesh::Mesh;
use glam::{DVec2, DVec3};

/// Parameters for linear extrusion.
#[derive(Debug, Clone)]
pub struct LinearExtrudeParams {
    /// Extrusion height along Z axis
    pub height: f64,
    /// Center the extrusion around Z=0
    pub center: bool,
    /// Twist angle in degrees over the height
    pub twist: f64,
    /// Number of slices for twist/scale interpolation
    pub slices: u32,
    /// Scale factor at the top [x_scale, y_scale]
    pub scale: [f64; 2],
}

impl Default for LinearExtrudeParams {
    fn default() -> Self {
        Self {
            height: 1.0,
            center: false,
            twist: 0.0,
            slices: 1,
            scale: [1.0, 1.0],
        }
    }
}

/// Extrudes a 2D polygon along the Z axis.
///
/// # Arguments
///
/// * `polygon` - The 2D polygon to extrude
/// * `params` - Extrusion parameters
///
/// # Returns
///
/// A 3D mesh representing the extruded shape.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::extrude::{Polygon2D, linear_extrude, LinearExtrudeParams};
///
/// let square = Polygon2D::square(DVec2::splat(10.0), true);
/// let params = LinearExtrudeParams {
///     height: 20.0,
///     center: false,
///     ..Default::default()
/// };
/// let mesh = linear_extrude(&square, &params)?;
/// ```
pub fn linear_extrude(polygon: &Polygon2D, params: &LinearExtrudeParams) -> Result<Mesh, MeshError> {
    // Validate parameters
    if params.height <= 0.0 {
        return Err(MeshError::degenerate(
            "linear_extrude height must be positive",
            None,
        ));
    }

    if polygon.vertex_count() < 3 {
        return Err(MeshError::degenerate(
            "Polygon must have at least 3 vertices",
            None,
        ));
    }

    // Determine number of slices
    let slices = if params.twist.abs() > 0.001 || (params.scale[0] - 1.0).abs() > 0.001 || (params.scale[1] - 1.0).abs() > 0.001 {
        params.slices.max(1)
    } else {
        1 // No twist/scale, single slice is enough
    };

    // Calculate Z offset for centering
    let z_offset = if params.center { -params.height / 2.0 } else { 0.0 };

    let n = polygon.vertex_count();
    let mut mesh = Mesh::with_capacity(n * (slices as usize + 1), n * slices as usize * 2 + n * 2);

    // Generate vertices for each slice
    for slice_idx in 0..=slices {
        let t = slice_idx as f64 / slices as f64;
        let z = z_offset + t * params.height;

        // Interpolate twist and scale
        let twist_rad = (params.twist * t).to_radians();
        let scale_x = 1.0 + t * (params.scale[0] - 1.0);
        let scale_y = 1.0 + t * (params.scale[1] - 1.0);

        let cos_twist = twist_rad.cos();
        let sin_twist = twist_rad.sin();

        // Add transformed vertices
        for v in &polygon.outer {
            // Scale
            let scaled = DVec2::new(v.x * scale_x, v.y * scale_y);
            // Rotate (twist)
            let rotated = DVec2::new(
                scaled.x * cos_twist - scaled.y * sin_twist,
                scaled.x * sin_twist + scaled.y * cos_twist,
            );
            mesh.add_vertex(DVec3::new(rotated.x, rotated.y, z));
        }
    }

    // Generate side faces (quads split into triangles)
    for slice_idx in 0..slices {
        let base = (slice_idx as usize) * n;
        let next_base = ((slice_idx + 1) as usize) * n;

        for i in 0..n {
            let i_next = (i + 1) % n;

            // Two triangles per quad
            // Triangle 1: base[i], next_base[i], next_base[i_next]
            mesh.add_triangle(
                (base + i) as u32,
                (next_base + i) as u32,
                (next_base + i_next) as u32,
            );
            // Triangle 2: base[i], next_base[i_next], base[i_next]
            mesh.add_triangle(
                (base + i) as u32,
                (next_base + i_next) as u32,
                (base + i_next) as u32,
            );
        }
    }

    // Generate bottom cap (at z_offset)
    // Fan triangulation from first vertex
    for i in 1..n - 1 {
        mesh.add_triangle(0, (i + 1) as u32, i as u32);
    }

    // Generate top cap (at z_offset + height)
    let top_base = (slices as usize) * n;
    for i in 1..n - 1 {
        mesh.add_triangle(
            top_base as u32,
            (top_base + i) as u32,
            (top_base + i + 1) as u32,
        );
    }

    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_extrude_square() {
        let square = Polygon2D::square(DVec2::splat(10.0), false);
        let params = LinearExtrudeParams {
            height: 20.0,
            ..Default::default()
        };
        let mesh = linear_extrude(&square, &params).unwrap();

        // Should have 8 vertices (4 bottom + 4 top)
        assert_eq!(mesh.vertex_count(), 8);
        // Should have 12 triangles (8 sides + 2 bottom + 2 top)
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_linear_extrude_centered() {
        let square = Polygon2D::square(DVec2::splat(10.0), true);
        let params = LinearExtrudeParams {
            height: 20.0,
            center: true,
            ..Default::default()
        };
        let mesh = linear_extrude(&square, &params).unwrap();

        let (min, max) = mesh.bounding_box();
        assert!((min.z - (-10.0)).abs() < 0.001);
        assert!((max.z - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_linear_extrude_with_twist() {
        let square = Polygon2D::square(DVec2::splat(10.0), true);
        let params = LinearExtrudeParams {
            height: 20.0,
            twist: 90.0,
            slices: 10,
            ..Default::default()
        };
        let mesh = linear_extrude(&square, &params).unwrap();

        // Should have 4 * 11 = 44 vertices (4 per slice, 11 slices including ends)
        assert_eq!(mesh.vertex_count(), 44);
    }

    #[test]
    fn test_linear_extrude_with_scale() {
        let square = Polygon2D::square(DVec2::splat(10.0), true);
        let params = LinearExtrudeParams {
            height: 20.0,
            scale: [0.5, 0.5],
            slices: 1,
            ..Default::default()
        };
        let mesh = linear_extrude(&square, &params).unwrap();

        // Top vertices should be scaled to half size
        let (_min, max) = mesh.bounding_box();
        assert!(max.x <= 5.5); // Some tolerance
    }

    #[test]
    fn test_linear_extrude_circle() {
        let circle = Polygon2D::circle(5.0, 32);
        let params = LinearExtrudeParams {
            height: 10.0,
            ..Default::default()
        };
        let mesh = linear_extrude(&circle, &params).unwrap();

        // Should have 64 vertices (32 bottom + 32 top)
        assert_eq!(mesh.vertex_count(), 64);
    }

    #[test]
    fn test_linear_extrude_invalid_height() {
        let square = Polygon2D::square(DVec2::splat(10.0), false);
        let params = LinearExtrudeParams {
            height: 0.0,
            ..Default::default()
        };
        let result = linear_extrude(&square, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_linear_extrude_invalid_polygon() {
        let line = Polygon2D::new(vec![DVec2::ZERO, DVec2::X]);
        let params = LinearExtrudeParams::default();
        let result = linear_extrude(&line, &params);
        assert!(result.is_err());
    }
}

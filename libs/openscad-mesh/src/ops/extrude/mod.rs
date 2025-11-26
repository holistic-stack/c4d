//! # Extrusion Operations
//!
//! Browser-safe algorithms for 2D to 3D extrusion:
//! - **linear_extrude**: Extrude 2D shape along Z axis with optional twist/scale
//! - **rotate_extrude**: Revolve 2D shape around Z axis
//!
//! ## OpenSCAD Compatibility
//!
//! These implementations match OpenSCAD's behavior:
//! - `linear_extrude(height, center, twist, slices, scale)`
//! - `rotate_extrude(angle, convexity)`

mod linear;
mod rotate;

#[cfg(test)]
mod tests;

pub use linear::{linear_extrude, LinearExtrudeParams};
pub use rotate::{rotate_extrude, RotateExtrudeParams};

use glam::DVec2;

/// A 2D polygon for extrusion operations.
///
/// Represents a closed 2D shape that can be extruded into 3D.
#[derive(Debug, Clone)]
pub struct Polygon2D {
    /// Outer boundary vertices in counter-clockwise order
    pub outer: Vec<DVec2>,
    /// Optional holes (each in clockwise order)
    pub holes: Vec<Vec<DVec2>>,
}

impl Polygon2D {
    /// Creates a new polygon from outer boundary vertices.
    ///
    /// # Arguments
    ///
    /// * `outer` - Vertices in counter-clockwise order
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let square = Polygon2D::new(vec![
    ///     DVec2::new(0.0, 0.0),
    ///     DVec2::new(1.0, 0.0),
    ///     DVec2::new(1.0, 1.0),
    ///     DVec2::new(0.0, 1.0),
    /// ]);
    /// ```
    pub fn new(outer: Vec<DVec2>) -> Self {
        Self {
            outer,
            holes: Vec::new(),
        }
    }

    /// Creates a polygon with holes.
    ///
    /// # Arguments
    ///
    /// * `outer` - Outer boundary in counter-clockwise order
    /// * `holes` - Inner holes, each in clockwise order
    pub fn with_holes(outer: Vec<DVec2>, holes: Vec<Vec<DVec2>>) -> Self {
        Self { outer, holes }
    }

    /// Creates a square polygon.
    ///
    /// # Arguments
    ///
    /// * `size` - Width and height
    /// * `center` - If true, center at origin
    pub fn square(size: DVec2, center: bool) -> Self {
        let (x, y) = if center {
            (-size.x / 2.0, -size.y / 2.0)
        } else {
            (0.0, 0.0)
        };

        Self::new(vec![
            DVec2::new(x, y),
            DVec2::new(x + size.x, y),
            DVec2::new(x + size.x, y + size.y),
            DVec2::new(x, y + size.y),
        ])
    }

    /// Creates a circle polygon.
    ///
    /// # Arguments
    ///
    /// * `radius` - Circle radius
    /// * `segments` - Number of segments
    pub fn circle(radius: f64, segments: u32) -> Self {
        let mut vertices = Vec::with_capacity(segments as usize);
        for i in 0..segments {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (segments as f64);
            vertices.push(DVec2::new(
                radius * angle.cos(),
                radius * angle.sin(),
            ));
        }
        Self::new(vertices)
    }

    /// Returns the number of vertices in the outer boundary.
    pub fn vertex_count(&self) -> usize {
        self.outer.len()
    }

    /// Returns true if the polygon has holes.
    pub fn has_holes(&self) -> bool {
        !self.holes.is_empty()
    }

    /// Translates the polygon by the given offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - Translation vector in 2D
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut circle = Polygon2D::circle(5.0, 32);
    /// circle.translate(DVec2::new(10.0, 0.0)); // Move circle right by 10
    /// ```
    pub fn translate(&mut self, offset: DVec2) {
        for vertex in &mut self.outer {
            *vertex += offset;
        }
        for hole in &mut self.holes {
            for vertex in hole {
                *vertex += offset;
            }
        }
    }
}

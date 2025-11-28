//! # CrossSection Module
//!
//! 2D polygon operations for extrusions and 2D primitives.
//!
//! ## Structure
//!
//! - `primitives`: Circle, Square, Polygon mesh builders
//! - `extrude`: Linear and rotate extrusions
//! - `ops`: Offset, Projection operations
//!
//! ## OpenSCAD Compatibility
//!
//! All 2D operations match OpenSCAD's output:
//! - Circle uses $fn/$fa/$fs for segment calculation
//! - Polygon supports paths for holes
//! - Linear extrude supports twist, scale, slices

pub mod primitives;
pub mod extrude;
pub mod ops;

// =============================================================================
// CROSSSECTION STRUCT
// =============================================================================

/// 2D polygon for extrusion operations.
///
/// Represents a closed 2D polygon that can be extruded to 3D.
///
/// ## Example
///
/// ```rust
/// use manifold_rs::CrossSection;
///
/// let circle = CrossSection::circle(5.0, 16);
/// let square = CrossSection::square([10.0, 10.0], true);
/// ```
#[derive(Debug, Clone, Default)]
pub struct CrossSection {
    /// Polygon vertices (x, y pairs)
    pub vertices: Vec<[f64; 2]>,
}

impl CrossSection {
    /// Create empty cross section.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create circle with given radius and segments.
    ///
    /// ## Parameters
    ///
    /// - `radius`: Circle radius
    /// - `segments`: Number of segments around circumference
    #[must_use]
    pub fn circle(radius: f64, segments: u32) -> Self {
        let n = segments.max(3) as usize;
        let mut vertices = Vec::with_capacity(n);
        
        for i in 0..n {
            let theta = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            vertices.push([
                radius * theta.cos(),
                radius * theta.sin(),
            ]);
        }
        
        Self { vertices }
    }

    /// Create square with given size.
    ///
    /// ## Parameters
    ///
    /// - `size`: [width, height]
    /// - `center`: If true, center at origin
    #[must_use]
    pub fn square(size: [f64; 2], center: bool) -> Self {
        let [w, h] = size;
        let (x0, x1) = if center { (-w / 2.0, w / 2.0) } else { (0.0, w) };
        let (y0, y1) = if center { (-h / 2.0, h / 2.0) } else { (0.0, h) };
        
        Self {
            vertices: vec![
                [x0, y0],
                [x1, y0],
                [x1, y1],
                [x0, y1],
            ],
        }
    }

    /// Create cross section from polygon vertices.
    ///
    /// ## Parameters
    ///
    /// - `vertices`: Polygon vertex coordinates
    #[must_use]
    pub fn from_vertices(vertices: Vec<[f64; 2]>) -> Self {
        Self { vertices }
    }

    /// Check if cross section is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices.len() < 3
    }

    /// Get vertex count.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test circle creation.
    #[test]
    fn test_circle() {
        let circle = CrossSection::circle(5.0, 16);
        assert_eq!(circle.vertex_count(), 16);
    }

    /// Test square creation.
    #[test]
    fn test_square() {
        let square = CrossSection::square([10.0, 10.0], false);
        assert_eq!(square.vertex_count(), 4);
    }

    /// Test centered square.
    #[test]
    fn test_square_centered() {
        let square = CrossSection::square([10.0, 10.0], true);
        assert_eq!(square.vertex_count(), 4);
        
        // Should have vertices on both sides of origin
        let has_negative = square.vertices.iter().any(|v| v[0] < 0.0);
        let has_positive = square.vertices.iter().any(|v| v[0] > 0.0);
        assert!(has_negative && has_positive);
    }
}

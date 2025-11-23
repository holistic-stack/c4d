//! 2D geometry representation supporting primitives (square, circle, polygon).
//!
//! A `CrossSection` consists of one or more contours (polygons), where each contour is
//! a list of 2D points (`Vec2`). It supports conversion to 3D polygons (via triangulation)
//! and enforcement of winding orders (CCW for outer, CW for holes).

use crate::core::vec2::Vec2;
use crate::error::Result;

/// A 2D cross-section composed of multiple contours (polygons).
///
/// Each contour is a closed loop of points.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossSection {
    /// List of contours. Each contour is a vector of points.
    pub contours: Vec<Vec<Vec2>>,
}

impl CrossSection {
    /// Creates a new empty `CrossSection`.
    pub fn new() -> Self {
        Self { contours: Vec::new() }
    }

    /// Creates a `CrossSection` from a list of contours.
    pub fn from_contours(contours: Vec<Vec<Vec2>>) -> Self {
        Self { contours }
    }

    /// Creates a square (rectangle) cross-section.
    ///
    /// # Arguments
    /// * `size` - The dimensions (width, height).
    /// * `center` - Whether the square is centered at the origin.
    pub fn square(size: Vec2, center: bool) -> Self {
        let (x, y) = (size.x, size.y);
        let (min_x, min_y) = if center {
            (-x / 2.0, -y / 2.0)
        } else {
            (0.0, 0.0)
        };
        let max_x = min_x + x;
        let max_y = min_y + y;

        // Counter-clockwise winding
        let contour = vec![
            Vec2::new(min_x, min_y),
            Vec2::new(max_x, min_y),
            Vec2::new(max_x, max_y),
            Vec2::new(min_x, max_y),
        ];

        Self {
            contours: vec![contour],
        }
    }

    /// Creates a circle cross-section (approximated as a regular polygon).
    ///
    /// # Arguments
    /// * `radius` - The radius of the circle.
    /// * `fragments` - The number of segments to approximate the circle.
    pub fn circle(radius: f64, fragments: u32) -> Self {
        let fragments = fragments.max(3);
        let mut contour = Vec::with_capacity(fragments as usize);
        let step = std::f64::consts::TAU / fragments as f64;

        for i in 0..fragments {
            let theta = i as f64 * step;
            let (sin, cos) = theta.sin_cos();
            contour.push(Vec2::new(cos * radius, sin * radius));
        }

        Self {
            contours: vec![contour],
        }
    }

    /// Creates a polygon cross-section from points and optional paths.
    ///
    /// If `paths` is provided, it defines the connectivity (indices into `points`).
    /// If `paths` is empty/None, `points` is treated as a single contour.
    ///
    /// # Arguments
    /// * `points` - The vertices of the polygon.
    /// * `paths` - Optional list of paths (indices).
    pub fn polygon(points: &[Vec2], paths: Option<&[Vec<usize>]>) -> Result<Self> {
        let contours = if let Some(paths) = paths {
            let mut contours = Vec::with_capacity(paths.len());
            for path in paths {
                let mut contour = Vec::with_capacity(path.len());
                for &idx in path {
                    if idx >= points.len() {
                        return Err(crate::error::Error::InvalidTopology(format!(
                            "Polygon index {} out of bounds (len {})",
                            idx,
                            points.len()
                        )));
                    }
                    contour.push(points[idx]);
                }
                contours.push(contour);
            }
            contours
        } else {
            vec![points.to_vec()]
        };

        Ok(Self { contours })
    }

    // TODO: Implement triangulation (to_polygons) using earcutr for 3D conversion.
    // TODO: Implement winding order enforcement.
}

#[cfg(test)]
mod tests;

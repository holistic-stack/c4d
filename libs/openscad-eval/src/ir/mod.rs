//! Intermediate representation nodes produced by the evaluator.
//!
//! The IR stays intentionally small for Task 1.1. Future phases will expand the
//! enum with booleans and transforms.

use config::constants::EPSILON_TOLERANCE;
use glam::{DMat4, DVec3, DVec2};
use thiserror::Error;

use openscad_ast::Span;

/// Geometry node describing solid primitives.
///
/// # Examples
/// ```
/// use glam::DVec3;
/// use openscad_eval::ir::GeometryNode;
/// use openscad_ast::Span;
/// let span = Span::new(0, 10).unwrap();
/// let node = GeometryNode::cube(DVec3::splat(1.0), false, span).unwrap();
/// assert_eq!(node.size(), DVec3::new(1.0, 1.0, 1.0));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryNode {
    /// Axis-aligned cube primitive defined by its XYZ dimensions.
    Cube { size: DVec3, center: bool, span: Span },
    /// Sphere primitive defined by its radius and resolution settings.
    Sphere { radius: f64, segments: u32, span: Span },
    /// Cylinder primitive defined by its height, radii and resolution settings.
    Cylinder {
        height: f64,
        radius_bottom: f64,
        radius_top: f64,
        center: bool,
        segments: u32,
        span: Span,
    },
    /// Axis-aligned square primitive defined by its XY dimensions.
    Square { size: DVec2, center: bool, span: Span },
    /// Circle primitive defined by its radius and resolution settings.
    Circle { radius: f64, segments: u32, span: Span },
    /// Polygon primitive defined by its points, paths and convexity.
    Polygon {
        points: Vec<DVec2>,
        paths: Vec<Vec<usize>>,
        convexity: u32,
        span: Span,
    },
    /// Transformation applied to a child geometry.
    Transform {
        matrix: DMat4,
        child: Box<GeometryNode>,
        span: Span,
    },
}

impl GeometryNode {
    /// Constructs a cube node while validating all sides exceed the tolerance.
    pub fn cube(size: DVec3, center: bool, span: Span) -> Result<Self, GeometryValidationError> {
        if size.min_element() <= EPSILON_TOLERANCE {
            return Err(GeometryValidationError::CubeSizeTooSmall { size });
        }
        Ok(Self::Cube { size, center, span })
    }

    /// Constructs a sphere node.
    pub fn sphere(radius: f64, segments: u32, span: Span) -> Result<Self, GeometryValidationError> {
        if radius <= EPSILON_TOLERANCE {
            return Err(GeometryValidationError::SphereRadiusTooSmall { radius });
        }
        Ok(Self::Sphere { radius, segments, span })
    }

    /// Constructs a cylinder node.
    pub fn cylinder(
        height: f64,
        radius_bottom: f64,
        radius_top: f64,
        center: bool,
        segments: u32,
        span: Span,
    ) -> Result<Self, GeometryValidationError> {
        if height <= EPSILON_TOLERANCE {
            return Err(GeometryValidationError::CylinderHeightTooSmall { height });
        }
        // At least one radius must be positive
        if radius_bottom <= 0.0 && radius_top <= 0.0 {
             return Err(GeometryValidationError::CylinderRadiiTooSmall);
        }
        Ok(Self::Cylinder {
            height,
            radius_bottom,
            radius_top,
            center,
            segments,
            span,
        })
    }

    /// Constructs a square node.
    pub fn square(size: DVec2, center: bool, span: Span) -> Result<Self, GeometryValidationError> {
        if size.min_element() <= EPSILON_TOLERANCE {
            return Err(GeometryValidationError::SquareSizeTooSmall { size });
        }
        Ok(Self::Square { size, center, span })
    }

    /// Constructs a circle node.
    pub fn circle(radius: f64, segments: u32, span: Span) -> Result<Self, GeometryValidationError> {
        if radius <= EPSILON_TOLERANCE {
            return Err(GeometryValidationError::CircleRadiusTooSmall { radius });
        }
        Ok(Self::Circle { radius, segments, span })
    }

    /// Constructs a polygon node.
    pub fn polygon(
        points: Vec<DVec2>,
        paths: Vec<Vec<usize>>,
        convexity: u32,
        span: Span,
    ) -> Result<Self, GeometryValidationError> {
        if points.len() < 3 {
            return Err(GeometryValidationError::PolygonTooFewPoints { count: points.len() });
        }
        Ok(Self::Polygon {
            points,
            paths,
            convexity,
            span,
        })
    }

    /// Returns the primary size vector for the node.
    pub fn size(&self) -> DVec3 {
        match self {
            GeometryNode::Cube { size, .. } => *size,
            GeometryNode::Sphere { radius, .. } => DVec3::splat(*radius * 2.0),
            GeometryNode::Cylinder { height, radius_bottom, radius_top, .. } => {
                let r = radius_bottom.max(*radius_top);
                DVec3::new(r * 2.0, r * 2.0, *height)
            },
            GeometryNode::Square { size, .. } => DVec3::new(size.x, size.y, 0.0),
            GeometryNode::Circle { radius, .. } => DVec3::new(radius * 2.0, radius * 2.0, 0.0),
            GeometryNode::Polygon { .. } => DVec3::ZERO, // Bounding box calculation required for Polygon
            // Transform doesn't have a simple size, return child size for now or TODO
            GeometryNode::Transform { child, .. } => child.size(),
        }
    }
}

/// Errors raised when IR construction fails validation.
#[derive(Debug, Error, PartialEq)]
pub enum GeometryValidationError {
    /// Cube size must exceed the configured epsilon tolerance.
    #[error("cube dimensions must exceed tolerance: {size:?}")]
    CubeSizeTooSmall { size: DVec3 },
    /// Sphere radius must exceed the configured epsilon tolerance.
    #[error("sphere radius must exceed tolerance: {radius}")]
    SphereRadiusTooSmall { radius: f64 },
    /// Cylinder height must exceed tolerance.
    #[error("cylinder height must exceed tolerance: {height}")]
    CylinderHeightTooSmall { height: f64 },
    /// Cylinder radii must be non-negative and at least one positive.
    #[error("cylinder radii must be non-negative and at least one positive")]
    CylinderRadiiTooSmall,
    /// Square size must exceed tolerance.
    #[error("square dimensions must exceed tolerance: {size:?}")]
    SquareSizeTooSmall { size: DVec2 },
    /// Circle radius must exceed tolerance.
    #[error("circle radius must exceed tolerance: {radius}")]
    CircleRadiusTooSmall { radius: f64 },
    /// Polygon must have at least 3 points.
    #[error("polygon must have at least 3 points, got {count}")]
    PolygonTooFewPoints { count: usize },
}

#[cfg(test)]
mod tests;

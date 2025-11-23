//! Intermediate representation nodes produced by the evaluator.
//!
//! The IR stays intentionally small for Task 1.1. Future phases will expand the
//! enum with booleans and transforms.

use config::constants::EPSILON_TOLERANCE;
use glam::{DMat4, DVec3, DVec2, DVec4};
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
    /// Linear extrusion of a 2D child geometry.
    LinearExtrude {
        height: f64,
        twist: f64,
        slices: u32,
        center: bool,
        scale: DVec2,
        convexity: u32,
        child: Box<GeometryNode>,
        span: Span,
    },
    /// Rotational extrusion of a 2D child geometry.
    RotateExtrude {
        angle: f64,
        convexity: u32,
        segments: u32,
        child: Box<GeometryNode>,
        span: Span,
    },
    /// Transformation applied to a child geometry.
    Transform {
        matrix: DMat4,
        child: Box<GeometryNode>,
        span: Span,
    },
    /// Resize operation to fit geometry into target dimensions.
    Resize {
        new_size: DVec3,
        auto: Vec<bool>, // x, y, z auto flags
        child: Box<GeometryNode>,
        span: Span,
    },
    /// Color operation to apply RGBA color to geometry.
    Color {
        color: DVec4,
        child: Box<GeometryNode>,
        span: Span,
    },
    /// Union of multiple children.
    Union {
        children: Vec<GeometryNode>,
        span: Span,
    },
    /// Difference of children (first - others).
    Difference {
        children: Vec<GeometryNode>,
        span: Span,
    },
    /// Intersection of multiple children.
    Intersection {
        children: Vec<GeometryNode>,
        span: Span,
    },
    /// Convex hull of multiple children.
    Hull {
        children: Vec<GeometryNode>,
        span: Span,
    },
    /// Minkowski sum of multiple children.
    Minkowski {
        children: Vec<GeometryNode>,
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
            GeometryNode::Polygon { .. } => DVec3::ZERO,
            GeometryNode::LinearExtrude { height, .. } => DVec3::new(0.0, 0.0, *height),
            GeometryNode::RotateExtrude { .. } => DVec3::ZERO,
            GeometryNode::Transform { child, .. } => child.size(),
            GeometryNode::Resize { new_size, .. } => *new_size,
            GeometryNode::Color { child, .. } => child.size(),
            GeometryNode::Union { children, .. } => children.first().map(|c| c.size()).unwrap_or(DVec3::ZERO),
            GeometryNode::Difference { children, .. } => children.first().map(|c| c.size()).unwrap_or(DVec3::ZERO),
            GeometryNode::Intersection { children, .. } => children.first().map(|c| c.size()).unwrap_or(DVec3::ZERO),
            GeometryNode::Hull { .. } => DVec3::ZERO,
            GeometryNode::Minkowski { .. } => DVec3::ZERO,
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

//! Intermediate representation nodes produced by the evaluator.
//!
//! The IR stays intentionally small for Task 1.1. Future phases will expand the
//! enum with booleans and transforms.

use config::constants::EPSILON_TOLERANCE;
use glam::{DMat4, DVec3};
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

    /// Returns the primary size vector for the node.
    pub fn size(&self) -> DVec3 {
        match self {
            GeometryNode::Cube { size, .. } => *size,
            GeometryNode::Sphere { radius, .. } => DVec3::splat(*radius * 2.0),
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
}

#[cfg(test)]
mod tests;

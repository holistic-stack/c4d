//! # Plane for BSP Operations
//!
//! Plane representation with point classification.

use super::vertex::Vertex;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Epsilon for floating point comparisons.
const EPSILON: f64 = 1e-5;

// =============================================================================
// CLASSIFICATION
// =============================================================================

/// Classification of a point relative to a plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    /// Point is in front of plane (positive side).
    Front,
    /// Point is behind plane (negative side).
    Back,
    /// Point is on the plane.
    Coplanar,
    /// Polygon spans the plane (has vertices on both sides).
    Spanning,
}

// =============================================================================
// PLANE
// =============================================================================

/// A plane in 3D space defined by normal and distance from origin.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Normal vector (unit length).
    normal: Vertex,
    /// Distance from origin along normal.
    w: f64,
}

impl Plane {
    /// Create plane from normal and distance.
    pub fn new(normal: Vertex, w: f64) -> Self {
        Self { normal, w }
    }

    /// Create plane from three points.
    ///
    /// Points should be in counter-clockwise order when viewed from front.
    pub fn from_points(a: &Vertex, b: &Vertex, c: &Vertex) -> Option<Self> {
        let ab = b.sub(a);
        let ac = c.sub(a);
        let normal = ab.cross(&ac).normalize();

        // Check for degenerate triangle
        if normal.length() < EPSILON {
            return None;
        }

        let w = normal.dot(a);
        Some(Self { normal, w })
    }

    /// Get the plane normal.
    pub fn normal(&self) -> Vertex {
        self.normal
    }

    /// Get the plane distance.
    pub fn w(&self) -> f64 {
        self.w
    }

    /// Flip the plane (reverse normal).
    pub fn flip(&self) -> Plane {
        Plane {
            normal: self.normal.negate(),
            w: -self.w,
        }
    }

    /// Classify a point relative to this plane.
    pub fn classify_point(&self, point: &Vertex) -> Classification {
        let dist = self.signed_distance(point);
        if dist > EPSILON {
            Classification::Front
        } else if dist < -EPSILON {
            Classification::Back
        } else {
            Classification::Coplanar
        }
    }

    /// Signed distance from point to plane.
    ///
    /// Positive = front, negative = back, zero = on plane.
    pub fn signed_distance(&self, point: &Vertex) -> f64 {
        self.normal.dot(point) - self.w
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_from_points() {
        let a = Vertex::new(0.0, 0.0, 0.0);
        let b = Vertex::new(1.0, 0.0, 0.0);
        let c = Vertex::new(0.0, 1.0, 0.0);

        let plane = Plane::from_points(&a, &b, &c).unwrap();

        // Normal should point in +Z direction
        assert!((plane.normal.z - 1.0).abs() < EPSILON);
        assert!(plane.normal.x.abs() < EPSILON);
        assert!(plane.normal.y.abs() < EPSILON);
    }

    #[test]
    fn test_plane_classify_point() {
        let plane = Plane::new(Vertex::new(0.0, 0.0, 1.0), 0.0);

        let front = Vertex::new(0.0, 0.0, 1.0);
        let back = Vertex::new(0.0, 0.0, -1.0);
        let on = Vertex::new(1.0, 1.0, 0.0);

        assert_eq!(plane.classify_point(&front), Classification::Front);
        assert_eq!(plane.classify_point(&back), Classification::Back);
        assert_eq!(plane.classify_point(&on), Classification::Coplanar);
    }

    #[test]
    fn test_plane_flip() {
        let plane = Plane::new(Vertex::new(0.0, 0.0, 1.0), 5.0);
        let flipped = plane.flip();

        assert!((flipped.normal.z + 1.0).abs() < EPSILON);
        assert!((flipped.w + 5.0).abs() < EPSILON);
    }
}

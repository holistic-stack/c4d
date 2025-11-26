//! # Plane
//!
//! Plane representation for BSP tree partitioning.
//! Uses robust geometric predicates for numerical stability.

use config::constants::EPSILON;
use glam::DVec3;

/// Classification of a point relative to a plane.
///
/// Used to determine which side of a plane a vertex lies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointClassification {
    /// Point is in front of the plane (positive side)
    Front,
    /// Point is behind the plane (negative side)
    Back,
    /// Point lies on the plane (within EPSILON tolerance)
    Coplanar,
}

/// Classification of a polygon relative to a plane.
///
/// Used to determine how to partition polygons in BSP construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonClassification {
    /// All vertices are in front of the plane
    Front,
    /// All vertices are behind the plane
    Back,
    /// All vertices lie on the plane
    Coplanar,
    /// Polygon spans the plane (some vertices front, some back)
    Spanning,
}

/// A plane in 3D space defined by normal and distance from origin.
///
/// The plane equation is: normal · point = distance
///
/// # Example
///
/// ```rust,ignore
/// use glam::DVec3;
/// use openscad_mesh::ops::boolean::plane::Plane;
///
/// // XY plane at z=0
/// let plane = Plane::new(DVec3::Z, 0.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Unit normal vector pointing to the "front" side
    pub normal: DVec3,
    /// Signed distance from origin along normal
    pub distance: f64,
}

impl Plane {
    /// Creates a new plane from normal and distance.
    ///
    /// # Arguments
    ///
    /// * `normal` - Normal vector (will be normalized)
    /// * `distance` - Signed distance from origin
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let plane = Plane::new(DVec3::Z, 5.0);
    /// ```
    pub fn new(normal: DVec3, distance: f64) -> Self {
        let n = normal.normalize();
        Self {
            normal: n,
            distance,
        }
    }

    /// Creates a plane from three points (counter-clockwise winding).
    ///
    /// The normal points towards the viewer when vertices appear counter-clockwise.
    ///
    /// # Arguments
    ///
    /// * `a`, `b`, `c` - Three non-collinear points on the plane
    ///
    /// # Returns
    ///
    /// Some(Plane) if points are non-collinear, None otherwise.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let plane = Plane::from_points(
    ///     DVec3::ZERO,
    ///     DVec3::X,
    ///     DVec3::Y,
    /// ).unwrap();
    /// assert_eq!(plane.normal, DVec3::Z);
    /// ```
    pub fn from_points(a: DVec3, b: DVec3, c: DVec3) -> Option<Self> {
        let ab = b - a;
        let ac = c - a;
        let normal = ab.cross(ac);

        let len_sq = normal.length_squared();
        if len_sq < EPSILON * EPSILON {
            return None; // Degenerate (collinear points)
        }

        let normal = normal / len_sq.sqrt();
        let distance = normal.dot(a);

        Some(Self { normal, distance })
    }

    /// Classifies a point relative to this plane.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to classify
    ///
    /// # Returns
    ///
    /// The classification (Front, Back, or Coplanar).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let plane = Plane::new(DVec3::Z, 0.0);
    /// assert_eq!(plane.classify_point(DVec3::new(0.0, 0.0, 1.0)), PointClassification::Front);
    /// assert_eq!(plane.classify_point(DVec3::new(0.0, 0.0, -1.0)), PointClassification::Back);
    /// ```
    #[inline]
    pub fn classify_point(&self, point: DVec3) -> PointClassification {
        let dist = self.signed_distance(point);
        if dist > EPSILON {
            PointClassification::Front
        } else if dist < -EPSILON {
            PointClassification::Back
        } else {
            PointClassification::Coplanar
        }
    }

    /// Computes the signed distance from a point to this plane.
    ///
    /// Positive = front, Negative = back, Zero = on plane.
    ///
    /// # Arguments
    ///
    /// * `point` - The point to measure from
    ///
    /// # Returns
    ///
    /// The signed distance.
    #[inline]
    pub fn signed_distance(&self, point: DVec3) -> f64 {
        self.normal.dot(point) - self.distance
    }

    /// Flips the plane to face the opposite direction.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut plane = Plane::new(DVec3::Z, 5.0);
    /// plane.flip();
    /// assert_eq!(plane.normal, -DVec3::Z);
    /// assert_eq!(plane.distance, -5.0);
    /// ```
    pub fn flip(&mut self) {
        self.normal = -self.normal;
        self.distance = -self.distance;
    }

    /// Returns a flipped copy of this plane.
    pub fn flipped(&self) -> Self {
        Self {
            normal: -self.normal,
            distance: -self.distance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::constants::EPSILON;

    #[test]
    fn test_plane_new() {
        let plane = Plane::new(DVec3::Z, 5.0);
        assert!((plane.normal - DVec3::Z).length() < EPSILON);
        assert!((plane.distance - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_plane_from_points() {
        let plane = Plane::from_points(
            DVec3::ZERO,
            DVec3::X,
            DVec3::Y,
        ).unwrap();
        
        // Normal should be +Z (CCW winding)
        assert!((plane.normal - DVec3::Z).length() < EPSILON);
        assert!(plane.distance.abs() < EPSILON);
    }

    #[test]
    fn test_plane_from_points_degenerate() {
        // Collinear points
        let result = Plane::from_points(
            DVec3::ZERO,
            DVec3::X,
            DVec3::new(2.0, 0.0, 0.0),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_classify_point_front() {
        let plane = Plane::new(DVec3::Z, 0.0);
        assert_eq!(
            plane.classify_point(DVec3::new(0.0, 0.0, 1.0)),
            PointClassification::Front
        );
    }

    #[test]
    fn test_classify_point_back() {
        let plane = Plane::new(DVec3::Z, 0.0);
        assert_eq!(
            plane.classify_point(DVec3::new(0.0, 0.0, -1.0)),
            PointClassification::Back
        );
    }

    #[test]
    fn test_classify_point_coplanar() {
        let plane = Plane::new(DVec3::Z, 0.0);
        assert_eq!(
            plane.classify_point(DVec3::new(5.0, 5.0, 0.0)),
            PointClassification::Coplanar
        );
    }

    #[test]
    fn test_signed_distance() {
        let plane = Plane::new(DVec3::Z, 5.0);
        
        // Point at z=10 is 5 units in front
        assert!((plane.signed_distance(DVec3::new(0.0, 0.0, 10.0)) - 5.0).abs() < EPSILON);
        
        // Point at z=0 is 5 units behind
        assert!((plane.signed_distance(DVec3::ZERO) - (-5.0)).abs() < EPSILON);
    }

    #[test]
    fn test_plane_flip() {
        let mut plane = Plane::new(DVec3::Z, 5.0);
        plane.flip();
        
        assert!((plane.normal - (-DVec3::Z)).length() < EPSILON);
        assert!((plane.distance - (-5.0)).abs() < EPSILON);
    }

    #[test]
    fn test_plane_flipped() {
        let plane = Plane::new(DVec3::Z, 5.0);
        let flipped = plane.flipped();
        
        assert!((flipped.normal - (-DVec3::Z)).length() < EPSILON);
        assert!((flipped.distance - (-5.0)).abs() < EPSILON);
    }
}

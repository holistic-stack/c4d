//! # Vertex
//!
//! Vertex representation for BSP boolean operations.
//! Each vertex has a position and can be interpolated for polygon splitting.

use glam::DVec3;

/// A vertex in 3D space for BSP operations.
///
/// Contains position data that can be linearly interpolated
/// when polygons are split by planes.
///
/// # Example
///
/// ```rust,ignore
/// use glam::DVec3;
/// use openscad_mesh::ops::boolean::vertex::Vertex;
///
/// let v = Vertex::new(DVec3::new(1.0, 2.0, 3.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vertex {
    /// Position in 3D space
    pub position: DVec3,
}

impl Vertex {
    /// Creates a new vertex at the given position.
    ///
    /// # Arguments
    ///
    /// * `position` - The 3D position of the vertex
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let v = Vertex::new(DVec3::new(0.0, 0.0, 0.0));
    /// ```
    #[inline]
    pub fn new(position: DVec3) -> Self {
        Self { position }
    }

    /// Linearly interpolates between this vertex and another.
    ///
    /// Used when splitting polygons along a plane.
    ///
    /// # Arguments
    ///
    /// * `other` - The other vertex to interpolate towards
    /// * `t` - Interpolation factor (0.0 = self, 1.0 = other)
    ///
    /// # Returns
    ///
    /// A new vertex at the interpolated position.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let v1 = Vertex::new(DVec3::ZERO);
    /// let v2 = Vertex::new(DVec3::X);
    /// let mid = v1.lerp(&v2, 0.5);
    /// assert_eq!(mid.position.x, 0.5);
    /// ```
    #[inline]
    pub fn lerp(&self, other: &Vertex, t: f64) -> Vertex {
        Vertex {
            position: self.position.lerp(other.position, t),
        }
    }
}

impl From<DVec3> for Vertex {
    fn from(position: DVec3) -> Self {
        Self::new(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_new() {
        let v = Vertex::new(DVec3::new(1.0, 2.0, 3.0));
        assert_eq!(v.position.x, 1.0);
        assert_eq!(v.position.y, 2.0);
        assert_eq!(v.position.z, 3.0);
    }

    #[test]
    fn test_vertex_lerp() {
        let v1 = Vertex::new(DVec3::ZERO);
        let v2 = Vertex::new(DVec3::new(10.0, 20.0, 30.0));
        
        let mid = v1.lerp(&v2, 0.5);
        assert_eq!(mid.position.x, 5.0);
        assert_eq!(mid.position.y, 10.0);
        assert_eq!(mid.position.z, 15.0);
    }

    #[test]
    fn test_vertex_lerp_endpoints() {
        let v1 = Vertex::new(DVec3::ZERO);
        let v2 = Vertex::new(DVec3::ONE);
        
        let at_start = v1.lerp(&v2, 0.0);
        assert_eq!(at_start.position, DVec3::ZERO);
        
        let at_end = v1.lerp(&v2, 1.0);
        assert_eq!(at_end.position, DVec3::ONE);
    }

    #[test]
    fn test_vertex_from_dvec3() {
        let v: Vertex = DVec3::new(1.0, 2.0, 3.0).into();
        assert_eq!(v.position, DVec3::new(1.0, 2.0, 3.0));
    }
}

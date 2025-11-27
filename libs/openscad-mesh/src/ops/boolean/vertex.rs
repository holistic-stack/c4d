//! # Vertex for BSP Operations
//!
//! 3D vertex with linear interpolation support.

// =============================================================================
// VERTEX
// =============================================================================

/// 3D vertex for BSP operations.
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Z coordinate.
    pub z: f64,
}

impl Vertex {
    /// Create new vertex.
    ///
    /// ## Parameters
    ///
    /// - `x`: X coordinate
    /// - `y`: Y coordinate
    /// - `z`: Z coordinate
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Linear interpolation between two vertices.
    ///
    /// ## Parameters
    ///
    /// - `other`: Target vertex
    /// - `t`: Interpolation factor (0.0 = self, 1.0 = other)
    ///
    /// ## Returns
    ///
    /// Interpolated vertex.
    pub fn lerp(&self, other: &Vertex, t: f64) -> Vertex {
        Vertex {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    /// Dot product with another vertex (as vector).
    pub fn dot(&self, other: &Vertex) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product with another vertex (as vector).
    pub fn cross(&self, other: &Vertex) -> Vertex {
        Vertex {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Subtract another vertex.
    pub fn sub(&self, other: &Vertex) -> Vertex {
        Vertex {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    /// Vector length.
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalize to unit length.
    pub fn normalize(&self) -> Vertex {
        let len = self.length();
        if len > 1e-10 {
            Vertex {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }

    /// Negate the vertex (flip direction).
    pub fn negate(&self) -> Vertex {
        Vertex {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_new() {
        let v = Vertex::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_vertex_lerp() {
        let a = Vertex::new(0.0, 0.0, 0.0);
        let b = Vertex::new(10.0, 10.0, 10.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.x - 5.0).abs() < 1e-10);
        assert!((mid.y - 5.0).abs() < 1e-10);
        assert!((mid.z - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_vertex_dot() {
        let a = Vertex::new(1.0, 0.0, 0.0);
        let b = Vertex::new(0.0, 1.0, 0.0);
        assert!((a.dot(&b)).abs() < 1e-10); // Perpendicular
    }

    #[test]
    fn test_vertex_cross() {
        let a = Vertex::new(1.0, 0.0, 0.0);
        let b = Vertex::new(0.0, 1.0, 0.0);
        let c = a.cross(&b);
        assert!((c.x).abs() < 1e-10);
        assert!((c.y).abs() < 1e-10);
        assert!((c.z - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vertex_normalize() {
        let v = Vertex::new(3.0, 4.0, 0.0);
        let n = v.normalize();
        assert!((n.length() - 1.0).abs() < 1e-10);
    }
}

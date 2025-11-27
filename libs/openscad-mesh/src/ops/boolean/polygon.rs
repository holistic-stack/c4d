//! # Polygon for BSP Operations
//!
//! Convex polygon with plane and splitting support.

use super::plane::{Classification, Plane};
use super::vertex::Vertex;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Epsilon for floating point comparisons.
const EPSILON: f64 = 1e-5;

// =============================================================================
// POLYGON
// =============================================================================

/// A convex polygon with associated plane.
#[derive(Debug, Clone)]
pub struct Polygon {
    /// Vertices in counter-clockwise order.
    vertices: Vec<Vertex>,
    /// Plane containing this polygon.
    plane: Plane,
}

impl Polygon {
    /// Create polygon from vertices.
    ///
    /// Returns None if vertices don't form a valid polygon.
    pub fn from_vertices(vertices: Vec<Vertex>) -> Option<Self> {
        if vertices.len() < 3 {
            return None;
        }

        let plane = Plane::from_points(&vertices[0], &vertices[1], &vertices[2])?;
        Some(Self { vertices, plane })
    }

    /// Get polygon vertices.
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// Get polygon plane.
    pub fn plane(&self) -> &Plane {
        &self.plane
    }

    /// Flip the polygon (reverse winding order and plane).
    pub fn flip(&self) -> Polygon {
        let mut vertices = self.vertices.clone();
        vertices.reverse();
        Polygon {
            vertices,
            plane: self.plane.flip(),
        }
    }

    /// Classify this polygon relative to a plane.
    pub fn classify(&self, plane: &Plane) -> Classification {
        let mut front_count = 0;
        let mut back_count = 0;

        for v in &self.vertices {
            match plane.classify_point(v) {
                Classification::Front => front_count += 1,
                Classification::Back => back_count += 1,
                _ => {}
            }
        }

        if front_count > 0 && back_count > 0 {
            Classification::Spanning
        } else if front_count > 0 {
            Classification::Front
        } else if back_count > 0 {
            Classification::Back
        } else {
            Classification::Coplanar
        }
    }

    /// Split polygon by a plane.
    ///
    /// ## Parameters
    ///
    /// - `plane`: Splitting plane
    /// - `coplanar_front`: Output for coplanar polygons facing same direction
    /// - `coplanar_back`: Output for coplanar polygons facing opposite direction
    /// - `front`: Output for polygons in front of plane
    /// - `back`: Output for polygons behind plane
    pub fn split(
        &self,
        plane: &Plane,
        coplanar_front: &mut Vec<Polygon>,
        coplanar_back: &mut Vec<Polygon>,
        front: &mut Vec<Polygon>,
        back: &mut Vec<Polygon>,
    ) {
        match self.classify(plane) {
            Classification::Coplanar => {
                // Check if polygon faces same direction as plane
                if self.plane.normal().dot(&plane.normal()) > 0.0 {
                    coplanar_front.push(self.clone());
                } else {
                    coplanar_back.push(self.clone());
                }
            }
            Classification::Front => {
                front.push(self.clone());
            }
            Classification::Back => {
                back.push(self.clone());
            }
            Classification::Spanning => {
                // Split the polygon
                let mut front_verts = Vec::new();
                let mut back_verts = Vec::new();

                for i in 0..self.vertices.len() {
                    let j = (i + 1) % self.vertices.len();
                    let vi = &self.vertices[i];
                    let vj = &self.vertices[j];

                    let ti = plane.classify_point(vi);
                    let tj = plane.classify_point(vj);

                    // Add current vertex to appropriate list(s)
                    if ti != Classification::Back {
                        front_verts.push(*vi);
                    }
                    if ti != Classification::Front {
                        back_verts.push(*vi);
                    }

                    // If edge crosses plane, add intersection point
                    if (ti == Classification::Front && tj == Classification::Back)
                        || (ti == Classification::Back && tj == Classification::Front)
                    {
                        let di = plane.signed_distance(vi);
                        let dj = plane.signed_distance(vj);
                        let t = di / (di - dj);
                        let intersection = vi.lerp(vj, t);
                        front_verts.push(intersection);
                        back_verts.push(intersection);
                    }
                }

                // Create new polygons if they have enough vertices
                if front_verts.len() >= 3 {
                    if let Some(poly) = Polygon::from_vertices(front_verts) {
                        front.push(poly);
                    }
                }
                if back_verts.len() >= 3 {
                    if let Some(poly) = Polygon::from_vertices(back_verts) {
                        back.push(poly);
                    }
                }
            }
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_triangle() -> Polygon {
        Polygon::from_vertices(vec![
            Vertex::new(0.0, 0.0, 0.0),
            Vertex::new(1.0, 0.0, 0.0),
            Vertex::new(0.5, 1.0, 0.0),
        ])
        .unwrap()
    }

    #[test]
    fn test_polygon_from_vertices() {
        let poly = create_triangle();
        assert_eq!(poly.vertices.len(), 3);
    }

    #[test]
    fn test_polygon_flip() {
        let poly = create_triangle();
        let flipped = poly.flip();

        // Vertices should be reversed
        assert_eq!(flipped.vertices[0].x, poly.vertices[2].x);
        assert_eq!(flipped.vertices[2].x, poly.vertices[0].x);
    }

    #[test]
    fn test_polygon_classify_front() {
        let poly = Polygon::from_vertices(vec![
            Vertex::new(0.0, 0.0, 1.0),
            Vertex::new(1.0, 0.0, 1.0),
            Vertex::new(0.5, 1.0, 1.0),
        ])
        .unwrap();

        let plane = Plane::new(Vertex::new(0.0, 0.0, 1.0), 0.0);
        assert_eq!(poly.classify(&plane), Classification::Front);
    }

    #[test]
    fn test_polygon_classify_back() {
        let poly = Polygon::from_vertices(vec![
            Vertex::new(0.0, 0.0, -1.0),
            Vertex::new(1.0, 0.0, -1.0),
            Vertex::new(0.5, 1.0, -1.0),
        ])
        .unwrap();

        let plane = Plane::new(Vertex::new(0.0, 0.0, 1.0), 0.0);
        assert_eq!(poly.classify(&plane), Classification::Back);
    }

    #[test]
    fn test_polygon_split_spanning() {
        // Triangle that spans z=0 plane
        let poly = Polygon::from_vertices(vec![
            Vertex::new(0.0, 0.0, -1.0),
            Vertex::new(1.0, 0.0, -1.0),
            Vertex::new(0.5, 0.0, 1.0),
        ])
        .unwrap();

        let plane = Plane::new(Vertex::new(0.0, 0.0, 1.0), 0.0);

        let mut cf = Vec::new();
        let mut cb = Vec::new();
        let mut f = Vec::new();
        let mut b = Vec::new();

        poly.split(&plane, &mut cf, &mut cb, &mut f, &mut b);

        assert!(!f.is_empty(), "Should have front polygon");
        assert!(!b.is_empty(), "Should have back polygon");
    }
}

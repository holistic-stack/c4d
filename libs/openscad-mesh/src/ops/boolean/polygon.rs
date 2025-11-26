//! # Polygon
//!
//! Polygon representation for BSP boolean operations.
//! Supports splitting polygons along planes for BSP tree construction.

use super::plane::{Plane, PointClassification, PolygonClassification};
use super::vertex::Vertex;

/// A convex polygon for BSP operations.
///
/// Polygons are assumed to be convex and planar. They can be split
/// along arbitrary planes during BSP tree construction.
///
/// # Example
///
/// ```rust,ignore
/// use glam::DVec3;
/// use openscad_mesh::ops::boolean::polygon::Polygon;
/// use openscad_mesh::ops::boolean::vertex::Vertex;
///
/// let triangle = Polygon::new(vec![
///     Vertex::new(DVec3::ZERO),
///     Vertex::new(DVec3::X),
///     Vertex::new(DVec3::Y),
/// ]);
/// ```
#[derive(Debug, Clone)]
pub struct Polygon {
    /// Vertices in counter-clockwise order
    pub vertices: Vec<Vertex>,
    /// Cached plane (computed lazily)
    plane: Option<Plane>,
}

impl Polygon {
    /// Creates a new polygon from vertices.
    ///
    /// Vertices should be in counter-clockwise order when viewed from
    /// the front (positive normal direction).
    ///
    /// # Arguments
    ///
    /// * `vertices` - At least 3 vertices in CCW order
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let poly = Polygon::new(vec![v0, v1, v2]);
    /// ```
    pub fn new(vertices: Vec<Vertex>) -> Self {
        Self {
            vertices,
            plane: None,
        }
    }

    /// Returns the plane containing this polygon.
    ///
    /// Computes and caches the plane on first call.
    pub fn plane(&mut self) -> Option<Plane> {
        if self.plane.is_none() && self.vertices.len() >= 3 {
            self.plane = Plane::from_points(
                self.vertices[0].position,
                self.vertices[1].position,
                self.vertices[2].position,
            );
        }
        self.plane
    }

    /// Returns the plane without caching (for immutable access).
    pub fn get_plane(&self) -> Option<Plane> {
        if self.vertices.len() >= 3 {
            Plane::from_points(
                self.vertices[0].position,
                self.vertices[1].position,
                self.vertices[2].position,
            )
        } else {
            None
        }
    }

    /// Flips the polygon to face the opposite direction.
    ///
    /// Reverses vertex order and flips the plane normal.
    pub fn flip(&mut self) {
        self.vertices.reverse();
        if let Some(ref mut plane) = self.plane {
            plane.flip();
        }
    }

    /// Returns a flipped copy of this polygon.
    pub fn flipped(&self) -> Self {
        let mut vertices = self.vertices.clone();
        vertices.reverse();
        Self {
            vertices,
            plane: self.plane.map(|p| p.flipped()),
        }
    }

    /// Classifies this polygon relative to a plane.
    ///
    /// # Arguments
    ///
    /// * `plane` - The plane to classify against
    ///
    /// # Returns
    ///
    /// The classification (Front, Back, Coplanar, or Spanning).
    pub fn classify(&self, plane: &Plane) -> PolygonClassification {
        let mut front_count = 0;
        let mut back_count = 0;

        for vertex in &self.vertices {
            match plane.classify_point(vertex.position) {
                PointClassification::Front => front_count += 1,
                PointClassification::Back => back_count += 1,
                PointClassification::Coplanar => {}
            }
        }

        if front_count > 0 && back_count > 0 {
            PolygonClassification::Spanning
        } else if front_count > 0 {
            PolygonClassification::Front
        } else if back_count > 0 {
            PolygonClassification::Back
        } else {
            PolygonClassification::Coplanar
        }
    }

    /// Splits this polygon by a plane.
    ///
    /// # Arguments
    ///
    /// * `plane` - The splitting plane
    ///
    /// # Returns
    ///
    /// A tuple of (front_polygons, back_polygons).
    /// Each may be empty if the polygon is entirely on one side.
    ///
    /// # Algorithm
    ///
    /// For each edge crossing the plane, compute the intersection point
    /// and add it to both the front and back polygon being constructed.
    pub fn split(&self, plane: &Plane) -> (Vec<Polygon>, Vec<Polygon>) {
        let classification = self.classify(plane);

        match classification {
            PolygonClassification::Front => (vec![self.clone()], vec![]),
            PolygonClassification::Back => (vec![], vec![self.clone()]),
            PolygonClassification::Coplanar => {
                // Check if polygon faces same direction as plane
                if let Some(poly_plane) = self.get_plane() {
                    if poly_plane.normal.dot(plane.normal) > 0.0 {
                        (vec![self.clone()], vec![])
                    } else {
                        (vec![], vec![self.clone()])
                    }
                } else {
                    (vec![], vec![])
                }
            }
            PolygonClassification::Spanning => {
                self.split_spanning(plane)
            }
        }
    }

    /// Splits a spanning polygon by a plane.
    ///
    /// Internal method called when polygon definitely spans the plane.
    fn split_spanning(&self, plane: &Plane) -> (Vec<Polygon>, Vec<Polygon>) {
        let mut front_verts = Vec::new();
        let mut back_verts = Vec::new();

        let n = self.vertices.len();
        for i in 0..n {
            let vi = &self.vertices[i];
            let vj = &self.vertices[(i + 1) % n];

            let ti = plane.classify_point(vi.position);
            let tj = plane.classify_point(vj.position);

            // Add current vertex to appropriate list(s)
            match ti {
                PointClassification::Front => {
                    front_verts.push(vi.clone());
                }
                PointClassification::Back => {
                    back_verts.push(vi.clone());
                }
                PointClassification::Coplanar => {
                    front_verts.push(vi.clone());
                    back_verts.push(vi.clone());
                }
            }

            // Check if edge crosses plane
            let crosses = matches!(
                (ti, tj),
                (PointClassification::Front, PointClassification::Back) |
                (PointClassification::Back, PointClassification::Front)
            );

            if crosses {
                // Compute intersection point
                let di = plane.signed_distance(vi.position);
                let dj = plane.signed_distance(vj.position);
                let t = di / (di - dj);

                // Clamp t to avoid numerical issues
                let t = t.clamp(0.0, 1.0);

                let intersection = vi.lerp(vj, t);
                front_verts.push(intersection.clone());
                back_verts.push(intersection);
            }
        }

        // Build result polygons
        let front = if front_verts.len() >= 3 {
            vec![Polygon::new(front_verts)]
        } else {
            vec![]
        };

        let back = if back_verts.len() >= 3 {
            vec![Polygon::new(back_verts)]
        } else {
            vec![]
        };

        (front, back)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::constants::EPSILON;
    use glam::DVec3;

    fn make_triangle() -> Polygon {
        Polygon::new(vec![
            Vertex::new(DVec3::ZERO),
            Vertex::new(DVec3::X),
            Vertex::new(DVec3::Y),
        ])
    }

    #[test]
    fn test_polygon_new() {
        let poly = make_triangle();
        assert_eq!(poly.vertices.len(), 3);
    }

    #[test]
    fn test_polygon_plane() {
        let mut poly = make_triangle();
        let plane = poly.plane().unwrap();
        
        // Triangle in XY plane should have Z normal
        assert!((plane.normal - DVec3::Z).length() < EPSILON);
    }

    #[test]
    fn test_polygon_flip() {
        let mut poly = make_triangle();
        let original_first = poly.vertices[0].position;
        let original_last = poly.vertices[2].position;
        
        poly.flip();
        
        assert_eq!(poly.vertices[0].position, original_last);
        assert_eq!(poly.vertices[2].position, original_first);
    }

    #[test]
    fn test_classify_front() {
        let poly = Polygon::new(vec![
            Vertex::new(DVec3::new(0.0, 0.0, 1.0)),
            Vertex::new(DVec3::new(1.0, 0.0, 1.0)),
            Vertex::new(DVec3::new(0.0, 1.0, 1.0)),
        ]);
        
        let plane = Plane::new(DVec3::Z, 0.0);
        assert_eq!(poly.classify(&plane), PolygonClassification::Front);
    }

    #[test]
    fn test_classify_back() {
        let poly = Polygon::new(vec![
            Vertex::new(DVec3::new(0.0, 0.0, -1.0)),
            Vertex::new(DVec3::new(1.0, 0.0, -1.0)),
            Vertex::new(DVec3::new(0.0, 1.0, -1.0)),
        ]);
        
        let plane = Plane::new(DVec3::Z, 0.0);
        assert_eq!(poly.classify(&plane), PolygonClassification::Back);
    }

    #[test]
    fn test_classify_coplanar() {
        let poly = make_triangle(); // In XY plane at z=0
        let plane = Plane::new(DVec3::Z, 0.0);
        assert_eq!(poly.classify(&plane), PolygonClassification::Coplanar);
    }

    #[test]
    fn test_classify_spanning() {
        let poly = Polygon::new(vec![
            Vertex::new(DVec3::new(0.0, 0.0, -1.0)),
            Vertex::new(DVec3::new(1.0, 0.0, 1.0)),
            Vertex::new(DVec3::new(0.0, 1.0, 0.0)),
        ]);
        
        let plane = Plane::new(DVec3::Z, 0.0);
        assert_eq!(poly.classify(&plane), PolygonClassification::Spanning);
    }

    #[test]
    fn test_split_front() {
        let poly = Polygon::new(vec![
            Vertex::new(DVec3::new(0.0, 0.0, 1.0)),
            Vertex::new(DVec3::new(1.0, 0.0, 1.0)),
            Vertex::new(DVec3::new(0.0, 1.0, 1.0)),
        ]);
        
        let plane = Plane::new(DVec3::Z, 0.0);
        let (front, back) = poly.split(&plane);
        
        assert_eq!(front.len(), 1);
        assert_eq!(back.len(), 0);
    }

    #[test]
    fn test_split_back() {
        let poly = Polygon::new(vec![
            Vertex::new(DVec3::new(0.0, 0.0, -1.0)),
            Vertex::new(DVec3::new(1.0, 0.0, -1.0)),
            Vertex::new(DVec3::new(0.0, 1.0, -1.0)),
        ]);
        
        let plane = Plane::new(DVec3::Z, 0.0);
        let (front, back) = poly.split(&plane);
        
        assert_eq!(front.len(), 0);
        assert_eq!(back.len(), 1);
    }

    #[test]
    fn test_split_spanning() {
        // Triangle spanning z=0 plane
        let poly = Polygon::new(vec![
            Vertex::new(DVec3::new(0.0, 0.0, -1.0)),
            Vertex::new(DVec3::new(1.0, 0.0, 1.0)),
            Vertex::new(DVec3::new(0.0, 1.0, 1.0)),
        ]);
        
        let plane = Plane::new(DVec3::Z, 0.0);
        let (front, back) = poly.split(&plane);
        
        // Should produce both front and back polygons
        assert!(!front.is_empty());
        assert!(!back.is_empty());
    }
}

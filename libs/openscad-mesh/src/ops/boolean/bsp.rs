//! # BSP Tree for CSG Operations
//!
//! Binary Space Partitioning tree implementation for boolean operations.

use super::polygon::Polygon;

// =============================================================================
// BSP NODE
// =============================================================================

/// A node in the BSP tree.
pub struct BspNode {
    /// Splitting plane (from first polygon).
    plane: Option<super::plane::Plane>,
    /// Front subtree (positive side of plane).
    front: Option<Box<BspNode>>,
    /// Back subtree (negative side of plane).
    back: Option<Box<BspNode>>,
    /// Polygons coplanar with splitting plane.
    polygons: Vec<Polygon>,
}

impl BspNode {
    /// Create new BSP tree from polygons.
    pub fn new(polygons: Vec<Polygon>) -> Self {
        let mut node = Self {
            plane: None,
            front: None,
            back: None,
            polygons: Vec::new(),
        };
        node.build(polygons);
        node
    }

    /// Build tree from polygons.
    fn build(&mut self, polygons: Vec<Polygon>) {
        if polygons.is_empty() {
            return;
        }

        // Use first polygon's plane as splitting plane
        if self.plane.is_none() {
            self.plane = Some(*polygons[0].plane());
        }

        let plane = self.plane.as_ref().unwrap();

        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();

        for poly in polygons {
            let mut coplanar_front = Vec::new();
            let mut coplanar_back = Vec::new();
            let mut front = Vec::new();
            let mut back = Vec::new();

            poly.split(
                plane,
                &mut coplanar_front,
                &mut coplanar_back,
                &mut front,
                &mut back,
            );

            self.polygons.extend(coplanar_front);
            self.polygons.extend(coplanar_back);
            front_polys.extend(front);
            back_polys.extend(back);
        }

        if !front_polys.is_empty() {
            if self.front.is_none() {
                self.front = Some(Box::new(BspNode {
                    plane: None,
                    front: None,
                    back: None,
                    polygons: Vec::new(),
                }));
            }
            self.front.as_mut().unwrap().build(front_polys);
        }

        if !back_polys.is_empty() {
            if self.back.is_none() {
                self.back = Some(Box::new(BspNode {
                    plane: None,
                    front: None,
                    back: None,
                    polygons: Vec::new(),
                }));
            }
            self.back.as_mut().unwrap().build(back_polys);
        }
    }

    /// Invert the tree (swap inside and outside).
    pub fn invert(&mut self) {
        // Flip all polygons
        for poly in &mut self.polygons {
            *poly = poly.flip();
        }

        // Flip the plane
        if let Some(ref mut plane) = self.plane {
            *plane = plane.flip();
        }

        // Swap front and back
        std::mem::swap(&mut self.front, &mut self.back);

        // Recursively invert children
        if let Some(ref mut front) = self.front {
            front.invert();
        }
        if let Some(ref mut back) = self.back {
            back.invert();
        }
    }

    /// Clip polygons to this tree.
    ///
    /// Removes parts of polygons that are inside this tree.
    fn clip_polygons(&self, polygons: Vec<Polygon>) -> Vec<Polygon> {
        if self.plane.is_none() {
            return polygons;
        }

        let plane = self.plane.as_ref().unwrap();
        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();

        for poly in polygons {
            let mut coplanar_front = Vec::new();
            let mut coplanar_back = Vec::new();
            let mut front = Vec::new();
            let mut back = Vec::new();

            poly.split(
                plane,
                &mut coplanar_front,
                &mut coplanar_back,
                &mut front,
                &mut back,
            );

            front_polys.extend(coplanar_front);
            front_polys.extend(front);
            back_polys.extend(coplanar_back);
            back_polys.extend(back);
        }

        // Recursively clip
        let front_result = if let Some(ref front) = self.front {
            front.clip_polygons(front_polys)
        } else {
            front_polys
        };

        let back_result = if let Some(ref back) = self.back {
            back.clip_polygons(back_polys)
        } else {
            Vec::new() // Discard polygons behind if no back tree
        };

        let mut result = front_result;
        result.extend(back_result);
        result
    }

    /// Remove polygons from this tree that are inside another tree.
    pub fn clip_to(&mut self, other: &BspNode) {
        self.polygons = other.clip_polygons(std::mem::take(&mut self.polygons));

        if let Some(ref mut front) = self.front {
            front.clip_to(other);
        }
        if let Some(ref mut back) = self.back {
            back.clip_to(other);
        }
    }

    /// Get all polygons from this tree.
    pub fn all_polygons(&self) -> Vec<Polygon> {
        let mut result = self.polygons.clone();

        if let Some(ref front) = self.front {
            result.extend(front.all_polygons());
        }
        if let Some(ref back) = self.back {
            result.extend(back.all_polygons());
        }

        result
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::vertex::Vertex;

    fn create_triangle_polygon(z: f64) -> Polygon {
        Polygon::from_vertices(vec![
            Vertex::new(0.0, 0.0, z),
            Vertex::new(1.0, 0.0, z),
            Vertex::new(0.5, 1.0, z),
        ])
        .unwrap()
    }

    #[test]
    fn test_bsp_new_empty() {
        let node = BspNode::new(Vec::new());
        assert!(node.plane.is_none());
        assert!(node.polygons.is_empty());
    }

    #[test]
    fn test_bsp_new_single() {
        let poly = create_triangle_polygon(0.0);
        let node = BspNode::new(vec![poly]);
        assert!(node.plane.is_some());
        assert_eq!(node.polygons.len(), 1);
    }

    #[test]
    fn test_bsp_invert() {
        let poly = create_triangle_polygon(0.0);
        let mut node = BspNode::new(vec![poly]);

        let original_normal = node.plane.as_ref().unwrap().normal();
        node.invert();
        let inverted_normal = node.plane.as_ref().unwrap().normal();

        // Normal should be flipped
        assert!((original_normal.z + inverted_normal.z).abs() < 1e-10);
    }

    #[test]
    fn test_bsp_all_polygons() {
        let polys = vec![
            create_triangle_polygon(0.0),
            create_triangle_polygon(1.0),
            create_triangle_polygon(-1.0),
        ];
        let node = BspNode::new(polys);

        let all = node.all_polygons();
        assert_eq!(all.len(), 3);
    }
}

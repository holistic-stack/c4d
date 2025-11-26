//! # BSP Tree
//!
//! Binary Space Partitioning tree for CSG boolean operations.
//! Based on the csg.js algorithm by Evan Wallace.
//!
//! ## Algorithm
//!
//! Each BSP node contains:
//! - A dividing plane
//! - Polygons coplanar with the plane
//! - Front subtree (polygons in front of plane)
//! - Back subtree (polygons behind plane)
//!
//! ## Operations
//!
//! - `clip_to`: Remove polygons from this tree that are inside another tree
//! - `invert`: Flip all polygons and swap front/back subtrees
//! - `all_polygons`: Collect all polygons from the tree

use super::polygon::Polygon;

/// A node in the BSP tree.
///
/// Each node partitions space using a plane and stores polygons
/// coplanar with that plane.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::boolean::bsp::BspNode;
///
/// let polygons = mesh_to_polygons(&mesh);
/// let tree = BspNode::new(polygons);
/// ```
#[derive(Debug, Clone)]
pub struct BspNode {
    /// Polygons coplanar with this node's plane
    polygons: Vec<Polygon>,
    /// Front subtree (polygons in front of plane)
    front: Option<Box<BspNode>>,
    /// Back subtree (polygons behind plane)
    back: Option<Box<BspNode>>,
}

impl BspNode {
    /// Creates a new BSP tree from a list of polygons.
    ///
    /// # Arguments
    ///
    /// * `polygons` - Polygons to build the tree from
    ///
    /// # Returns
    ///
    /// A BSP tree containing all the polygons.
    pub fn new(polygons: Vec<Polygon>) -> Self {
        let mut node = Self {
            polygons: Vec::new(),
            front: None,
            back: None,
        };

        if !polygons.is_empty() {
            node.build(polygons);
        }

        node
    }

    /// Builds the BSP tree from polygons.
    ///
    /// Uses the first polygon's plane as the splitting plane.
    fn build(&mut self, mut polygons: Vec<Polygon>) {
        if polygons.is_empty() {
            return;
        }

        // Use first polygon's plane as splitting plane
        let mut first_poly = polygons.remove(0);
        let plane = match first_poly.plane() {
            Some(p) => p,
            None => {
                // Degenerate polygon, try next
                if !polygons.is_empty() {
                    self.build(polygons);
                }
                return;
            }
        };

        // Add first polygon to coplanar list
        self.polygons.push(first_poly);

        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();

        // Classify and split remaining polygons
        for poly in polygons {
            let (front, back) = poly.split(&plane);
            front_polys.extend(front);
            back_polys.extend(back);
        }

        // Recursively build subtrees
        if !front_polys.is_empty() {
            self.front = Some(Box::new(BspNode::new(front_polys)));
        }

        if !back_polys.is_empty() {
            self.back = Some(Box::new(BspNode::new(back_polys)));
        }
    }

    /// Inverts this BSP tree (flips all polygons and swaps subtrees).
    ///
    /// Used for implementing difference and intersection operations.
    pub fn invert(&mut self) {
        // Flip all polygons at this node
        for poly in &mut self.polygons {
            poly.flip();
        }

        // Swap front and back subtrees
        std::mem::swap(&mut self.front, &mut self.back);

        // Recursively invert subtrees
        if let Some(ref mut front) = self.front {
            front.invert();
        }
        if let Some(ref mut back) = self.back {
            back.invert();
        }
    }

    /// Clips polygons to this BSP tree.
    ///
    /// Removes parts of polygons that are inside the solid represented
    /// by this tree.
    ///
    /// # Arguments
    ///
    /// * `polygons` - Polygons to clip
    ///
    /// # Returns
    ///
    /// Polygons that are outside this tree's solid.
    pub fn clip_polygons(&self, polygons: Vec<Polygon>) -> Vec<Polygon> {
        if self.polygons.is_empty() {
            return polygons;
        }

        // Get splitting plane from first polygon
        let plane = match self.polygons[0].get_plane() {
            Some(p) => p,
            None => return polygons,
        };

        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();

        for poly in polygons {
            let (front, back) = poly.split(&plane);
            front_polys.extend(front);
            back_polys.extend(back);
        }

        // Recursively clip front polygons
        if let Some(ref front) = self.front {
            front_polys = front.clip_polygons(front_polys);
        }

        // Recursively clip back polygons (or discard if no back tree)
        if let Some(ref back) = self.back {
            back_polys = back.clip_polygons(back_polys);
        } else {
            // No back tree means back polygons are inside solid - discard them
            back_polys.clear();
        }

        // Combine results
        front_polys.extend(back_polys);
        front_polys
    }

    /// Clips this tree's polygons to another tree.
    ///
    /// Removes parts of this tree's polygons that are inside the other tree.
    ///
    /// # Arguments
    ///
    /// * `other` - The tree to clip against
    pub fn clip_to(&mut self, other: &BspNode) {
        self.polygons = other.clip_polygons(std::mem::take(&mut self.polygons));

        if let Some(ref mut front) = self.front {
            front.clip_to(other);
        }
        if let Some(ref mut back) = self.back {
            back.clip_to(other);
        }
    }

    /// Collects all polygons from this tree.
    ///
    /// # Returns
    ///
    /// A vector containing all polygons in the tree.
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

    /// Returns the number of polygons in this tree.
    pub fn polygon_count(&self) -> usize {
        let mut count = self.polygons.len();

        if let Some(ref front) = self.front {
            count += front.polygon_count();
        }
        if let Some(ref back) = self.back {
            count += back.polygon_count();
        }

        count
    }

    /// Returns the depth of this tree.
    pub fn depth(&self) -> usize {
        let front_depth = self.front.as_ref().map_or(0, |f| f.depth());
        let back_depth = self.back.as_ref().map_or(0, |b| b.depth());
        1 + front_depth.max(back_depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::vertex::Vertex;
    use glam::DVec3;

    fn make_triangle_polygon(z: f64) -> Polygon {
        Polygon::new(vec![
            Vertex::new(DVec3::new(0.0, 0.0, z)),
            Vertex::new(DVec3::new(1.0, 0.0, z)),
            Vertex::new(DVec3::new(0.0, 1.0, z)),
        ])
    }

    #[test]
    fn test_bsp_new_empty() {
        let tree = BspNode::new(vec![]);
        assert_eq!(tree.polygon_count(), 0);
    }

    #[test]
    fn test_bsp_new_single() {
        let poly = make_triangle_polygon(0.0);
        let tree = BspNode::new(vec![poly]);
        assert_eq!(tree.polygon_count(), 1);
    }

    #[test]
    fn test_bsp_new_multiple() {
        let polys = vec![
            make_triangle_polygon(0.0),
            make_triangle_polygon(1.0),
            make_triangle_polygon(-1.0),
        ];
        let tree = BspNode::new(polys);
        assert_eq!(tree.polygon_count(), 3);
    }

    #[test]
    fn test_bsp_all_polygons() {
        let polys = vec![
            make_triangle_polygon(0.0),
            make_triangle_polygon(1.0),
        ];
        let tree = BspNode::new(polys);
        let all = tree.all_polygons();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_bsp_invert() {
        let poly = make_triangle_polygon(0.0);
        let original_normal = poly.get_plane().unwrap().normal;
        
        let mut tree = BspNode::new(vec![poly]);
        tree.invert();
        
        let inverted_normal = tree.polygons[0].get_plane().unwrap().normal;
        
        // Normal should be flipped
        assert!((original_normal + inverted_normal).length() < 0.001);
    }

    #[test]
    fn test_bsp_depth() {
        let tree = BspNode::new(vec![make_triangle_polygon(0.0)]);
        assert!(tree.depth() >= 1);
    }

    #[test]
    fn test_bsp_clip_polygons_front() {
        // Create a tree from a polygon at z=0
        let tree = BspNode::new(vec![make_triangle_polygon(0.0)]);
        
        // Clip a polygon at z=1 (in front)
        let to_clip = vec![make_triangle_polygon(1.0)];
        let result = tree.clip_polygons(to_clip);
        
        // Front polygon should survive
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_bsp_clip_polygons_back() {
        // Create a tree from a polygon at z=0
        let tree = BspNode::new(vec![make_triangle_polygon(0.0)]);
        
        // Clip a polygon at z=-1 (behind)
        let to_clip = vec![make_triangle_polygon(-1.0)];
        let result = tree.clip_polygons(to_clip);
        
        // Back polygon should be clipped (removed)
        assert_eq!(result.len(), 0);
    }
}

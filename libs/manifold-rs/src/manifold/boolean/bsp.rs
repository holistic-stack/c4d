//! # BSP Tree Implementation
//!
//! Binary Space Partitioning tree for boolean operations.
//!
//! ## Algorithm Overview
//!
//! The BSP tree recursively partitions 3D space using planes derived from polygon faces.
//! Each node stores:
//! - A splitting plane
//! - Polygons coplanar with that plane
//! - Front subtree (positive side)
//! - Back subtree (negative side)
//!
//! ## Boolean Operations
//!
//! - **Union**: Keep (A outside B) ∪ (B outside A)
//! - **Difference**: Keep (A outside B) ∪ (B inside A, reversed)
//! - **Intersection**: Keep (A inside B) ∪ (B inside A)
//!
//! ## Limitations vs Manifold
//!
//! BSP splits polygons along arbitrary planes, causing ~44% more triangles than
//! Manifold's edge-intersection algorithm. See `tasks.md` for future optimization plans.
//!
//! ## References
//!
//! - Naylor, B. (1990). "Binary Space Partitioning Trees"
//! - Thibault, W. C., & Naylor, B. F. (1987). "Set operations on polyhedra using BSP trees"

use crate::mesh::Mesh;
use super::geometry::{dot, point_inside_mesh};
use super::polygon::{BspPolygon, Plane, PolygonClassification, split_polygon};

// =============================================================================
// BSP NODE
// =============================================================================

/// BSP tree node for mesh partitioning.
///
/// ## Structure
///
/// ```text
///           [Plane]
///          /       \
///      Front       Back
///     (+ side)   (- side)
/// ```
///
/// Leaf nodes have `plane = None` and empty front/back.
#[derive(Debug)]
pub struct BspNode {
    /// Splitting plane (None for leaf nodes)
    plane: Option<Plane>,
    /// Polygons coplanar with this node's plane
    polygons: Vec<BspPolygon>,
    /// Front subtree (positive side of plane)
    front: Option<Box<BspNode>>,
    /// Back subtree (negative side of plane)
    back: Option<Box<BspNode>>,
}

impl BspNode {
    /// Create empty BSP node.
    pub fn new() -> Self {
        Self {
            plane: None,
            polygons: Vec::new(),
            front: None,
            back: None,
        }
    }

    /// Build BSP tree from polygons.
    ///
    /// ## Algorithm
    ///
    /// 1. Use first polygon's plane as splitting plane
    /// 2. Classify each polygon: front, back, coplanar, or spanning
    /// 3. Split spanning polygons
    /// 4. Recursively build front/back subtrees
    ///
    /// ## Complexity
    ///
    /// O(n² log n) worst case, O(n log n) average for well-distributed geometry.
    pub fn build(&mut self, polygons: Vec<BspPolygon>) {
        if polygons.is_empty() {
            return;
        }

        // Use first polygon's plane as splitting plane
        if self.plane.is_none() {
            self.plane = Some(Plane::from_polygon(&polygons[0]));
        }

        let plane = self.plane.unwrap();
        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();

        for poly in polygons {
            let (classification, front_part, back_part) = split_polygon(&poly, &plane);
            
            match classification {
                PolygonClassification::Coplanar => {
                    // Store with this node (both front and back facing)
                    self.polygons.push(poly);
                }
                PolygonClassification::Front => {
                    front_polys.push(poly);
                }
                PolygonClassification::Back => {
                    back_polys.push(poly);
                }
                PolygonClassification::Spanning => {
                    if let Some(fp) = front_part {
                        front_polys.push(fp);
                    }
                    if let Some(bp) = back_part {
                        back_polys.push(bp);
                    }
                }
            }
        }

        // Build subtrees
        if !front_polys.is_empty() {
            self.front = Some(Box::new(BspNode::new()));
            self.front.as_mut().unwrap().build(front_polys);
        }

        if !back_polys.is_empty() {
            self.back = Some(Box::new(BspNode::new()));
            self.back.as_mut().unwrap().build(back_polys);
        }
    }

    /// Clip polygons using mesh for robust leaf classification.
    ///
    /// ## Parameters
    ///
    /// - `polygons`: Polygons to clip
    /// - `mesh`: Original mesh for point-in-mesh tests at leaves
    /// - `keep_inside`: If true, keep polygons inside mesh; if false, keep outside
    ///
    /// ## Why Robust Classification?
    ///
    /// Standard BSP clipping relies on implicit leaf classification which can fail
    /// for complex geometry. This method uses explicit ray-casting at leaf nodes.
    pub fn clip_polygons_robust(
        &self,
        polygons: Vec<BspPolygon>,
        mesh: &Mesh,
        keep_inside: bool,
    ) -> Vec<BspPolygon> {
        if self.plane.is_none() {
            // Leaf node: verify each polygon against mesh
            return polygons.into_iter().filter(|poly| {
                let center = poly.centroid();
                let is_inside = point_inside_mesh(&center, mesh);
                if keep_inside { is_inside } else { !is_inside }
            }).collect();
        }

        let plane = self.plane.unwrap();
        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();

        // Classify and split polygons
        for poly in polygons {
            let (classification, front_part, back_part) = split_polygon(&poly, &plane);
            
            match classification {
                PolygonClassification::Coplanar => {
                    // Route based on normal direction relative to plane
                    let facing_same = dot(&poly.normal, &plane.normal) > 0.0;
                    if facing_same {
                        front_polys.push(poly);
                    } else {
                        back_polys.push(poly);
                    }
                }
                PolygonClassification::Front => front_polys.push(poly),
                PolygonClassification::Back => back_polys.push(poly),
                PolygonClassification::Spanning => {
                    if let Some(fp) = front_part {
                        front_polys.push(fp);
                    }
                    if let Some(bp) = back_part {
                        back_polys.push(bp);
                    }
                }
            }
        }

        // Recursively clip in subtrees
        let mut result = self.clip_subtree_robust(
            &self.front,
            front_polys,
            mesh,
            keep_inside,
        );
        
        result.extend(self.clip_subtree_robust(
            &self.back,
            back_polys,
            mesh,
            keep_inside,
        ));

        result
    }

    /// Helper to clip in a subtree (or at leaf if subtree is None).
    fn clip_subtree_robust(
        &self,
        subtree: &Option<Box<BspNode>>,
        polygons: Vec<BspPolygon>,
        mesh: &Mesh,
        keep_inside: bool,
    ) -> Vec<BspPolygon> {
        if let Some(ref node) = subtree {
            node.clip_polygons_robust(polygons, mesh, keep_inside)
        } else {
            // Missing child = implicit leaf, check against mesh
            polygons.into_iter().filter(|poly| {
                let center = poly.centroid();
                let is_inside = point_inside_mesh(&center, mesh);
                if keep_inside { is_inside } else { !is_inside }
            }).collect()
        }
    }

    // =========================================================================
    // LEGACY METHODS (kept for reference, not actively used)
    // =========================================================================

    /// Invert the BSP tree (swap inside/outside).
    ///
    /// Flips all polygons and swaps front/back children.
    #[allow(dead_code)]
    pub fn invert(&mut self) {
        for poly in &mut self.polygons {
            poly.flip();
        }

        if let Some(ref mut plane) = self.plane {
            plane.normal = [-plane.normal[0], -plane.normal[1], -plane.normal[2]];
            plane.w = -plane.w;
        }

        std::mem::swap(&mut self.front, &mut self.back);

        if let Some(ref mut front) = self.front {
            front.invert();
        }
        if let Some(ref mut back) = self.back {
            back.invert();
        }
    }

    /// Standard polygon clipping (may misclassify at leaves).
    #[allow(dead_code)]
    pub fn clip_polygons(&self, polygons: Vec<BspPolygon>) -> Vec<BspPolygon> {
        if self.plane.is_none() {
            return polygons;
        }

        let plane = self.plane.unwrap();
        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();

        for poly in polygons {
            let (classification, front_part, back_part) = split_polygon(&poly, &plane);
            
            match classification {
                PolygonClassification::Coplanar | PolygonClassification::Front => {
                    front_polys.push(poly);
                }
                PolygonClassification::Back => {
                    back_polys.push(poly);
                }
                PolygonClassification::Spanning => {
                    if let Some(fp) = front_part {
                        front_polys.push(fp);
                    }
                    if let Some(bp) = back_part {
                        back_polys.push(bp);
                    }
                }
            }
        }

        let mut result = if let Some(ref front) = self.front {
            front.clip_polygons(front_polys)
        } else {
            front_polys
        };

        let back_result = if let Some(ref back) = self.back {
            back.clip_polygons(back_polys)
        } else {
            Vec::new() // Discard back polygons if no back tree
        };

        result.extend(back_result);
        result
    }

    /// Clip this tree to another tree.
    #[allow(dead_code)]
    pub fn clip_to(&mut self, other: &BspNode) {
        self.polygons = other.clip_polygons(std::mem::take(&mut self.polygons));
        
        if let Some(ref mut front) = self.front {
            front.clip_to(other);
        }
        if let Some(ref mut back) = self.back {
            back.clip_to(other);
        }
    }

    /// Collect all polygons from tree.
    #[allow(dead_code)]
    pub fn all_polygons(&self) -> Vec<BspPolygon> {
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

impl Default for BspNode {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsp_node_new() {
        let node = BspNode::new();
        assert!(node.plane.is_none());
        assert!(node.polygons.is_empty());
        assert!(node.front.is_none());
        assert!(node.back.is_none());
    }

    #[test]
    fn test_bsp_build_empty() {
        let mut node = BspNode::new();
        node.build(vec![]);
        assert!(node.plane.is_none());
    }

    #[test]
    fn test_bsp_build_single_polygon() {
        let mut node = BspNode::new();
        let poly = BspPolygon::with_normal(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            [0.0, 0.0, 1.0],
        );
        node.build(vec![poly]);
        
        assert!(node.plane.is_some());
        assert_eq!(node.polygons.len(), 1);
    }
}

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
//!
//! ## Stack Safety
//!
//! All operations use iterative algorithms with explicit stacks to avoid
//! stack overflow in WASM environments where stack size is limited (~1MB).

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
    /// Creates a new BSP tree from polygons.
    ///
    /// Uses iterative construction to avoid stack overflow in WASM.
    ///
    /// # Algorithm Complexity
    ///
    /// - Uses O(N) iteration instead of O(N²) `remove(0)` operations
    /// - `swap_remove` for O(1) splitter extraction
    /// - Pre-allocated vectors to reduce reallocations
    ///
    /// # Arguments
    ///
    /// * `polygons` - Polygons to build the tree from
    ///
    /// # Returns
    ///
    /// A BSP tree containing all the polygons.
    pub fn new(polygons: Vec<Polygon>) -> Self {
        let mut root = Self {
            polygons: Vec::new(),
            front: None,
            back: None,
        };

        if polygons.is_empty() {
            return root;
        }

        // Build iteratively using a work stack
        // Each item is (node_ptr, polygons_to_add)
        // We use raw pointers for the stack since we need mutable access
        type WorkItem = (*mut BspNode, Vec<Polygon>);
        let mut stack: Vec<WorkItem> = vec![(&mut root as *mut BspNode, polygons)];

        while let Some((node_ptr, polys)) = stack.pop() {
            if polys.is_empty() {
                continue;
            }

            // Safety: we control all pointers and they point to valid nodes
            let node = unsafe { &mut *node_ptr };

            // Convert to owned vec for manipulation
            let mut polys = polys;

            // Find first valid polygon with a plane - simple O(N) scan
            // Note: More sophisticated splitter selection (sampling heuristic) was tested
            // but the overhead outweighed benefits for typical mesh sizes
            let splitter_idx = polys.iter().position(|p| p.get_plane().is_some());

            // No valid splitter found - skip this node
            let splitter_idx = match splitter_idx {
                Some(idx) => idx,
                None => continue,
            };
            
            // Use swap_remove for O(1) removal of splitter polygon
            let mut splitter = polys.swap_remove(splitter_idx);
            let plane = match splitter.plane() {
                Some(p) => p,
                None => continue, // Should not happen since we checked above
            };
            node.polygons.push(splitter);

            // Pre-allocate with estimated capacity to reduce reallocations
            let estimated_size = polys.len() / 2 + 1;
            let mut front_polys = Vec::with_capacity(estimated_size);
            let mut back_polys = Vec::with_capacity(estimated_size);

            // Classify and split remaining polygons - O(N) iteration
            for poly in polys {
                let (front, back) = poly.split(&plane);
                front_polys.extend(front);
                back_polys.extend(back);
            }

            // Create child nodes and add to stack
            if !front_polys.is_empty() {
                node.front = Some(Box::new(BspNode {
                    polygons: Vec::new(),
                    front: None,
                    back: None,
                }));
                if let Some(ref mut front) = node.front {
                    stack.push((front.as_mut() as *mut BspNode, front_polys));
                }
            }

            if !back_polys.is_empty() {
                node.back = Some(Box::new(BspNode {
                    polygons: Vec::new(),
                    front: None,
                    back: None,
                }));
                if let Some(ref mut back) = node.back {
                    stack.push((back.as_mut() as *mut BspNode, back_polys));
                }
            }
        }

        root
    }

    /// Inverts this BSP tree (flips all polygons and swaps subtrees).
    ///
    /// Uses iterative traversal to avoid stack overflow in WASM.
    /// Used for implementing difference and intersection operations.
    pub fn invert(&mut self) {
        // Use iterative traversal with explicit stack
        let mut stack: Vec<*mut BspNode> = vec![self as *mut BspNode];

        while let Some(node_ptr) = stack.pop() {
            // Safety: we control all pointers and they point to valid nodes
            let node = unsafe { &mut *node_ptr };

            // Flip all polygons at this node
            for poly in &mut node.polygons {
                poly.flip();
            }

            // Swap front and back subtrees
            std::mem::swap(&mut node.front, &mut node.back);

            // Add children to stack
            if let Some(ref mut front) = node.front {
                stack.push(front.as_mut() as *mut BspNode);
            }
            if let Some(ref mut back) = node.back {
                stack.push(back.as_mut() as *mut BspNode);
            }
        }
    }

    /// Clips polygons to this BSP tree.
    ///
    /// Uses iterative traversal to avoid stack overflow in WASM.
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

        // Work item: (node, front_polygons, back_polygons, is_processing_front)
        // We process nodes iteratively, tracking front/back polygons separately
        
        // Use a simpler approach: process one level at a time
        let mut result = Vec::new();
        
        // Stack of (node_ref, polygons_to_clip)
        let mut stack: Vec<(&BspNode, Vec<Polygon>)> = vec![(self, polygons)];
        
        while let Some((node, polys)) = stack.pop() {
            if polys.is_empty() {
                continue;
            }
            
            if node.polygons.is_empty() {
                result.extend(polys);
                continue;
            }

            // Get splitting plane from first polygon
            let plane = match node.polygons[0].get_plane() {
                Some(p) => p,
                None => {
                    result.extend(polys);
                    continue;
                }
            };

            let mut front_polys = Vec::new();
            let mut back_polys = Vec::new();

            for poly in polys {
                let (front, back) = poly.split(&plane);
                front_polys.extend(front);
                back_polys.extend(back);
            }

            // Process front polygons
            if let Some(ref front) = node.front {
                stack.push((front.as_ref(), front_polys));
            } else {
                result.extend(front_polys);
            }

            // Process back polygons (or discard if no back tree)
            if let Some(ref back) = node.back {
                stack.push((back.as_ref(), back_polys));
            }
            // If no back tree, back polygons are inside solid - discard them
        }

        result
    }

    /// Clips this tree's polygons to another tree.
    ///
    /// Uses iterative traversal to avoid stack overflow in WASM.
    /// Removes parts of this tree's polygons that are inside the other tree.
    ///
    /// # Arguments
    ///
    /// * `other` - The tree to clip against
    pub fn clip_to(&mut self, other: &BspNode) {
        // Use iterative traversal with explicit stack
        let mut stack: Vec<*mut BspNode> = vec![self as *mut BspNode];

        while let Some(node_ptr) = stack.pop() {
            // Safety: we control all pointers and they point to valid nodes
            let node = unsafe { &mut *node_ptr };

            node.polygons = other.clip_polygons(std::mem::take(&mut node.polygons));

            // Add children to stack
            if let Some(ref mut front) = node.front {
                stack.push(front.as_mut() as *mut BspNode);
            }
            if let Some(ref mut back) = node.back {
                stack.push(back.as_mut() as *mut BspNode);
            }
        }
    }

    /// Collects all polygons from this tree.
    ///
    /// Uses iterative traversal to avoid stack overflow in WASM.
    ///
    /// # Returns
    ///
    /// A vector containing all polygons in the tree.
    pub fn all_polygons(&self) -> Vec<Polygon> {
        let mut result = Vec::new();
        let mut stack: Vec<&BspNode> = vec![self];

        while let Some(node) = stack.pop() {
            result.extend(node.polygons.iter().cloned());

            if let Some(ref front) = node.front {
                stack.push(front.as_ref());
            }
            if let Some(ref back) = node.back {
                stack.push(back.as_ref());
            }
        }

        result
    }

    /// Returns the number of polygons in this tree.
    ///
    /// Uses iterative traversal to avoid stack overflow in WASM.
    #[allow(dead_code)]
    pub fn polygon_count(&self) -> usize {
        let mut count = 0;
        let mut stack: Vec<&BspNode> = vec![self];

        while let Some(node) = stack.pop() {
            count += node.polygons.len();

            if let Some(ref front) = node.front {
                stack.push(front.as_ref());
            }
            if let Some(ref back) = node.back {
                stack.push(back.as_ref());
            }
        }

        count
    }

    /// Returns the depth of this tree.
    ///
    /// Uses iterative traversal to avoid stack overflow in WASM.
    #[allow(dead_code)]
    pub fn depth(&self) -> usize {
        let mut max_depth = 0;
        // Stack of (node, current_depth)
        let mut stack: Vec<(&BspNode, usize)> = vec![(self, 1)];

        while let Some((node, depth)) = stack.pop() {
            max_depth = max_depth.max(depth);

            if let Some(ref front) = node.front {
                stack.push((front.as_ref(), depth + 1));
            }
            if let Some(ref back) = node.back {
                stack.push((back.as_ref(), depth + 1));
            }
        }

        max_depth
    }
}

impl Drop for BspNode {
    fn drop(&mut self) {
        // Iterative drop to avoid stack overflow
        let mut stack = Vec::new();
        
        if let Some(front) = self.front.take() { stack.push(front); }
        if let Some(back) = self.back.take() { stack.push(back); }
        
        while let Some(mut node) = stack.pop() {
            // Move children to stack before node is dropped
            if let Some(front) = node.front.take() { stack.push(front); }
            if let Some(back) = node.back.take() { stack.push(back); }
            // node is dropped here, but since children are None, it won't recurse
        }
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

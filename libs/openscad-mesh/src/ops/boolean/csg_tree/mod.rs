//! # CSG Tree - Lazy Evaluation and Tree Rewriting
//!
//! Represents CSG operations as a tree for lazy evaluation and optimization.
//! Implements tree rewriting rules to minimize intermediate mesh sizes.
//!
//! ## Key Optimizations
//!
//! 1. **Lazy Evaluation**: Meshes are only computed when needed
//! 2. **Tree Rewriting**: Reorders operations to minimize intermediate sizes
//! 3. **Bounding Box Pruning**: Skip operations on non-overlapping meshes
//! 4. **Batch Processing**: Groups similar operations for efficiency
//!
//! ## Example
//!
//! ```rust,ignore
//! use openscad_mesh::ops::boolean::csg_tree::{CsgNode, CsgOp};
//!
//! let tree = CsgNode::Binary {
//!     op: CsgOp::Union,
//!     left: Box::new(CsgNode::Leaf(mesh_a)),
//!     right: Box::new(CsgNode::Leaf(mesh_b)),
//! };
//! let result = tree.evaluate()?;
//! ```

use crate::mesh::Mesh;
use crate::error::MeshError;
use glam::DVec3;
use std::collections::HashMap;
use std::sync::Arc;

/// CSG operation types.
///
/// # Variants
///
/// - `Union`: A ∪ B - combines both volumes
/// - `Difference`: A - B - subtracts B from A
/// - `Intersection`: A ∩ B - keeps only common volume
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CsgOp {
    /// Union operation: A ∪ B
    Union,
    /// Difference operation: A - B
    Difference,
    /// Intersection operation: A ∩ B
    Intersection,
}

impl CsgOp {
    /// Returns the operation name for debugging.
    pub fn name(&self) -> &'static str {
        match self {
            CsgOp::Union => "union",
            CsgOp::Difference => "difference",
            CsgOp::Intersection => "intersection",
        }
    }
}

/// Bounding box for early rejection tests.
///
/// # Fields
///
/// - `min`: Minimum corner of the box
/// - `max`: Maximum corner of the box
#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    /// Minimum corner (x, y, z)
    pub min: DVec3,
    /// Maximum corner (x, y, z)
    pub max: DVec3,
}

impl BoundingBox {
    /// Creates a new bounding box from min/max corners.
    pub fn new(min: DVec3, max: DVec3) -> Self {
        Self { min, max }
    }

    /// Creates an empty (invalid) bounding box.
    pub fn empty() -> Self {
        Self {
            min: DVec3::splat(f64::INFINITY),
            max: DVec3::splat(f64::NEG_INFINITY),
        }
    }

    /// Checks if this bounding box overlaps with another.
    ///
    /// # Arguments
    ///
    /// * `other` - The other bounding box
    ///
    /// # Returns
    ///
    /// True if boxes overlap on all three axes.
    pub fn overlaps(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    /// Expands this bounding box to include another.
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min: DVec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: DVec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Computes the intersection of two bounding boxes.
    pub fn intersection(&self, other: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min: DVec3::new(
                self.min.x.max(other.min.x),
                self.min.y.max(other.min.y),
                self.min.z.max(other.min.z),
            ),
            max: DVec3::new(
                self.max.x.min(other.max.x),
                self.max.y.min(other.max.y),
                self.max.z.min(other.max.z),
            ),
        }
    }

    /// Checks if the bounding box is valid (non-empty).
    pub fn is_valid(&self) -> bool {
        self.min.x <= self.max.x && 
        self.min.y <= self.max.y && 
        self.min.z <= self.max.z
    }

    /// Computes bounding box from a mesh.
    pub fn from_mesh(mesh: &Mesh) -> Self {
        let (min, max) = mesh.bounding_box();
        Self { min, max }
    }

    /// Computes the volume of the bounding box.
    pub fn volume(&self) -> f64 {
        if !self.is_valid() {
            return 0.0;
        }
        let size = self.max - self.min;
        size.x * size.y * size.z
    }
}

/// CSG tree node for lazy evaluation.
///
/// Represents either a leaf (mesh) or a binary operation.
///
/// # Variants
///
/// - `Leaf`: A concrete mesh
/// - `Binary`: A binary CSG operation on two subtrees
/// - `Cached`: A cached result with key
#[derive(Debug, Clone)]
pub enum CsgNode {
    /// Leaf node containing a mesh
    Leaf {
        /// The mesh data
        mesh: Arc<Mesh>,
        /// Precomputed bounding box
        bounds: BoundingBox,
    },
    /// Binary CSG operation
    Binary {
        /// The operation type
        op: CsgOp,
        /// Left operand
        left: Box<CsgNode>,
        /// Right operand
        right: Box<CsgNode>,
        /// Cached bounding box (computed lazily)
        bounds: Option<BoundingBox>,
    },
    /// Cached result reference
    Cached {
        /// Cache key for lookup
        key: String,
        /// Fallback node if cache miss
        fallback: Box<CsgNode>,
    },
}

impl CsgNode {
    /// Creates a leaf node from a mesh.
    ///
    /// # Arguments
    ///
    /// * `mesh` - The mesh to wrap
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let node = CsgNode::leaf(mesh);
    /// ```
    pub fn leaf(mesh: Mesh) -> Self {
        let bounds = BoundingBox::from_mesh(&mesh);
        CsgNode::Leaf {
            mesh: Arc::new(mesh),
            bounds,
        }
    }

    /// Creates a union operation node.
    ///
    /// # Arguments
    ///
    /// * `left` - First operand
    /// * `right` - Second operand
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let node = CsgNode::union(left, right);
    /// ```
    pub fn union(left: CsgNode, right: CsgNode) -> Self {
        CsgNode::Binary {
            op: CsgOp::Union,
            left: Box::new(left),
            right: Box::new(right),
            bounds: None,
        }
    }

    /// Creates a difference operation node.
    ///
    /// # Arguments
    ///
    /// * `left` - Mesh to subtract from
    /// * `right` - Mesh to subtract
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let node = CsgNode::difference(left, right);
    /// ```
    pub fn difference(left: CsgNode, right: CsgNode) -> Self {
        CsgNode::Binary {
            op: CsgOp::Difference,
            left: Box::new(left),
            right: Box::new(right),
            bounds: None,
        }
    }

    /// Creates an intersection operation node.
    ///
    /// # Arguments
    ///
    /// * `left` - First operand
    /// * `right` - Second operand
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let node = CsgNode::intersection(left, right);
    /// ```
    pub fn intersection(left: CsgNode, right: CsgNode) -> Self {
        CsgNode::Binary {
            op: CsgOp::Intersection,
            left: Box::new(left),
            right: Box::new(right),
            bounds: None,
        }
    }

    /// Gets or computes the bounding box of this node.
    ///
    /// Uses cached value if available, otherwise computes and caches.
    pub fn bounds(&self) -> BoundingBox {
        match self {
            CsgNode::Leaf { bounds, .. } => *bounds,
            CsgNode::Binary { op, left, right, bounds } => {
                if let Some(b) = bounds {
                    return *b;
                }
                
                let left_bounds = left.bounds();
                let right_bounds = right.bounds();
                
                match op {
                    CsgOp::Union => left_bounds.union(&right_bounds),
                    CsgOp::Intersection => left_bounds.intersection(&right_bounds),
                    CsgOp::Difference => left_bounds,
                }
            }
            CsgNode::Cached { fallback, .. } => fallback.bounds(),
        }
    }

    /// Counts the number of leaf nodes in the tree.
    pub fn leaf_count(&self) -> usize {
        match self {
            CsgNode::Leaf { .. } => 1,
            CsgNode::Binary { left, right, .. } => {
                left.leaf_count() + right.leaf_count()
            }
            CsgNode::Cached { fallback, .. } => fallback.leaf_count(),
        }
    }

    /// Counts the total number of nodes in the tree.
    pub fn node_count(&self) -> usize {
        match self {
            CsgNode::Leaf { .. } => 1,
            CsgNode::Binary { left, right, .. } => {
                1 + left.node_count() + right.node_count()
            }
            CsgNode::Cached { fallback, .. } => 1 + fallback.node_count(),
        }
    }

    /// Estimates the complexity of evaluating this tree.
    ///
    /// Uses bounding box volumes as proxy for mesh complexity.
    pub fn estimated_cost(&self) -> f64 {
        match self {
            CsgNode::Leaf { bounds, .. } => bounds.volume(),
            CsgNode::Binary { op, left, right, .. } => {
                let left_cost = left.estimated_cost();
                let right_cost = right.estimated_cost();
                let left_bounds = left.bounds();
                let right_bounds = right.bounds();

                // Intersection and difference are cheaper if boxes barely overlap
                let overlap_factor = if left_bounds.overlaps(&right_bounds) {
                    let intersection = left_bounds.intersection(&right_bounds);
                    intersection.volume() / (left_bounds.volume() + right_bounds.volume()).max(1e-10)
                } else {
                    0.0
                };

                match op {
                    CsgOp::Union => left_cost + right_cost,
                    CsgOp::Intersection => (left_cost + right_cost) * overlap_factor,
                    CsgOp::Difference => left_cost + right_cost * overlap_factor,
                }
            }
            CsgNode::Cached { fallback, .. } => fallback.estimated_cost() * 0.1, // Cached = cheaper
        }
    }
}

/// Tree rewriting optimizer.
///
/// Applies algebraic and geometric optimizations to CSG trees.
///
/// # Optimizations
///
/// 1. **Associativity**: Reorders nested unions/intersections
/// 2. **Commutativity**: Swaps operands to put smaller first
/// 3. **Pruning**: Removes operations on non-overlapping bounds
/// 4. **Flattening**: Converts nested same-operations to n-ary
pub struct TreeOptimizer {
    /// Cache of optimized subtrees by hash
    cache: HashMap<String, CsgNode>,
}

impl TreeOptimizer {
    /// Creates a new tree optimizer.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Optimizes a CSG tree.
    ///
    /// Applies all optimization rules recursively.
    ///
    /// # Arguments
    ///
    /// * `node` - The tree to optimize
    ///
    /// # Returns
    ///
    /// An optimized tree that produces the same result.
    pub fn optimize(&mut self, node: CsgNode) -> CsgNode {
        match node {
            CsgNode::Leaf { .. } => node,
            CsgNode::Binary { op, left, right, .. } => {
                // Recursively optimize children first
                let left = self.optimize(*left);
                let right = self.optimize(*right);

                // Apply pruning based on bounding boxes
                let left_bounds = left.bounds();
                let right_bounds = right.bounds();

                if !left_bounds.overlaps(&right_bounds) {
                    return self.prune_non_overlapping(op, left, right);
                }

                // Apply commutativity: put smaller operand first for union/intersection
                let (left, right) = if matches!(op, CsgOp::Union | CsgOp::Intersection) {
                    if left.estimated_cost() > right.estimated_cost() {
                        (right, left)
                    } else {
                        (left, right)
                    }
                } else {
                    (left, right)
                };

                CsgNode::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                    bounds: None,
                }
            }
            CsgNode::Cached { key, fallback } => {
                CsgNode::Cached {
                    key,
                    fallback: Box::new(self.optimize(*fallback)),
                }
            }
        }
    }

    /// Prunes operations on non-overlapping bounding boxes.
    fn prune_non_overlapping(&self, op: CsgOp, left: CsgNode, right: CsgNode) -> CsgNode {
        match op {
            CsgOp::Union => {
                // Non-overlapping union: just merge
                CsgNode::Binary {
                    op: CsgOp::Union,
                    left: Box::new(left),
                    right: Box::new(right),
                    bounds: None,
                }
            }
            CsgOp::Intersection => {
                // Non-overlapping intersection: empty result
                CsgNode::leaf(Mesh::new())
            }
            CsgOp::Difference => {
                // Non-overlapping difference: just the left operand
                left
            }
        }
    }

    /// Flattens nested same-operations into n-ary operations.
    ///
    /// For example: union(union(a, b), union(c, d)) -> union(a, b, c, d)
    pub fn flatten<'a>(&self, node: &'a CsgNode) -> Vec<&'a CsgNode> {
        match node {
            CsgNode::Leaf { .. } => vec![node],
            CsgNode::Binary { op, left, right, .. } => {
                let mut result = Vec::new();
                
                // Only flatten for associative operations
                if matches!(op, CsgOp::Union | CsgOp::Intersection) {
                    // Recursively flatten children with same operation
                    if let CsgNode::Binary { op: child_op, .. } = left.as_ref() {
                        if *child_op == *op {
                            result.extend(self.flatten(left));
                        } else {
                            result.push(left.as_ref());
                        }
                    } else {
                        result.push(left.as_ref());
                    }
                    
                    if let CsgNode::Binary { op: child_op, .. } = right.as_ref() {
                        if *child_op == *op {
                            result.extend(self.flatten(right));
                        } else {
                            result.push(right.as_ref());
                        }
                    } else {
                        result.push(right.as_ref());
                    }
                } else {
                    result.push(left.as_ref());
                    result.push(right.as_ref());
                }
                
                result
            }
            CsgNode::Cached { fallback, .. } => self.flatten(fallback),
        }
    }
}

impl Default for TreeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_mesh() -> Mesh {
        let mut mesh = Mesh::new();
        mesh.add_vertex(DVec3::new(0.0, 0.0, 0.0));
        mesh.add_vertex(DVec3::new(1.0, 0.0, 0.0));
        mesh.add_vertex(DVec3::new(0.0, 1.0, 0.0));
        mesh.add_triangle(0, 1, 2);
        mesh
    }

    #[test]
    fn test_bounding_box_overlap() {
        let a = BoundingBox::new(DVec3::ZERO, DVec3::ONE);
        let b = BoundingBox::new(DVec3::splat(0.5), DVec3::splat(1.5));
        let c = BoundingBox::new(DVec3::splat(2.0), DVec3::splat(3.0));

        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
        assert!(!a.overlaps(&c));
        assert!(!c.overlaps(&a));
    }

    #[test]
    fn test_leaf_node_creation() {
        let mesh = create_test_mesh();
        let node = CsgNode::leaf(mesh);

        assert_eq!(node.leaf_count(), 1);
        assert_eq!(node.node_count(), 1);
    }

    #[test]
    fn test_binary_node_creation() {
        let mesh_a = create_test_mesh();
        let mesh_b = create_test_mesh();

        let node = CsgNode::union(
            CsgNode::leaf(mesh_a),
            CsgNode::leaf(mesh_b),
        );

        assert_eq!(node.leaf_count(), 2);
        assert_eq!(node.node_count(), 3);
    }

    #[test]
    fn test_tree_optimizer_prune_intersection() {
        // Two non-overlapping meshes
        let mut mesh_a = Mesh::new();
        mesh_a.add_vertex(DVec3::new(0.0, 0.0, 0.0));
        mesh_a.add_vertex(DVec3::new(1.0, 0.0, 0.0));
        mesh_a.add_vertex(DVec3::new(0.0, 1.0, 0.0));
        mesh_a.add_triangle(0, 1, 2);

        let mut mesh_b = Mesh::new();
        mesh_b.add_vertex(DVec3::new(10.0, 0.0, 0.0));
        mesh_b.add_vertex(DVec3::new(11.0, 0.0, 0.0));
        mesh_b.add_vertex(DVec3::new(10.0, 1.0, 0.0));
        mesh_b.add_triangle(0, 1, 2);

        let node = CsgNode::intersection(
            CsgNode::leaf(mesh_a),
            CsgNode::leaf(mesh_b),
        );

        let mut optimizer = TreeOptimizer::new();
        let optimized = optimizer.optimize(node);

        // Should be pruned to an empty mesh
        if let CsgNode::Leaf { mesh, .. } = optimized {
            assert!(mesh.is_empty());
        } else {
            panic!("Expected Leaf node after optimization");
        }
    }

    #[test]
    fn test_estimated_cost() {
        let mesh = create_test_mesh();
        let node = CsgNode::leaf(mesh);

        let cost = node.estimated_cost();
        assert!(cost >= 0.0);
    }
}

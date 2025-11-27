//! # Boolean Operations
//!
//! CSG (Constructive Solid Geometry) boolean operations with multiple backends.
//!
//! ## Backends
//!
//! - **Manifold**: Fast intersection-based algorithm (default)
//! - **BSP**: Traditional BSP tree algorithm (fallback)
//!
//! ## Features
//!
//! - **Lazy Evaluation**: CSG tree defers computation
//! - **Tree Rewriting**: Optimizes operation order
//! - **Mesh Caching**: Caches repeated module calls
//! - **Spatial Indexing**: O(n log n) triangle queries
//!
//! ## Operations
//!
//! - **Union**: A ∪ B - combines both volumes
//! - **Difference**: A - B - subtracts B from A
//! - **Intersection**: A ∩ B - keeps only common volume
//!
//! ## Browser Safety
//!
//! Pure Rust implementation with no native dependencies.
//! Uses `robust` crate for exact geometric predicates.

mod bsp;
mod plane;
mod polygon;
mod vertex;

// New Manifold-like modules
pub mod cache;
pub mod csg_tree;
pub mod halfedge;
pub mod manifold;

#[cfg(test)]
mod tests;

use crate::error::MeshError;
use crate::mesh::Mesh;
use bsp::BspNode;
use polygon::Polygon;

/// Computes the union of two meshes (A ∪ B).
///
/// Returns a mesh containing all volume from both inputs.
///
/// # Arguments
///
/// * `a` - First mesh
/// * `b` - Second mesh
///
/// # Returns
///
/// A new mesh representing A ∪ B.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::boolean::union;
///
/// let result = union(&mesh_a, &mesh_b)?;
/// ```
pub fn union(a: &Mesh, b: &Mesh) -> Result<Mesh, MeshError> {
    // Handle empty meshes
    if a.is_empty() {
        return Ok(b.clone());
    }
    if b.is_empty() {
        return Ok(a.clone());
    }

    // Check if bounding boxes overlap - if not, just merge
    if !bounding_boxes_overlap(a, b) {
        return Ok(merge_meshes(a, b));
    }

    // Convert meshes to polygons
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    // Build BSP trees
    let mut bsp_a = BspNode::new(polys_a);
    let mut bsp_b = BspNode::new(polys_b);

    // Union algorithm: A.clipTo(B); B.clipTo(A); B.invert(); B.clipTo(A); B.invert()
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();

    // Combine polygons
    let mut result_polys = bsp_a.all_polygons();
    result_polys.extend(bsp_b.all_polygons());

    // Convert back to mesh
    polygons_to_mesh(&result_polys)
}

/// Computes the difference of two meshes (A - B).
///
/// Returns a mesh containing volume from A that is not in B.
///
/// # Arguments
///
/// * `a` - Mesh to subtract from
/// * `b` - Mesh to subtract
///
/// # Returns
///
/// A new mesh representing A - B.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::boolean::difference;
///
/// let result = difference(&mesh_a, &mesh_b)?;
/// ```
pub fn difference(a: &Mesh, b: &Mesh) -> Result<Mesh, MeshError> {
    // Handle empty meshes
    if a.is_empty() {
        return Ok(Mesh::new());
    }
    if b.is_empty() {
        return Ok(a.clone());
    }

    // Check if bounding boxes overlap - if not, return A unchanged
    if !bounding_boxes_overlap(a, b) {
        return Ok(a.clone());
    }

    // Convert meshes to polygons
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    // Build BSP trees
    let mut bsp_a = BspNode::new(polys_a);
    let mut bsp_b = BspNode::new(polys_b);

    // Difference algorithm: A.invert(); A.clipTo(B); B.clipTo(A); B.invert(); B.clipTo(A); B.invert(); A.build(B.allPolygons()); A.invert()
    bsp_a.invert();
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();

    // Combine polygons
    let mut result_polys = bsp_a.all_polygons();
    result_polys.extend(bsp_b.all_polygons());

    // Build final tree and invert back
    let mut result = BspNode::new(result_polys);
    result.invert();

    // Convert back to mesh
    polygons_to_mesh(&result.all_polygons())
}

/// Computes the intersection of two meshes (A ∩ B).
///
/// Returns a mesh containing only the volume common to both inputs.
///
/// # Arguments
///
/// * `a` - First mesh
/// * `b` - Second mesh
///
/// # Returns
///
/// A new mesh representing A ∩ B.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_mesh::ops::boolean::intersection;
///
/// let result = intersection(&mesh_a, &mesh_b)?;
/// ```
pub fn intersection(a: &Mesh, b: &Mesh) -> Result<Mesh, MeshError> {
    // Handle empty meshes
    if a.is_empty() || b.is_empty() {
        return Ok(Mesh::new());
    }

    // Check if bounding boxes overlap - if not, result is empty
    if !bounding_boxes_overlap(a, b) {
        return Ok(Mesh::new());
    }

    // Convert meshes to polygons
    let polys_a = mesh_to_polygons(a);
    let polys_b = mesh_to_polygons(b);

    // Build BSP trees
    let mut bsp_a = BspNode::new(polys_a);
    let mut bsp_b = BspNode::new(polys_b);

    // Intersection algorithm: A.invert(); B.clipTo(A); B.invert(); A.clipTo(B); B.clipTo(A)
    bsp_a.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);

    // Combine polygons
    let mut result_polys = bsp_a.all_polygons();
    result_polys.extend(bsp_b.all_polygons());

    // Build final tree and invert back
    let mut result = BspNode::new(result_polys);
    result.invert();

    // Convert back to mesh
    polygons_to_mesh(&result.all_polygons())
}

/// Checks if two mesh bounding boxes overlap.
fn bounding_boxes_overlap(a: &Mesh, b: &Mesh) -> bool {
    let (min_a, max_a) = a.bounding_box();
    let (min_b, max_b) = b.bounding_box();

    // Check overlap on all three axes
    min_a.x <= max_b.x && max_a.x >= min_b.x &&
    min_a.y <= max_b.y && max_a.y >= min_b.y &&
    min_a.z <= max_b.z && max_a.z >= min_b.z
}

/// Merges two non-overlapping meshes.
fn merge_meshes(a: &Mesh, b: &Mesh) -> Mesh {
    let mut result = a.clone();
    result.merge(b);
    result
}

/// Converts a mesh to a list of polygons.
fn mesh_to_polygons(mesh: &Mesh) -> Vec<Polygon> {
    mesh.triangles()
        .iter()
        .map(|tri| {
            let v0 = vertex::Vertex::new(mesh.vertex(tri[0]));
            let v1 = vertex::Vertex::new(mesh.vertex(tri[1]));
            let v2 = vertex::Vertex::new(mesh.vertex(tri[2]));
            Polygon::new(vec![v0, v1, v2])
        })
        .collect()
}

/// Converts a list of polygons back to a mesh.
fn polygons_to_mesh(polygons: &[Polygon]) -> Result<Mesh, MeshError> {
    let mut mesh = Mesh::new();

    for poly in polygons {
        if poly.vertices.len() < 3 {
            continue;
        }

        // Add vertices
        let base_idx = mesh.vertex_count() as u32;
        for v in &poly.vertices {
            mesh.add_vertex(v.position);
        }

        // Fan triangulation for polygons with more than 3 vertices
        for i in 1..poly.vertices.len() - 1 {
            mesh.add_triangle(base_idx, base_idx + i as u32, base_idx + (i + 1) as u32);
        }
    }

    Ok(mesh)
}

// =============================================================================
// CSG BACKEND SELECTION
// =============================================================================

/// CSG backend selection for boolean operations.
///
/// Choose between Manifold (fast) and BSP (traditional) algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CsgBackend {
    /// Manifold-like intersection-based algorithm (faster for large meshes)
    #[default]
    Manifold,
    /// Traditional BSP tree algorithm (more robust for edge cases)
    Bsp,
}

/// Computes union using the specified backend.
///
/// # Arguments
///
/// * `a` - First mesh
/// * `b` - Second mesh  
/// * `backend` - Which algorithm to use
///
/// # Example
///
/// ```rust,ignore
/// let result = union_with_backend(&mesh_a, &mesh_b, CsgBackend::Manifold)?;
/// ```
pub fn union_with_backend(a: &Mesh, b: &Mesh, backend: CsgBackend) -> Result<Mesh, MeshError> {
    match backend {
        CsgBackend::Manifold => manifold::union(a, b),
        CsgBackend::Bsp => union(a, b),
    }
}

/// Computes difference using the specified backend.
///
/// # Arguments
///
/// * `a` - Mesh to subtract from
/// * `b` - Mesh to subtract
/// * `backend` - Which algorithm to use
///
/// # Example
///
/// ```rust,ignore
/// let result = difference_with_backend(&mesh_a, &mesh_b, CsgBackend::Manifold)?;
/// ```
pub fn difference_with_backend(a: &Mesh, b: &Mesh, backend: CsgBackend) -> Result<Mesh, MeshError> {
    match backend {
        CsgBackend::Manifold => manifold::difference(a, b),
        CsgBackend::Bsp => difference(a, b),
    }
}

/// Computes intersection using the specified backend.
///
/// # Arguments
///
/// * `a` - First mesh
/// * `b` - Second mesh
/// * `backend` - Which algorithm to use
///
/// # Example
///
/// ```rust,ignore
/// let result = intersection_with_backend(&mesh_a, &mesh_b, CsgBackend::Manifold)?;
/// ```
pub fn intersection_with_backend(a: &Mesh, b: &Mesh, backend: CsgBackend) -> Result<Mesh, MeshError> {
    match backend {
        CsgBackend::Manifold => manifold::intersection(a, b),
        CsgBackend::Bsp => intersection(a, b),
    }
}

// =============================================================================
// CSG EVALUATOR WITH OPTIMIZATION
// =============================================================================

use csg_tree::{CsgNode, CsgOp, TreeOptimizer};
use cache::{CacheKey, MeshCache};
use std::sync::Arc;

/// CSG evaluator with tree optimization and caching.
///
/// Evaluates CSG trees with:
/// - Tree rewriting to minimize intermediate mesh sizes
/// - Mesh caching for repeated operations
/// - Automatic backend selection based on mesh complexity
///
/// # Example
///
/// ```rust,ignore
/// let mut evaluator = CsgEvaluator::new();
/// let tree = CsgNode::union(
///     CsgNode::leaf(mesh_a),
///     CsgNode::leaf(mesh_b),
/// );
/// let result = evaluator.evaluate(tree)?;
/// ```
pub struct CsgEvaluator {
    /// Tree optimizer for rewriting
    optimizer: TreeOptimizer,
    /// Mesh cache
    cache: MeshCache,
    /// Backend to use
    backend: CsgBackend,
    /// Triangle threshold for backend selection
    auto_backend_threshold: usize,
}

impl CsgEvaluator {
    /// Creates a new CSG evaluator.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let evaluator = CsgEvaluator::new();
    /// ```
    pub fn new() -> Self {
        Self {
            optimizer: TreeOptimizer::new(),
            cache: MeshCache::new(500),
            backend: CsgBackend::Manifold,
            auto_backend_threshold: 1000,
        }
    }

    /// Creates an evaluator with a specific backend.
    pub fn with_backend(backend: CsgBackend) -> Self {
        Self {
            backend,
            ..Self::new()
        }
    }

    /// Evaluates a CSG tree.
    ///
    /// Applies optimization and caching automatically.
    ///
    /// # Arguments
    ///
    /// * `tree` - The CSG tree to evaluate
    ///
    /// # Returns
    ///
    /// The resulting mesh.
    pub fn evaluate(&mut self, tree: CsgNode) -> Result<Mesh, MeshError> {
        // Optimize the tree first
        let optimized = self.optimizer.optimize(tree);
        
        // Evaluate recursively
        self.eval_node(&optimized)
    }

    /// Recursively evaluates a CSG node.
    fn eval_node(&mut self, node: &CsgNode) -> Result<Mesh, MeshError> {
        match node {
            CsgNode::Leaf { mesh, .. } => Ok(Mesh::clone(mesh.as_ref())),
            
            CsgNode::Binary { op, left, right, .. } => {
                // Evaluate children
                let left_mesh = self.eval_node(left)?;
                let right_mesh = self.eval_node(right)?;

                // Select backend based on complexity
                let total_tris = left_mesh.triangle_count() + right_mesh.triangle_count();
                let backend = if total_tris > self.auto_backend_threshold {
                    CsgBackend::Manifold
                } else {
                    self.backend
                };

                // Perform the operation
                match op {
                    CsgOp::Union => union_with_backend(&left_mesh, &right_mesh, backend),
                    CsgOp::Difference => difference_with_backend(&left_mesh, &right_mesh, backend),
                    CsgOp::Intersection => intersection_with_backend(&left_mesh, &right_mesh, backend),
                }
            }
            
            CsgNode::Cached { key, fallback } => {
                // Try cache first
                if let Some(cached) = self.cache.get(&CacheKey::new(key.clone())) {
                    return Ok((*cached).clone());
                }

                // Evaluate fallback and cache result
                let result = self.eval_node(fallback)?;
                self.cache.put(CacheKey::new(key.clone()), result.clone());
                Ok(result)
            }
        }
    }

    /// Returns cache statistics.
    pub fn cache_stats(&self) -> &cache::CacheStats {
        self.cache.stats()
    }

    /// Clears the cache.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for CsgEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

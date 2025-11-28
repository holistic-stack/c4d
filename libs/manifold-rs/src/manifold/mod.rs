//! # Manifold Module
//!
//! 3D solid operations based on Manifold-3D algorithms.
//!
//! ## Structure
//!
//! - `constructors`: Cube, Sphere, Cylinder, Polyhedron primitives
//! - `boolean`: Union, Difference, Intersection operations
//! - `hull`: Convex hull computation
//! - `minkowski`: Minkowski sum
//!
//! ## Algorithm Reference
//!
//! Based on [Manifold-3D](https://github.com/elalish/manifold):
//! - Guaranteed manifold output (watertight meshes)
//! - Robust boolean operations with exact predicates
//! - Efficient BVH-based collision detection

pub mod constructors;
pub mod boolean;
pub mod hull;
pub mod minkowski;

use crate::mesh::Mesh;

// =============================================================================
// MANIFOLD STRUCT
// =============================================================================

/// 3D solid with CSG operations.
///
/// Represents a watertight (manifold) triangle mesh that can undergo
/// boolean operations while maintaining topological validity.
///
/// ## Example
///
/// ```ignore
/// use manifold_rs::Manifold;
///
/// let cube = Manifold::cube([10.0, 10.0, 10.0], false);
/// let sphere = Manifold::sphere(5.0, 32);
/// let result = cube.subtract(&sphere);
/// let mesh = result.get_mesh();
/// ```
#[derive(Debug, Clone)]
pub struct Manifold {
    /// Internal mesh representation.
    mesh: Mesh,
}

impl Manifold {
    /// Create Manifold from existing mesh.
    ///
    /// ## Parameters
    ///
    /// - `mesh`: Triangle mesh (assumed to be manifold)
    #[must_use]
    pub fn from_mesh(mesh: Mesh) -> Self {
        Self { mesh }
    }

    /// Get the output mesh.
    ///
    /// Returns a copy of the internal mesh for rendering.
    #[must_use]
    pub fn get_mesh(&self) -> Mesh {
        self.mesh.clone()
    }

    /// Get mesh reference.
    #[must_use]
    pub fn mesh(&self) -> &Mesh {
        &self.mesh
    }

    /// Get mutable mesh reference.
    pub fn mesh_mut(&mut self) -> &mut Mesh {
        &mut self.mesh
    }

    /// Check if manifold is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mesh.is_empty()
    }

    /// Get vertex count.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.mesh.vertex_count()
    }

    /// Get triangle count.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.mesh.triangle_count()
    }
}

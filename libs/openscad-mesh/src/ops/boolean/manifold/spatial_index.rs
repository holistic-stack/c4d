//! # Spatial Index for Fast Triangle Queries
//!
//! Uses spatial hashing for O(1) average-case triangle lookups.
//! Essential for efficient CSG operations on large meshes.
//!
//! ## Features
//!
//! - **Spatial hashing**: Grid-based acceleration structure
//! - **Ray queries**: Find triangles intersecting a ray
//! - **Box queries**: Find triangles in an axis-aligned box
//!
//! ## Example
//!
//! ```rust,ignore
//! use openscad_mesh::ops::boolean::manifold::SpatialIndex;
//!
//! let index = SpatialIndex::from_mesh(&mesh);
//! let candidates = index.query_ray(origin, direction);
//! ```

use crate::mesh::Mesh;
use glam::DVec3;
use std::collections::HashMap;

/// Cell size multiplier for spatial hashing.
/// Larger values = fewer cells but more triangles per cell.
const CELL_SIZE_FACTOR: f64 = 2.0;

/// Minimum cell size to avoid too many cells.
const MIN_CELL_SIZE: f64 = 0.1;

/// 3D cell coordinate for spatial hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CellCoord {
    x: i32,
    y: i32,
    z: i32,
}

impl CellCoord {
    /// Creates a cell coordinate from 3D position.
    fn from_position(pos: DVec3, cell_size: f64) -> Self {
        Self {
            x: (pos.x / cell_size).floor() as i32,
            y: (pos.y / cell_size).floor() as i32,
            z: (pos.z / cell_size).floor() as i32,
        }
    }
}

/// Spatial index for fast triangle queries.
///
/// Uses a hash grid to map 3D space to triangle lists.
///
/// # Example
///
/// ```rust,ignore
/// let index = SpatialIndex::from_mesh(&mesh);
/// let candidates = index.query_box(min, max);
/// ```
#[derive(Debug)]
pub struct SpatialIndex {
    /// Hash grid: cell -> triangle indices
    grid: HashMap<CellCoord, Vec<usize>>,
    /// Cell size for hashing
    cell_size: f64,
    /// Mesh bounding box min
    bounds_min: DVec3,
    /// Mesh bounding box max
    bounds_max: DVec3,
    /// Total triangle count
    triangle_count: usize,
}

impl SpatialIndex {
    /// Creates a spatial index from a mesh.
    ///
    /// Automatically determines optimal cell size based on mesh bounds.
    ///
    /// # Arguments
    ///
    /// * `mesh` - The mesh to index
    ///
    /// # Returns
    ///
    /// A new spatial index for the mesh.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let index = SpatialIndex::from_mesh(&mesh);
    /// ```
    pub fn from_mesh(mesh: &Mesh) -> Self {
        let (bounds_min, bounds_max) = mesh.bounding_box();
        let triangles = mesh.triangles();
        let triangle_count = triangles.len();

        // Compute optimal cell size based on mesh extent and triangle count
        let extent = bounds_max - bounds_min;
        let avg_extent = (extent.x + extent.y + extent.z) / 3.0;
        let cell_size = (avg_extent / (triangle_count as f64).cbrt() * CELL_SIZE_FACTOR)
            .max(MIN_CELL_SIZE);

        let mut grid: HashMap<CellCoord, Vec<usize>> = HashMap::new();

        // Index each triangle
        for (i, tri) in triangles.iter().enumerate() {
            let v0 = mesh.vertex(tri[0]);
            let v1 = mesh.vertex(tri[1]);
            let v2 = mesh.vertex(tri[2]);

            // Get bounding box of triangle
            let tri_min = v0.min(v1).min(v2);
            let tri_max = v0.max(v1).max(v2);

            // Insert into all overlapping cells
            let cell_min = CellCoord::from_position(tri_min, cell_size);
            let cell_max = CellCoord::from_position(tri_max, cell_size);

            for cx in cell_min.x..=cell_max.x {
                for cy in cell_min.y..=cell_max.y {
                    for cz in cell_min.z..=cell_max.z {
                        let cell = CellCoord { x: cx, y: cy, z: cz };
                        grid.entry(cell).or_default().push(i);
                    }
                }
            }
        }

        Self {
            grid,
            cell_size,
            bounds_min,
            bounds_max,
            triangle_count,
        }
    }

    /// Queries triangles in an axis-aligned bounding box.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum corner of query box
    /// * `max` - Maximum corner of query box
    ///
    /// # Returns
    ///
    /// Vector of triangle indices that may intersect the box.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let candidates = index.query_box(DVec3::ZERO, DVec3::ONE);
    /// ```
    pub fn query_box(&self, min: DVec3, max: DVec3) -> Vec<usize> {
        let cell_min = CellCoord::from_position(min, self.cell_size);
        let cell_max = CellCoord::from_position(max, self.cell_size);

        let mut result = Vec::new();
        let mut seen = vec![false; self.triangle_count];

        for cx in cell_min.x..=cell_max.x {
            for cy in cell_min.y..=cell_max.y {
                for cz in cell_min.z..=cell_max.z {
                    let cell = CellCoord { x: cx, y: cy, z: cz };
                    if let Some(tris) = self.grid.get(&cell) {
                        for &tri_idx in tris {
                            if !seen[tri_idx] {
                                seen[tri_idx] = true;
                                result.push(tri_idx);
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Queries triangles along a ray.
    ///
    /// Uses DDA-like algorithm to traverse cells along the ray.
    ///
    /// # Arguments
    ///
    /// * `origin` - Ray origin
    /// * `direction` - Ray direction (should be normalized)
    ///
    /// # Returns
    ///
    /// Vector of triangle indices that the ray may intersect.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let candidates = index.query_ray(origin, DVec3::X);
    /// ```
    pub fn query_ray(&self, origin: DVec3, direction: DVec3) -> Vec<usize> {
        // Clamp ray to mesh bounds
        let mut result = Vec::new();
        let mut seen = vec![false; self.triangle_count];

        // Traverse cells along the ray using 3D DDA
        let step = DVec3::new(
            if direction.x >= 0.0 { 1.0 } else { -1.0 },
            if direction.y >= 0.0 { 1.0 } else { -1.0 },
            if direction.z >= 0.0 { 1.0 } else { -1.0 },
        );

        let mut current = CellCoord::from_position(origin, self.cell_size);

        // Compute t_max and t_delta for DDA
        let inv_dir = DVec3::new(
            if direction.x.abs() > 1e-10 { 1.0 / direction.x } else { 1e10 },
            if direction.y.abs() > 1e-10 { 1.0 / direction.y } else { 1e10 },
            if direction.z.abs() > 1e-10 { 1.0 / direction.z } else { 1e10 },
        );

        let t_delta = DVec3::new(
            (self.cell_size * inv_dir.x).abs(),
            (self.cell_size * inv_dir.y).abs(),
            (self.cell_size * inv_dir.z).abs(),
        );

        let cell_boundary = DVec3::new(
            (current.x as f64 + if step.x > 0.0 { 1.0 } else { 0.0 }) * self.cell_size,
            (current.y as f64 + if step.y > 0.0 { 1.0 } else { 0.0 }) * self.cell_size,
            (current.z as f64 + if step.z > 0.0 { 1.0 } else { 0.0 }) * self.cell_size,
        );

        let mut t_max = DVec3::new(
            (cell_boundary.x - origin.x) * inv_dir.x,
            (cell_boundary.y - origin.y) * inv_dir.y,
            (cell_boundary.z - origin.z) * inv_dir.z,
        );

        // Maximum ray distance (based on mesh bounds)
        let max_t = ((self.bounds_max - self.bounds_min).length() / self.cell_size) as i32 + 10;

        for _ in 0..max_t {
            // Add triangles in current cell
            if let Some(tris) = self.grid.get(&current) {
                for &tri_idx in tris {
                    if !seen[tri_idx] {
                        seen[tri_idx] = true;
                        result.push(tri_idx);
                    }
                }
            }

            // Move to next cell
            if t_max.x < t_max.y && t_max.x < t_max.z {
                current.x += step.x as i32;
                t_max.x += t_delta.x;
            } else if t_max.y < t_max.z {
                current.y += step.y as i32;
                t_max.y += t_delta.y;
            } else {
                current.z += step.z as i32;
                t_max.z += t_delta.z;
            }

            // Check if we're outside the mesh bounds (with margin)
            let margin = 2;
            let bounds_min_cell = CellCoord::from_position(self.bounds_min, self.cell_size);
            let bounds_max_cell = CellCoord::from_position(self.bounds_max, self.cell_size);

            if current.x < bounds_min_cell.x - margin || current.x > bounds_max_cell.x + margin ||
               current.y < bounds_min_cell.y - margin || current.y > bounds_max_cell.y + margin ||
               current.z < bounds_min_cell.z - margin || current.z > bounds_max_cell.z + margin
            {
                break;
            }
        }

        result
    }

    /// Returns the number of cells in the index.
    pub fn cell_count(&self) -> usize {
        self.grid.len()
    }

    /// Returns the cell size used for hashing.
    pub fn cell_size(&self) -> f64 {
        self.cell_size
    }

    /// Returns the average triangles per cell.
    pub fn avg_triangles_per_cell(&self) -> f64 {
        if self.grid.is_empty() {
            return 0.0;
        }
        let total: usize = self.grid.values().map(|v| v.len()).sum();
        total as f64 / self.grid.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_mesh() -> Mesh {
        let mut mesh = Mesh::new();
        
        // Create a simple cube-like mesh
        mesh.add_vertex(DVec3::new(0.0, 0.0, 0.0));
        mesh.add_vertex(DVec3::new(1.0, 0.0, 0.0));
        mesh.add_vertex(DVec3::new(1.0, 1.0, 0.0));
        mesh.add_vertex(DVec3::new(0.0, 1.0, 0.0));
        mesh.add_vertex(DVec3::new(0.0, 0.0, 1.0));
        mesh.add_vertex(DVec3::new(1.0, 0.0, 1.0));
        mesh.add_vertex(DVec3::new(1.0, 1.0, 1.0));
        mesh.add_vertex(DVec3::new(0.0, 1.0, 1.0));

        // Bottom face
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        // Top face
        mesh.add_triangle(4, 6, 5);
        mesh.add_triangle(4, 7, 6);

        mesh
    }

    #[test]
    fn test_spatial_index_creation() {
        let mesh = create_test_mesh();
        let index = SpatialIndex::from_mesh(&mesh);

        assert!(index.cell_count() > 0);
        assert!(index.cell_size() > 0.0);
    }

    #[test]
    fn test_query_box() {
        let mesh = create_test_mesh();
        let index = SpatialIndex::from_mesh(&mesh);

        // Query the entire mesh bounds
        let candidates = index.query_box(DVec3::ZERO, DVec3::ONE);
        assert_eq!(candidates.len(), 4); // All 4 triangles
    }

    #[test]
    fn test_query_box_partial() {
        let mesh = create_test_mesh();
        let index = SpatialIndex::from_mesh(&mesh);

        // Query only lower half
        let candidates = index.query_box(
            DVec3::new(-0.1, -0.1, -0.1),
            DVec3::new(1.1, 1.1, 0.5),
        );

        // Should get at least the bottom triangles
        assert!(candidates.len() >= 2);
    }

    #[test]
    fn test_query_ray() {
        let mesh = create_test_mesh();
        let index = SpatialIndex::from_mesh(&mesh);

        // Ray through the mesh
        let candidates = index.query_ray(
            DVec3::new(0.5, 0.5, -1.0),
            DVec3::Z,
        );

        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_query_ray_miss() {
        let mesh = create_test_mesh();
        let index = SpatialIndex::from_mesh(&mesh);

        // Ray that misses the mesh
        let candidates = index.query_ray(
            DVec3::new(100.0, 100.0, -1.0),
            DVec3::Z,
        );

        assert!(candidates.is_empty());
    }

    #[test]
    fn test_avg_triangles_per_cell() {
        let mesh = create_test_mesh();
        let index = SpatialIndex::from_mesh(&mesh);

        let avg = index.avg_triangles_per_cell();
        assert!(avg > 0.0);
    }
}

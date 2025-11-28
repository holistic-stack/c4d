//! # Polygon Operations
//!
//! Polygon data structures and operations for BSP boolean operations.
//!
//! ## Contents
//!
//! - **Data structures**: `BspPolygon`, `Plane`, `PolygonClassification`
//! - **Split operations**: `split_polygon` for BSP tree construction
//! - **Merge operations**: `merge_coplanar_polygons` for optimization
//! - **Conversion**: `mesh_to_polygons`, `polygons_to_mesh`
//!
//! ## Design Principles
//!
//! - **SRP**: Only polygon-related logic, no BSP tree traversal
//! - **DRY**: Reusable polygon operations
//! - **Testable**: Pure functions with clear inputs/outputs

use crate::mesh::Mesh;
use super::geometry::{dot, cross, normalize, compute_triangle_normal, EPSILON};
use std::collections::HashMap;

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/// Splitting plane defined by normal and distance from origin.
///
/// The plane equation is: `dot(normal, point) = w`
///
/// Points with `dot(normal, point) > w` are on the front (positive) side.
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Unit normal vector pointing to front side
    pub normal: [f32; 3],
    /// Signed distance from origin: `w = dot(normal, point_on_plane)`
    pub w: f32,
}

impl Plane {
    /// Create plane from polygon (uses first vertex and normal).
    pub fn from_polygon(poly: &BspPolygon) -> Self {
        Self {
            normal: poly.normal,
            w: dot(&poly.normal, &poly.vertices[0]),
        }
    }
}

/// Polygon in BSP tree with vertices and precomputed normal.
///
/// ## Invariants
///
/// - `vertices.len() >= 3` (valid polygon)
/// - `normal` is unit length and consistent with vertex winding
/// - Vertices are in counter-clockwise order when viewed from front
#[derive(Debug, Clone)]
pub struct BspPolygon {
    /// Polygon vertices in counter-clockwise order
    pub vertices: Vec<[f32; 3]>,
    /// Unit normal vector (precomputed for efficiency)
    pub normal: [f32; 3],
}

impl BspPolygon {
    /// Create polygon from vertices (computes normal automatically).
    #[allow(dead_code)]
    pub fn new(vertices: Vec<[f32; 3]>) -> Self {
        let normal = if vertices.len() >= 3 {
            compute_triangle_normal(&vertices[0], &vertices[1], &vertices[2])
        } else {
            [0.0, 0.0, 1.0]
        };
        Self { vertices, normal }
    }

    /// Create polygon with explicit normal.
    pub fn with_normal(vertices: Vec<[f32; 3]>, normal: [f32; 3]) -> Self {
        Self { vertices, normal }
    }

    /// Compute centroid (average of all vertices).
    pub fn centroid(&self) -> [f32; 3] {
        let n = self.vertices.len() as f32;
        let sum = self.vertices.iter().fold([0.0, 0.0, 0.0], |acc, v| {
            [acc[0] + v[0], acc[1] + v[1], acc[2] + v[2]]
        });
        [sum[0] / n, sum[1] / n, sum[2] / n]
    }

    /// Reverse vertex order and flip normal (for inside-out conversion).
    pub fn flip(&mut self) {
        self.vertices.reverse();
        self.normal = [-self.normal[0], -self.normal[1], -self.normal[2]];
    }
}

/// Classification of polygon relative to splitting plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PolygonClassification {
    /// All vertices on plane (within EPSILON)
    Coplanar,
    /// All vertices on front side (positive)
    Front,
    /// All vertices on back side (negative)
    Back,
    /// Vertices on both sides (needs splitting)
    Spanning,
}

// =============================================================================
// POLYGON SPLITTING
// =============================================================================

/// Classify and optionally split polygon by plane.
///
/// ## Algorithm
///
/// 1. Classify each vertex as front (+1), back (-1), or on-plane (0)
/// 2. If all same classification, return that classification
/// 3. If spanning, compute intersection points and split
///
/// ## Parameters
///
/// - `poly`: Polygon to classify/split
/// - `plane`: Splitting plane
///
/// ## Returns
///
/// Tuple of `(classification, front_part, back_part)`:
/// - For `Coplanar`, `Front`, `Back`: both parts are `None`
/// - For `Spanning`: one or both parts contain the split polygons
pub fn split_polygon(
    poly: &BspPolygon,
    plane: &Plane,
) -> (PolygonClassification, Option<BspPolygon>, Option<BspPolygon>) {
    // Classify each vertex
    let mut front_count = 0;
    let mut back_count = 0;
    let mut types = Vec::with_capacity(poly.vertices.len());
    
    for v in &poly.vertices {
        let dist = dot(&plane.normal, v) - plane.w;
        if dist < -EPSILON {
            types.push(-1);
            back_count += 1;
        } else if dist > EPSILON {
            types.push(1);
            front_count += 1;
        } else {
            types.push(0); // On plane
        }
    }
    
    // Determine classification
    if front_count == 0 && back_count == 0 {
        return (PolygonClassification::Coplanar, None, None);
    }
    if back_count == 0 {
        return (PolygonClassification::Front, None, None);
    }
    if front_count == 0 {
        return (PolygonClassification::Back, None, None);
    }
    
    // Spanning: split the polygon
    let (front_verts, back_verts) = compute_split_vertices(poly, plane, &types);
    
    let front_poly = if front_verts.len() >= 3 {
        Some(BspPolygon::with_normal(front_verts, poly.normal))
    } else {
        None
    };
    
    let back_poly = if back_verts.len() >= 3 {
        Some(BspPolygon::with_normal(back_verts, poly.normal))
    } else {
        None
    };
    
    (PolygonClassification::Spanning, front_poly, back_poly)
}

/// Compute vertices for front and back parts of a spanning polygon.
fn compute_split_vertices(
    poly: &BspPolygon,
    plane: &Plane,
    types: &[i32],
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let mut front_verts = Vec::new();
    let mut back_verts = Vec::new();
    
    for i in 0..poly.vertices.len() {
        let j = (i + 1) % poly.vertices.len();
        let ti = types[i];
        let tj = types[j];
        let vi = poly.vertices[i];
        let vj = poly.vertices[j];
        
        // Add current vertex to appropriate side(s)
        if ti != -1 {
            front_verts.push(vi);
        }
        if ti != 1 {
            back_verts.push(vi);
        }
        
        // If edge crosses plane, compute and add intersection point
        if (ti == -1 && tj == 1) || (ti == 1 && tj == -1) {
            let intersection = compute_plane_intersection(&vi, &vj, plane);
            front_verts.push(intersection);
            back_verts.push(intersection);
        }
    }
    
    (front_verts, back_verts)
}

/// Compute intersection point of edge with plane.
fn compute_plane_intersection(v1: &[f32; 3], v2: &[f32; 3], plane: &Plane) -> [f32; 3] {
    let edge = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
    let denom = dot(&plane.normal, &edge);
    
    // Avoid division by zero (shouldn't happen for spanning edges)
    if denom.abs() < 1e-10 {
        return *v1;
    }
    
    let t = (plane.w - dot(&plane.normal, v1)) / denom;
    
    [
        v1[0] + t * edge[0],
        v1[1] + t * edge[1],
        v1[2] + t * edge[2],
    ]
}

// =============================================================================
// POLYGON MERGING
// =============================================================================

/// Merge coplanar polygons to reduce BSP fragmentation.
///
/// ## Algorithm
///
/// 1. Group polygons by plane (quantized normal + distance)
/// 2. Within each group, repeatedly merge adjacent polygons
/// 3. Stop when no more merges possible
///
/// ## Why This Helps
///
/// BSP splitting creates many small triangles. Merging reduces:
/// - Triangle count (better performance)
/// - Vertex duplication (smaller meshes)
///
/// ## Performance
///
/// O(n²) within each coplanar group, but groups are typically small.
pub fn merge_coplanar_polygons(polygons: Vec<BspPolygon>) -> Vec<BspPolygon> {
    // Group by plane
    let mut groups: HashMap<[i32; 4], Vec<BspPolygon>> = HashMap::new();
    
    for poly in polygons {
        if poly.vertices.len() < 3 {
            continue;
        }
        let key = plane_key(&poly);
        groups.entry(key).or_default().push(poly);
    }
    
    // Merge within each group
    let mut result = Vec::new();
    
    for (_key, group) in groups {
        result.extend(merge_polygon_group(group));
    }
    
    result
}

/// Compute quantized plane key for grouping coplanar polygons.
///
/// Uses integer quantization to handle floating-point imprecision.
fn plane_key(poly: &BspPolygon) -> [i32; 4] {
    let n = normalize(&poly.normal);
    let d = if !poly.vertices.is_empty() {
        -(n[0] * poly.vertices[0][0] + n[1] * poly.vertices[0][1] + n[2] * poly.vertices[0][2])
    } else {
        0.0
    };
    
    // Quantize: 1000 = ~0.001 tolerance (about 0.06° for normals)
    [
        (n[0] * 1000.0).round() as i32,
        (n[1] * 1000.0).round() as i32,
        (n[2] * 1000.0).round() as i32,
        (d * 1000.0).round() as i32,
    ]
}

/// Merge polygons within a single coplanar group.
fn merge_polygon_group(mut group: Vec<BspPolygon>) -> Vec<BspPolygon> {
    let mut merged = true;
    
    while merged {
        merged = false;
        let mut new_group = Vec::new();
        let mut used = vec![false; group.len()];
        
        for i in 0..group.len() {
            if used[i] {
                continue;
            }
            
            let mut current = group[i].clone();
            used[i] = true;
            
            // Try to merge with any other unused polygon
            for j in 0..group.len() {
                if used[j] || i == j {
                    continue;
                }
                
                if let Some(m) = try_merge_polygons(&current, &group[j]) {
                    current = m;
                    used[j] = true;
                    merged = true;
                }
            }
            
            new_group.push(current);
        }
        
        group = new_group;
    }
    
    group
}

/// Try to merge two coplanar polygons that share an edge.
///
/// ## Algorithm
///
/// Finds a shared edge (same vertices in reverse order) and merges
/// the polygons by removing the shared edge and concatenating vertices.
///
/// ## Returns
///
/// `Some(merged)` if polygons share an edge, `None` otherwise
fn try_merge_polygons(p1: &BspPolygon, p2: &BspPolygon) -> Option<BspPolygon> {
    let n1 = p1.vertices.len();
    let n2 = p2.vertices.len();
    
    // Find shared edge
    for i in 0..n1 {
        let a1 = &p1.vertices[i];
        let b1 = &p1.vertices[(i + 1) % n1];
        
        for j in 0..n2 {
            let a2 = &p2.vertices[j];
            let b2 = &p2.vertices[(j + 1) % n2];
            
            // Shared edge: a1==b2 and b1==a2 (reversed)
            if vertices_equal(a1, b2) && vertices_equal(b1, a2) {
                return Some(merge_at_edge(p1, p2, i, j));
            }
        }
    }
    
    None
}

/// Check if two vertices are approximately equal.
const VERTEX_EPSILON: f32 = 1e-4;

fn vertices_equal(a: &[f32; 3], b: &[f32; 3]) -> bool {
    (a[0] - b[0]).abs() < VERTEX_EPSILON &&
    (a[1] - b[1]).abs() < VERTEX_EPSILON &&
    (a[2] - b[2]).abs() < VERTEX_EPSILON
}

/// Merge two polygons at their shared edge.
fn merge_at_edge(p1: &BspPolygon, p2: &BspPolygon, edge1: usize, edge2: usize) -> BspPolygon {
    let n1 = p1.vertices.len();
    let n2 = p2.vertices.len();
    let mut merged = Vec::new();
    
    // Add p1 vertices up to and including shared edge start
    for k in 0..=edge1 {
        merged.push(p1.vertices[k]);
    }
    
    // Add p2 vertices after shared edge (skipping the shared edge)
    for k in 2..n2 {
        merged.push(p2.vertices[(edge2 + k) % n2]);
    }
    
    // Add p1 vertices after shared edge end
    for k in (edge1 + 2)..n1 {
        merged.push(p1.vertices[k]);
    }
    
    // Remove collinear vertices
    let cleaned = remove_collinear_vertices(&merged);
    
    BspPolygon::with_normal(cleaned, p1.normal)
}

/// Remove collinear vertices from polygon boundary.
///
/// Vertices are collinear if the cross product of adjacent edges is ~zero.
fn remove_collinear_vertices(vertices: &[[f32; 3]]) -> Vec<[f32; 3]> {
    if vertices.len() < 3 {
        return vertices.to_vec();
    }
    
    let mut result = Vec::new();
    let n = vertices.len();
    
    for i in 0..n {
        let prev = &vertices[(i + n - 1) % n];
        let curr = &vertices[i];
        let next = &vertices[(i + 1) % n];
        
        // Edges from prev to curr and curr to next
        let v1 = [curr[0] - prev[0], curr[1] - prev[1], curr[2] - prev[2]];
        let v2 = [next[0] - curr[0], next[1] - curr[1], next[2] - curr[2]];
        
        let c = cross(&v1, &v2);
        let cross_len = (c[0]*c[0] + c[1]*c[1] + c[2]*c[2]).sqrt();
        let v1_len = (v1[0]*v1[0] + v1[1]*v1[1] + v1[2]*v1[2]).sqrt();
        let v2_len = (v2[0]*v2[0] + v2[1]*v2[1] + v2[2]*v2[2]).sqrt();
        
        // Keep if not collinear (significant cross product)
        if v1_len > 1e-9 && v2_len > 1e-9 && cross_len / (v1_len * v2_len) > 1e-4 {
            result.push(*curr);
        }
    }
    
    result
}

// =============================================================================
// MESH CONVERSION
// =============================================================================

/// Convert mesh triangles to BSP polygons.
///
/// Also performs initial coplanar merge to reduce BSP tree depth.
pub fn mesh_to_polygons(mesh: &Mesh) -> Vec<BspPolygon> {
    let mut polygons = Vec::new();
    
    for i in (0..mesh.indices.len()).step_by(3) {
        let i0 = mesh.indices[i] as usize * 3;
        let i1 = mesh.indices[i + 1] as usize * 3;
        let i2 = mesh.indices[i + 2] as usize * 3;
        
        let v0 = [mesh.vertices[i0], mesh.vertices[i0 + 1], mesh.vertices[i0 + 2]];
        let v1 = [mesh.vertices[i1], mesh.vertices[i1 + 1], mesh.vertices[i1 + 2]];
        let v2 = [mesh.vertices[i2], mesh.vertices[i2 + 1], mesh.vertices[i2 + 2]];
        
        let normal = compute_triangle_normal(&v0, &v1, &v2);
        
        polygons.push(BspPolygon::with_normal(vec![v0, v1, v2], normal));
    }
    
    // Pre-merge to reduce BSP fragmentation
    merge_coplanar_polygons(polygons)
}

/// Convert BSP polygons back to mesh with vertex welding.
///
/// ## Process
///
/// 1. Merge coplanar polygons
/// 2. Fan-triangulate each polygon
/// 3. Weld identical vertices
pub fn polygons_to_mesh(polygons: &[BspPolygon]) -> Mesh {
    let merged = merge_coplanar_polygons(polygons.to_vec());
    
    let mut mesh = Mesh::new();
    let mut welder = VertexWelder::new();
    
    for poly in &merged {
        if poly.vertices.len() < 3 {
            continue;
        }
        
        // Fan triangulation from first vertex
        let idx0 = welder.add(&mut mesh, poly.vertices[0], poly.normal);
        
        for i in 1..poly.vertices.len() - 1 {
            let idx1 = welder.add(&mut mesh, poly.vertices[i], poly.normal);
            let idx2 = welder.add(&mut mesh, poly.vertices[i + 1], poly.normal);
            mesh.add_triangle(idx0, idx1, idx2);
        }
    }
    
    mesh
}

// =============================================================================
// VERTEX WELDING
// =============================================================================

/// Helper for welding duplicate vertices during mesh construction.
///
/// Uses spatial hashing to efficiently find vertices at the same position.
/// Vertices are welded if position matches AND normals are similar (for smooth shading).
pub struct VertexWelder {
    /// Spatial hash: quantized position -> list of vertex indices
    cache: HashMap<[i32; 3], Vec<u32>>,
}

impl VertexWelder {
    /// Create new vertex welder.
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }
    
    /// Add vertex to mesh, returning index (may reuse existing vertex).
    ///
    /// ## Welding Criteria
    ///
    /// - Position within 1e-4 units
    /// - Normal dot product > 0.9 (within ~25°)
    pub fn add(&mut self, mesh: &mut Mesh, pos: [f32; 3], normal: [f32; 3]) -> u32 {
        // Quantize position for spatial hash
        let key = [
            (pos[0] * 10000.0) as i32,
            (pos[1] * 10000.0) as i32,
            (pos[2] * 10000.0) as i32,
        ];
        
        // Check for existing vertex at this position
        if let Some(indices) = self.cache.get(&key) {
            for &idx in indices {
                let i = idx as usize * 3;
                let v = [mesh.vertices[i], mesh.vertices[i+1], mesh.vertices[i+2]];
                let dist_sq = (v[0]-pos[0]).powi(2) + (v[1]-pos[1]).powi(2) + (v[2]-pos[2]).powi(2);
                
                if dist_sq < 1e-8 {
                    let n = [mesh.normals[i], mesh.normals[i+1], mesh.normals[i+2]];
                    let dot_n = n[0]*normal[0] + n[1]*normal[1] + n[2]*normal[2];
                    
                    // Weld if normals are similar (~25° threshold)
                    if dot_n > 0.9 {
                        return idx;
                    }
                }
            }
        }
        
        // Add new vertex
        let idx = mesh.add_vertex(pos[0], pos[1], pos[2], normal[0], normal[1], normal[2]);
        self.cache.entry(key).or_default().push(idx);
        idx
    }
}

impl Default for VertexWelder {
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
    fn test_split_polygon_front() {
        let poly = BspPolygon::with_normal(
            vec![[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.5, 1.0, 1.0]],
            [0.0, 0.0, 1.0],
        );
        let plane = Plane { normal: [0.0, 0.0, 1.0], w: 0.0 };
        
        let (class, _, _) = split_polygon(&poly, &plane);
        assert_eq!(class, PolygonClassification::Front);
    }

    #[test]
    fn test_split_polygon_back() {
        let poly = BspPolygon::with_normal(
            vec![[0.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.5, 1.0, -1.0]],
            [0.0, 0.0, -1.0],
        );
        let plane = Plane { normal: [0.0, 0.0, 1.0], w: 0.0 };
        
        let (class, _, _) = split_polygon(&poly, &plane);
        assert_eq!(class, PolygonClassification::Back);
    }

    #[test]
    fn test_split_polygon_spanning() {
        let poly = BspPolygon::with_normal(
            vec![[0.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.5, 0.0, 1.0]],
            [0.0, 1.0, 0.0],
        );
        let plane = Plane { normal: [0.0, 0.0, 1.0], w: 0.0 };
        
        let (class, front, back) = split_polygon(&poly, &plane);
        assert_eq!(class, PolygonClassification::Spanning);
        assert!(front.is_some());
        assert!(back.is_some());
    }

    #[test]
    fn test_vertices_equal() {
        assert!(vertices_equal(&[0.0, 0.0, 0.0], &[0.00001, 0.0, 0.0]));
        assert!(!vertices_equal(&[0.0, 0.0, 0.0], &[1.0, 0.0, 0.0]));
    }
}

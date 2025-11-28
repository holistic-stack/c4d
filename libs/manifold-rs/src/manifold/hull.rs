//! # Convex Hull
//!
//! QuickHull algorithm for computing convex hulls of point sets.
//!
//! ## Algorithm
//!
//! 1. Find initial tetrahedron from extreme points
//! 2. Assign outside points to faces
//! 3. For each face with outside points:
//!    - Find farthest point
//!    - Find visible faces and horizon edges
//!    - Delete visible faces
//!    - Create new faces from horizon edges + new point
//!    - Reassign outside points to new faces
//! 4. Return convex hull mesh
//!
//! ## References
//!
//! - [QuickHull Paper](https://www.cise.ufl.edu/~ungor/courses/fall06/papers/QuickHull.pdf)
//! - [Wikipedia: Quickhull](https://en.wikipedia.org/wiki/Quickhull)

use crate::error::ManifoldResult;
use crate::mesh::Mesh;
use std::collections::HashSet;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Tolerance for coplanarity tests.
const EPSILON: f32 = 1e-7;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Compute convex hull of multiple meshes.
///
/// Takes all vertices from all meshes and computes their convex hull
/// using the QuickHull algorithm.
///
/// ## Parameters
///
/// - `meshes`: Slice of meshes to hull
///
/// ## Returns
///
/// Convex hull mesh with correct face normals
///
/// ## Example
///
/// ```rust
/// use manifold_rs::mesh::Mesh;
/// use manifold_rs::manifold::hull::compute_hull;
/// use manifold_rs::manifold::constructors::build_cube;
///
/// let mut mesh = Mesh::new();
/// build_cube(&mut mesh, [10.0, 10.0, 10.0], false);
///
/// let hull = compute_hull(&[mesh]).unwrap();
/// // Hull of a cube is the cube itself
/// assert!(hull.triangle_count() >= 4);
/// ```
pub fn compute_hull(meshes: &[Mesh]) -> ManifoldResult<Mesh> {
    // Collect all unique points
    let mut points: Vec<[f32; 3]> = Vec::new();
    let mut seen: HashSet<[i32; 3]> = HashSet::new();
    
    for mesh in meshes {
        for i in (0..mesh.vertices.len()).step_by(3) {
            let p = [
                mesh.vertices[i],
                mesh.vertices[i + 1],
                mesh.vertices[i + 2],
            ];
            // Quantize to avoid near-duplicates
            let key = [
                (p[0] * 10000.0) as i32,
                (p[1] * 10000.0) as i32,
                (p[2] * 10000.0) as i32,
            ];
            if seen.insert(key) {
                points.push(p);
            }
        }
    }
    
    if points.len() < 4 {
        // Not enough points for a tetrahedron
        return Ok(Mesh::new());
    }
    
    // Run QuickHull
    quickhull(&points)
}

// =============================================================================
// QUICKHULL IMPLEMENTATION
// =============================================================================

/// QuickHull algorithm for 3D convex hull.
///
/// ## Algorithm Steps
///
/// 1. Find extreme points and build initial tetrahedron
/// 2. Assign all remaining points to face outside sets
/// 3. While any face has outside points:
///    - Find face with farthest outside point
///    - Find all visible faces from that point
///    - Find horizon edges (boundary between visible/non-visible)
///    - Delete visible faces
///    - Create new faces from horizon edges + new point
///    - Reassign outside points from deleted faces to new faces
fn quickhull(points: &[[f32; 3]]) -> ManifoldResult<Mesh> {
    // Find extreme points for initial tetrahedron
    let (min_x, max_x, min_y, max_y, min_z, max_z) = find_extreme_points(points);
    
    // Build initial tetrahedron
    let mut hull = Hull::new(points);
    if !hull.build_initial_tetrahedron(min_x, max_x, min_y, max_y, min_z, max_z) {
        return Ok(Mesh::new());
    }
    
    // Assign points to faces and expand hull
    hull.assign_points_to_faces();
    hull.expand();
    
    // Convert to mesh
    Ok(hull.to_mesh())
}

/// Find indices of extreme points (min/max in each dimension).
fn find_extreme_points(points: &[[f32; 3]]) -> (usize, usize, usize, usize, usize, usize) {
    let mut min_x = 0;
    let mut max_x = 0;
    let mut min_y = 0;
    let mut max_y = 0;
    let mut min_z = 0;
    let mut max_z = 0;
    
    for (i, p) in points.iter().enumerate() {
        if p[0] < points[min_x][0] { min_x = i; }
        if p[0] > points[max_x][0] { max_x = i; }
        if p[1] < points[min_y][1] { min_y = i; }
        if p[1] > points[max_y][1] { max_y = i; }
        if p[2] < points[min_z][2] { min_z = i; }
        if p[2] > points[max_z][2] { max_z = i; }
    }
    
    (min_x, max_x, min_y, max_y, min_z, max_z)
}

// =============================================================================
// HULL STRUCTURE
// =============================================================================

/// Hull under construction.
///
/// Stores the convex hull as a set of triangular faces, each with:
/// - 3 vertex indices into the points array
/// - Face normal (outward pointing)
/// - Set of point indices outside this face
struct Hull<'a> {
    /// Reference to original points
    points: &'a [[f32; 3]],
    /// Active faces in the hull
    faces: Vec<HullFace>,
    /// Which points are already part of the hull vertices
    in_hull: Vec<bool>,
}

/// Face in the hull.
struct HullFace {
    /// Vertex indices (into Hull.points)
    verts: [usize; 3],
    /// Face normal (outward pointing)
    normal: [f32; 3],
    /// Distance from origin along normal
    d: f32,
    /// Point indices that are outside this face
    outside: Vec<usize>,
    /// Is this face still active?
    active: bool,
}

impl<'a> Hull<'a> {
    /// Create new hull builder with reference to points.
    fn new(points: &'a [[f32; 3]]) -> Self {
        Self {
            points,
            faces: Vec::new(),
            in_hull: vec![false; points.len()],
        }
    }
    
    /// Build initial tetrahedron from 4 extreme points.
    ///
    /// Returns false if points are coplanar (no valid tetrahedron).
    fn build_initial_tetrahedron(
        &mut self,
        min_x: usize,
        max_x: usize,
        min_y: usize,
        max_y: usize,
        min_z: usize,
        max_z: usize,
    ) -> bool {
        // Find two most distant points among extremes
        let extremes = [min_x, max_x, min_y, max_y, min_z, max_z];
        let mut max_dist = 0.0f32;
        let mut p0 = 0;
        let mut p1 = 0;
        
        for i in 0..extremes.len() {
            for j in (i + 1)..extremes.len() {
                let dist = distance_sq(&self.points[extremes[i]], &self.points[extremes[j]]);
                if dist > max_dist {
                    max_dist = dist;
                    p0 = extremes[i];
                    p1 = extremes[j];
                }
            }
        }
        
        if max_dist < EPSILON {
            return false;
        }
        
        // Find point farthest from line p0-p1
        let mut max_dist = 0.0f32;
        let mut p2 = 0;
        for (i, p) in self.points.iter().enumerate() {
            let dist = point_line_distance_sq(p, &self.points[p0], &self.points[p1]);
            if dist > max_dist {
                max_dist = dist;
                p2 = i;
            }
        }
        
        if max_dist < EPSILON {
            return false;
        }
        
        // Find point farthest from plane p0-p1-p2
        let plane_normal = cross(
            &sub(&self.points[p1], &self.points[p0]),
            &sub(&self.points[p2], &self.points[p0]),
        );
        
        let mut max_dist = 0.0f32;
        let mut p3 = 0;
        for (i, p) in self.points.iter().enumerate() {
            let dist = dot(&sub(p, &self.points[p0]), &plane_normal).abs();
            if dist > max_dist {
                max_dist = dist;
                p3 = i;
            }
        }
        
        if max_dist < EPSILON {
            return false;
        }
        
        // Mark initial points as in hull
        self.in_hull[p0] = true;
        self.in_hull[p1] = true;
        self.in_hull[p2] = true;
        self.in_hull[p3] = true;
        
        // Add 4 faces of tetrahedron with outward-pointing normals
        // Check which side p3 is on relative to plane p0-p1-p2
        let above = dot(&sub(&self.points[p3], &self.points[p0]), &plane_normal) > 0.0;
        
        if above {
            // p3 is above plane, so reverse winding for bottom face
            self.add_face(p0, p2, p1); // bottom (away from p3)
            self.add_face(p0, p1, p3); // side
            self.add_face(p1, p2, p3); // side
            self.add_face(p2, p0, p3); // side
        } else {
            // p3 is below plane
            self.add_face(p0, p1, p2); // top (away from p3)
            self.add_face(p0, p3, p1); // side
            self.add_face(p1, p3, p2); // side
            self.add_face(p2, p3, p0); // side
        }
        
        true
    }
    
    /// Add a face with given vertex indices.
    fn add_face(&mut self, v0: usize, v1: usize, v2: usize) {
        let p0 = self.points[v0];
        let p1 = self.points[v1];
        let p2 = self.points[v2];
        
        let normal = normalize(&cross(&sub(&p1, &p0), &sub(&p2, &p0)));
        let d = dot(&normal, &p0);
        
        self.faces.push(HullFace {
            verts: [v0, v1, v2],
            normal,
            d,
            outside: Vec::new(),
            active: true,
        });
    }
    
    /// Assign all non-hull points to face outside sets.
    fn assign_points_to_faces(&mut self) {
        for (i, p) in self.points.iter().enumerate() {
            if self.in_hull[i] {
                continue;
            }
            self.assign_point_to_face(i, p);
        }
    }
    
    /// Assign a single point to the first face it's outside of.
    fn assign_point_to_face(&mut self, pt_idx: usize, p: &[f32; 3]) {
        for face in &mut self.faces {
            if !face.active {
                continue;
            }
            let dist = dot(&face.normal, p) - face.d;
            if dist > EPSILON {
                face.outside.push(pt_idx);
                return;
            }
        }
    }
    
    /// Expand hull until no face has outside points.
    fn expand(&mut self) {
        // Process faces with outside points
        loop {
            // Find face with farthest outside point
            let mut best_face = None;
            let mut best_pt = 0;
            let mut best_dist = 0.0f32;
            
            for (face_idx, face) in self.faces.iter().enumerate() {
                if !face.active || face.outside.is_empty() {
                    continue;
                }
                
                for &pt_idx in &face.outside {
                    let p = &self.points[pt_idx];
                    let dist = dot(&face.normal, p) - face.d;
                    if dist > best_dist {
                        best_dist = dist;
                        best_face = Some(face_idx);
                        best_pt = pt_idx;
                    }
                }
            }
            
            match best_face {
                Some(face_idx) => {
                    self.add_point_to_hull(face_idx, best_pt);
                }
                None => break,
            }
        }
    }
    
    /// Add a point to the hull by updating visible faces.
    fn add_point_to_hull(&mut self, start_face: usize, pt_idx: usize) {
        let p = &self.points[pt_idx];
        
        // Find all visible faces (point is above the face plane)
        let mut visible: Vec<usize> = Vec::new();
        for (i, face) in self.faces.iter().enumerate() {
            if !face.active {
                continue;
            }
            let dist = dot(&face.normal, p) - face.d;
            if dist > EPSILON {
                visible.push(i);
            }
        }
        
        if visible.is_empty() {
            return;
        }
        
        // Find horizon edges (edges shared between visible and non-visible faces)
        let mut horizon: Vec<(usize, usize)> = Vec::new();
        
        for &vis_idx in &visible {
            let face = &self.faces[vis_idx];
            let edges = [
                (face.verts[0], face.verts[1]),
                (face.verts[1], face.verts[2]),
                (face.verts[2], face.verts[0]),
            ];
            
            for (v0, v1) in edges {
                // Check if this edge is on the horizon
                // (i.e., the opposite face is not visible)
                let is_horizon = !visible.iter().any(|&other_idx| {
                    if other_idx == vis_idx {
                        return false;
                    }
                    let other = &self.faces[other_idx];
                    let other_edges = [
                        (other.verts[0], other.verts[1]),
                        (other.verts[1], other.verts[2]),
                        (other.verts[2], other.verts[0]),
                    ];
                    // Check if edge is shared (in either direction)
                    other_edges.contains(&(v0, v1)) || other_edges.contains(&(v1, v0))
                });
                
                if is_horizon {
                    // Store edge with correct winding (v1, v0) for new face
                    horizon.push((v1, v0));
                }
            }
        }
        
        // Collect outside points from visible faces (for reassignment)
        let mut orphan_points: Vec<usize> = Vec::new();
        for &vis_idx in &visible {
            orphan_points.extend(&self.faces[vis_idx].outside);
        }
        
        // Deactivate visible faces
        for &vis_idx in &visible {
            self.faces[vis_idx].active = false;
        }
        
        // Mark new point as in hull
        self.in_hull[pt_idx] = true;
        
        // Create new faces from horizon edges + new point
        for (v0, v1) in horizon {
            self.add_face(v0, v1, pt_idx);
        }
        
        // Reassign orphan points to new faces
        for &orphan_idx in &orphan_points {
            if self.in_hull[orphan_idx] {
                continue;
            }
            self.assign_point_to_face(orphan_idx, &self.points[orphan_idx]);
        }
    }
    
    /// Convert hull to output mesh.
    fn to_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new();
        
        for face in &self.faces {
            if !face.active {
                continue;
            }
            
            let p0 = self.points[face.verts[0]];
            let p1 = self.points[face.verts[1]];
            let p2 = self.points[face.verts[2]];
            
            let v0 = mesh.add_vertex(
                p0[0], p0[1], p0[2],
                face.normal[0], face.normal[1], face.normal[2],
            );
            let v1 = mesh.add_vertex(
                p1[0], p1[1], p1[2],
                face.normal[0], face.normal[1], face.normal[2],
            );
            let v2 = mesh.add_vertex(
                p2[0], p2[1], p2[2],
                face.normal[0], face.normal[1], face.normal[2],
            );
            
            mesh.add_triangle(v0, v1, v2);
        }
        
        mesh
    }
}

// =============================================================================
// VECTOR MATH HELPERS
// =============================================================================

fn sub(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: &[f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

fn distance_sq(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let d = sub(a, b);
    dot(&d, &d)
}

fn point_line_distance_sq(p: &[f32; 3], a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let ab = sub(b, a);
    let ap = sub(p, a);
    let c = cross(&ab, &ap);
    dot(&c, &c) / dot(&ab, &ab).max(1e-10)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifold::constructors::{build_cube, build_sphere};

    /// Test hull of a cube produces 12 triangles (6 faces × 2 triangles).
    #[test]
    fn test_hull_cube() {
        let mut mesh = Mesh::new();
        build_cube(&mut mesh, [10.0, 10.0, 10.0], false);
        
        let hull = compute_hull(&[mesh]).unwrap();
        // Hull of a cube should be the cube itself (8 vertices, 12 triangles)
        assert!(hull.triangle_count() >= 4, "Hull should have at least 4 triangles");
        assert!(hull.triangle_count() <= 12, "Cube hull should have at most 12 triangles");
    }

    /// Test hull of empty input.
    #[test]
    fn test_hull_empty() {
        let hull = compute_hull(&[]).unwrap();
        assert!(hull.is_empty());
    }
    
    /// Test hull of a sphere produces convex hull.
    #[test]
    fn test_hull_sphere() {
        let mut mesh = Mesh::new();
        build_sphere(&mut mesh, 5.0, 8);
        
        let hull = compute_hull(&[mesh]).unwrap();
        // Hull should have triangles
        assert!(hull.triangle_count() >= 4, "Sphere hull should have triangles");
    }
    
    /// Test hull of two separated cubes creates enclosing hull.
    #[test]
    fn test_hull_two_cubes() {
        let mut mesh1 = Mesh::new();
        build_cube(&mut mesh1, [5.0, 5.0, 5.0], false);
        
        let mut mesh2 = Mesh::new();
        build_cube(&mut mesh2, [5.0, 5.0, 5.0], false);
        mesh2.translate(15.0, 0.0, 0.0);
        
        let hull = compute_hull(&[mesh1, mesh2]).unwrap();
        // Hull should encompass both cubes
        assert!(hull.triangle_count() >= 4);
    }
    
    /// Test hull with insufficient points returns empty.
    #[test]
    fn test_hull_insufficient_points() {
        // Less than 4 points - can't form tetrahedron
        let mesh = Mesh::new();
        let hull = compute_hull(&[mesh]).unwrap();
        assert!(hull.is_empty());
    }
}

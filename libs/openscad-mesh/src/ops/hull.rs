//! # Convex Hull (QuickHull Algorithm)
//!
//! Computes the 3D convex hull using QuickHull with proper outside set tracking.
//! 
//! ## Algorithm
//! 
//! 1. Find initial tetrahedron from extremal points
//! 2. Assign each remaining point to the outside set of the face it's farthest from
//! 3. For each face with a non-empty outside set:
//!    a. Find the farthest point in that outside set
//!    b. Find all faces visible from that point
//!    c. Extract horizon edges (boundary of visible region)
//!    d. Create new faces from horizon edges to the farthest point
//!    e. Redistribute outside sets from deleted faces to new faces

use crate::Mesh;
use std::collections::{HashMap, HashSet};

/// Epsilon for floating point comparisons.
const EPSILON: f32 = 1e-6;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Compute the 3D convex hull of a set of points.
pub fn convex_hull(points: &[[f32; 3]]) -> Mesh {
    // Deduplicate points
    let unique_points = deduplicate_points(points);
    
    if unique_points.len() < 4 {
        return Mesh::new();
    }

    let mut qh = QuickHull::new(&unique_points);
    if !qh.build() {
        return Mesh::new();
    }
    qh.to_mesh()
}

// =============================================================================
// FACE STRUCTURE
// =============================================================================

/// A face with its outside set.
#[derive(Clone)]
struct Face {
    /// Vertex indices (CCW winding when viewed from outside).
    vertices: [usize; 3],
    /// Outward-pointing normal (not normalized).
    normal: [f32; 3],
    /// D coefficient in plane equation: normal · point = D.
    d: f32,
    /// Points outside this face (indices into point array).
    outside: Vec<usize>,
    /// Is this face active?
    active: bool,
}

impl Face {
    /// Create a new face from 3 vertices with correct outward normal.
    fn new(points: &[[f32; 3]], v0: usize, v1: usize, v2: usize, centroid: &[f32; 3]) -> Self {
        let p0 = points[v0];
        let p1 = points[v1];
        let p2 = points[v2];
        
        let edge1 = sub(&p1, &p0);
        let edge2 = sub(&p2, &p0);
        let mut normal = cross(&edge1, &edge2);
        let mut d = dot(&normal, &p0);
        
        // Check if normal points toward centroid (wrong direction)
        let face_center = [
            (p0[0] + p1[0] + p2[0]) / 3.0,
            (p0[1] + p1[1] + p2[1]) / 3.0,
            (p0[2] + p1[2] + p2[2]) / 3.0,
        ];
        let to_centroid = sub(centroid, &face_center);
        
        let mut vertices = [v0, v1, v2];
        if dot(&normal, &to_centroid) > 0.0 {
            // Normal points toward centroid, flip it
            vertices = [v0, v2, v1];
            normal = [-normal[0], -normal[1], -normal[2]];
            d = -d;
        }
        
        Self {
            vertices,
            normal,
            d,
            outside: Vec::new(),
            active: true,
        }
    }
    
    /// Signed distance from point to face plane.
    /// Positive = point is outside (above the plane).
    fn signed_distance(&self, points: &[[f32; 3]], point_idx: usize) -> f32 {
        let p = points[point_idx];
        dot(&self.normal, &p) - self.d
    }
}

// =============================================================================
// QUICKHULL
// =============================================================================

struct QuickHull<'a> {
    points: &'a [[f32; 3]],
    faces: Vec<Face>,
}

impl<'a> QuickHull<'a> {
    fn new(points: &'a [[f32; 3]]) -> Self {
        Self {
            points,
            faces: Vec::new(),
        }
    }
    
    /// Build the convex hull. Returns false if degenerate.
    fn build(&mut self) -> bool {
        // Find initial tetrahedron
        let initial = match self.find_initial_tetrahedron() {
            Some(t) => t,
            None => return false,
        };
        
        let (p0, p1, p2, p3) = initial;
        
        // Compute centroid
        let centroid = [
            (self.points[p0][0] + self.points[p1][0] + self.points[p2][0] + self.points[p3][0]) / 4.0,
            (self.points[p0][1] + self.points[p1][1] + self.points[p2][1] + self.points[p3][1]) / 4.0,
            (self.points[p0][2] + self.points[p1][2] + self.points[p2][2] + self.points[p3][2]) / 4.0,
        ];
        
        // Create initial 4 faces
        self.faces.push(Face::new(self.points, p0, p1, p2, &centroid));
        self.faces.push(Face::new(self.points, p0, p2, p3, &centroid));
        self.faces.push(Face::new(self.points, p0, p3, p1, &centroid));
        self.faces.push(Face::new(self.points, p1, p3, p2, &centroid));
        
        // Build initial outside sets
        let initial_set: HashSet<usize> = [p0, p1, p2, p3].into_iter().collect();
        
        for point_idx in 0..self.points.len() {
            if initial_set.contains(&point_idx) { continue; }
            self.assign_point_to_face(point_idx);
        }
        
        // Process faces until all outside sets are empty
        self.process_all();
        
        true
    }
    
    /// Assign a point to the face it's farthest from (if outside any face).
    fn assign_point_to_face(&mut self, point_idx: usize) {
        let mut best_face = None;
        let mut best_dist = EPSILON;
        
        for (face_idx, face) in self.faces.iter().enumerate() {
            if !face.active { continue; }
            let dist = face.signed_distance(self.points, point_idx);
            if dist > best_dist {
                best_dist = dist;
                best_face = Some(face_idx);
            }
        }
        
        if let Some(face_idx) = best_face {
            self.faces[face_idx].outside.push(point_idx);
        }
    }
    
    /// Process all faces with outside points.
    fn process_all(&mut self) {
        loop {
            // Find a face with outside points
            let face_with_outside = self.faces.iter()
                .enumerate()
                .find(|(_, f)| f.active && !f.outside.is_empty())
                .map(|(i, _)| i);
            
            match face_with_outside {
                Some(face_idx) => self.process_face(face_idx),
                None => break,
            }
        }
    }
    
    /// Process a face by extending hull to its farthest outside point.
    fn process_face(&mut self, face_idx: usize) {
        // Find farthest point in this face's outside set
        let (farthest_idx, _) = {
            let face = &self.faces[face_idx];
            let mut best_point = face.outside[0];
            let mut best_dist = face.signed_distance(self.points, best_point);
            
            for &point_idx in &face.outside[1..] {
                let dist = face.signed_distance(self.points, point_idx);
                if dist > best_dist {
                    best_dist = dist;
                    best_point = point_idx;
                }
            }
            (best_point, best_dist)
        };
        
        // Find all visible faces using BFS from the starting face
        let mut visible: HashSet<usize> = HashSet::new();
        let mut to_visit: Vec<usize> = vec![face_idx];
        visible.insert(face_idx);
        
        while let Some(current_idx) = to_visit.pop() {
            // Find neighbors (faces sharing an edge)
            for (other_idx, other_face) in self.faces.iter().enumerate() {
                if !other_face.active { continue; }
                if visible.contains(&other_idx) { continue; }
                
                // Check if they share an edge
                if !self.faces_share_edge(current_idx, other_idx) { continue; }
                
                // Check if visible from farthest point
                let dist = other_face.signed_distance(self.points, farthest_idx);
                if dist > EPSILON {
                    visible.insert(other_idx);
                    to_visit.push(other_idx);
                }
            }
        }
        
        // Find horizon edges (edges between visible and non-visible faces)
        let horizon_edges = self.find_horizon_edges(&visible);
        
        // Collect points to reassign from visible faces
        let mut points_to_reassign: Vec<usize> = Vec::new();
        for &vis_idx in &visible {
            for &point_idx in &self.faces[vis_idx].outside {
                if point_idx != farthest_idx {
                    points_to_reassign.push(point_idx);
                }
            }
        }
        
        // Mark visible faces as inactive
        for &vis_idx in &visible {
            self.faces[vis_idx].active = false;
        }
        
        // Create new faces from horizon edges to farthest point
        let centroid = self.compute_centroid();
        for (v0, v1) in horizon_edges {
            // Create face with correct winding
            let new_face = Face::new(self.points, v1, v0, farthest_idx, &centroid);
            self.faces.push(new_face);
        }
        
        // Reassign points to new faces
        for point_idx in points_to_reassign {
            self.assign_point_to_face(point_idx);
        }
    }
    
    /// Check if two faces share an edge.
    fn faces_share_edge(&self, face_a: usize, face_b: usize) -> bool {
        let a = &self.faces[face_a].vertices;
        let b = &self.faces[face_b].vertices;
        
        // Count shared vertices
        let mut shared = 0;
        for &va in a {
            for &vb in b {
                if va == vb { shared += 1; }
            }
        }
        shared >= 2
    }
    
    /// Find horizon edges (edges between visible and non-visible faces).
    fn find_horizon_edges(&self, visible: &HashSet<usize>) -> Vec<(usize, usize)> {
        let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
        
        // Count each edge in visible faces
        for &face_idx in visible {
            let v = self.faces[face_idx].vertices;
            for i in 0..3 {
                let v0 = v[i];
                let v1 = v[(i + 1) % 3];
                let key = if v0 < v1 { (v0, v1) } else { (v1, v0) };
                *edge_count.entry(key).or_insert(0) += 1;
            }
        }
        
        // Horizon edges appear exactly once (boundary of visible region)
        let mut horizon: Vec<(usize, usize)> = Vec::new();
        for &face_idx in visible {
            let v = self.faces[face_idx].vertices;
            for i in 0..3 {
                let v0 = v[i];
                let v1 = v[(i + 1) % 3];
                let key = if v0 < v1 { (v0, v1) } else { (v1, v0) };
                if edge_count[&key] == 1 {
                    // This is a horizon edge - preserve original direction from face
                    horizon.push((v0, v1));
                }
            }
        }
        
        horizon
    }
    
    /// Compute centroid of active hull.
    fn compute_centroid(&self) -> [f32; 3] {
        let mut sum = [0.0, 0.0, 0.0];
        let mut count = 0;
        
        for face in &self.faces {
            if !face.active { continue; }
            for &idx in &face.vertices {
                let p = self.points[idx];
                sum[0] += p[0];
                sum[1] += p[1];
                sum[2] += p[2];
                count += 1;
            }
        }
        
        if count > 0 {
            sum[0] /= count as f32;
            sum[1] /= count as f32;
            sum[2] /= count as f32;
        }
        sum
    }
    
    /// Find initial tetrahedron.
    fn find_initial_tetrahedron(&self) -> Option<(usize, usize, usize, usize)> {
        if self.points.len() < 4 {
            return None;
        }

        // Find extremal points on X axis
        let (mut min_x, mut max_x) = (0, 0);
        for (i, p) in self.points.iter().enumerate() {
            if p[0] < self.points[min_x][0] { min_x = i; }
            if p[0] > self.points[max_x][0] { max_x = i; }
        }

        if (self.points[max_x][0] - self.points[min_x][0]).abs() < EPSILON {
            // Try Y axis
            for (i, p) in self.points.iter().enumerate() {
                if p[1] < self.points[min_x][1] { min_x = i; }
                if p[1] > self.points[max_x][1] { max_x = i; }
            }
        }
        
        if min_x == max_x && self.points.len() >= 4 {
            return Some((0, 1, 2, 3));
        }

        // Find point farthest from line
        let p0 = self.points[min_x];
        let p1 = self.points[max_x];
        let mut max_dist = 0.0f32;
        let mut p2_idx = min_x;
        
        for (i, p) in self.points.iter().enumerate() {
            if i == min_x || i == max_x { continue; }
            let dist = point_to_line_distance(p, &p0, &p1);
            if dist > max_dist {
                max_dist = dist;
                p2_idx = i;
            }
        }
        
        if max_dist < EPSILON {
            return None;
        }

        // Find point farthest from plane
        let p2 = self.points[p2_idx];
        let normal = cross(&sub(&p1, &p0), &sub(&p2, &p0));
        let mut max_dist = 0.0f32;
        let mut p3_idx = min_x;
        
        for (i, p) in self.points.iter().enumerate() {
            if i == min_x || i == max_x || i == p2_idx { continue; }
            let dist = dot(&sub(p, &p0), &normal).abs();
            if dist > max_dist {
                max_dist = dist;
                p3_idx = i;
            }
        }
        
        if max_dist < EPSILON {
            return None;
        }

        Some((min_x, max_x, p2_idx, p3_idx))
    }
    
    /// Convert to mesh.
    fn to_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new();
        
        for face in &self.faces {
            if !face.active { continue; }
            
            let p0 = self.points[face.vertices[0]];
            let p1 = self.points[face.vertices[1]];
            let p2 = self.points[face.vertices[2]];
            
            // Compute normalized normal
            let len = length(&face.normal);
            let n = if len > EPSILON {
                [face.normal[0] / len, face.normal[1] / len, face.normal[2] / len]
            } else {
                [0.0, 0.0, 1.0]
            };
            
            let v0 = mesh.add_vertex(p0[0], p0[1], p0[2], n[0], n[1], n[2]);
            let v1 = mesh.add_vertex(p1[0], p1[1], p1[2], n[0], n[1], n[2]);
            let v2 = mesh.add_vertex(p2[0], p2[1], p2[2], n[0], n[1], n[2]);
            mesh.add_triangle(v0, v1, v2);
        }
        
        mesh
    }
}

// =============================================================================
// HELPERS
// =============================================================================

fn deduplicate_points(points: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let tolerance = 1e-6;
    let inv_tol = 1.0 / tolerance;
    let mut seen: HashMap<(i64, i64, i64), usize> = HashMap::new();
    let mut result = Vec::new();
    
    for point in points {
        let key = (
            (point[0] * inv_tol).round() as i64,
            (point[1] * inv_tol).round() as i64,
            (point[2] * inv_tol).round() as i64,
        );
        
        if !seen.contains_key(&key) {
            seen.insert(key, result.len());
            result.push(*point);
        }
    }
    
    result
}

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

fn length(v: &[f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn point_to_line_distance(point: &[f32; 3], line_start: &[f32; 3], line_end: &[f32; 3]) -> f32 {
    let line_dir = sub(line_end, line_start);
    let point_vec = sub(point, line_start);
    let cross_vec = cross(&line_dir, &point_vec);
    let cross_len = length(&cross_vec);
    let line_len = length(&line_dir);
    if line_len < EPSILON { 0.0 } else { cross_len / line_len }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hull_tetrahedron() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];
        let mesh = convex_hull(&points);
        assert_eq!(mesh.triangle_count(), 4);
    }

    #[test]
    fn test_hull_cube() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let mesh = convex_hull(&points);
        // Cube should have 12 triangles (6 faces * 2)
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_hull_with_interior_points() {
        let mut points = vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [0.0, 10.0, 0.0],
            [0.0, 0.0, 10.0],
        ];
        points.push([2.0, 2.0, 2.0]);
        points.push([3.0, 3.0, 3.0]);
        
        let mesh = convex_hull(&points);
        assert_eq!(mesh.triangle_count(), 4);
    }

    #[test]
    fn test_hull_empty() {
        let points: Vec<[f32; 3]> = vec![];
        let mesh = convex_hull(&points);
        assert!(mesh.is_empty());
    }

    #[test]
    fn test_hull_insufficient_points() {
        let points = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let mesh = convex_hull(&points);
        assert!(mesh.is_empty());
    }
    
    #[test]
    fn test_hull_two_spheres() {
        use std::f32::consts::PI;
        
        let mut points = Vec::new();
        let segments = 16;
        
        // Sphere 1 at origin
        for j in 0..=segments/2 {
            let phi = PI * j as f32 / (segments/2) as f32;
            for i in 0..segments {
                let theta = 2.0 * PI * i as f32 / segments as f32;
                points.push([
                    5.0 * phi.sin() * theta.cos(),
                    5.0 * phi.sin() * theta.sin(),
                    5.0 * phi.cos(),
                ]);
            }
        }
        
        // Sphere 2 at (20, 0, 0)
        for j in 0..=segments/2 {
            let phi = PI * j as f32 / (segments/2) as f32;
            for i in 0..segments {
                let theta = 2.0 * PI * i as f32 / segments as f32;
                points.push([
                    20.0 + 5.0 * phi.sin() * theta.cos(),
                    5.0 * phi.sin() * theta.sin(),
                    5.0 * phi.cos(),
                ]);
            }
        }
        
        let mesh = convex_hull(&points);
        
        println!("Two spheres: {} input points, {} output triangles", 
                 points.len(), mesh.triangle_count());
        
        // Should produce a pill shape with reasonable number of triangles
        // Max triangles for n points is 2n-4, so for ~288 points, max ~572
        assert!(mesh.triangle_count() > 50, "Too few triangles: {}", mesh.triangle_count());
        assert!(mesh.triangle_count() < 600, "Too many triangles: {}", mesh.triangle_count());
    }
    
    #[test]
    fn test_hull_sphere_mesh_quality() {
        use std::f32::consts::PI;
        
        let mut points = Vec::new();
        let segments = 32;
        let rings = segments / 2;
        
        for j in 0..=rings {
            let phi = PI * j as f32 / rings as f32;
            for i in 0..segments {
                let theta = 2.0 * PI * i as f32 / segments as f32;
                points.push([
                    10.0 * phi.sin() * theta.cos(),
                    10.0 * phi.sin() * theta.sin(),
                    10.0 * phi.cos(),
                ]);
            }
        }
        
        let mesh = convex_hull(&points);
        
        println!("Single sphere: {} input points, {} output triangles",
                 points.len(), mesh.triangle_count());
        
        // For 544 points on a sphere, expect around 1000 triangles (2n-4)
        assert!(mesh.triangle_count() >= 100, "Too few: {}", mesh.triangle_count());
        assert!(mesh.triangle_count() <= 1200, "Too many: {}", mesh.triangle_count());
    }
}

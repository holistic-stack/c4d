//! # QuickHull Algorithm
//!
//! 3D convex hull computation using the QuickHull algorithm.
//! Based on the original algorithm by Barber, Dobkin, and Huhdanpaa.
//!
//! ## Algorithm Steps
//!
//! 1. Find 6 extreme points (min/max on each axis)
//! 2. Build initial tetrahedron from 4 non-coplanar points
//! 3. Assign remaining points to faces they're outside of
//! 4. For each face with outside points:
//!    a. Find farthest point
//!    b. Find horizon edges (boundary of visible faces)
//!    c. Create new faces from horizon to farthest point
//!    d. Reassign outside points to new faces
//! 5. Repeat until no faces have outside points

use crate::error::MeshError;
use crate::mesh::Mesh;
use config::constants::EPSILON;
use glam::DVec3;
use std::collections::HashSet;

/// Computes the convex hull of a set of 3D points.
///
/// # Arguments
///
/// * `points` - Points to compute hull of (at least 4 non-coplanar)
///
/// # Returns
///
/// A mesh representing the convex hull.
///
/// # Example
///
/// ```rust,ignore
/// let points = vec![
///     DVec3::new(0.0, 0.0, 0.0),
///     DVec3::new(1.0, 0.0, 0.0),
///     DVec3::new(0.0, 1.0, 0.0),
///     DVec3::new(0.0, 0.0, 1.0),
/// ];
/// let hull = convex_hull(&points)?;
/// ```
pub fn convex_hull(points: &[DVec3]) -> Result<Mesh, MeshError> {
    if points.len() < 4 {
        return Err(MeshError::degenerate(
            "Convex hull requires at least 4 points",
            None,
        ));
    }

    // Remove duplicate points
    let unique_points = remove_duplicates(points);
    if unique_points.len() < 4 {
        return Err(MeshError::degenerate(
            "Convex hull requires at least 4 unique points",
            None,
        ));
    }

    // Build initial tetrahedron
    let (initial_faces, remaining) = build_initial_simplex(&unique_points)?;

    // Run QuickHull iteration
    let final_faces = quickhull_iterate(initial_faces, remaining, &unique_points)?;

    // Convert faces to mesh
    faces_to_mesh(&final_faces, &unique_points)
}

/// A face of the convex hull (triangle).
#[derive(Debug, Clone)]
struct HullFace {
    /// Indices of the three vertices
    vertices: [usize; 3],
    /// Outward-pointing normal
    normal: DVec3,
    /// Distance from origin along normal
    distance: f64,
    /// Points outside this face (indices into points array)
    outside_points: Vec<usize>,
}

impl HullFace {
    /// Creates a new face from three vertex indices.
    fn new(v0: usize, v1: usize, v2: usize, points: &[DVec3]) -> Self {
        let p0 = points[v0];
        let p1 = points[v1];
        let p2 = points[v2];

        let edge1 = p1 - p0;
        let edge2 = p2 - p0;
        let normal = edge1.cross(edge2).normalize();
        let distance = normal.dot(p0);

        Self {
            vertices: [v0, v1, v2],
            normal,
            distance,
            outside_points: Vec::new(),
        }
    }

    /// Returns the signed distance from a point to this face's plane.
    fn signed_distance(&self, point: DVec3) -> f64 {
        self.normal.dot(point) - self.distance
    }

    /// Returns true if the point is outside (in front of) this face.
    fn is_outside(&self, point: DVec3) -> bool {
        self.signed_distance(point) > EPSILON
    }

    /// Finds the farthest outside point.
    fn farthest_point(&self, points: &[DVec3]) -> Option<usize> {
        self.outside_points
            .iter()
            .max_by(|&&a, &&b| {
                let da = self.signed_distance(points[a]);
                let db = self.signed_distance(points[b]);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }
}

/// Removes duplicate points within EPSILON tolerance.
fn remove_duplicates(points: &[DVec3]) -> Vec<DVec3> {
    let mut unique = Vec::with_capacity(points.len());
    for p in points {
        let is_duplicate = unique.iter().any(|u: &DVec3| (*u - *p).length() < EPSILON);
        if !is_duplicate {
            unique.push(*p);
        }
    }
    unique
}

/// Builds the initial tetrahedron from extreme points.
fn build_initial_simplex(points: &[DVec3]) -> Result<(Vec<HullFace>, Vec<usize>), MeshError> {
    // Find extreme points on each axis
    let mut min_x = 0;
    let mut max_x = 0;
    let mut min_y = 0;
    let mut max_y = 0;
    let mut min_z = 0;
    let mut max_z = 0;

    for (i, p) in points.iter().enumerate() {
        if p.x < points[min_x].x { min_x = i; }
        if p.x > points[max_x].x { max_x = i; }
        if p.y < points[min_y].y { min_y = i; }
        if p.y > points[max_y].y { max_y = i; }
        if p.z < points[min_z].z { min_z = i; }
        if p.z > points[max_z].z { max_z = i; }
    }

    // Find two points with maximum distance
    let extremes = [min_x, max_x, min_y, max_y, min_z, max_z];
    let (p0, p1) = find_farthest_pair(&extremes, points);

    // Find third point farthest from line p0-p1
    let p2 = find_farthest_from_line(p0, p1, points)?;

    // Find fourth point farthest from plane p0-p1-p2
    let p3 = find_farthest_from_plane(p0, p1, p2, points)?;

    // Create initial tetrahedron faces (ensure outward normals)
    let centroid = (points[p0] + points[p1] + points[p2] + points[p3]) / 4.0;
    let mut faces = vec![
        create_face_outward(p0, p1, p2, centroid, points),
        create_face_outward(p0, p2, p3, centroid, points),
        create_face_outward(p0, p3, p1, centroid, points),
        create_face_outward(p1, p3, p2, centroid, points),
    ];

    // Assign remaining points to faces
    let used: HashSet<usize> = [p0, p1, p2, p3].into_iter().collect();
    let remaining: Vec<usize> = (0..points.len()).filter(|i| !used.contains(i)).collect();

    for &idx in &remaining {
        let point = points[idx];
        for face in &mut faces {
            if face.is_outside(point) {
                face.outside_points.push(idx);
                break;
            }
        }
    }

    Ok((faces, remaining))
}

/// Finds the pair of points with maximum distance.
fn find_farthest_pair(indices: &[usize], points: &[DVec3]) -> (usize, usize) {
    let mut max_dist = 0.0;
    let mut best = (indices[0], indices[1]);

    for (i, &a) in indices.iter().enumerate() {
        for &b in indices.iter().skip(i + 1) {
            let dist = (points[a] - points[b]).length_squared();
            if dist > max_dist {
                max_dist = dist;
                best = (a, b);
            }
        }
    }
    best
}

/// Finds the point farthest from a line.
fn find_farthest_from_line(p0: usize, p1: usize, points: &[DVec3]) -> Result<usize, MeshError> {
    let line_dir = (points[p1] - points[p0]).normalize();
    let mut max_dist = 0.0;
    let mut best = None;

    for (i, p) in points.iter().enumerate() {
        if i == p0 || i == p1 {
            continue;
        }
        let v = *p - points[p0];
        let proj = v.dot(line_dir) * line_dir;
        let dist = (v - proj).length();
        if dist > max_dist {
            max_dist = dist;
            best = Some(i);
        }
    }

    best.ok_or_else(|| MeshError::degenerate("All points are collinear", None))
}

/// Finds the point farthest from a plane.
fn find_farthest_from_plane(p0: usize, p1: usize, p2: usize, points: &[DVec3]) -> Result<usize, MeshError> {
    let edge1 = points[p1] - points[p0];
    let edge2 = points[p2] - points[p0];
    let normal = edge1.cross(edge2).normalize();

    let mut max_dist = 0.0;
    let mut best = None;

    for (i, p) in points.iter().enumerate() {
        if i == p0 || i == p1 || i == p2 {
            continue;
        }
        let dist = (normal.dot(*p - points[p0])).abs();
        if dist > max_dist {
            max_dist = dist;
            best = Some(i);
        }
    }

    best.ok_or_else(|| MeshError::degenerate("All points are coplanar", None))
}

/// Creates a face with outward-pointing normal.
fn create_face_outward(v0: usize, v1: usize, v2: usize, centroid: DVec3, points: &[DVec3]) -> HullFace {
    let face = HullFace::new(v0, v1, v2, points);
    let face_center = (points[v0] + points[v1] + points[v2]) / 3.0;
    let to_centroid = centroid - face_center;

    // If normal points toward centroid, flip the face
    if face.normal.dot(to_centroid) > 0.0 {
        HullFace::new(v0, v2, v1, points)
    } else {
        face
    }
}

/// Main QuickHull iteration.
fn quickhull_iterate(
    mut faces: Vec<HullFace>,
    _remaining: Vec<usize>,
    points: &[DVec3],
) -> Result<Vec<HullFace>, MeshError> {
    let max_iterations = points.len() * 2;
    let mut iteration = 0;

    loop {
        iteration += 1;
        if iteration > max_iterations {
            break;
        }

        // Find a face with outside points
        let face_idx = faces.iter().position(|f| !f.outside_points.is_empty());
        let face_idx = match face_idx {
            Some(idx) => idx,
            None => break, // No more outside points
        };

        // Find farthest point from this face
        let farthest = match faces[face_idx].farthest_point(points) {
            Some(p) => p,
            None => continue,
        };

        // Find all faces visible from this point
        let visible: Vec<usize> = faces
            .iter()
            .enumerate()
            .filter(|(_, f)| f.is_outside(points[farthest]))
            .map(|(i, _)| i)
            .collect();

        if visible.is_empty() {
            continue;
        }

        // Find horizon edges (edges of visible faces not shared with other visible faces)
        let horizon = find_horizon_edges(&faces, &visible);

        // Collect outside points from visible faces
        let mut reassign: Vec<usize> = Vec::new();
        for &idx in &visible {
            reassign.extend(&faces[idx].outside_points);
        }
        reassign.retain(|&p| p != farthest);

        // Remove visible faces (in reverse order to preserve indices)
        let mut visible_sorted = visible.clone();
        visible_sorted.sort_by(|a, b| b.cmp(a));
        for idx in visible_sorted {
            faces.swap_remove(idx);
        }

        // Create new faces from horizon edges to farthest point
        let centroid = compute_centroid(&faces, points);
        for (e0, e1) in horizon {
            let new_face = create_face_outward(e0, e1, farthest, centroid, points);
            faces.push(new_face);
        }

        // Reassign outside points to new faces
        for &idx in &reassign {
            let point = points[idx];
            for face in &mut faces {
                if face.is_outside(point) {
                    face.outside_points.push(idx);
                    break;
                }
            }
        }
    }

    Ok(faces)
}

/// Finds horizon edges from visible faces.
fn find_horizon_edges(faces: &[HullFace], visible: &[usize]) -> Vec<(usize, usize)> {
    let _visible_set: HashSet<usize> = visible.iter().copied().collect();
    let mut edge_count: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();

    for &idx in visible {
        let v = faces[idx].vertices;
        let edges = [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])];
        for (a, b) in edges {
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // Horizon edges appear exactly once among visible faces
    let mut horizon = Vec::new();
    for &idx in visible {
        let v = faces[idx].vertices;
        let edges = [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])];
        for (a, b) in edges {
            let key = if a < b { (a, b) } else { (b, a) };
            if edge_count[&key] == 1 {
                // Preserve winding order
                horizon.push((a, b));
            }
        }
    }

    horizon
}

/// Computes centroid of current hull.
fn compute_centroid(faces: &[HullFace], points: &[DVec3]) -> DVec3 {
    let mut sum = DVec3::ZERO;
    let mut count = 0;
    let mut seen: HashSet<usize> = HashSet::new();

    for face in faces {
        for &v in &face.vertices {
            if seen.insert(v) {
                sum += points[v];
                count += 1;
            }
        }
    }

    if count > 0 {
        sum / count as f64
    } else {
        DVec3::ZERO
    }
}

/// Converts hull faces to a mesh.
fn faces_to_mesh(faces: &[HullFace], points: &[DVec3]) -> Result<Mesh, MeshError> {
    // Collect unique vertices used in faces
    let mut used_vertices: HashSet<usize> = HashSet::new();
    for face in faces {
        for &v in &face.vertices {
            used_vertices.insert(v);
        }
    }

    // Create index mapping
    let mut vertex_map: std::collections::HashMap<usize, u32> = std::collections::HashMap::new();
    let mut mesh = Mesh::with_capacity(used_vertices.len(), faces.len());

    for &v in &used_vertices {
        let new_idx = mesh.vertex_count() as u32;
        vertex_map.insert(v, new_idx);
        mesh.add_vertex(points[v]);
    }

    // Add faces
    for face in faces {
        let v0 = vertex_map[&face.vertices[0]];
        let v1 = vertex_map[&face.vertices[1]];
        let v2 = vertex_map[&face.vertices[2]];
        mesh.add_triangle(v0, v1, v2);
    }

    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convex_hull_tetrahedron() {
        let points = vec![
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(1.0, 0.0, 0.0),
            DVec3::new(0.5, 1.0, 0.0),
            DVec3::new(0.5, 0.5, 1.0),
        ];
        let mesh = convex_hull(&points).unwrap();
        
        // Tetrahedron has 4 vertices and 4 faces
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.triangle_count(), 4);
    }

    #[test]
    fn test_convex_hull_cube_vertices() {
        // 8 vertices of a unit cube
        let points = vec![
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(1.0, 0.0, 0.0),
            DVec3::new(1.0, 1.0, 0.0),
            DVec3::new(0.0, 1.0, 0.0),
            DVec3::new(0.0, 0.0, 1.0),
            DVec3::new(1.0, 0.0, 1.0),
            DVec3::new(1.0, 1.0, 1.0),
            DVec3::new(0.0, 1.0, 1.0),
        ];
        let mesh = convex_hull(&points).unwrap();
        
        // Cube has 8 vertices and 12 triangles (6 faces * 2)
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_convex_hull_with_interior_points() {
        // Cube vertices plus interior point
        let mut points = vec![
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(1.0, 0.0, 0.0),
            DVec3::new(1.0, 1.0, 0.0),
            DVec3::new(0.0, 1.0, 0.0),
            DVec3::new(0.0, 0.0, 1.0),
            DVec3::new(1.0, 0.0, 1.0),
            DVec3::new(1.0, 1.0, 1.0),
            DVec3::new(0.0, 1.0, 1.0),
        ];
        // Add interior point
        points.push(DVec3::new(0.5, 0.5, 0.5));
        
        let mesh = convex_hull(&points).unwrap();
        
        // Interior point should not affect hull
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_convex_hull_too_few_points() {
        let points = vec![
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(1.0, 0.0, 0.0),
            DVec3::new(0.5, 1.0, 0.0),
        ];
        let result = convex_hull(&points);
        assert!(result.is_err());
    }
}

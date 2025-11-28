//! # 2D Operation Mesh Builders
//!
//! Builds meshes for 2D operations (offset, projection).
//! Uses pure Rust algorithms for browser safety.

use crate::mesh::Mesh;
use openscad_eval::GeometryNode;
use std::f32::consts::PI;

use super::build_mesh_into;
use super::extrusions::extract_2d_outline;

// =============================================================================
// OFFSET (2D Polygon Offset)
// =============================================================================

/// Build offset 2D shape.
///
/// ## Algorithm
///
/// Simple polygon offset using parallel edge shifting:
/// 1. For each edge, compute the outward normal
/// 2. Shift edge by delta in normal direction
/// 3. Handle corners with miter/chamfer joins
/// 4. Render as flat mesh at Z=0
///
/// ## OpenSCAD offset
///
/// ```text
/// offset(r = 5) circle(10);
/// offset(delta = 2, chamfer = true) square(10);
/// ```
pub fn build_offset_2d(mesh: &mut Mesh, delta: f64, chamfer: bool, child: &GeometryNode) {
    // Extract outline from child
    let outline = extract_2d_outline(child);
    if outline.len() < 3 {
        return;
    }

    let d = delta as f32;
    
    // Compute offset outline
    let offset_outline = offset_polygon(&outline, d, chamfer);
    if offset_outline.len() < 3 {
        return;
    }

    // Render as flat 2D mesh at Z=0
    let n = [0.0f32, 0.0, 1.0];
    
    // Fan triangulation from first vertex
    let first = mesh.add_vertex(offset_outline[0][0], offset_outline[0][1], 0.0, n[0], n[1], n[2]);
    
    for i in 1..offset_outline.len() - 1 {
        let v1 = mesh.add_vertex(offset_outline[i][0], offset_outline[i][1], 0.0, n[0], n[1], n[2]);
        let v2 = mesh.add_vertex(offset_outline[i + 1][0], offset_outline[i + 1][1], 0.0, n[0], n[1], n[2]);
        mesh.add_triangle(first, v1, v2);
    }
}

/// Offset a polygon by the given delta.
///
/// Positive delta expands, negative delta shrinks.
/// Uses miter joins by default, chamfer if specified.
fn offset_polygon(outline: &[[f32; 2]], delta: f32, chamfer: bool) -> Vec<[f32; 2]> {
    let n = outline.len();
    if n < 3 {
        return vec![];
    }

    let mut result = Vec::with_capacity(n * 2); // May grow for chamfers

    for i in 0..n {
        let prev = if i == 0 { n - 1 } else { i - 1 };
        let next = (i + 1) % n;

        // Edge vectors
        let e1 = [
            outline[i][0] - outline[prev][0],
            outline[i][1] - outline[prev][1],
        ];
        let e2 = [
            outline[next][0] - outline[i][0],
            outline[next][1] - outline[i][1],
        ];

        // Normalize and get normals (perpendicular, pointing outward for CCW polygon)
        let len1 = (e1[0] * e1[0] + e1[1] * e1[1]).sqrt();
        let len2 = (e2[0] * e2[0] + e2[1] * e2[1]).sqrt();
        
        if len1 < 1e-10 || len2 < 1e-10 {
            result.push(outline[i]);
            continue;
        }

        // For CCW winding, rotate 90° CCW (left) to get outward normal
        let n1 = [e1[1] / len1, -e1[0] / len1];
        let n2 = [e2[1] / len2, -e2[0] / len2];

        // Average normal for miter join
        let avg_x = n1[0] + n2[0];
        let avg_y = n1[1] + n2[1];
        let avg_len = (avg_x * avg_x + avg_y * avg_y).sqrt();

        if avg_len < 1e-10 {
            // Parallel edges, use first normal
            result.push([
                outline[i][0] + n1[0] * delta,
                outline[i][1] + n1[1] * delta,
            ]);
        } else if chamfer {
            // Chamfer: add two points
            result.push([
                outline[i][0] + n1[0] * delta,
                outline[i][1] + n1[1] * delta,
            ]);
            result.push([
                outline[i][0] + n2[0] * delta,
                outline[i][1] + n2[1] * delta,
            ]);
        } else {
            // Miter: compute intersection point
            let nx = avg_x / avg_len;
            let ny = avg_y / avg_len;
            
            // Miter factor to maintain proper distance
            let dot = n1[0] * nx + n1[1] * ny;
            let miter = if dot.abs() > 0.1 { delta / dot } else { delta };
            
            result.push([
                outline[i][0] + nx * miter,
                outline[i][1] + ny * miter,
            ]);
        }
    }

    result
}

// =============================================================================
// PROJECTION (3D to 2D)
// =============================================================================

/// Build projection of 3D shape to 2D.
///
/// ## Algorithm
///
/// - `cut=false`: Project all vertices onto XY plane, keeping outline
/// - `cut=true`: Only include vertices where Z ≈ 0 (cross-section)
///
/// ## OpenSCAD projection
///
/// ```text
/// projection() sphere(10);
/// projection(cut = true) cube(10);
/// ```
pub fn build_projection(mesh: &mut Mesh, cut: bool, child: &GeometryNode) -> Result<(), crate::error::MeshError> {
    // Build child 3D mesh
    let mut child_mesh = Mesh::new();
    build_mesh_into(&mut child_mesh, child)?;

    if child_mesh.vertices.is_empty() {
        return Ok(());
    }

    // Collect 2D points from 3D mesh
    let points: Vec<[f32; 2]> = if cut {
        // Cut mode: only vertices at Z ≈ 0
        collect_cut_points(&child_mesh)
    } else {
        // Project mode: all vertices projected to XY
        collect_projected_points(&child_mesh)
    };

    if points.len() < 3 {
        return Ok(());
    }

    // Compute convex hull of 2D points for simple visualization
    let hull = compute_2d_convex_hull(&points);
    if hull.len() < 3 {
        return Ok(());
    }

    // Render as flat 2D mesh at Z=0
    let n = [0.0f32, 0.0, 1.0];
    
    // Fan triangulation from first vertex
    let first = mesh.add_vertex(hull[0][0], hull[0][1], 0.0, n[0], n[1], n[2]);
    
    for i in 1..hull.len() - 1 {
        let v1 = mesh.add_vertex(hull[i][0], hull[i][1], 0.0, n[0], n[1], n[2]);
        let v2 = mesh.add_vertex(hull[i + 1][0], hull[i + 1][1], 0.0, n[0], n[1], n[2]);
        mesh.add_triangle(first, v1, v2);
    }

    Ok(())
}

/// Collect vertices at Z ≈ 0 for cut mode.
fn collect_cut_points(mesh: &Mesh) -> Vec<[f32; 2]> {
    let z_tolerance = 0.001;
    let mut points = Vec::new();
    
    for i in (0..mesh.vertices.len()).step_by(3) {
        let z = mesh.vertices[i + 2];
        if z.abs() < z_tolerance {
            points.push([mesh.vertices[i], mesh.vertices[i + 1]]);
        }
    }
    
    points
}

/// Collect all vertices projected to XY plane.
fn collect_projected_points(mesh: &Mesh) -> Vec<[f32; 2]> {
    let mut points = Vec::new();
    
    for i in (0..mesh.vertices.len()).step_by(3) {
        points.push([mesh.vertices[i], mesh.vertices[i + 1]]);
    }
    
    points
}

/// Compute 2D convex hull using gift wrapping algorithm.
fn compute_2d_convex_hull(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    if points.len() < 3 {
        return points.to_vec();
    }

    // Find leftmost point
    let mut start = 0;
    for i in 1..points.len() {
        if points[i][0] < points[start][0] {
            start = i;
        }
    }

    let mut hull = Vec::new();
    let mut current = start;
    
    loop {
        hull.push(points[current]);
        let mut next = 0;
        
        for i in 1..points.len() {
            if next == current {
                next = i;
                continue;
            }
            
            // Check if point i is more counter-clockwise than next
            let cross = cross_2d(
                &points[current],
                &points[next],
                &points[i],
            );
            
            if cross > 0.0 || (cross == 0.0 && distance_sq(&points[current], &points[i]) > distance_sq(&points[current], &points[next])) {
                next = i;
            }
        }
        
        current = next;
        if current == start {
            break;
        }
        
        // Safety limit
        if hull.len() > points.len() {
            break;
        }
    }

    hull
}

/// 2D cross product (z-component of 3D cross).
fn cross_2d(o: &[f32; 2], a: &[f32; 2], b: &[f32; 2]) -> f32 {
    (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
}

/// Squared distance between two 2D points.
fn distance_sq(a: &[f32; 2], b: &[f32; 2]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    dx * dx + dy * dy
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_polygon_expand() {
        let square = vec![
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 10.0],
            [0.0, 10.0],
        ];
        let result = offset_polygon(&square, 1.0, false);
        assert_eq!(result.len(), 4);
        // Should be larger
        assert!(result[0][0] < 0.0 || result[0][1] < 0.0);
    }

    #[test]
    fn test_offset_polygon_shrink() {
        let square = vec![
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 10.0],
            [0.0, 10.0],
        ];
        let result = offset_polygon(&square, -1.0, false);
        assert_eq!(result.len(), 4);
        // Should be smaller
        assert!(result[0][0] > 0.0 && result[0][1] > 0.0);
    }

    #[test]
    fn test_offset_polygon_chamfer() {
        let square = vec![
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 10.0],
            [0.0, 10.0],
        ];
        let result = offset_polygon(&square, 1.0, true);
        // Chamfer adds extra vertices at corners
        assert!(result.len() >= 4);
    }

    #[test]
    fn test_build_offset_2d() {
        let mut mesh = Mesh::new();
        let child = GeometryNode::Circle { radius: 10.0, fn_: 16 };
        build_offset_2d(&mut mesh, 2.0, false, &child);
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_build_projection() {
        let mut mesh = Mesh::new();
        let child = GeometryNode::Cube { size: [10.0, 10.0, 10.0], center: false };
        build_projection(&mut mesh, false, &child).unwrap();
        assert!(mesh.vertex_count() > 0);
    }

    #[test]
    fn test_2d_convex_hull() {
        let points = vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [0.5, 0.5],
            [1.0, 1.0],
            [0.0, 1.0],
        ];
        let hull = compute_2d_convex_hull(&points);
        assert_eq!(hull.len(), 4); // Square hull, interior point excluded
    }
}

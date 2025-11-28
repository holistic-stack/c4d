//! # 2D Primitive Mesh Builders
//!
//! Builds thin 3D meshes from 2D primitives for preview rendering.
//!
//! ## Note
//!
//! These create thin slabs (z=0 to z=0.01) for 2D shape visualization.
//! For actual 3D geometry, use extrusions.

use crate::mesh::Mesh;
use std::f32::consts::PI;

// =============================================================================
// CIRCLE
// =============================================================================

/// Build circle mesh (thin slab for preview).
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// circle(r, $fn=segments);
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh
/// - `radius`: Circle radius
/// - `segments`: Number of segments
pub fn build_circle_mesh(mesh: &mut Mesh, radius: f64, segments: u32) {
    let r = radius as f32;
    let n = segments.max(3) as usize;
    let z = 0.0; // Thin slab at z=0
    
    // Generate vertices around circle
    let mut ring: Vec<u32> = Vec::with_capacity(n);
    
    for i in 0..n {
        let theta = 2.0 * PI * i as f32 / n as f32;
        let x = r * theta.cos();
        let y = r * theta.sin();
        let v = mesh.add_vertex(x, y, z, 0.0, 0.0, 1.0);
        ring.push(v);
    }
    
    // Triangulate (fan from first vertex)
    for i in 1..n - 1 {
        mesh.add_triangle(ring[0], ring[i], ring[i + 1]);
    }
}

// =============================================================================
// SQUARE
// =============================================================================

/// Build square mesh (thin slab for preview).
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// square([width, height], center);
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh
/// - `size`: [width, height]
/// - `center`: If true, center at origin
pub fn build_square_mesh(mesh: &mut Mesh, size: [f64; 2], center: bool) {
    let [w, h] = [size[0] as f32, size[1] as f32];
    let z = 0.0;
    
    let (x0, x1) = if center { (-w / 2.0, w / 2.0) } else { (0.0, w) };
    let (y0, y1) = if center { (-h / 2.0, h / 2.0) } else { (0.0, h) };
    
    let v0 = mesh.add_vertex(x0, y0, z, 0.0, 0.0, 1.0);
    let v1 = mesh.add_vertex(x1, y0, z, 0.0, 0.0, 1.0);
    let v2 = mesh.add_vertex(x1, y1, z, 0.0, 0.0, 1.0);
    let v3 = mesh.add_vertex(x0, y1, z, 0.0, 0.0, 1.0);
    
    mesh.add_triangle(v0, v1, v2);
    mesh.add_triangle(v0, v2, v3);
}

// =============================================================================
// POLYGON
// =============================================================================

/// Build polygon mesh (thin slab for preview).
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// polygon(points, paths);
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh
/// - `points`: Polygon vertices
/// - `paths`: Optional path definitions (for holes)
pub fn build_polygon_mesh(mesh: &mut Mesh, points: &[[f64; 2]], paths: Option<&[Vec<usize>]>) {
    if points.len() < 3 {
        return;
    }
    
    let z = 0.0;
    
    // If no paths, treat all points as single polygon
    let paths = paths.unwrap_or(&[]);
    
    if paths.is_empty() {
        // Simple polygon - fan triangulation
        let pts: Vec<[f32; 2]> = points.iter()
            .map(|p| [p[0] as f32, p[1] as f32])
            .collect();
        
        let first = mesh.add_vertex(pts[0][0], pts[0][1], z, 0.0, 0.0, 1.0);
        
        for i in 1..pts.len() - 1 {
            let v1 = mesh.add_vertex(pts[i][0], pts[i][1], z, 0.0, 0.0, 1.0);
            let v2 = mesh.add_vertex(pts[i + 1][0], pts[i + 1][1], z, 0.0, 0.0, 1.0);
            mesh.add_triangle(first, v1, v2);
        }
    } else {
        // Multiple paths - triangulate each
        for path in paths {
            if path.len() < 3 {
                continue;
            }
            
            let pts: Vec<[f32; 2]> = path.iter()
                .filter_map(|&i| points.get(i).map(|p| [p[0] as f32, p[1] as f32]))
                .collect();
            
            if pts.len() < 3 {
                continue;
            }
            
            let first = mesh.add_vertex(pts[0][0], pts[0][1], z, 0.0, 0.0, 1.0);
            
            for i in 1..pts.len() - 1 {
                let v1 = mesh.add_vertex(pts[i][0], pts[i][1], z, 0.0, 0.0, 1.0);
                let v2 = mesh.add_vertex(pts[i + 1][0], pts[i + 1][1], z, 0.0, 0.0, 1.0);
                mesh.add_triangle(first, v1, v2);
            }
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test circle mesh.
    #[test]
    fn test_build_circle() {
        let mut mesh = Mesh::new();
        build_circle_mesh(&mut mesh, 5.0, 16);
        assert_eq!(mesh.vertex_count(), 16);
        assert_eq!(mesh.triangle_count(), 14); // n-2 triangles for fan
    }

    /// Test square mesh.
    #[test]
    fn test_build_square() {
        let mut mesh = Mesh::new();
        build_square_mesh(&mut mesh, [10.0, 10.0], false);
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.triangle_count(), 2);
    }

    /// Test polygon mesh.
    #[test]
    fn test_build_polygon() {
        let mut mesh = Mesh::new();
        let points = [[0.0, 0.0], [10.0, 0.0], [5.0, 10.0]];
        build_polygon_mesh(&mut mesh, &points, None);
        assert_eq!(mesh.triangle_count(), 1);
    }
}

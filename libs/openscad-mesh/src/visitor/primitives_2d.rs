//! # 2D Primitive Mesh Builders
//!
//! Builds flat meshes at Z=0 for circle, square, and polygon primitives.
//! These are rendered as thin 3D shapes for visualization.

use crate::mesh::Mesh;
use std::f32::consts::PI;

// =============================================================================
// CIRCLE
// =============================================================================

/// Build circle as a flat disk at Z=0.
///
/// ## OpenSCAD Circle
///
/// 2D circle primitive. Rendered as flat triangulated disk for visualization.
///
/// ## Example
///
/// ```text
/// circle(10, $fn=16);
/// ```
pub fn build_circle_2d(mesh: &mut Mesh, radius: f64, fn_: u32) {
    let r = radius as f32;
    let segments = fn_.max(3) as usize;
    
    // Center vertex
    let center = mesh.add_vertex(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    
    // Edge vertices
    let mut edge_verts = Vec::with_capacity(segments);
    for i in 0..segments {
        let theta = 2.0 * PI * i as f32 / segments as f32;
        let x = r * theta.cos();
        let y = r * theta.sin();
        let v = mesh.add_vertex(x, y, 0.0, 0.0, 0.0, 1.0);
        edge_verts.push(v);
    }
    
    // Triangles (fan from center)
    for i in 0..segments {
        let next = (i + 1) % segments;
        mesh.add_triangle(center, edge_verts[i], edge_verts[next]);
    }
}

// =============================================================================
// SQUARE
// =============================================================================

/// Build square as a flat rectangle at Z=0.
///
/// ## OpenSCAD Square
///
/// 2D square/rectangle primitive. Rendered as two triangles for visualization.
///
/// ## Example
///
/// ```text
/// square([20, 15], center=true);
/// ```
pub fn build_square_2d(mesh: &mut Mesh, size: [f64; 2], center: bool) {
    let [sx, sy] = [size[0] as f32, size[1] as f32];
    
    let (min_x, max_x, min_y, max_y) = if center {
        (-sx / 2.0, sx / 2.0, -sy / 2.0, sy / 2.0)
    } else {
        (0.0, sx, 0.0, sy)
    };
    
    // Normal pointing up (+Z)
    let n = [0.0, 0.0, 1.0];
    
    let v0 = mesh.add_vertex(min_x, min_y, 0.0, n[0], n[1], n[2]);
    let v1 = mesh.add_vertex(max_x, min_y, 0.0, n[0], n[1], n[2]);
    let v2 = mesh.add_vertex(max_x, max_y, 0.0, n[0], n[1], n[2]);
    let v3 = mesh.add_vertex(min_x, max_y, 0.0, n[0], n[1], n[2]);
    
    mesh.add_triangle(v0, v1, v2);
    mesh.add_triangle(v0, v2, v3);
}

// =============================================================================
// POLYGON
// =============================================================================

/// Build polygon as flat triangulated shape at Z=0.
///
/// ## OpenSCAD Polygon
///
/// 2D polygon primitive with optional paths. Uses fan triangulation.
/// Note: Fan triangulation works for convex polygons; concave polygons
/// may need ear-clipping for correct results.
///
/// ## Example
///
/// ```text
/// polygon(points=[[0,0], [10,0], [10,10], [0,10]]);
/// ```
pub fn build_polygon_2d(mesh: &mut Mesh, points: &[[f64; 2]], paths: Option<&Vec<Vec<usize>>>) {
    if points.len() < 3 {
        return;
    }
    
    // Normal pointing up (+Z)
    let n = [0.0f32, 0.0, 1.0];
    
    // If no paths provided, use all points as single outline
    let outlines: Vec<Vec<usize>> = match paths {
        Some(p) if !p.is_empty() => p.clone(),
        _ => vec![(0..points.len()).collect()],
    };
    
    for outline in outlines {
        if outline.len() < 3 {
            continue;
        }
        
        // Get vertices for this outline
        let outline_points: Vec<[f32; 2]> = outline.iter()
            .filter_map(|&idx| {
                if idx < points.len() {
                    Some([points[idx][0] as f32, points[idx][1] as f32])
                } else {
                    None
                }
            })
            .collect();
        
        if outline_points.len() < 3 {
            continue;
        }
        
        // Simple fan triangulation (works for convex polygons)
        // TODO: Use proper ear-clipping for concave polygons
        let first = mesh.add_vertex(
            outline_points[0][0], outline_points[0][1], 0.0,
            n[0], n[1], n[2]
        );
        
        for i in 1..outline_points.len() - 1 {
            let v1 = mesh.add_vertex(
                outline_points[i][0], outline_points[i][1], 0.0,
                n[0], n[1], n[2]
            );
            let v2 = mesh.add_vertex(
                outline_points[i + 1][0], outline_points[i + 1][1], 0.0,
                n[0], n[1], n[2]
            );
            mesh.add_triangle(first, v1, v2);
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_circle_2d() {
        let mut mesh = Mesh::new();
        build_circle_2d(&mut mesh, 10.0, 16);
        assert_eq!(mesh.vertex_count(), 17); // center + 16 edge
        assert_eq!(mesh.triangle_count(), 16);
    }

    #[test]
    fn test_build_square_2d() {
        let mut mesh = Mesh::new();
        build_square_2d(&mut mesh, [10.0, 5.0], false);
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.triangle_count(), 2);
    }

    #[test]
    fn test_build_square_2d_centered() {
        let mut mesh = Mesh::new();
        build_square_2d(&mut mesh, [10.0, 10.0], true);
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.triangle_count(), 2);
    }

    #[test]
    fn test_build_polygon_2d() {
        let mut mesh = Mesh::new();
        let points = [
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 10.0],
            [0.0, 10.0],
        ];
        build_polygon_2d(&mut mesh, &points, None);
        assert!(mesh.vertex_count() > 0);
        assert_eq!(mesh.triangle_count(), 2);
    }

    #[test]
    fn test_build_polygon_2d_with_path() {
        let mut mesh = Mesh::new();
        let points = [
            [0.0, 0.0],
            [10.0, 0.0],
            [10.0, 10.0],
            [0.0, 10.0],
        ];
        let paths = vec![vec![0, 1, 2, 3]];
        build_polygon_2d(&mut mesh, &points, Some(&paths));
        assert!(mesh.vertex_count() > 0);
    }
}

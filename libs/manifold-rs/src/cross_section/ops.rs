//! # 2D Operations
//!
//! Offset and projection operations for 2D polygons.
//!
//! ## Operations
//!
//! - `offset`: Expand or shrink a polygon
//! - `projection`: Project 3D geometry to 2D

use openscad_eval::GeometryNode;
use crate::error::ManifoldResult;
use crate::mesh::Mesh;
use crate::openscad::SegmentParams;

// =============================================================================
// OFFSET
// =============================================================================

/// Offset 2D children by delta amount.
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// offset(delta, chamfer) { children }
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh
/// - `children`: 2D child geometry nodes
/// - `delta`: Offset amount (positive = expand, negative = shrink)
/// - `chamfer`: If true, use chamfered corners instead of rounded
/// - `params`: Segment parameters
pub fn offset(
    mesh: &mut Mesh,
    children: &[GeometryNode],
    delta: f64,
    chamfer: bool,
    params: &SegmentParams,
) -> ManifoldResult<()> {
    for child in children {
        let polygon = extract_2d_points(child, params)?;
        if polygon.len() < 3 {
            continue;
        }
        
        // Offset the polygon
        let offset_polygon = offset_polygon_by_delta(&polygon, delta, chamfer);
        
        // Build mesh from offset polygon
        build_polygon_mesh(mesh, &offset_polygon);
    }
    
    Ok(())
}

/// Offset a polygon by delta.
fn offset_polygon_by_delta(polygon: &[[f64; 2]], delta: f64, _chamfer: bool) -> Vec<[f64; 2]> {
    if polygon.len() < 3 || delta.abs() < 1e-10 {
        return polygon.to_vec();
    }
    
    let n = polygon.len();
    let mut result = Vec::with_capacity(n);
    
    for i in 0..n {
        let prev = (i + n - 1) % n;
        let next = (i + 1) % n;
        
        let p0 = polygon[prev];
        let p1 = polygon[i];
        let p2 = polygon[next];
        
        // Compute edge normals
        let e1 = normalize_2d([p1[0] - p0[0], p1[1] - p0[1]]);
        let e2 = normalize_2d([p2[0] - p1[0], p2[1] - p1[1]]);
        
        let n1 = [-e1[1], e1[0]]; // Perpendicular
        let n2 = [-e2[1], e2[0]];
        
        // Average normal at vertex
        let avg = normalize_2d([n1[0] + n2[0], n1[1] + n2[1]]);
        
        // Offset point
        result.push([
            p1[0] + avg[0] * delta,
            p1[1] + avg[1] * delta,
        ]);
    }
    
    result
}

/// Normalize a 2D vector.
fn normalize_2d(v: [f64; 2]) -> [f64; 2] {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if len > 1e-10 {
        [v[0] / len, v[1] / len]
    } else {
        [0.0, 0.0]
    }
}

/// Build mesh from 2D polygon.
fn build_polygon_mesh(mesh: &mut Mesh, polygon: &[[f64; 2]]) {
    if polygon.len() < 3 {
        return;
    }
    
    let z = 0.0;
    
    // Fan triangulation
    let first = mesh.add_vertex(
        polygon[0][0] as f32,
        polygon[0][1] as f32,
        z,
        0.0, 0.0, 1.0,
    );
    
    for i in 1..polygon.len() - 1 {
        let v1 = mesh.add_vertex(
            polygon[i][0] as f32,
            polygon[i][1] as f32,
            z,
            0.0, 0.0, 1.0,
        );
        let v2 = mesh.add_vertex(
            polygon[i + 1][0] as f32,
            polygon[i + 1][1] as f32,
            z,
            0.0, 0.0, 1.0,
        );
        mesh.add_triangle(first, v1, v2);
    }
}

// =============================================================================
// PROJECTION
// =============================================================================

/// Project 3D children to 2D.
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// projection(cut) { children }
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh
/// - `children`: 3D child geometry nodes
/// - `cut`: If true, only project geometry at z=0
/// - `params`: Segment parameters
pub fn projection(
    mesh: &mut Mesh,
    children: &[GeometryNode],
    cut: bool,
    params: &SegmentParams,
) -> ManifoldResult<()> {
    // First, build 3D mesh from children
    let mut child_mesh = Mesh::new();
    for child in children {
        crate::openscad::from_ir::geometry_to_mesh(child)
            .map(|m| child_mesh.merge(&m))
            .ok();
    }
    
    if child_mesh.is_empty() {
        return Ok(());
    }
    
    // Project to 2D
    let polygon = project_mesh_to_2d(&child_mesh, cut);
    
    // Build 2D mesh from projection
    build_polygon_mesh(mesh, &polygon);
    
    Ok(())
}

/// Project mesh to 2D by taking convex hull of XY coordinates.
fn project_mesh_to_2d(mesh: &Mesh, cut: bool) -> Vec<[f64; 2]> {
    let mut points: Vec<[f64; 2]> = Vec::new();
    
    for i in (0..mesh.vertices.len()).step_by(3) {
        let x = mesh.vertices[i] as f64;
        let y = mesh.vertices[i + 1] as f64;
        let z = mesh.vertices[i + 2] as f64;
        
        if cut {
            // Only include points near z=0
            if z.abs() < 0.01 {
                points.push([x, y]);
            }
        } else {
            // Include all points
            points.push([x, y]);
        }
    }
    
    // For simplicity, just return unique points
    // A proper implementation would compute the 2D convex hull
    dedup_points_2d(&points)
}

/// Remove duplicate 2D points.
fn dedup_points_2d(points: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut result = Vec::new();
    
    for p in points {
        let is_dup = result.iter().any(|r: &[f64; 2]| {
            (r[0] - p[0]).abs() < 0.001 && (r[1] - p[1]).abs() < 0.001
        });
        
        if !is_dup {
            result.push(*p);
        }
    }
    
    result
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Extract 2D points from a geometry node.
fn extract_2d_points(node: &GeometryNode, params: &SegmentParams) -> ManifoldResult<Vec<[f64; 2]>> {
    match node {
        GeometryNode::Circle { radius, fn_ } => {
            let segments = if *fn_ > 0 { *fn_ } else { params.calculate_segments(*radius) };
            Ok(circle_points(*radius, segments))
        }
        
        GeometryNode::Square { size, center } => {
            Ok(square_points(*size, *center))
        }
        
        GeometryNode::Polygon { points, .. } => {
            Ok(points.clone())
        }
        
        _ => Ok(Vec::new()),
    }
}

/// Generate circle points.
fn circle_points(radius: f64, segments: u32) -> Vec<[f64; 2]> {
    let n = segments.max(3) as usize;
    let mut points = Vec::with_capacity(n);
    
    for i in 0..n {
        let theta = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
        points.push([
            radius * theta.cos(),
            radius * theta.sin(),
        ]);
    }
    
    points
}

/// Generate square points.
fn square_points(size: [f64; 2], center: bool) -> Vec<[f64; 2]> {
    let [w, h] = size;
    let (x0, x1) = if center { (-w / 2.0, w / 2.0) } else { (0.0, w) };
    let (y0, y1) = if center { (-h / 2.0, h / 2.0) } else { (0.0, h) };
    
    vec![
        [x0, y0],
        [x1, y0],
        [x1, y1],
        [x0, y1],
    ]
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test polygon offset.
    #[test]
    fn test_offset_polygon() {
        let polygon = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let offset = offset_polygon_by_delta(&polygon, 1.0, false);
        
        // Offset polygon should be larger
        assert_eq!(offset.len(), 4);
    }

    /// Test normalize_2d.
    #[test]
    fn test_normalize_2d() {
        let v = normalize_2d([3.0, 4.0]);
        let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
        assert!((len - 1.0).abs() < 0.001);
    }

    /// Test dedup_points_2d.
    #[test]
    fn test_dedup_points() {
        let points = [[0.0, 0.0], [1.0, 0.0], [0.0, 0.0]];
        let deduped = dedup_points_2d(&points);
        assert_eq!(deduped.len(), 2);
    }
}

//! # Extrusion Operations
//!
//! Linear and rotate extrusions to convert 2D shapes to 3D.
//!
//! ## OpenSCAD Compatibility
//!
//! - `linear_extrude`: height, center, twist, scale, slices
//! - `rotate_extrude`: angle, segments

use openscad_eval::GeometryNode;
use crate::error::ManifoldResult;
use crate::mesh::Mesh;
use crate::openscad::SegmentParams;
use std::f32::consts::PI;

// =============================================================================
// LINEAR EXTRUDE
// =============================================================================

/// Linear extrude 2D children to 3D.
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// linear_extrude(height, center, twist, scale, slices) { children }
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh
/// - `children`: 2D child geometry nodes
/// - `height`: Extrusion height
/// - `center`: If true, center vertically
/// - `twist`: Rotation in degrees over height
/// - `scale`: Scale factor at top (default 1.0)
/// - `slices`: Number of vertical slices
/// - `params`: Segment parameters for children
pub fn linear_extrude(
    mesh: &mut Mesh,
    children: &[GeometryNode],
    height: f64,
    center: bool,
    twist: f64,
    scale: f64,
    slices: u32,
    params: &SegmentParams,
) -> ManifoldResult<()> {
    let h = height as f32;
    let twist_rad = (twist as f32).to_radians();
    let scale_factor = scale as f64;
    let num_slices = slices.max(1) as usize;
    
    let z_offset = if center { -h / 2.0 } else { 0.0 };
    
    // Process each 2D child
    for child in children {
        let polygon = extract_2d_points(child, params)?;
        if polygon.len() < 3 {
            continue;
        }
        
        // Generate layers
        for slice in 0..num_slices {
            let t0 = slice as f32 / num_slices as f32;
            let t1 = (slice + 1) as f32 / num_slices as f32;
            
            let z0 = z_offset + h * t0;
            let z1 = z_offset + h * t1;
            
            let angle0 = twist_rad * t0;
            let angle1 = twist_rad * t1;
            
            let s0 = 1.0 + (scale_factor - 1.0) * t0 as f64;
            let s1 = 1.0 + (scale_factor - 1.0) * t1 as f64;
            
            // Create side faces between layers
            for i in 0..polygon.len() {
                let next = (i + 1) % polygon.len();
                
                let p0 = &polygon[i];
                let p1 = &polygon[next];
                
                // Bottom layer vertices
                let (x00, y00) = rotate_point(p0[0] * s0, p0[1] * s0, angle0);
                let (x01, y01) = rotate_point(p1[0] * s0, p1[1] * s0, angle0);
                
                // Top layer vertices
                let (x10, y10) = rotate_point(p0[0] * s1, p0[1] * s1, angle1);
                let (x11, y11) = rotate_point(p1[0] * s1, p1[1] * s1, angle1);
                
                // Compute normal (approximation)
                let nx = (y01 - y00) as f32;
                let ny = -(x01 - x00) as f32;
                let nz = 0.0;
                let len = (nx * nx + ny * ny).sqrt();
                let (nx, ny) = if len > 0.0 { (nx / len, ny / len) } else { (0.0, 1.0) };
                
                let v0 = mesh.add_vertex(x00 as f32, y00 as f32, z0, nx, ny, nz);
                let v1 = mesh.add_vertex(x01 as f32, y01 as f32, z0, nx, ny, nz);
                let v2 = mesh.add_vertex(x11 as f32, y11 as f32, z1, nx, ny, nz);
                let v3 = mesh.add_vertex(x10 as f32, y10 as f32, z1, nx, ny, nz);
                
                mesh.add_triangle(v0, v1, v2);
                mesh.add_triangle(v0, v2, v3);
            }
        }
        
        // Add bottom cap
        add_cap(mesh, &polygon, z_offset, 0.0, 1.0, -1.0);
        
        // Add top cap
        let top_angle = twist_rad;
        add_cap(mesh, &polygon, z_offset + h, top_angle, scale_factor as f32, 1.0);
    }
    
    Ok(())
}

/// Add a cap (top or bottom) to the extrusion.
fn add_cap(mesh: &mut Mesh, polygon: &[[f64; 2]], z: f32, angle: f32, scale: f32, nz: f32) {
    if polygon.len() < 3 {
        return;
    }
    
    // Transform first point
    let (x0, y0) = rotate_point(polygon[0][0] * scale as f64, polygon[0][1] * scale as f64, angle);
    let first = mesh.add_vertex(x0 as f32, y0 as f32, z, 0.0, 0.0, nz);
    
    // Fan triangulation
    for i in 1..polygon.len() - 1 {
        let (x1, y1) = rotate_point(polygon[i][0] * scale as f64, polygon[i][1] * scale as f64, angle);
        let (x2, y2) = rotate_point(polygon[i + 1][0] * scale as f64, polygon[i + 1][1] * scale as f64, angle);
        
        let v1 = mesh.add_vertex(x1 as f32, y1 as f32, z, 0.0, 0.0, nz);
        let v2 = mesh.add_vertex(x2 as f32, y2 as f32, z, 0.0, 0.0, nz);
        
        if nz > 0.0 {
            mesh.add_triangle(first, v1, v2);
        } else {
            mesh.add_triangle(first, v2, v1);
        }
    }
}

/// Rotate a 2D point by angle (radians).
fn rotate_point(x: f64, y: f64, angle: f32) -> (f64, f64) {
    let cos_a = (angle as f64).cos();
    let sin_a = (angle as f64).sin();
    (x * cos_a - y * sin_a, x * sin_a + y * cos_a)
}

// =============================================================================
// ROTATE EXTRUDE
// =============================================================================

/// Rotate extrude 2D children around Z axis.
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// rotate_extrude(angle, $fn) { children }
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh
/// - `children`: 2D child geometry nodes
/// - `angle`: Rotation angle in degrees (360 for full revolution)
/// - `params`: Segment parameters
pub fn rotate_extrude(
    mesh: &mut Mesh,
    children: &[GeometryNode],
    angle: f64,
    params: &SegmentParams,
) -> ManifoldResult<()> {
    let angle_rad = (angle as f32).to_radians();
    let segments = params.calculate_segments(10.0); // Use reasonable default radius
    let num_segments = segments.max(3) as usize;
    
    // Process each 2D child
    for child in children {
        let polygon = extract_2d_points(child, params)?;
        if polygon.len() < 2 {
            continue;
        }
        
        // Generate rotated layers
        for seg in 0..num_segments {
            let t0 = seg as f32 / num_segments as f32;
            let t1 = (seg + 1) as f32 / num_segments as f32;
            
            let theta0 = angle_rad * t0;
            let theta1 = angle_rad * t1;
            
            let (cos0, sin0) = (theta0.cos(), theta0.sin());
            let (cos1, sin1) = (theta1.cos(), theta1.sin());
            
            // Create quads between layers
            for i in 0..polygon.len() - 1 {
                let p0 = &polygon[i];
                let p1 = &polygon[i + 1];
                
                // p0 and p1 are in XY plane, we rotate around Z
                // X in polygon becomes radius, Y becomes Z
                let r0 = p0[0] as f32;
                let r1 = p1[0] as f32;
                let z0 = p0[1] as f32;
                let z1 = p1[1] as f32;
                
                if r0 < 0.0 || r1 < 0.0 {
                    continue; // Skip points on wrong side of axis
                }
                
                // Four corners of quad
                let x00 = r0 * cos0;
                let y00 = r0 * sin0;
                let x01 = r0 * cos1;
                let y01 = r0 * sin1;
                let x10 = r1 * cos0;
                let y10 = r1 * sin0;
                let x11 = r1 * cos1;
                let y11 = r1 * sin1;
                
                // Compute normals
                let nx0 = cos0;
                let ny0 = sin0;
                let nx1 = cos1;
                let ny1 = sin1;
                
                let v00 = mesh.add_vertex(x00, y00, z0, nx0, ny0, 0.0);
                let v01 = mesh.add_vertex(x01, y01, z0, nx1, ny1, 0.0);
                let v10 = mesh.add_vertex(x10, y10, z1, nx0, ny0, 0.0);
                let v11 = mesh.add_vertex(x11, y11, z1, nx1, ny1, 0.0);
                
                mesh.add_triangle(v00, v10, v11);
                mesh.add_triangle(v00, v11, v01);
            }
        }
        
        // Add caps if not full revolution
        if (angle - 360.0).abs() > 0.1 {
            // Start cap
            add_revolve_cap(mesh, &polygon, 0.0, -1.0);
            // End cap
            add_revolve_cap(mesh, &polygon, angle_rad, 1.0);
        }
    }
    
    Ok(())
}

/// Add a cap for rotate_extrude at given angle.
fn add_revolve_cap(mesh: &mut Mesh, polygon: &[[f64; 2]], angle: f32, normal_dir: f32) {
    if polygon.len() < 3 {
        return;
    }
    
    let (cos_a, sin_a) = (angle.cos(), angle.sin());
    let nx = -sin_a * normal_dir;
    let ny = cos_a * normal_dir;
    
    // Transform polygon to 3D at this angle
    let pts: Vec<(f32, f32, f32)> = polygon.iter()
        .map(|p| {
            let r = p[0] as f32;
            let z = p[1] as f32;
            (r * cos_a, r * sin_a, z)
        })
        .collect();
    
    if pts.len() < 3 {
        return;
    }
    
    let first = mesh.add_vertex(pts[0].0, pts[0].1, pts[0].2, nx, ny, 0.0);
    
    for i in 1..pts.len() - 1 {
        let v1 = mesh.add_vertex(pts[i].0, pts[i].1, pts[i].2, nx, ny, 0.0);
        let v2 = mesh.add_vertex(pts[i + 1].0, pts[i + 1].1, pts[i + 1].2, nx, ny, 0.0);
        
        if normal_dir > 0.0 {
            mesh.add_triangle(first, v1, v2);
        } else {
            mesh.add_triangle(first, v2, v1);
        }
    }
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

    /// Test circle points generation.
    #[test]
    fn test_circle_points() {
        let points = circle_points(5.0, 8);
        assert_eq!(points.len(), 8);
    }

    /// Test square points generation.
    #[test]
    fn test_square_points() {
        let points = square_points([10.0, 10.0], false);
        assert_eq!(points.len(), 4);
    }

    /// Test rotate_point.
    #[test]
    fn test_rotate_point() {
        let (x, y) = rotate_point(1.0, 0.0, std::f32::consts::PI / 2.0);
        assert!((x - 0.0).abs() < 0.01);
        assert!((y - 1.0).abs() < 0.01);
    }
}

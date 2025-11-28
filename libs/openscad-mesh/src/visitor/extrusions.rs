//! # Extrusion Mesh Builders
//!
//! Builds 3D meshes from 2D shapes via linear and rotational extrusion.
//! Implements Manifold-style algorithms with OpenSCAD parameter compatibility.

use crate::mesh::Mesh;
use openscad_eval::GeometryNode;
use std::f32::consts::PI;

// =============================================================================
// LINEAR EXTRUDE
// =============================================================================

/// Build linear extrusion of a 2D shape.
///
/// ## OpenSCAD linear_extrude
///
/// Extrudes a 2D shape along the Z axis with optional twist and scale.
///
/// - `height`: Extrusion height along Z
/// - `twist`: Rotation in degrees over the height
/// - `scale`: Scale factor at top [sx, sy]
/// - `slices`: Number of intermediate layers (for twist/scale)
/// - `center`: If true, center along Z axis
///
/// ## Example
///
/// ```text
/// linear_extrude(height=10, twist=90, slices=20)
///     circle(5);
/// ```
pub fn build_linear_extrude(
    mesh: &mut Mesh,
    height: f64,
    twist: f64,
    scale: [f64; 2],
    slices: u32,
    center: bool,
    child: &GeometryNode,
) {
    // Extract 2D outline from child
    let outline = extract_2d_outline(child);
    if outline.len() < 3 {
        return;
    }

    let h = height as f32;
    let num_slices = slices.max(1) as usize;
    let twist_rad = (twist as f32).to_radians();
    let z_offset = if center { -h / 2.0 } else { 0.0 };

    // For each slice, generate a ring of vertices
    let mut rings: Vec<Vec<u32>> = Vec::with_capacity(num_slices + 1);
    
    for slice_idx in 0..=num_slices {
        let t = slice_idx as f32 / num_slices as f32;
        let z = z_offset + t * h;
        let angle = t * twist_rad;
        let sx = 1.0 + t * (scale[0] as f32 - 1.0);
        let sy = 1.0 + t * (scale[1] as f32 - 1.0);
        
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        let mut ring = Vec::with_capacity(outline.len());
        for &[ox, oy] in &outline {
            // Apply scale
            let x = ox * sx;
            let y = oy * sy;
            // Apply twist rotation
            let rx = x * cos_a - y * sin_a;
            let ry = x * sin_a + y * cos_a;
            
            // Normal pointing outward (approximate)
            let len = (rx * rx + ry * ry).sqrt();
            let (nx, ny) = if len > 0.0 { (rx / len, ry / len) } else { (0.0, 1.0) };
            
            let v = mesh.add_vertex(rx, ry, z, nx, ny, 0.0);
            ring.push(v);
        }
        rings.push(ring);
    }

    // Connect rings with side faces
    for ring_idx in 0..num_slices {
        let ring_a = &rings[ring_idx];
        let ring_b = &rings[ring_idx + 1];
        
        for i in 0..outline.len() {
            let next = (i + 1) % outline.len();
            
            // Two triangles per quad
            mesh.add_triangle(ring_a[i], ring_a[next], ring_b[next]);
            mesh.add_triangle(ring_a[i], ring_b[next], ring_b[i]);
        }
    }

    // Bottom cap (first ring)
    add_cap(mesh, &rings[0], false);
    
    // Top cap (last ring)
    add_cap(mesh, &rings[num_slices], true);
}

// =============================================================================
// ROTATE EXTRUDE
// =============================================================================

/// Build rotational extrusion of a 2D shape.
///
/// ## OpenSCAD rotate_extrude
///
/// Rotates a 2D shape around the Z axis.
/// The 2D shape should be in the positive X half-plane.
///
/// - `angle`: Sweep angle in degrees (default 360)
/// - `fn_`: Number of segments around the rotation
///
/// ## Example
///
/// ```text
/// rotate_extrude($fn=32)
///     translate([15, 0]) circle(5);
/// ```
pub fn build_rotate_extrude(
    mesh: &mut Mesh,
    angle: f64,
    fn_: u32,
    child: &GeometryNode,
) {
    // Extract 2D outline from child
    let outline = extract_2d_outline(child);
    if outline.len() < 2 {
        return;
    }

    let angle_rad = (angle as f32).to_radians();
    let segments = fn_.max(3) as usize;
    let full_rotation = (angle - 360.0).abs() < 0.001;

    // Generate rings by rotating the outline around Z
    let mut rings: Vec<Vec<u32>> = Vec::with_capacity(segments + 1);
    let num_steps = if full_rotation { segments } else { segments + 1 };
    
    for step in 0..num_steps {
        let t = step as f32 / segments as f32;
        let theta = t * angle_rad;
        let cos_t = theta.cos();
        let sin_t = theta.sin();
        
        let mut ring = Vec::with_capacity(outline.len());
        for &[x, y] in &outline {
            // Rotate point (x, 0, y) around Z axis
            let rx = x * cos_t;
            let ry = x * sin_t;
            let rz = y;
            
            // Normal: radial direction
            let nx = cos_t;
            let ny = sin_t;
            
            let v = mesh.add_vertex(rx, ry, rz, nx, ny, 0.0);
            ring.push(v);
        }
        rings.push(ring);
    }

    // Connect rings with side faces
    let connection_count = if full_rotation { segments } else { segments };
    for ring_idx in 0..connection_count {
        let ring_a = &rings[ring_idx];
        let next_idx = if full_rotation && ring_idx == segments - 1 { 0 } else { ring_idx + 1 };
        let ring_b = &rings[next_idx];
        
        for i in 0..outline.len() - 1 {
            // Two triangles per quad
            mesh.add_triangle(ring_a[i], ring_a[i + 1], ring_b[i + 1]);
            mesh.add_triangle(ring_a[i], ring_b[i + 1], ring_b[i]);
        }
    }

    // End caps for partial rotation
    if !full_rotation && rings.len() >= 2 {
        // Start cap
        add_sweep_cap(mesh, &rings[0], false);
        // End cap  
        add_sweep_cap(mesh, &rings[rings.len() - 1], true);
    }
}

// =============================================================================
// HELPERS
// =============================================================================

/// Extract 2D outline from a child geometry node.
/// Returns list of [x, y] points.
pub fn extract_2d_outline(child: &GeometryNode) -> Vec<[f32; 2]> {
    match child {
        GeometryNode::Circle { radius, fn_ } => {
            let r = *radius as f32;
            let segments = (*fn_).max(3) as usize;
            let mut outline = Vec::with_capacity(segments);
            for i in 0..segments {
                let theta = 2.0 * PI * i as f32 / segments as f32;
                outline.push([r * theta.cos(), r * theta.sin()]);
            }
            outline
        }
        GeometryNode::Square { size, center } => {
            let [sx, sy] = [size[0] as f32, size[1] as f32];
            if *center {
                vec![
                    [-sx/2.0, -sy/2.0],
                    [sx/2.0, -sy/2.0],
                    [sx/2.0, sy/2.0],
                    [-sx/2.0, sy/2.0],
                ]
            } else {
                vec![
                    [0.0, 0.0],
                    [sx, 0.0],
                    [sx, sy],
                    [0.0, sy],
                ]
            }
        }
        GeometryNode::Polygon { points, .. } => {
            points.iter()
                .map(|p| [p[0] as f32, p[1] as f32])
                .collect()
        }
        GeometryNode::Translate { offset, child } => {
            let mut outline = extract_2d_outline(child);
            let ox = offset[0] as f32;
            let oy = offset[1] as f32;
            for pt in &mut outline {
                pt[0] += ox;
                pt[1] += oy;
            }
            outline
        }
        GeometryNode::Scale { factors, child } => {
            let mut outline = extract_2d_outline(child);
            let sx = factors[0] as f32;
            let sy = factors[1] as f32;
            for pt in &mut outline {
                pt[0] *= sx;
                pt[1] *= sy;
            }
            outline
        }
        GeometryNode::Offset { delta, chamfer, child } => {
            let outline = extract_2d_outline(child);
            if outline.len() < 3 {
                return outline;
            }
            offset_polygon_inline(&outline, *delta as f32, *chamfer)
        }
        _ => Vec::new(),
    }
}

/// Inline polygon offset for use in extract_2d_outline.
/// This avoids circular dependency with ops_2d module.
fn offset_polygon_inline(outline: &[[f32; 2]], delta: f32, chamfer: bool) -> Vec<[f32; 2]> {
    let n = outline.len();
    if n < 3 {
        return vec![];
    }

    let mut result = Vec::with_capacity(n * 2);

    for i in 0..n {
        let prev = if i == 0 { n - 1 } else { i - 1 };
        let next = (i + 1) % n;

        let e1 = [
            outline[i][0] - outline[prev][0],
            outline[i][1] - outline[prev][1],
        ];
        let e2 = [
            outline[next][0] - outline[i][0],
            outline[next][1] - outline[i][1],
        ];

        let len1 = (e1[0] * e1[0] + e1[1] * e1[1]).sqrt();
        let len2 = (e2[0] * e2[0] + e2[1] * e2[1]).sqrt();
        
        if len1 < 1e-10 || len2 < 1e-10 {
            result.push(outline[i]);
            continue;
        }

        let n1 = [e1[1] / len1, -e1[0] / len1];
        let n2 = [e2[1] / len2, -e2[0] / len2];

        let avg_x = n1[0] + n2[0];
        let avg_y = n1[1] + n2[1];
        let avg_len = (avg_x * avg_x + avg_y * avg_y).sqrt();

        if avg_len < 1e-10 {
            result.push([
                outline[i][0] + n1[0] * delta,
                outline[i][1] + n1[1] * delta,
            ]);
        } else if chamfer {
            result.push([
                outline[i][0] + n1[0] * delta,
                outline[i][1] + n1[1] * delta,
            ]);
            result.push([
                outline[i][0] + n2[0] * delta,
                outline[i][1] + n2[1] * delta,
            ]);
        } else {
            let nx = avg_x / avg_len;
            let ny = avg_y / avg_len;
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

/// Add a cap (top or bottom) from a ring of vertices.
fn add_cap(mesh: &mut Mesh, ring: &[u32], top: bool) {
    if ring.len() < 3 {
        return;
    }
    
    // Fan triangulation from first vertex
    for i in 1..ring.len() - 1 {
        if top {
            mesh.add_triangle(ring[0], ring[i], ring[i + 1]);
        } else {
            mesh.add_triangle(ring[0], ring[i + 1], ring[i]);
        }
    }
}

/// Add end cap for rotate_extrude (polygon in XZ plane at given rotation).
fn add_sweep_cap(mesh: &mut Mesh, ring: &[u32], top: bool) {
    if ring.len() < 3 {
        return;
    }
    
    // Simple fan triangulation
    for i in 1..ring.len() - 1 {
        if top {
            mesh.add_triangle(ring[0], ring[i], ring[i + 1]);
        } else {
            mesh.add_triangle(ring[0], ring[i + 1], ring[i]);
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
    fn test_extract_circle_outline() {
        let node = GeometryNode::Circle { radius: 10.0, fn_: 8 };
        let outline = extract_2d_outline(&node);
        assert_eq!(outline.len(), 8);
    }

    #[test]
    fn test_extract_square_outline() {
        let node = GeometryNode::Square { size: [10.0, 5.0], center: false };
        let outline = extract_2d_outline(&node);
        assert_eq!(outline.len(), 4);
    }

    #[test]
    fn test_extract_translated_outline() {
        let child = GeometryNode::Circle { radius: 5.0, fn_: 4 };
        let node = GeometryNode::Translate {
            offset: [10.0, 0.0, 0.0],
            child: Box::new(child),
        };
        let outline = extract_2d_outline(&node);
        assert_eq!(outline.len(), 4);
        // First point should be at (15, 0) = radius + offset
        assert!((outline[0][0] - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_build_linear_extrude() {
        let mut mesh = Mesh::new();
        let child = GeometryNode::Circle { radius: 5.0, fn_: 8 };
        build_linear_extrude(&mut mesh, 10.0, 0.0, [1.0, 1.0], 1, false, &child);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_build_linear_extrude_with_twist() {
        let mut mesh = Mesh::new();
        let child = GeometryNode::Square { size: [10.0, 5.0], center: true };
        build_linear_extrude(&mut mesh, 20.0, 180.0, [1.0, 1.0], 10, false, &child);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_build_rotate_extrude() {
        let mut mesh = Mesh::new();
        let circle = GeometryNode::Circle { radius: 3.0, fn_: 8 };
        let child = GeometryNode::Translate {
            offset: [10.0, 0.0, 0.0],
            child: Box::new(circle),
        };
        build_rotate_extrude(&mut mesh, 360.0, 16, &child);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }
}

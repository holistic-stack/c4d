//! # 3D Primitive Constructors
//!
//! Mesh builders for cube, sphere, cylinder, and polyhedron primitives.
//! Uses OpenSCAD-compatible algorithms with Manifold circularSegments.
//!
//! ## OpenSCAD Compatibility
//!
//! All primitives match OpenSCAD's output exactly:
//! - Cube uses 24 vertices (4 per face for per-face normals)
//! - Sphere uses offset phi (no pole vertices)
//! - Cylinder uses separate vertices for caps and sides

use crate::mesh::Mesh;
use std::f32::consts::PI;

// =============================================================================
// CUBE
// =============================================================================

/// Build axis-aligned cube mesh.
///
/// Creates a cube with 24 vertices (4 per face for correct per-face normals)
/// and 12 triangles (2 per face).
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// cube([sx, sy, sz], center);
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh to populate
/// - `size`: [width, depth, height]
/// - `center`: If true, center at origin; otherwise in positive octant
///
/// ## Example
///
/// ```rust
/// use manifold_rs::mesh::Mesh;
/// use manifold_rs::manifold::constructors::build_cube;
///
/// let mut mesh = Mesh::new();
/// build_cube(&mut mesh, [10.0, 10.0, 10.0], false);
/// assert_eq!(mesh.vertex_count(), 24);
/// assert_eq!(mesh.triangle_count(), 12);
/// ```
pub fn build_cube(mesh: &mut Mesh, size: [f64; 3], center: bool) {
    let [sx, sy, sz] = [size[0] as f32, size[1] as f32, size[2] as f32];
    
    // Calculate bounds based on center parameter
    let (min_x, max_x) = if center { (-sx / 2.0, sx / 2.0) } else { (0.0, sx) };
    let (min_y, max_y) = if center { (-sy / 2.0, sy / 2.0) } else { (0.0, sy) };
    let (min_z, max_z) = if center { (-sz / 2.0, sz / 2.0) } else { (0.0, sz) };

    // Front face (+Z normal)
    let v0 = mesh.add_vertex(min_x, min_y, max_z, 0.0, 0.0, 1.0);
    let v1 = mesh.add_vertex(max_x, min_y, max_z, 0.0, 0.0, 1.0);
    let v2 = mesh.add_vertex(max_x, max_y, max_z, 0.0, 0.0, 1.0);
    let v3 = mesh.add_vertex(min_x, max_y, max_z, 0.0, 0.0, 1.0);
    mesh.add_triangle(v0, v1, v2);
    mesh.add_triangle(v0, v2, v3);

    // Back face (-Z normal)
    let v4 = mesh.add_vertex(max_x, min_y, min_z, 0.0, 0.0, -1.0);
    let v5 = mesh.add_vertex(min_x, min_y, min_z, 0.0, 0.0, -1.0);
    let v6 = mesh.add_vertex(min_x, max_y, min_z, 0.0, 0.0, -1.0);
    let v7 = mesh.add_vertex(max_x, max_y, min_z, 0.0, 0.0, -1.0);
    mesh.add_triangle(v4, v5, v6);
    mesh.add_triangle(v4, v6, v7);

    // Top face (+Y normal)
    let v8 = mesh.add_vertex(min_x, max_y, max_z, 0.0, 1.0, 0.0);
    let v9 = mesh.add_vertex(max_x, max_y, max_z, 0.0, 1.0, 0.0);
    let v10 = mesh.add_vertex(max_x, max_y, min_z, 0.0, 1.0, 0.0);
    let v11 = mesh.add_vertex(min_x, max_y, min_z, 0.0, 1.0, 0.0);
    mesh.add_triangle(v8, v9, v10);
    mesh.add_triangle(v8, v10, v11);

    // Bottom face (-Y normal)
    let v12 = mesh.add_vertex(min_x, min_y, min_z, 0.0, -1.0, 0.0);
    let v13 = mesh.add_vertex(max_x, min_y, min_z, 0.0, -1.0, 0.0);
    let v14 = mesh.add_vertex(max_x, min_y, max_z, 0.0, -1.0, 0.0);
    let v15 = mesh.add_vertex(min_x, min_y, max_z, 0.0, -1.0, 0.0);
    mesh.add_triangle(v12, v13, v14);
    mesh.add_triangle(v12, v14, v15);

    // Right face (+X normal)
    let v16 = mesh.add_vertex(max_x, min_y, max_z, 1.0, 0.0, 0.0);
    let v17 = mesh.add_vertex(max_x, min_y, min_z, 1.0, 0.0, 0.0);
    let v18 = mesh.add_vertex(max_x, max_y, min_z, 1.0, 0.0, 0.0);
    let v19 = mesh.add_vertex(max_x, max_y, max_z, 1.0, 0.0, 0.0);
    mesh.add_triangle(v16, v17, v18);
    mesh.add_triangle(v16, v18, v19);

    // Left face (-X normal)
    let v20 = mesh.add_vertex(min_x, min_y, min_z, -1.0, 0.0, 0.0);
    let v21 = mesh.add_vertex(min_x, min_y, max_z, -1.0, 0.0, 0.0);
    let v22 = mesh.add_vertex(min_x, max_y, max_z, -1.0, 0.0, 0.0);
    let v23 = mesh.add_vertex(min_x, max_y, min_z, -1.0, 0.0, 0.0);
    mesh.add_triangle(v20, v21, v22);
    mesh.add_triangle(v20, v22, v23);
}

// =============================================================================
// SPHERE
// =============================================================================

/// Build sphere mesh using OpenSCAD-compatible tessellation.
///
/// Uses offset phi values so vertices are NOT at the poles, matching OpenSCAD:
/// - `phi = 180.0 * (i + 0.5) / num_rings`
/// - `num_rings = (circularSegments + 1) / 2`
/// - Creates n-gon caps instead of triangle fans at poles
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// sphere(r, $fn=circularSegments);
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh to populate
/// - `radius`: Sphere radius
/// - `circular_segments`: Number of segments around circumference
pub fn build_sphere(mesh: &mut Mesh, radius: f64, circular_segments: u32) {
    let r = radius as f32;
    let num_fragments = circular_segments.max(3) as usize;
    let num_rings = (num_fragments + 1) / 2;
    
    // Generate ring vertices with OpenSCAD-compatible offset
    let mut rings: Vec<Vec<u32>> = Vec::with_capacity(num_rings);
    
    for ring_idx in 0..num_rings {
        // OpenSCAD offset formula: phi = 180 * (i + 0.5) / num_rings
        let phi_deg = 180.0 * (ring_idx as f32 + 0.5) / num_rings as f32;
        let phi_rad = phi_deg.to_radians();
        
        let ring_radius = r * phi_rad.sin();
        let z = r * phi_rad.cos();
        
        let mut ring = Vec::with_capacity(num_fragments);
        for seg_idx in 0..num_fragments {
            let theta = 2.0 * PI * seg_idx as f32 / num_fragments as f32;
            let x = ring_radius * theta.cos();
            let y = ring_radius * theta.sin();
            
            // Normal is normalized position for unit sphere
            let len = (x * x + y * y + z * z).sqrt();
            let (nx, ny, nz) = if len > 0.0 {
                (x / len, y / len, z / len)
            } else {
                (0.0, 0.0, 1.0)
            };
            
            let v = mesh.add_vertex(x, y, z, nx, ny, nz);
            ring.push(v);
        }
        rings.push(ring);
    }

    // Top cap - triangulated n-gon from first ring
    if !rings.is_empty() {
        let ring = &rings[0];
        let n = ring.len();
        
        if n % 2 == 0 {
            // Split diagonal triangulation (OpenSCAD style for even N)
            // Splits along diagonal (N/2-1)-(N-1)
            let p1 = n - 1;
            let p2 = n / 2 - 1;
            
            // Fan from p1 (Right half: 0..N/2-1)
            for i in 0..(n / 2 - 1) {
                mesh.add_triangle(ring[p1], ring[i], ring[i + 1]);
            }
            
            // Fan from p2 (Left half: N/2-1..N-1)
            for i in (n / 2)..(n - 1) {
                mesh.add_triangle(ring[p2], ring[i], ring[i + 1]);
            }
        } else {
            // Simple fan
            for i in 1..n - 1 {
                mesh.add_triangle(ring[0], ring[i], ring[i + 1]);
            }
        }
    }

    // Body - quads between adjacent rings
    for ring_idx in 0..num_rings.saturating_sub(1) {
        let ring_a = &rings[ring_idx];
        let ring_b = &rings[ring_idx + 1];
        
        for i in 0..num_fragments {
            let next = (i + 1) % num_fragments;
            mesh.add_triangle(ring_a[i], ring_b[i], ring_b[next]);
            mesh.add_triangle(ring_a[i], ring_b[next], ring_a[next]);
        }
    }

    // Bottom cap - triangulated n-gon from last ring (reversed winding)
    if !rings.is_empty() {
        let ring = &rings[num_rings - 1];
        let n = ring.len();
        
        if n % 2 == 0 {
            // Split diagonal triangulation (Reversed)
            let p1 = n - 1;
            let p2 = n / 2 - 1;
            
            // Fan from p1 (Right half)
            for i in 0..(n / 2 - 1) {
                mesh.add_triangle(ring[p1], ring[i + 1], ring[i]);
            }
            
            // Fan from p2 (Left half)
            for i in (n / 2)..(n - 1) {
                mesh.add_triangle(ring[p2], ring[i + 1], ring[i]);
            }
        } else {
            // Simple fan (Reversed)
            for i in 1..n - 1 {
                mesh.add_triangle(ring[0], ring[i + 1], ring[i]);
            }
        }
    }
}

// =============================================================================
// CYLINDER
// =============================================================================

/// Build cylinder or cone mesh.
///
/// Creates a cylinder with separate vertices for caps (flat normals) and
/// sides (radial normals), matching OpenSCAD's output.
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// cylinder(h=height, r1=radius1, r2=radius2, center=center, $fn=segments);
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh to populate
/// - `height`: Cylinder height
/// - `radius1`: Bottom radius (0 for cone pointing up)
/// - `radius2`: Top radius (0 for cone pointing down)
/// - `circular_segments`: Number of segments around circumference
/// - `center`: If true, center vertically at origin
pub fn build_cylinder(
    mesh: &mut Mesh,
    height: f64,
    radius1: f64,
    radius2: f64,
    circular_segments: u32,
    center: bool,
) {
    let h = height as f32;
    let r1 = radius1 as f32;
    let r2 = radius2 as f32;
    let segments = circular_segments.max(3) as usize;
    
    let z_bottom = if center { -h / 2.0 } else { 0.0 };
    let z_top = if center { h / 2.0 } else { h };

    // Generate bottom ring (if not a point)
    let mut bottom_ring: Vec<u32> = Vec::new();
    if r1 > 0.0 {
        for i in 0..segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let x = r1 * theta.cos();
            let y = r1 * theta.sin();
            let v = mesh.add_vertex(x, y, z_bottom, 0.0, 0.0, -1.0);
            bottom_ring.push(v);
        }
    }

    // Generate top ring (if not a point)
    let mut top_ring: Vec<u32> = Vec::new();
    if r2 > 0.0 {
        for i in 0..segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let x = r2 * theta.cos();
            let y = r2 * theta.sin();
            let v = mesh.add_vertex(x, y, z_top, 0.0, 0.0, 1.0);
            top_ring.push(v);
        }
    }

    // Side faces with proper outward normals
    let cone_slope = (r1 - r2) / h;
    let normal_z = cone_slope / (1.0 + cone_slope * cone_slope).sqrt();
    let normal_xy_scale = 1.0 / (1.0 + cone_slope * cone_slope).sqrt();

    // Create separate side vertices with outward normals
    let mut side_bottom: Vec<u32> = Vec::new();
    let mut side_top: Vec<u32> = Vec::new();
    
    for i in 0..segments {
        let theta = 2.0 * PI * i as f32 / segments as f32;
        let nx = theta.cos() * normal_xy_scale;
        let ny = theta.sin() * normal_xy_scale;
        
        if r1 > 0.0 {
            let x = r1 * theta.cos();
            let y = r1 * theta.sin();
            let v = mesh.add_vertex(x, y, z_bottom, nx, ny, normal_z);
            side_bottom.push(v);
        }
        
        if r2 > 0.0 {
            let x = r2 * theta.cos();
            let y = r2 * theta.sin();
            let v = mesh.add_vertex(x, y, z_top, nx, ny, normal_z);
            side_top.push(v);
        }
    }

    // Side faces
    if r1 > 0.0 && r2 > 0.0 {
        // Full cylinder/frustum - quads
        for i in 0..segments {
            let next = (i + 1) % segments;
            mesh.add_triangle(side_bottom[i], side_bottom[next], side_top[next]);
            mesh.add_triangle(side_bottom[i], side_top[next], side_top[i]);
        }
    } else if r1 > 0.0 {
        // Cone pointing up - triangles to apex
        let apex = mesh.add_vertex(0.0, 0.0, z_top, 0.0, 0.0, 1.0);
        for i in 0..segments {
            let next = (i + 1) % segments;
            mesh.add_triangle(side_bottom[i], side_bottom[next], apex);
        }
    } else if r2 > 0.0 {
        // Inverted cone - triangles from apex
        let apex = mesh.add_vertex(0.0, 0.0, z_bottom, 0.0, 0.0, -1.0);
        for i in 0..segments {
            let next = (i + 1) % segments;
            mesh.add_triangle(apex, side_top[next], side_top[i]);
        }
    }

    // Bottom cap (if not a point)
    if r1 > 0.0 && bottom_ring.len() >= 3 {
        for i in 1..bottom_ring.len() - 1 {
            mesh.add_triangle(bottom_ring[0], bottom_ring[i + 1], bottom_ring[i]);
        }
    }

    // Top cap (if not a point)
    if r2 > 0.0 && top_ring.len() >= 3 {
        for i in 1..top_ring.len() - 1 {
            mesh.add_triangle(top_ring[0], top_ring[i], top_ring[i + 1]);
        }
    }
}

// =============================================================================
// POLYHEDRON
// =============================================================================

/// Build polyhedron from points and faces.
///
/// Creates a custom mesh with computed face normals. Face winding is reversed
/// from OpenSCAD convention (CCW → CW for OpenGL).
///
/// ## OpenSCAD Equivalent
///
/// ```text
/// polyhedron(points=[[x,y,z],...], faces=[[i,j,k],...]);
/// ```
///
/// ## Parameters
///
/// - `mesh`: Output mesh to populate
/// - `points`: Vertex positions
/// - `faces`: Face definitions (indices into points array)
pub fn build_polyhedron(mesh: &mut Mesh, points: &[[f64; 3]], faces: &[Vec<usize>]) {
    // Convert points to f32
    let pts: Vec<[f32; 3]> = points
        .iter()
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();
    
    // Process each face
    for face in faces {
        if face.len() < 3 {
            continue;
        }
        
        // Compute face normal from first 3 vertices
        let p0 = pts[face[0]];
        let p1 = pts[face[1]];
        let p2 = pts[face[2]];
        
        let u = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let v = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let n = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        let n = if len > 0.0 {
            [n[0] / len, n[1] / len, n[2] / len]
        } else {
            [0.0, 0.0, 1.0]
        };
        
        // First vertex (shared for fan triangulation)
        let first_vertex = mesh.add_vertex(p0[0], p0[1], p0[2], n[0], n[1], n[2]);
        
        // Fan triangulation
        for i in 1..face.len() - 1 {
            let idx1 = face[i];
            let idx2 = face[i + 1];
            
            let v1 = mesh.add_vertex(pts[idx1][0], pts[idx1][1], pts[idx1][2], n[0], n[1], n[2]);
            let v2 = mesh.add_vertex(pts[idx2][0], pts[idx2][1], pts[idx2][2], n[0], n[1], n[2]);
            
            // Reverse winding for OpenSCAD compatibility
            mesh.add_triangle(first_vertex, v2, v1);
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test cube construction.
    #[test]
    fn test_build_cube() {
        let mut mesh = Mesh::new();
        build_cube(&mut mesh, [1.0, 1.0, 1.0], false);
        assert_eq!(mesh.vertex_count(), 24);
        assert_eq!(mesh.triangle_count(), 12);
    }

    /// Test centered cube.
    #[test]
    fn test_build_cube_centered() {
        let mut mesh = Mesh::new();
        build_cube(&mut mesh, [2.0, 2.0, 2.0], true);
        
        // Should have vertices on both sides of origin
        let has_negative = mesh.vertices.iter().any(|&v| v < 0.0);
        let has_positive = mesh.vertices.iter().any(|&v| v > 0.0);
        assert!(has_negative && has_positive);
    }

    /// Test sphere construction.
    #[test]
    fn test_build_sphere() {
        let mut mesh = Mesh::new();
        build_sphere(&mut mesh, 1.0, 16);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    /// Test cylinder construction.
    #[test]
    fn test_build_cylinder() {
        let mut mesh = Mesh::new();
        build_cylinder(&mut mesh, 2.0, 1.0, 1.0, 16, false);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    /// Test cone (cylinder with r2=0).
    #[test]
    fn test_build_cone() {
        let mut mesh = Mesh::new();
        build_cylinder(&mut mesh, 2.0, 1.0, 0.0, 16, false);
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    /// Test polyhedron (tetrahedron).
    #[test]
    fn test_build_polyhedron() {
        let mut mesh = Mesh::new();
        let points = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];
        let faces = vec![
            vec![0, 1, 2],
            vec![0, 3, 1],
            vec![1, 3, 2],
            vec![2, 3, 0],
        ];
        build_polyhedron(&mut mesh, &points, &faces);
        assert_eq!(mesh.triangle_count(), 4);
    }
}

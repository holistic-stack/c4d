//! # Sphere Primitive
//!
//! Generates mesh for sphere shapes using latitude/longitude tessellation.

use crate::error::MeshError;
use crate::mesh::Mesh;
use glam::DVec3;
use std::f64::consts::PI;

/// Creates a sphere mesh using latitude/longitude tessellation.
///
/// # Arguments
///
/// * `radius` - The radius of the sphere
/// * `segments` - Number of segments around the circumference
///
/// # Returns
///
/// A mesh representing the sphere.
///
/// # Algorithm
///
/// Uses OpenSCAD's sphere algorithm:
/// - num_rings = (segments + 1) / 2
/// - Each ring at polar angle phi = 180° * (i + 0.5) / num_rings
/// - No pole vertices - uses polygon caps
///
/// # Example
///
/// ```rust
/// use openscad_mesh::primitives::create_sphere;
///
/// let mesh = create_sphere(5.0, 32).unwrap();
/// assert!(mesh.vertex_count() > 0);
/// ```
pub fn create_sphere(radius: f64, segments: u32) -> Result<Mesh, MeshError> {
    if radius <= 0.0 {
        return Err(MeshError::degenerate(
            format!("Sphere radius must be positive: {}", radius),
            None,
        ));
    }

    if segments < 3 {
        return Err(MeshError::degenerate(
            format!("Sphere segments must be at least 3: {}", segments),
            None,
        ));
    }

    let num_rings = (segments + 1) / 2;
    let mut mesh = Mesh::new();

    // Generate vertices for each ring
    let mut rings: Vec<Vec<u32>> = Vec::with_capacity(num_rings as usize);

    for i in 0..num_rings {
        // Polar angle (0 = top, PI = bottom)
        // OpenSCAD uses offset formula: phi = PI * (i + 0.5) / num_rings
        let phi = PI * (i as f64 + 0.5) / num_rings as f64;
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let ring_radius = radius * sin_phi;
        let z = radius * cos_phi;

        let mut ring_indices = Vec::with_capacity(segments as usize);

        for j in 0..segments {
            // Azimuthal angle
            let theta = 2.0 * PI * j as f64 / segments as f64;
            let x = ring_radius * theta.cos();
            let y = ring_radius * theta.sin();

            let idx = mesh.add_vertex(DVec3::new(x, y, z));
            ring_indices.push(idx);
        }

        rings.push(ring_indices);
    }

    // Generate triangles

    // Top cap (first ring as polygon fan)
    let first_ring = &rings[0];
    for j in 1..segments - 1 {
        mesh.add_triangle(
            first_ring[0],
            first_ring[j as usize],
            first_ring[(j + 1) as usize],
        );
    }

    // Middle bands (quads between adjacent rings)
    for i in 0..num_rings - 1 {
        let ring_a = &rings[i as usize];
        let ring_b = &rings[(i + 1) as usize];

        for j in 0..segments {
            let j_next = (j + 1) % segments;

            let a0 = ring_a[j as usize];
            let a1 = ring_a[j_next as usize];
            let b0 = ring_b[j as usize];
            let b1 = ring_b[j_next as usize];

            // Two triangles per quad
            mesh.add_triangle(a0, b0, b1);
            mesh.add_triangle(a0, b1, a1);
        }
    }

    // Bottom cap (last ring as polygon fan, reversed)
    let last_ring = &rings[(num_rings - 1) as usize];
    for j in 1..segments - 1 {
        mesh.add_triangle(
            last_ring[0],
            last_ring[(j + 1) as usize],
            last_ring[j as usize],
        );
    }

    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_basic() {
        let mesh = create_sphere(5.0, 16).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_sphere_validates() {
        let mesh = create_sphere(5.0, 16).unwrap();
        assert!(mesh.validate());
    }

    #[test]
    fn test_sphere_bounding_box() {
        let radius = 5.0;
        let mesh = create_sphere(radius, 32).unwrap();
        let (min, max) = mesh.bounding_box();

        // Bounding box should be approximately [-r, -r, -r] to [r, r, r]
        // Allow some tolerance due to tessellation
        let tolerance = radius * 0.1;
        assert!(min.x >= -radius - tolerance);
        assert!(min.y >= -radius - tolerance);
        assert!(min.z >= -radius - tolerance);
        assert!(max.x <= radius + tolerance);
        assert!(max.y <= radius + tolerance);
        assert!(max.z <= radius + tolerance);
    }

    #[test]
    fn test_sphere_invalid_radius() {
        let result = create_sphere(0.0, 16);
        assert!(result.is_err());
    }

    #[test]
    fn test_sphere_negative_radius() {
        let result = create_sphere(-5.0, 16);
        assert!(result.is_err());
    }

    #[test]
    fn test_sphere_too_few_segments() {
        let result = create_sphere(5.0, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_sphere_high_resolution() {
        let mesh = create_sphere(10.0, 64).unwrap();
        assert!(mesh.vertex_count() > 100);
        assert!(mesh.validate());
    }
}

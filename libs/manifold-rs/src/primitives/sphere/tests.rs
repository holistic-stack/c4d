//! Sphere primitive tests.

use super::*;

/// Verifies a valid sphere configuration builds a closed manifold.
///
/// # Examples
/// ```rust
/// use manifold_rs::primitives::sphere::Sphere;
/// let sphere = Sphere::new(10.0, 16).expect("valid configuration");
/// let manifold = sphere.to_manifold().expect("manifold build succeeds");
/// assert!(manifold.validate().is_ok());
/// ```
#[test]
fn sphere_builds_valid_manifold() {
    let sphere = Sphere::new(10.0, 16).expect("valid configuration");
    let manifold = sphere.to_manifold().expect("manifold build succeeds");
    assert!(manifold.validate().is_ok());
}

/// Matches OpenSCAD's latitude/longitude construction where the top/bottom cap height
/// equals `r * cos(φ)` with `φ = 180° * 0.5 / num_rings`.
///
/// # Example
/// ```
/// // OpenSCAD reference: fragments=24 -> rings=12 -> φ=7.5° -> cap height ≈ 0.9914 * r
/// ```
#[test]
fn sphere_latlong_cap_height_matches_openscad() {
    let fragments = 24;
    let radius = 10.0;
    let sphere = Sphere::new(radius, fragments).expect("valid configuration");
    let manifold = sphere.to_manifold().expect("manifold build succeeds");

    let (min, max) = manifold.bounding_box();
    let expected_cap = expected_cap_height(radius, fragments);
    assert!((max.z - expected_cap).abs() < 1e-9);
    assert!((min.z + expected_cap).abs() < 1e-9);
}

/// Guarantees the latitude/longitude tessellation matches OpenSCAD's vertex/triangle counts.
/// The mesh contains `rings * fragments` vertices and the triangles follow the
/// `(rings - 1) * fragments * 2 + 2 * (fragments - 2)` formula emitted by `SphereNode`.
#[test]
fn sphere_latlong_counts_match_openscad() {
    let fragments = 32;
    let sphere = Sphere::new(4.0, fragments).expect("valid configuration");
    let manifold = sphere.to_manifold().expect("manifold build succeeds");

    let rings = (fragments + 1) / 2;
    let expected_vertices = (rings * fragments) as usize;
    assert_eq!(manifold.vertex_count(), expected_vertices);

    let expected_faces = ((rings - 1) * fragments * 2) + (fragments - 2) * 2;
    assert_eq!(manifold.face_count() as u32, expected_faces);
}

/// Rejects invalid radii to prevent degenerate meshes.
#[test]
fn sphere_rejects_non_positive_radius() {
    let err = Sphere::new(0.0, 8).expect_err("radius validation");
    assert!(matches!(err, Error::InvalidTopology(_)));
}

/// Clamps fragment counts to the OpenSCAD minimum (three) when the caller provides
/// insufficient segments, mirroring CurveDiscretizer.
#[test]
fn sphere_clamps_insufficient_segments() {
    let sphere = Sphere::new(1.0, 1).expect("segments are clamped");
    assert_eq!(sphere.fragments(), 3);
}

/// Computes the OpenSCAD-equivalent cap height for a given fragment count.
///
/// # Examples
/// ```
/// use manifold_rs::primitives::sphere::tests::expected_cap_height;
/// assert!((expected_cap_height(1.0, 24) - 0.9914448613738104).abs() < 1e-12);
/// ```
fn expected_cap_height(radius: f64, fragments: u32) -> f64 {
    let rings = (fragments + 1) / 2;
    let phi_deg = 180.0 * 0.5 / rings as f64;
    radius * phi_deg.to_radians().cos()
}

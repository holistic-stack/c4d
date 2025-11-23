use crate::Manifold;
use crate::error::Error;
use crate::primitives::triangulate::manifold_from_contours;
use crate::Vec3;

/// Creates a circle approximation.
pub fn circle(radius: f64, segments: u32) -> Result<Manifold, Error> {
    let segments = segments.max(3); // Minimum 3 segments for a triangle
    let mut points = Vec::with_capacity(segments as usize);

    for i in 0..segments {
        let theta = (i as f64) * 2.0 * std::f64::consts::PI / (segments as f64);
        let x = radius * theta.cos();
        let y = radius * theta.sin();
        points.push(Vec3::new(x, y, 0.0));
    }

    manifold_from_contours(vec![points]).map_err(|e| Error::MeshGeneration(e))
}

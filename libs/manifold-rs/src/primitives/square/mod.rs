use crate::Manifold;
use crate::error::Error;
use glam::DVec2;
use crate::primitives::triangulate::manifold_from_contours;
use crate::Vec3;

/// Creates a square (or rectangle) centered at the origin or in the first quadrant.
pub fn square(size: DVec2, center: bool) -> Result<Manifold, Error> {
    let x = size.x;
    let y = size.y;

    let points = if center {
        vec![
            Vec3::new(-x / 2.0, -y / 2.0, 0.0),
            Vec3::new(x / 2.0, -y / 2.0, 0.0),
            Vec3::new(x / 2.0, y / 2.0, 0.0),
            Vec3::new(-x / 2.0, y / 2.0, 0.0),
        ]
    } else {
        vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(x, 0.0, 0.0),
            Vec3::new(x, y, 0.0),
            Vec3::new(0.0, y, 0.0),
        ]
    };

    manifold_from_contours(vec![points]).map_err(|e| Error::MeshGeneration(e))
}

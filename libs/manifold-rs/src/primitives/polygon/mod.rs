use crate::Manifold;
use crate::error::Error;
use glam::DVec2;
use crate::primitives::triangulate::manifold_from_contours;
use crate::Vec3;

/// Creates a polygon from a list of points and optional paths.
///
/// If paths are not provided, assumes points form a single loop in order.
pub fn polygon(
    points: Vec<DVec2>,
    paths: Option<Vec<Vec<u32>>>,
    _convexity: u32,
) -> Result<Manifold, Error> {
    if points.len() < 3 {
        return Ok(Manifold::new());
    }

    let mut contours = Vec::new();

    if let Some(path_list) = paths {
        for path in path_list {
            let mut contour = Vec::new();
            for idx_u32 in path {
                let idx = idx_u32 as usize;
                if idx >= points.len() {
                    return Err(Error::IndexOutOfBounds(format!("Polygon path index {}", idx)));
                }
                let p = points[idx];
                contour.push(Vec3::new(p.x, p.y, 0.0));
            }
            contours.push(contour);
        }
    } else {
        // Single loop from all points
        let mut contour = Vec::new();
        for p in points {
            contour.push(Vec3::new(p.x, p.y, 0.0));
        }
        contours.push(contour);
    }

    manifold_from_contours(contours).map_err(|e| Error::MeshGeneration(e))
}

pub fn polygon_from_loops(loops: Vec<Vec<Vec3>>) -> Result<Manifold, String> {
    manifold_from_contours(loops)
}

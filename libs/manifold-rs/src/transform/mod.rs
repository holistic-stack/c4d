//! Transform helpers for applying evaluator matrices to manifolds.
//!
//! This module centralizes all transform logic so `from_ir` and future
//! components can share the same implementation, keeping SRP boundaries between
//! evaluation, IR conversion, and geometry kernels.

use glam::DMat4;

use crate::Manifold;

/// Applies a 4×4 affine transform matrix to the provided `Manifold`.
///
/// The helper delegates to `Manifold::transform` and exists primarily to keep
/// the `from_ir` logic focused on flow control.
///
/// # Examples
/// ```
/// use glam::{DMat4, DVec3};
/// use manifold_rs::{primitives::cube::cube, Vec3, transform::apply_transform};
///
/// let mut cube = cube(Vec3::splat(1.0), false).unwrap();
/// apply_transform(&mut cube, DMat4::from_translation(DVec3::new(5.0, 0.0, 0.0)));
/// let (min, max) = cube.bounding_box();
/// assert_eq!(min.x, 5.0);
/// assert_eq!(max.x, 6.0);
/// ```
pub fn apply_transform(manifold: &mut Manifold, matrix: DMat4) {
    manifold.transform(matrix);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::cube::cube;
    use crate::Vec3;

    #[test]
    fn translate_updates_bounding_box() {
        let mut mesh = cube(Vec3::splat(1.0), false).expect("cube");
        apply_transform(&mut mesh, DMat4::from_translation(glam::DVec3::new(2.0, 0.0, 0.0)));
        let (min, max) = mesh.bounding_box();
        assert_eq!(min, Vec3::new(2.0, 0.0, 0.0));
        assert_eq!(max, Vec3::new(3.0, 1.0, 1.0));
    }

    #[test]
    fn scale_preserves_vertex_count() {
        let mut mesh = cube(Vec3::new(1.0, 2.0, 3.0), false).expect("cube");
        apply_transform(&mut mesh, DMat4::from_scale(glam::DVec3::new(2.0, 3.0, 4.0)));
        assert_eq!(mesh.vertex_count(), 8);
    }
}

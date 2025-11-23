//! Resize operation for Manifold.

use crate::{Manifold, Vec3};
use glam::DMat4;

/// Resizes the manifold to the specified dimensions.
///
/// If a dimension is 0 (or auto), it scales proportionally to the other dimensions.
///
/// # Arguments
///
/// * `manifold` - The manifold to resize.
/// * `new_size` - The target size (X, Y, Z).
/// * `auto` - Flags indicating which dimensions should be auto-scaled.
pub fn resize(manifold: &mut Manifold, new_size: Vec3, auto: [bool; 3]) {
    let (min, max) = manifold.bounding_box();
    let current_size = max - min;

    // Handle empty mesh or point
    if current_size.x.abs() < 1e-9 && current_size.y.abs() < 1e-9 && current_size.z.abs() < 1e-9 {
        return;
    }

    let mut scale = Vec3::ONE;
    let mut auto_scale = 0.0;
    let mut auto_count = 0;

    // Calculate explicit scales
    for i in 0..3 {
        let size_dim = match i {
            0 => current_size.x,
            1 => current_size.y,
            2 => current_size.z,
            _ => 1.0,
        };
        let target_dim = match i {
            0 => new_size.x,
            1 => new_size.y,
            2 => new_size.z,
            _ => 0.0,
        };

        if !auto[i] && size_dim.abs() > 1e-9 {
            let s = target_dim / size_dim;
            match i {
                0 => scale.x = s,
                1 => scale.y = s,
                2 => scale.z = s,
                _ => {},
            }
            auto_scale += s;
            auto_count += 1;
        }
    }

    if auto_count > 0 {
        auto_scale /= auto_count as f64;
    } else {
        auto_scale = 1.0;
    }

    // Apply auto scales
    if auto[0] { scale.x = auto_scale; }
    if auto[1] { scale.y = auto_scale; }
    if auto[2] { scale.z = auto_scale; }

    manifold.transform(DMat4::from_scale(scale));
}

#[cfg(test)]
mod tests;

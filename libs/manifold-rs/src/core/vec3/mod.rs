//! Core vector types for geometry operations
//! 
//! This module provides type aliases and utilities for 3D geometry,
//! using glam's DVec3 for f64 precision as required by the architecture.

use glam::DVec3;

/// 3D vector with f64 precision
/// 
/// This is the primary vector type used throughout the manifold-rs library.
/// It provides double precision for accurate geometric computations.
pub type Vec3 = DVec3;

/// 2D vector with f64 precision
/// 
/// Used for 2D operations like polygon processing and 2D transformations.
pub type Vec2 = glam::DVec2;

/// 4x4 transformation matrix with f64 precision
/// 
/// Used for affine transformations (translation, rotation, scaling).
pub type Mat4 = glam::DMat4;

/// Quaternion with f64 precision
/// 
/// Used for rotation representations and spherical linear interpolation.
pub type Quat = glam::DQuat;

/// Axis-aligned bounding box
/// 
/// Represents a 3D bounding box with minimum and maximum coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    /// Creates a new bounding box from min and max coordinates
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
    
    /// Creates an empty bounding box
    pub fn empty() -> Self {
        Self {
            min: Vec3::splat(f64::INFINITY),
            max: Vec3::splat(f64::NEG_INFINITY),
        }
    }
    
    /// Expands the bounding box to include a point
    pub fn expand(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }
    
    /// Returns the center of the bounding box
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
    
    /// Returns the size of the bounding box
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
    
    /// Checks if the bounding box is valid (min <= max)
    pub fn is_valid(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y && self.min.z <= self.max.z
    }
    
    /// Checks if a point is inside the bounding box
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::empty()
    }
}

/// Utility functions for vector operations
pub mod utils {
    use super::*;
    use crate::config::EPSILON;
    
    /// Checks if two vectors are approximately equal within epsilon
    pub fn approx_eq(a: Vec3, b: Vec3) -> bool {
        (a - b).length_squared() < EPSILON * EPSILON
    }
    
    /// Computes the normal of a triangle given three vertices
    pub fn triangle_normal(v0: Vec3, v1: Vec3, v2: Vec3) -> Vec3 {
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        e1.cross(e2).normalize()
    }
    
    /// Computes the area of a triangle given three vertices
    pub fn triangle_area(v0: Vec3, v1: Vec3, v2: Vec3) -> f64 {
        let e1 = v1 - v0;
        let e2 = v2 - v0;
        e1.cross(e2).length() * 0.5
    }
    
    /// Computes the centroid of a triangle
    pub fn triangle_centroid(v0: Vec3, v1: Vec3, v2: Vec3) -> Vec3 {
        (v0 + v1 + v2) / 3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;

    #[test]
    fn test_vec3_type_alias() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn test_bounding_box_creation() {
        let min = Vec3::new(0.0, 0.0, 0.0);
        let max = Vec3::new(10.0, 10.0, 10.0);
        let bbox = BoundingBox::new(min, max);
        
        assert_eq!(bbox.min, min);
        assert_eq!(bbox.max, max);
        assert!(bbox.is_valid());
    }

    #[test]
    fn test_bounding_box_expansion() {
        let mut bbox = BoundingBox::empty();
        assert!(!bbox.is_valid());
        
        bbox.expand(Vec3::new(1.0, 2.0, 3.0));
        bbox.expand(Vec3::new(-1.0, -2.0, -3.0));
        
        assert_eq!(bbox.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(bbox.max, Vec3::new(1.0, 2.0, 3.0));
        assert!(bbox.is_valid());
    }

    #[test]
    fn test_bounding_box_center() {
        let bbox = BoundingBox::new(
            Vec3::new(-1.0, -2.0, -3.0),
            Vec3::new(1.0, 2.0, 3.0)
        );
        assert_eq!(bbox.center(), Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_triangle_normal() {
        let v0 = Vec3::new(0.0, 0.0, 0.0);
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        
        let normal = triangle_normal(v0, v1, v2);
        assert_eq!(normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_triangle_area() {
        let v0 = Vec3::new(0.0, 0.0, 0.0);
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        
        let area = triangle_area(v0, v1, v2);
        assert_eq!(area, 0.5);
    }
}
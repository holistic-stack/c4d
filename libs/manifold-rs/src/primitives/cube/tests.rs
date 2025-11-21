/// Tests for the cube primitive.
///
/// These tests verify that the cube primitive:
/// - Creates the correct number of vertices and faces
/// - Produces a valid manifold topology
/// - Handles centered vs non-centered positioning
/// - Rejects invalid inputs

#[cfg(test)]
mod tests {
    use crate::core::vec3::Vec3;
    use crate::primitives::cube::cube;
    use crate::ManifoldError;

    /// Test that a unit cube has 8 vertices.
    ///
    /// A cube should have exactly 8 corner vertices.
    #[test]
    fn test_cube_has_8_vertices() {
        let c = cube(Vec3::new(1.0, 1.0, 1.0), false).unwrap();
        assert_eq!(c.vertex_count(), 8);
    }

    /// Test that a cube has 12 triangular faces (2 per cube face).
    ///
    /// A cube has 6 faces, each triangulated into 2 triangles = 12 total.
    #[test]
    fn test_cube_has_12_faces() {
        let c = cube(Vec3::new(1.0, 1.0, 1.0), false).unwrap();
        assert_eq!(c.face_count(), 12);
    }

    /// Test that the cube passes validation.
    ///
    /// The half-edge structure should be topologically valid.
    #[test]
    fn test_cube_validates() {
        let c = cube(Vec3::new(1.0, 1.0, 1.0), false).unwrap();
        assert!(c.validate().is_ok());
    }

    /// Test bounding box for non-centered cube.
    ///
    /// A non-centered cube with size [2, 3, 4] should have:
    /// - min corner at origin (0, 0, 0)
    /// - max corner at (2, 3, 4)
    #[test]
    fn test_cube_bounding_box_non_centered() {
        let c = cube(Vec3::new(2.0, 3.0, 4.0), false).unwrap();
        let (min, max) = c.bounding_box();
        
        assert_eq!(min, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(max, Vec3::new(2.0, 3.0, 4.0));
    }

    /// Test bounding box for centered cube.
    ///
    /// A centered cube with size [2, 2, 2] should have:
    /// - min corner at (-1, -1, -1)
    /// - max corner at (1, 1, 1)
    #[test]
    fn test_cube_bounding_box_centered() {
        let c = cube(Vec3::new(2.0, 2.0, 2.0), true).unwrap();
        let (min, max) = c.bounding_box();
        
        assert_eq!(min, Vec3::new(-1.0, -1.0, -1.0));
        assert_eq!(max, Vec3::new(1.0, 1.0, 1.0));
    }

    /// Test that cube rejects negative size.
    ///
    /// Negative dimensions are invalid and should return an error.
    #[test]
    fn test_cube_rejects_negative_size() {
        let result = cube(Vec3::new(-1.0, 1.0, 1.0), false);
        assert!(matches!(result, Err(ManifoldError::InvalidTopology(_))));
    }

    /// Test that cube rejects zero size.
    ///
    /// Zero dimensions are invalid and should return an error.
    #[test]
    fn test_cube_rejects_zero_size() {
        let result = cube(Vec3::new(0.0, 1.0, 1.0), false);
        assert!(matches!(result, Err(ManifoldError::InvalidTopology(_))));
    }

    /// Test that all 24 half-edges exist (12 triangles * 3 edges / 2 for pairing).
    ///
    /// Each triangle has 3 edges, and edges are shared between triangles.
    /// 12 triangles * 3 edges = 36 half-edges total.
    #[test]
    fn test_cube_has_correct_edge_count() {
        let c = cube(Vec3::new(1.0, 1.0, 1.0), false).unwrap();
        // 12 faces * 3 edges per face = 36 half-edges
        assert_eq!(c.half_edges.len(), 36);
    }
}

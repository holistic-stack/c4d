//! Geometric primitives for manifold construction
//!
//! This module provides basic geometric primitives like cubes, spheres, cylinders,
//! and other shapes that can be used to construct more complex manifolds.

use crate::core::{HalfEdgeMesh, Vec3, TopologyResult, HalfEdgeId};

/// Cube primitive with configurable dimensions and center
#[derive(Debug, Clone, PartialEq)]
pub struct Cube {
    /// Center position of the cube
    pub center: Vec3,
    /// Size of the cube (width, height, depth)
    pub size: Vec3,
}

impl Cube {
    /// Creates a new cube centered at the origin with unit size
    pub fn new() -> Self {
        Self {
            center: Vec3::new(0.0, 0.0, 0.0),
            size: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// Creates a cube with the specified size, centered at origin
    pub fn from_size(size: Vec3) -> Self {
        Self {
            center: Vec3::new(0.0, 0.0, 0.0),
            size,
        }
    }

    /// Creates a cube with the specified center and size
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        Self { center, size }
    }

    /// Sets the center position of the cube
    pub fn with_center(mut self, center: Vec3) -> Self {
        self.center = center;
        self
    }

    /// Sets the size of the cube
    pub fn with_size(mut self, size: Vec3) -> Self {
        self.size = size;
        self
    }

    /// Converts this cube to a half-edge mesh
    pub fn to_mesh(&self) -> TopologyResult<HalfEdgeMesh> {
        let mut mesh = HalfEdgeMesh::new();

        // Calculate half-extents
        let half_size = self.size * 0.5;
        
        // Create 8 vertices for the cube
        let vertices = [
            // Bottom face (y = -half_size.y)
            mesh.add_vertex(Vec3::new(-half_size.x, -half_size.y, -half_size.z) + self.center), // 0
            mesh.add_vertex(Vec3::new( half_size.x, -half_size.y, -half_size.z) + self.center), // 1
            mesh.add_vertex(Vec3::new( half_size.x, -half_size.y,  half_size.z) + self.center), // 2
            mesh.add_vertex(Vec3::new(-half_size.x, -half_size.y,  half_size.z) + self.center), // 3
            
            // Top face (y = half_size.y)
            mesh.add_vertex(Vec3::new(-half_size.x,  half_size.y, -half_size.z) + self.center), // 4
            mesh.add_vertex(Vec3::new( half_size.x,  half_size.y, -half_size.z) + self.center), // 5
            mesh.add_vertex(Vec3::new( half_size.x,  half_size.y,  half_size.z) + self.center), // 6
            mesh.add_vertex(Vec3::new(-half_size.x,  half_size.y,  half_size.z) + self.center), // 7
        ];

        // Create 6 faces (12 triangles, 2 per face)
        let faces = vec![
            // Bottom face (2 triangles)
            (0, 1, 2), (0, 2, 3),
            // Top face (2 triangles)
            (4, 7, 6), (4, 6, 5),
            // Front face (2 triangles)
            (0, 4, 5), (0, 5, 1),
            // Back face (2 triangles)
            (2, 6, 7), (2, 7, 3),
            // Left face (2 triangles)
            (0, 3, 7), (0, 7, 4),
            // Right face (2 triangles)
            (1, 5, 6), (1, 6, 2),
        ];

        // Create faces and half-edges
        for (_i, &(v0_idx, v1_idx, v2_idx)) in faces.iter().enumerate() {
            let face_id = mesh.add_face();
            let edge_id = mesh.add_edge(HalfEdgeId(0), HalfEdgeId(0)); // Will be updated
            
            let v0 = vertices[v0_idx];
            let v1 = vertices[v1_idx];
            let v2 = vertices[v2_idx];
            
            // Create three half-edges for this triangle
            let he0 = mesh.add_half_edge(v0, edge_id);
            let he1 = mesh.add_half_edge(v1, edge_id);
            let he2 = mesh.add_half_edge(v2, edge_id);
            
            // Set up connectivity
            mesh.half_edge_mut(he0).unwrap().next = Some(he1);
            mesh.half_edge_mut(he1).unwrap().next = Some(he2);
            mesh.half_edge_mut(he2).unwrap().next = Some(he0);
            
            mesh.half_edge_mut(he0).unwrap().face = Some(face_id);
            mesh.half_edge_mut(he1).unwrap().face = Some(face_id);
            mesh.half_edge_mut(he2).unwrap().face = Some(face_id);
            
            mesh.face_mut(face_id).unwrap().halfedge = Some(he0);
            
            // Update edge to point to correct half-edges
            mesh.edge_mut(edge_id).unwrap().halfedges = [he0, he1]; // Simplified for now
        }

        // Validate the mesh topology
        mesh.validate_topology()?;
        
        Ok(mesh)
    }
}

impl Default for Cube {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::VertexId;

    #[test]
    fn test_cube_default() {
        let cube = Cube::default();
        assert_eq!(cube.center, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(cube.size, Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_cube_new() {
        let cube = Cube::new();
        assert_eq!(cube.center, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(cube.size, Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_cube_from_size() {
        let size = Vec3::new(2.0, 3.0, 4.0);
        let cube = Cube::from_size(size);
        assert_eq!(cube.center, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(cube.size, size);
    }

    #[test]
    fn test_cube_from_center_size() {
        let center = Vec3::new(1.0, 2.0, 3.0);
        let size = Vec3::new(4.0, 5.0, 6.0);
        let cube = Cube::from_center_size(center, size);
        assert_eq!(cube.center, center);
        assert_eq!(cube.size, size);
    }

    #[test]
    fn test_cube_builder_pattern() {
        let cube = Cube::new()
            .with_center(Vec3::new(1.0, 2.0, 3.0))
            .with_size(Vec3::new(4.0, 5.0, 6.0));
        
        assert_eq!(cube.center, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(cube.size, Vec3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_cube_to_mesh_basic() {
        let cube = Cube::new();
        let mesh = cube.to_mesh().unwrap();
        
        // A cube should have 8 vertices
        assert_eq!(mesh.vertex_count(), 8);
        
        // A cube should have 6 faces (12 triangles, 2 per face)
        assert_eq!(mesh.face_count(), 12);
        
        // Validate mesh topology
        assert!(mesh.validate_topology().is_ok());
    }

    #[test]
    fn test_cube_to_mesh_custom_size() {
        let cube = Cube::from_size(Vec3::new(2.0, 3.0, 4.0));
        let mesh = cube.to_mesh().unwrap();
        
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.face_count(), 12);
        assert!(mesh.validate_topology().is_ok());
        
        // Check that vertices span the expected range
        let bbox = mesh.bounding_box();
        assert_eq!(bbox.min, Vec3::new(-1.0, -1.5, -2.0));
        assert_eq!(bbox.max, Vec3::new(1.0, 1.5, 2.0));
    }

    #[test]
    fn test_cube_to_mesh_custom_center() {
        let center = Vec3::new(5.0, 6.0, 7.0);
        let cube = Cube::from_center_size(center, Vec3::new(2.0, 2.0, 2.0));
        let mesh = cube.to_mesh().unwrap();
        
        let bbox = mesh.bounding_box();
        assert_eq!(bbox.center(), center);
    }

    #[test]
    fn test_cube_to_mesh_unit_cube_vertices() {
        let cube = Cube::new();
        let mesh = cube.to_mesh().unwrap();
        
        // Check that vertices are in expected positions
        let expected_vertices = [
            Vec3::new(-0.5, -0.5, -0.5), // 0
            Vec3::new( 0.5, -0.5, -0.5), // 1
            Vec3::new( 0.5, -0.5,  0.5), // 2
            Vec3::new(-0.5, -0.5,  0.5), // 3
            Vec3::new(-0.5,  0.5, -0.5), // 4
            Vec3::new( 0.5,  0.5, -0.5), // 5
            Vec3::new( 0.5,  0.5,  0.5), // 6
            Vec3::new(-0.5,  0.5,  0.5), // 7
        ];
        
        for (i, &expected_pos) in expected_vertices.iter().enumerate() {
            let vertex = mesh.vertex(VertexId(i as u32)).unwrap();
            assert_eq!(vertex.position, expected_pos, "Vertex {} position mismatch", i);
        }
    }
}
//! # IR to Mesh Conversion
//!
//! Converts Geometry IR from openscad-eval into triangle meshes.

use crate::error::MeshError;
use crate::mesh::Mesh;
use crate::ops::{boolean, extrude, hull, minkowski, offset};
use crate::ops::extrude::{LinearExtrudeParams, Polygon2D, RotateExtrudeParams};
use crate::ops::offset::OffsetParams;
use crate::primitives::{create_cube, create_cylinder, create_sphere};
use glam::DVec2;
use openscad_eval::ir::{BooleanOperation, GeometryNode, OffsetAmount};

/// Converts a list of geometry nodes to a single mesh.
///
/// Multiple top-level nodes are combined via union.
pub fn geometry_to_mesh(nodes: &[GeometryNode]) -> Result<Mesh, MeshError> {
    if nodes.is_empty() {
        return Ok(Mesh::new());
    }

    if nodes.len() == 1 {
        return node_to_mesh(&nodes[0]);
    }

    // Multiple nodes: merge them
    let mut result = Mesh::new();
    for node in nodes {
        let mesh = node_to_mesh(node)?;
        result.merge(&mesh);
    }
    Ok(result)
}

/// Converts a single geometry node to a mesh.
pub fn node_to_mesh(node: &GeometryNode) -> Result<Mesh, MeshError> {
    match node {
        // 3D Primitives
        GeometryNode::Cube { size, center, span } => {
            create_cube(*size, *center).map_err(|e| match e {
                MeshError::DegenerateGeometry { message, .. } => {
                    MeshError::DegenerateGeometry {
                        message,
                        span: Some(*span),
                    }
                }
                other => other,
            })
        }

        GeometryNode::Sphere {
            radius, segments, span,
        } => create_sphere(*radius, *segments).map_err(|e| match e {
            MeshError::DegenerateGeometry { message, .. } => {
                MeshError::DegenerateGeometry {
                    message,
                    span: Some(*span),
                }
            }
            other => other,
        }),

        GeometryNode::Cylinder {
            height,
            radius_bottom,
            radius_top,
            center,
            segments,
            span,
        } => create_cylinder(*height, *radius_bottom, *radius_top, *center, *segments)
            .map_err(|e| match e {
                MeshError::DegenerateGeometry { message, .. } => {
                    MeshError::DegenerateGeometry {
                        message,
                        span: Some(*span),
                    }
                }
                other => other,
            }),

        GeometryNode::Polyhedron { points, faces, span, .. } => {
            create_polyhedron(points, faces).map_err(|e| match e {
                MeshError::InvalidTopology { message, .. } => {
                    MeshError::InvalidTopology {
                        message,
                        span: Some(*span),
                    }
                }
                other => other,
            })
        }

        // 2D Primitives (return empty mesh for now - need extrusion)
        GeometryNode::Square { span, .. }
        | GeometryNode::Circle { span, .. }
        | GeometryNode::Polygon { span, .. } => {
            Err(MeshError::unsupported(
                "2D primitives require extrusion for 3D rendering",
                Some(*span),
            ))
        }

        // Transformations
        GeometryNode::Transform {
            matrix, children, ..
        } => {
            let mut mesh = geometry_to_mesh(children)?;
            mesh.transform(matrix);
            Ok(mesh)
        }

        GeometryNode::Color { color, children, .. } => {
            let mut mesh = geometry_to_mesh(children)?;
            mesh.set_uniform_color(*color);
            Ok(mesh)
        }

        // Boolean Operations (using BSP trees)
        GeometryNode::Boolean {
            operation,
            children,
            span: _,
        } => {
            if children.is_empty() {
                return Ok(Mesh::new());
            }

            // Convert all children to meshes
            let meshes: Result<Vec<Mesh>, MeshError> = children
                .iter()
                .map(node_to_mesh)
                .collect();
            let meshes = meshes?;

            // Apply boolean operation
            let mut iter = meshes.into_iter();
            let mut result = iter.next().unwrap_or_default();

            match operation {
                BooleanOperation::Union => {
                    for mesh in iter {
                        result = boolean::union(&result, &mesh)?;
                    }
                }
                BooleanOperation::Difference => {
                    for mesh in iter {
                        result = boolean::difference(&result, &mesh)?;
                    }
                }
                BooleanOperation::Intersection => {
                    for mesh in iter {
                        result = boolean::intersection(&result, &mesh)?;
                    }
                }
            }
            Ok(result)
        }

        // Extrusions
        GeometryNode::LinearExtrude {
            height,
            center,
            twist,
            slices,
            scale,
            children,
            span,
        } => {
            // Convert 2D children to Polygon2D
            let polygon = children_to_polygon2d(children, *span)?;
            
            let params = LinearExtrudeParams {
                height: *height,
                center: *center,
                twist: *twist,
                slices: *slices,
                scale: *scale,
            };
            
            extrude::linear_extrude(&polygon, &params).map_err(|e| match e {
                MeshError::DegenerateGeometry { message, .. } => {
                    MeshError::DegenerateGeometry {
                        message,
                        span: Some(*span),
                    }
                }
                other => other,
            })
        }

        GeometryNode::RotateExtrude {
            angle,
            convexity: _,
            children,
            span,
        } => {
            // Convert 2D children to Polygon2D
            let polygon = children_to_polygon2d(children, *span)?;
            
            let params = RotateExtrudeParams {
                angle: *angle,
                segments: 32, // Default segments for rotation
            };
            
            extrude::rotate_extrude(&polygon, &params).map_err(|e| match e {
                MeshError::DegenerateGeometry { message, .. } => {
                    MeshError::DegenerateGeometry {
                        message,
                        span: Some(*span),
                    }
                }
                other => other,
            })
        }

        // Advanced Operations
        GeometryNode::Hull { children, span } => {
            if children.is_empty() {
                return Ok(Mesh::new());
            }

            // Convert all children to meshes
            let meshes: Result<Vec<Mesh>, MeshError> = children
                .iter()
                .map(node_to_mesh)
                .collect();
            let meshes = meshes?;

            // Compute hull of all meshes
            let mesh_refs: Vec<&Mesh> = meshes.iter().collect();
            hull::hull(&mesh_refs).map_err(|e| match e {
                MeshError::DegenerateGeometry { message, .. } => {
                    MeshError::DegenerateGeometry {
                        message,
                        span: Some(*span),
                    }
                }
                other => other,
            })
        }

        GeometryNode::Minkowski { children, convexity: _, span } => {
            if children.is_empty() {
                return Ok(Mesh::new());
            }

            // Convert all children to meshes
            let meshes: Result<Vec<Mesh>, MeshError> = children
                .iter()
                .map(node_to_mesh)
                .collect();
            let meshes = meshes?;

            // Compute Minkowski sum of all meshes
            let mesh_refs: Vec<&Mesh> = meshes.iter().collect();
            minkowski::minkowski(&mesh_refs).map_err(|e| match e {
                MeshError::DegenerateGeometry { message, .. } => {
                    MeshError::DegenerateGeometry {
                        message,
                        span: Some(*span),
                    }
                }
                other => other,
            })
        }

        GeometryNode::Offset { amount, chamfer, children, span } => {
            if children.is_empty() {
                return Ok(Mesh::new());
            }

            // Convert 2D children to Polygon2D
            let polygon = children_to_polygon2d(children, *span)?;

            // Convert OffsetAmount to offset value
            let offset_amount = match amount {
                OffsetAmount::Radius(r) => *r,
                OffsetAmount::Delta(d) => *d,
            };

            let params = OffsetParams {
                amount: offset_amount,
                chamfer: *chamfer,
            };

            // Apply offset to the polygon
            let offset_polygon = offset::offset_polygon(&polygon, &params)
                .map_err(|msg| MeshError::degenerate(&msg, Some(*span)))?;

            // Extrude the offset polygon to create a 3D mesh
            // Note: In OpenSCAD, offset is typically used with linear_extrude
            // For standalone offset, we create a thin 3D representation
            let extrude_params = LinearExtrudeParams {
                height: 0.01, // Minimal height for 2D representation
                center: true,
                twist: 0.0,
                slices: 1,
                scale: [1.0, 1.0],
            };

            extrude::linear_extrude(&offset_polygon, &extrude_params).map_err(|e| match e {
                MeshError::DegenerateGeometry { message, .. } => {
                    MeshError::DegenerateGeometry {
                        message,
                        span: Some(*span),
                    }
                }
                other => other,
            })
        }

        GeometryNode::Resize { children, .. } => {
            // For now, just return children without resizing
            geometry_to_mesh(children)
        }

        GeometryNode::Empty { .. } => Ok(Mesh::new()),
    }
}

/// Converts 2D geometry children to a Polygon2D for extrusion.
///
/// Handles Square, Circle, and Polygon 2D primitives.
/// Also handles wrapper nodes like Union and Translate to extract the inner 2D shape.
///
/// # Arguments
/// * `children` - The geometry nodes to extract 2D profile from
/// * `span` - Source location for error reporting
///
/// # Returns
/// A Polygon2D representing the 2D profile for extrusion
fn children_to_polygon2d(children: &[GeometryNode], span: openscad_ast::Span) -> Result<Polygon2D, MeshError> {
    if children.is_empty() {
        return Err(MeshError::degenerate(
            "Extrusion requires at least one 2D child",
            Some(span),
        ));
    }

    // For now, only handle the first child
    // TODO: Support multiple children (union of 2D shapes)
    let child = &children[0];

    extract_2d_polygon(child, span)
}

/// Recursively extracts a 2D polygon from a geometry node.
///
/// Unwraps wrapper nodes (Union, Translate, etc.) to find the inner 2D primitive.
///
/// # Arguments
/// * `node` - The geometry node to extract from
/// * `span` - Source location for error reporting
fn extract_2d_polygon(node: &GeometryNode, span: openscad_ast::Span) -> Result<Polygon2D, MeshError> {
    match node {
        // Direct 2D primitives
        GeometryNode::Square { size, center, .. } => {
            Ok(Polygon2D::square(DVec2::new(size[0], size[1]), *center))
        }

        GeometryNode::Circle { radius, segments, .. } => {
            Ok(Polygon2D::circle(*radius, *segments))
        }

        GeometryNode::Polygon { points, paths: _, .. } => {
            let vertices: Vec<DVec2> = points
                .iter()
                .map(|p| DVec2::new(p[0], p[1]))
                .collect();
            
            if vertices.len() < 3 {
                return Err(MeshError::degenerate(
                    "Polygon must have at least 3 vertices",
                    Some(span),
                ));
            }
            
            Ok(Polygon2D::new(vertices))
        }

        // Wrapper nodes - unwrap and recurse
        GeometryNode::Boolean { children, .. } => {
            if children.is_empty() {
                return Err(MeshError::degenerate(
                    "Boolean has no children for extrusion",
                    Some(span),
                ));
            }
            // TODO: Support union of multiple 2D shapes
            extract_2d_polygon(&children[0], span)
        }

        GeometryNode::Transform { matrix, children, .. } => {
            if children.is_empty() {
                return Err(MeshError::degenerate(
                    "Transform has no children for extrusion",
                    Some(span),
                ));
            }
            // Extract the 2D polygon and apply transformation
            // For 2D, we only use the translation component (w_axis.x and w_axis.y)
            let mut polygon = extract_2d_polygon(&children[0], span)?;
            let translation = DVec2::new(matrix.w_axis.x, matrix.w_axis.y);
            polygon.translate(translation);
            Ok(polygon)
        }

        _ => Err(MeshError::unsupported(
            "Extrusion requires 2D primitives (square, circle, polygon)",
            Some(span),
        )),
    }
}

/// Creates a polyhedron mesh from points and faces.
fn create_polyhedron(
    points: &[glam::DVec3],
    faces: &[Vec<u32>],
) -> Result<Mesh, MeshError> {
    if points.is_empty() {
        return Err(MeshError::invalid_topology("Polyhedron has no points", None));
    }

    if faces.is_empty() {
        return Err(MeshError::invalid_topology("Polyhedron has no faces", None));
    }

    let mut mesh = Mesh::with_capacity(points.len(), faces.len() * 2);

    // Add all vertices
    for point in points {
        mesh.add_vertex(*point);
    }

    // Add faces (triangulate if needed)
    for face in faces {
        if face.len() < 3 {
            return Err(MeshError::invalid_topology(
                format!("Face has fewer than 3 vertices: {:?}", face),
                None,
            ));
        }

        // Validate indices
        for &idx in face {
            if idx as usize >= points.len() {
                return Err(MeshError::invalid_topology(
                    format!("Face index {} out of range (max: {})", idx, points.len() - 1),
                    None,
                ));
            }
        }

        // Fan triangulation for polygons with more than 3 vertices
        // OpenSCAD reverses face winding, so we do too
        for i in 1..face.len() - 1 {
            mesh.add_triangle(
                face[0],
                face[i + 1] as u32,
                face[i] as u32,
            );
        }
    }

    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec3;
    use openscad_ast::Span;

    #[test]
    fn test_cube_to_mesh() {
        let node = GeometryNode::Cube {
            size: DVec3::splat(10.0),
            center: false,
            span: Span::default(),
        };
        let mesh = node_to_mesh(&node).unwrap();
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_sphere_to_mesh() {
        let node = GeometryNode::Sphere {
            radius: 5.0,
            segments: 16,
            span: Span::default(),
        };
        let mesh = node_to_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_transform_to_mesh() {
        let node = GeometryNode::Transform {
            matrix: glam::DMat4::from_translation(DVec3::new(10.0, 0.0, 0.0)),
            children: vec![GeometryNode::Cube {
                size: DVec3::splat(5.0),
                center: false,
                span: Span::default(),
            }],
            span: Span::default(),
        };
        let mesh = node_to_mesh(&node).unwrap();
        let (min, max) = mesh.bounding_box();
        assert!(min.x >= 10.0);
        assert!(max.x <= 15.0);
    }

    #[test]
    fn test_union_to_mesh() {
        let node = GeometryNode::Boolean {
            operation: BooleanOperation::Union,
            children: vec![
                GeometryNode::Cube {
                    size: DVec3::splat(5.0),
                    center: false,
                    span: Span::default(),
                },
                GeometryNode::Sphere {
                    radius: 3.0,
                    segments: 16,
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };
        let mesh = node_to_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 8); // More than just cube
    }

    #[test]
    fn test_empty_geometry() {
        let nodes: Vec<GeometryNode> = vec![];
        let mesh = geometry_to_mesh(&nodes).unwrap();
        assert!(mesh.is_empty());
    }

    #[test]
    fn test_linear_extrude_square() {
        let node = GeometryNode::LinearExtrude {
            height: 10.0,
            center: false,
            twist: 0.0,
            slices: 1,
            scale: [1.0, 1.0],
            children: vec![GeometryNode::Square {
                size: [5.0, 5.0],
                center: false,
                span: Span::default(),
            }],
            span: Span::default(),
        };
        let mesh = node_to_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
        
        // Check bounding box
        let (min, max) = mesh.bounding_box();
        assert!(min.z >= 0.0);
        assert!(max.z <= 10.0);
    }

    #[test]
    fn test_linear_extrude_circle() {
        let node = GeometryNode::LinearExtrude {
            height: 20.0,
            center: true,
            twist: 0.0,
            slices: 1,
            scale: [1.0, 1.0],
            children: vec![GeometryNode::Circle {
                radius: 5.0,
                segments: 32,
                span: Span::default(),
            }],
            span: Span::default(),
        };
        let mesh = node_to_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
        
        // Check centered
        let (min, max) = mesh.bounding_box();
        assert!((min.z - (-10.0)).abs() < 0.1);
        assert!((max.z - 10.0).abs() < 0.1);
    }

    #[test]
    fn test_rotate_extrude_square() {
        let node = GeometryNode::RotateExtrude {
            angle: 360.0,
            convexity: 1,
            children: vec![GeometryNode::Square {
                size: [2.0, 4.0],
                center: false,
                span: Span::default(),
            }],
            span: Span::default(),
        };
        let mesh = node_to_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
    }

    #[test]
    fn test_offset_square() {
        let node = GeometryNode::Offset {
            amount: OffsetAmount::Delta(1.0),
            chamfer: false,
            children: vec![GeometryNode::Square {
                size: [10.0, 10.0],
                center: true,
                span: Span::default(),
            }],
            span: Span::default(),
        };
        let mesh = node_to_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
        assert!(mesh.triangle_count() > 0);
        
        // Check that offset expanded the square
        let (_min, max) = mesh.bounding_box();
        assert!(max.x > 5.0, "Offset should expand the square");
    }

    #[test]
    fn test_hull_two_cubes() {
        let node = GeometryNode::Hull {
            children: vec![
                GeometryNode::Cube {
                    size: DVec3::splat(5.0),
                    center: false,
                    span: Span::default(),
                },
                GeometryNode::Transform {
                    matrix: glam::DMat4::from_translation(DVec3::new(10.0, 0.0, 0.0)),
                    children: vec![GeometryNode::Cube {
                        size: DVec3::splat(5.0),
                        center: false,
                        span: Span::default(),
                    }],
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };
        let mesh = node_to_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
        
        // Hull should span from 0 to 15
        let (min, max) = mesh.bounding_box();
        assert!(min.x <= 0.0);
        assert!(max.x >= 15.0);
    }

    #[test]
    fn test_minkowski_cube_sphere() {
        let node = GeometryNode::Minkowski {
            convexity: 1,
            children: vec![
                GeometryNode::Cube {
                    size: DVec3::splat(4.0),
                    center: true,
                    span: Span::default(),
                },
                GeometryNode::Sphere {
                    radius: 1.0,
                    segments: 8,
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };
        let mesh = node_to_mesh(&node).unwrap();
        assert!(mesh.vertex_count() > 0);
        
        // Minkowski should expand the cube by the sphere radius
        let (min, max) = mesh.bounding_box();
        assert!(min.x < -2.0, "Minkowski should expand beyond cube");
        assert!(max.x > 2.0, "Minkowski should expand beyond cube");
    }
}

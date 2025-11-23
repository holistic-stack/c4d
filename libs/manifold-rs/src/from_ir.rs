/// Conversion from IR to Manifold mesh.
///
/// This module bridges the evaluator and the geometry kernel.

use crate::{MeshBuffers, Vec3, Manifold, BooleanOp};
use crate::primitives::cube::cube;
use crate::primitives::sphere::Sphere;
use crate::primitives::square::square;
use crate::primitives::circle::circle;
use crate::primitives::polygon::polygon;
use crate::primitives::cylinder::cylinder;
use crate::primitives::polyhedron::polyhedron;
use crate::ops::resize::resize;
use crate::ops::extrude::linear_extrude;
use crate::ops::revolve::rotate_extrude;
use crate::ops::hull::hull;
use crate::ops::minkowski::minkowski;
use crate::ops::boolean::boolean;
use crate::transform::apply_transform;
use openscad_eval::{Evaluator, InMemoryFilesystem, GeometryNode, EvaluationError};
use openscad_ast::{Diagnostic, Span};

/// Compiles OpenSCAD source code to a mesh.
pub fn from_source(source: &str) -> Result<MeshBuffers, Vec<Diagnostic>> {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    
    let nodes = evaluator.evaluate_source(source).map_err(|e| {
        match e {
            EvaluationError::AstDiagnostics(diags) => diags,
            _ => {
                let span = Span::new(0, source.len()).unwrap_or_else(|_| Span::new(0, 1).unwrap());
                vec![Diagnostic::error(format!("Evaluation error: {}", e), span)]
            }
        }
    })?;

    if nodes.is_empty() {
        return Ok(MeshBuffers::new());
    }

    let mut combined = MeshBuffers::new();
    for node in &nodes {
        let manifold = convert_node(node)?;
        append_mesh(&mut combined, &manifold.to_mesh_buffers());
    }

    Ok(combined)
}

fn append_mesh(target: &mut MeshBuffers, source: &MeshBuffers) {
    let vertex_offset = target.vertex_count() as u32;
    target.vertices.extend_from_slice(&source.vertices);
    target
        .indices
        .extend(source.indices.iter().map(|idx| idx + vertex_offset));
}

fn convert_node(node: &GeometryNode) -> Result<Manifold, Vec<Diagnostic>> {
    match node {
        GeometryNode::Cube { size, center, span } => {
             cube(Vec3::new(size.x, size.y, size.z), *center)
                 .map_err(|e| {
                     vec![Diagnostic::error(format!("Manifold error: {}", e), *span)]
                 })
        }
        GeometryNode::Sphere { radius, segments, span } => {
            let generator = Sphere::new(*radius, *segments).map_err(|err| {
                vec![Diagnostic::error(format!("Manifold error: {}", err), *span)]
            })?;
            generator.to_manifold().map_err(|err| {
                vec![Diagnostic::error(format!("Manifold error: {}", err), *span)]
            })
        }
        GeometryNode::Cylinder { height, radius_bottom, radius_top, center, segments, span } => {
            cylinder(*height, *radius_bottom, *radius_top, *center, *segments).map_err(|e| {
                vec![Diagnostic::error(format!("Manifold error: {}", e), *span)]
            })
        }
        GeometryNode::Square { size, center, span } => {
            square(*size, *center).map_err(|e| {
                vec![Diagnostic::error(format!("Manifold error: {}", e), *span)]
            })
        }
        GeometryNode::Circle { radius, segments, span } => {
            circle(*radius, *segments).map_err(|e| {
                vec![Diagnostic::error(format!("Manifold error: {}", e), *span)]
            })
        }
        GeometryNode::Polygon { points, paths, convexity, span } => {
             // Convert paths from Vec<Vec<usize>> to Option<Vec<Vec<u32>>>
             let paths_converted = if paths.is_empty() {
                 None
             } else {
                 Some(paths.iter().map(|path| {
                     path.iter().map(|&idx| idx as u32).collect()
                 }).collect())
             };

             polygon(points.clone(), paths_converted, *convexity).map_err(|e| {
                 vec![Diagnostic::error(format!("Manifold error: {}", e), *span)]
             })
        }
        GeometryNode::Polyhedron { points, faces, convexity: _, span } => {
             let pts: Vec<Vec3> = points.iter().map(|p| Vec3::new(p.x, p.y, p.z)).collect();
             polyhedron(&pts, faces).map_err(|e| {
                 vec![Diagnostic::error(format!("Manifold error: {}", e), *span)]
             })
        }
        GeometryNode::LinearExtrude { height, twist, slices, center, scale, convexity: _, child, span } => {
             let child_mesh = convert_node(child)?;
             // Extract cross section using Manifold method
             let cs = child_mesh.to_cross_section().ok_or_else(|| {
                 vec![Diagnostic::error("Failed to extract cross section (not 2D)".to_string(), *span)]
             })?;

             linear_extrude(&cs, *height, *twist, *slices, *center, *scale).map_err(|e| {
                 vec![Diagnostic::error(format!("Extrude error: {}", e), *span)]
             })
        }
        GeometryNode::RotateExtrude { angle, convexity, segments, child, span } => {
             let child_mesh = convert_node(child)?;
             let cs = child_mesh.to_cross_section().ok_or_else(|| {
                 vec![Diagnostic::error("Failed to extract cross section (not 2D)".to_string(), *span)]
             })?;

             rotate_extrude(&cs, *angle, *convexity, *segments).map_err(|e| {
                 vec![Diagnostic::error(format!("Revolve error: {}", e), *span)]
             })
        }
        GeometryNode::Transform { matrix, child, span: _ } => {
            let mut m = convert_node(child)?;
            apply_transform(&mut m, *matrix);
            Ok(m)
        }
        GeometryNode::Resize { new_size, auto, child, span: _ } => {
            let mut m = convert_node(child)?;
            let auto_arr = [auto[0], auto[1], auto[2]];
            resize(&mut m, *new_size, auto_arr);
            Ok(m)
        }
        GeometryNode::Color { color, child, span: _ } => {
            let m = convert_node(child)?;
            Ok(m.with_color(*color))
        }
        GeometryNode::Union { children, span } => {
            if children.is_empty() { return Ok(Manifold::new()); }
            let mut result = convert_node(&children[0])?;
            for child in &children[1..] {
                let other = convert_node(child)?;
                result = boolean(&result, &other, BooleanOp::Union).map_err(|e| {
                    vec![Diagnostic::error(format!("Union error: {}", e), *span)]
                })?;
            }
            Ok(result)
        }
        GeometryNode::Difference { children, span } => {
            if children.is_empty() { return Ok(Manifold::new()); }
            let mut result = convert_node(&children[0])?;
            for child in &children[1..] {
                let other = convert_node(child)?;
                result = boolean(&result, &other, BooleanOp::Difference).map_err(|e| {
                    vec![Diagnostic::error(format!("Difference error: {}", e), *span)]
                })?;
            }
            Ok(result)
        }
        GeometryNode::Intersection { children, span } => {
            if children.is_empty() { return Ok(Manifold::new()); }
            let mut result = convert_node(&children[0])?;
            for child in &children[1..] {
                let other = convert_node(child)?;
                result = boolean(&result, &other, BooleanOp::Intersection).map_err(|e| {
                    vec![Diagnostic::error(format!("Intersection error: {}", e), *span)]
                })?;
            }
            Ok(result)
        }
        GeometryNode::Hull { children, span } => {
            let mut points = Vec::new();
            for child in children {
                let m = convert_node(child)?;
                for v in m.vertices {
                    points.push(v.position);
                }
            }
            hull(&points).map_err(|e| {
                vec![Diagnostic::error(format!("Hull error: {}", e), *span)]
            })
        }
        GeometryNode::Minkowski { children, span } => {
            if children.len() < 2 {
                return Err(vec![Diagnostic::error("Minkowski requires at least 2 children".to_string(), *span)]);
            }
            let mut result = convert_node(&children[0])?;
            for child in &children[1..] {
                let other = convert_node(child)?;
                result = minkowski(&result, &other).map_err(|e| {
                    vec![Diagnostic::error(format!("Minkowski error: {}", e), *span)]
                })?;
            }
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounding_box_from_buffers(buffers: &MeshBuffers) -> (Vec3, Vec3) {
        let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for chunk in buffers.vertices.chunks(3) {
            let x = chunk[0] as f64;
            let y = chunk[1] as f64;
            let z = chunk[2] as f64;
            min.x = min.x.min(x);
            min.y = min.y.min(y);
            min.z = min.z.min(z);
            max.x = max.x.max(x);
            max.y = max.y.max(y);
            max.z = max.z.max(z);
        }

        (min, max)
    }

    #[test]
    fn test_cube_generation() {
        let mesh = from_source("cube(10);").expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_cube_vector_generation() {
        let mesh = from_source("cube([1, 2, 3]);").expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 8);
    }

    #[test]
    fn test_multiple_top_level_nodes_are_combined() {
        let mesh = from_source("cube(2); translate([10,10,10]) cube([10,20,30]);")
            .expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 16);
        assert_eq!(mesh.triangle_count(), 24);
    }

    #[test]
    fn test_invalid_source() {
        let result = from_source("cube(");
        assert!(result.is_err());
    }

    #[test]
    fn test_sphere_generation() {
        let mesh = from_source("sphere(10);").expect("compilation succeeds");
        assert!(mesh.vertex_count() > 6);
        assert!(mesh.triangle_count() > 8);
    }

    #[test]
    fn test_cylinder_generation() {
        let mesh = from_source("cylinder(h=10, r1=5, r2=5);").expect("compilation succeeds");
        assert!(mesh.vertex_count() >= 6);
    }

    #[test]
    fn test_polyhedron_generation() {
        let source = "polyhedron(points=[[0,0,0], [1,0,0], [0,1,0], [0,0,1]], faces=[[0,1,2], [0,3,1], [0,2,3], [1,3,2]]);";
        let mesh = from_source(source).expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.triangle_count(), 4);
    }

    #[test]
    fn test_translate_generation() {
        let mesh = from_source("translate([10, 0, 0]) cube(1);").expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 8);
    }

    #[test]
    fn test_translated_cube_bounding_box() {
        let mesh = from_source("translate([5, 0, 0]) cube(2);").expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 12);
        let (min, max) = bounding_box_from_buffers(&mesh);
        assert_eq!(min, Vec3::new(5.0, 0.0, 0.0));
        assert_eq!(max, Vec3::new(7.0, 2.0, 2.0));
    }

    #[test]
    fn test_rotated_cube_bounding_box_swaps_axes() {
        let mesh = from_source("rotate([0,0,90]) cube([1,2,3]);").expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 12);
        let (min, max) = bounding_box_from_buffers(&mesh);
        assert!((min.x + 2.0).abs() < 1e-6);
        assert!((min.y - 0.0).abs() < 1e-6);
        assert!((min.z - 0.0).abs() < 1e-6);
        assert!((max.x - 0.0).abs() < 1e-6);
        assert!((max.y - 1.0).abs() < 1e-6);
        assert!((max.z - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_preserves_topology() {
        let mesh = from_source("scale([2,3,4]) cube(1);").expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 8);
        assert_eq!(mesh.triangle_count(), 12);
    }

    #[test]
    fn test_square_generation() {
        let mesh = from_source("square(10);").expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.triangle_count(), 2); // Single sided (2 triangles)
    }

    #[test]
    fn test_circle_generation() {
        let mesh = from_source("circle(10);").expect("compilation succeeds");
        assert!(mesh.vertex_count() >= 3);
    }

    #[test]
    fn test_polygon_generation() {
        let mesh = from_source("polygon([[0,0], [10,0], [0,10]]);").expect("compilation succeeds");
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1); // Single sided
    }
}

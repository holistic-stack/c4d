use crate::{Diagnostic, Span};
use tree_sitter::Node;
use super::shared::{get_positional_arg, parse_number_list_of_lists, parse_u32};

/// Parses arguments for the `polyhedron` primitive.
///
/// `polyhedron(points, faces, convexity)`
///
/// # Arguments
/// * `args` - The argument list CST node.
/// * `source` - The original source code.
///
/// # Returns
/// * `Ok(tuple)` - (points, faces, convexity)
pub fn parse_polyhedron_arguments(
    args: &Node,
    source: &str,
) -> Result<(Vec<[f64; 3]>, Vec<Vec<usize>>, u32), Vec<Diagnostic>> {

    // Helper to get named arg leniently
    let find_named = |name: &str| -> Option<Node> {
        let mut cursor = args.walk();
        for child in args.children(&mut cursor) {
            if child.kind() == "assignment" {
                // Try field name "left"
                let lhs = child.child_by_field_name("left")
                    .or_else(|| child.child(0)); // Fallback to index 0

                if let Some(lhs) = lhs {
                    let lhs_text = &source[lhs.byte_range()].trim();
                    if *lhs_text == name {
                        return child.child_by_field_name("right")
                            .or_else(|| {
                                // Fallback to finding expression child (index >= 2 usually)
                                let mut c2 = child.walk();
                                for sub in child.children(&mut c2) {
                                    if sub.kind() != "identifier" && sub.kind() != "=" && sub.kind() != lhs.kind() {
                                         // Heuristic: anything not identifier or =
                                         // return Some(sub);
                                    }
                                }
                                // Usually index 2
                                child.child(2)
                            });
                    }
                }
            }
        }
        None
    };

    // Points: Positional 0 or Named "points"
    let points_node = find_named("points")
        .or_else(|| get_positional_arg(args, 0))
        .ok_or_else(|| {
            let mut cursor = args.walk();
            let children_info: Vec<String> = args.children(&mut cursor).map(|c| {
                 format!("{}: '{}'", c.kind(), &source[c.byte_range()])
            }).collect();
            vec![Diagnostic::error(
                format!("Missing required argument: points. Children: {:?}", children_info),
                Span::new(args.start_byte(), args.end_byte()).unwrap(),
            )]
        })?;

    // Faces: Positional 1 or Named "faces" (or "triangles")
    let faces_node = find_named("faces")
        .or_else(|| find_named("triangles"))
        .or_else(|| get_positional_arg(args, 1))
        .ok_or_else(|| {
            vec![Diagnostic::error(
                "Missing required argument: faces".to_string(),
                Span::new(args.start_byte(), args.end_byte()).unwrap(),
            )]
        })?;

    // Convexity: Named "convexity", default 1
    let convexity_node = find_named("convexity");

    // Parse points
    let points_list = parse_number_list_of_lists(&points_node, source)?;
    // Convert to [f64; 3]
    let points: Vec<[f64; 3]> = points_list.into_iter().map(|p| {
        if p.len() >= 3 {
            [p[0], p[1], p[2]]
        } else {
            // Pad with 0
            [
                *p.get(0).unwrap_or(&0.0),
                *p.get(1).unwrap_or(&0.0),
                *p.get(2).unwrap_or(&0.0)
            ]
        }
    }).collect();

    // Parse faces (list of list of indices)
    let faces_list_f64 = parse_number_list_of_lists(&faces_node, source)?;
    let faces: Vec<Vec<usize>> = faces_list_f64.into_iter().map(|face| {
        face.into_iter().map(|idx| idx as usize).collect()
    }).collect();

    // Parse convexity
    let convexity = if let Some(node) = convexity_node {
        parse_u32(&node, source)?
    } else {
        1
    };

    Ok((points, faces, convexity))
}

#[cfg(test)]
mod tests {
    use crate::parse_to_ast;
    use crate::ast_types::Statement;

    #[test]
    fn test_parse_polyhedron_named() {
        let source = "polyhedron(points=[[0,0,0],[1,0,0],[0,1,0],[0,0,1]], faces=[[0,1,2],[0,3,1],[0,2,3],[1,3,2]]);";
        let ast = parse_to_ast(source).expect("parse succeeds");
        assert_eq!(ast.len(), 1);

        match &ast[0] {
            Statement::Polyhedron { points, faces, convexity, .. } => {
                assert_eq!(points.len(), 4);
                assert_eq!(faces.len(), 4);
                assert_eq!(*convexity, 1);
            }
            _ => panic!("Expected Polyhedron"),
        }
    }
}

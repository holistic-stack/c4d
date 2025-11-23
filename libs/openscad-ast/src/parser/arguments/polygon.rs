//! Argument parsing for `polygon` primitive.

use crate::{Diagnostic, Span};
use super::shared::{parse_f64, parse_u32};
use tree_sitter::Node;

/// Parses arguments for `polygon(points, paths, convexity)`.
///
/// # Arguments
///
/// * `args_node` - The `arguments` node from the CST.
/// * `source` - The source code.
///
/// # Returns
///
/// * `Ok((points, paths, convexity))` - Parsed arguments.
/// * `Err(Vec<Diagnostic>)` - Parse errors.
pub fn parse_polygon_arguments(
    args_node: &Node,
    source: &str,
) -> Result<(Vec<[f64; 2]>, Option<Vec<Vec<usize>>>, u32), Vec<Diagnostic>> {
    let mut cursor = args_node.walk();
    let children: Vec<_> = args_node.children(&mut cursor).collect();

    let mut points: Option<Vec<[f64; 2]>> = None;
    let mut paths: Option<Vec<Vec<usize>>> = None;
    let mut convexity: u32 = 1; // Default convexity

    let mut positional_index = 0;

    for child in children {
        if child.kind() == "(" || child.kind() == ")" || child.kind() == "," {
            continue;
        }

        if child.kind() == "assignment" {
            // Named argument
            let mut assign_cursor = child.walk();
            let subchildren: Vec<_> = child.children(&mut assign_cursor).collect();

            let name_node = subchildren.iter().find(|n| n.kind() == "identifier");
            let value_node = subchildren.iter().find(|n| n.kind() != "identifier" && n.kind() != "=");

            if let (Some(name_node), Some(value_node)) = (name_node, value_node) {
                let name = &source[name_node.byte_range()];
                match name {
                    "points" => points = Some(parse_points(value_node, source)?),
                    "paths" => paths = Some(parse_paths(value_node, source)?),
                    "convexity" => convexity = parse_u32(value_node, source)?,
                    _ => {
                         // Warn or ignore
                    }
                }
            }
        } else {
            // Positional argument
            match positional_index {
                0 => points = Some(parse_points(&child, source)?),
                1 => paths = Some(parse_paths(&child, source)?),
                2 => convexity = parse_u32(&child, source)?,
                _ => {
                     return Err(vec![Diagnostic::error(
                        "Too many positional arguments for polygon()",
                        Span::new(child.start_byte(), child.end_byte()).unwrap(),
                    )]);
                }
            }
            positional_index += 1;
        }
    }

    let points = points.ok_or_else(|| vec![Diagnostic::error(
        "Missing argument: points",
        Span::new(args_node.start_byte(), args_node.end_byte()).unwrap(),
    )])?;

    Ok((points, paths, convexity))
}

fn parse_points(node: &Node, source: &str) -> Result<Vec<[f64; 2]>, Vec<Diagnostic>> {
    let mut points = Vec::new();

    // Expecting a vector of vectors: [[x,y], [x,y], ...]
    // Check if it's a vector/list
    if node.kind() != "vector" && node.kind() != "list" {
         return Err(vec![Diagnostic::error(
            "Points must be a list of 2D vectors",
            Span::new(node.start_byte(), node.end_byte()).unwrap(),
        )]);
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
         if child.kind() == "vector" || child.kind() == "list" {
             let mut sub_cursor = child.walk();
             let mut coords = Vec::new();
             for sub_child in child.children(&mut sub_cursor) {
                 if sub_child.kind() == "number" || sub_child.kind() == "integer" || sub_child.kind() == "float" {
                     coords.push(parse_f64(&sub_child, source)?);
                 }
             }
             if coords.len() == 2 {
                 points.push([coords[0], coords[1]]);
             } else {
                 return Err(vec![Diagnostic::error(
                    format!("Point must have 2 coordinates, got {}", coords.len()),
                    Span::new(child.start_byte(), child.end_byte()).unwrap(),
                )]);
             }
         }
    }

    Ok(points)
}

fn parse_paths(node: &Node, source: &str) -> Result<Vec<Vec<usize>>, Vec<Diagnostic>> {
    // paths can be a single list of indices [0, 1, 2] OR a list of lists [[0,1,2], [3,4,5]]

    if node.kind() != "vector" && node.kind() != "list" {
         return Err(vec![Diagnostic::error(
            "Paths must be a list of indices or list of lists of indices",
            Span::new(node.start_byte(), node.end_byte()).unwrap(),
        )]);
    }

    // Check depth
    let mut is_nested = false;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "vector" || child.kind() == "list" {
            is_nested = true;
            break;
        }
    }

    if is_nested {
         let mut paths = Vec::new();
         let mut cursor = node.walk();
         for child in node.children(&mut cursor) {
             if child.kind() == "vector" || child.kind() == "list" {
                 let mut sub_cursor = child.walk();
                 let mut path = Vec::new();
                 for sub_child in child.children(&mut sub_cursor) {
                     if sub_child.kind() == "number" || sub_child.kind() == "integer" || sub_child.kind() == "float" {
                         path.push(parse_u32(&sub_child, source)? as usize);
                     }
                 }
                 paths.push(path);
             }
         }
         Ok(paths)
    } else {
        // Single path
        let mut path = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
             if child.kind() == "number" || child.kind() == "integer" || child.kind() == "float" {
                 path.push(parse_u32(&child, source)? as usize);
             }
        }
        Ok(vec![path])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter_openscad_parser::parse_source;

    fn get_args_node(source: &str) -> tree_sitter::Tree {
        parse_source(source).unwrap()
    }

    #[test]
    fn test_polygon_simple() {
        let source = "polygon([[0,0], [10,0], [0,10]]);";
        let tree = get_args_node(source);
        let root = tree.root_node();
        let args = root.child(0).unwrap().child(0).unwrap().child(1).unwrap();

        let (points, paths, convexity) = parse_polygon_arguments(&args, source).unwrap();
        assert_eq!(points.len(), 3);
        assert_eq!(paths, None); // Implicit paths
        assert_eq!(convexity, 1);
    }

    #[test]
    fn test_polygon_with_paths() {
        let source = "polygon([[0,0], [10,0], [0,10]], [[0,1,2]]);";
        let tree = get_args_node(source);
        let root = tree.root_node();
        let args = root.child(0).unwrap().child(0).unwrap().child(1).unwrap();

        let (points, paths, _) = parse_polygon_arguments(&args, source).unwrap();
        assert_eq!(points.len(), 3);
        assert!(paths.is_some());
        assert_eq!(paths.unwrap()[0], vec![0, 1, 2]);
    }
}

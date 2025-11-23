//! Argument parsing for `square` primitive.

use crate::{ast_types::SquareSize, Diagnostic, Span};
use super::shared::{parse_bool, parse_f64};
use tree_sitter::Node;

/// Parses arguments for `square(size, center)`.
///
/// # Arguments
///
/// * `args_node` - The `arguments` node from the CST.
/// * `source` - The source code.
///
/// # Returns
///
/// * `Ok((size, center))` - Parsed arguments.
/// * `Err(Vec<Diagnostic>)` - Parse errors.
pub fn parse_square_arguments(
    args_node: &Node,
    source: &str,
) -> Result<(SquareSize, bool), Vec<Diagnostic>> {
    let mut cursor = args_node.walk();
    let children: Vec<_> = args_node.children(&mut cursor).collect();

    let mut size: Option<SquareSize> = None;
    let mut center: bool = false; // Default center=false

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
                    "size" => {
                        size = Some(parse_square_size(value_node, source)?);
                    }
                    "center" => {
                        center = parse_bool(value_node, source)?;
                    }
                    _ => {
                         return Err(vec![Diagnostic::warning(
                            format!("Unknown argument: {}", name),
                            Span::new(name_node.start_byte(), name_node.end_byte()).unwrap(),
                        )]);
                    }
                }
            }
        } else {
            // Positional argument
            match positional_index {
                0 => {
                    size = Some(parse_square_size(&child, source)?);
                }
                1 => {
                    center = parse_bool(&child, source)?;
                }
                _ => {
                     return Err(vec![Diagnostic::error(
                        "Too many positional arguments for square()",
                        Span::new(child.start_byte(), child.end_byte()).unwrap(),
                    )]);
                }
            }
            positional_index += 1;
        }
    }

    // Defaults
    let size = size.ok_or_else(|| vec![Diagnostic::error(
        "Missing argument: size",
        Span::new(args_node.start_byte(), args_node.end_byte()).unwrap(),
    )])?;

    Ok((size, center))
}

fn parse_square_size(node: &Node, source: &str) -> Result<SquareSize, Vec<Diagnostic>> {
    if node.kind() == "vector" || node.kind() == "list" { // Assuming "vector" or "list" based on grammar
         let mut cursor = node.walk();
        let mut values = Vec::new();

        for child in node.children(&mut cursor) {
            if child.kind() == "number" || child.kind() == "integer" || child.kind() == "float" {
                 let val = parse_f64(&child, source)?;
                 values.push(val);
            }
        }

        if values.len() == 2 {
            Ok(SquareSize::Vector([values[0], values[1]]))
        } else {
            Err(vec![Diagnostic::error(
                format!("Vector must have 2 elements, got {}", values.len()),
                Span::new(node.start_byte(), node.end_byte()).unwrap(),
            )])
        }
    } else {
         let val = parse_f64(node, source)?;
         Ok(SquareSize::Scalar(val))
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
    fn test_square_scalar() {
        let source = "square(10);";
        let tree = get_args_node(source);
        let root = tree.root_node();
        // module_item -> module_call -> arguments
        let args = root.child(0).unwrap().child(0).unwrap().child(1).unwrap();

        let (size, center) = parse_square_arguments(&args, source).unwrap();
        assert_eq!(size, SquareSize::Scalar(10.0));
        assert_eq!(center, false);
    }

    #[test]
    fn test_square_vector() {
        let source = "square([10, 20]);";
        let tree = get_args_node(source);
        let root = tree.root_node();
        let args = root.child(0).unwrap().child(0).unwrap().child(1).unwrap();

        let (size, center) = parse_square_arguments(&args, source).unwrap();
        assert_eq!(size, SquareSize::Vector([10.0, 20.0]));
        assert_eq!(center, false);
    }

    #[test]
    fn test_square_named() {
        let source = "square(size=15, center=true);";
        let tree = get_args_node(source);
        let root = tree.root_node();
        let args = root.child(0).unwrap().child(0).unwrap().child(1).unwrap();

        let (size, center) = parse_square_arguments(&args, source).unwrap();
        assert_eq!(size, SquareSize::Scalar(15.0));
        assert_eq!(center, true);
    }
}

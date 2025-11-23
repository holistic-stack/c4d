//! Argument parsing for `circle` primitive.

use crate::{Diagnostic, Span};
use super::shared::{parse_f64, parse_u32};
use tree_sitter::Node;

/// Parses arguments for `circle(r|d, $fn, $fa, $fs)`.
///
/// # Arguments
///
/// * `args_node` - The `arguments` node from the CST.
/// * `source` - The source code.
///
/// # Returns
///
/// * `Ok((radius, fa, fs, fn_))` - Parsed arguments.
/// * `Err(Vec<Diagnostic>)` - Parse errors.
pub fn parse_circle_arguments(
    args_node: &Node,
    source: &str,
) -> Result<(f64, Option<f64>, Option<f64>, Option<u32>), Vec<Diagnostic>> {
    let mut cursor = args_node.walk();
    let children: Vec<_> = args_node.children(&mut cursor).collect();

    let mut radius: Option<f64> = None;
    let mut diameter: Option<f64> = None;
    let mut fa: Option<f64> = None;
    let mut fs: Option<f64> = None;
    let mut fn_: Option<u32> = None;

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
                    "r" => radius = Some(parse_f64(value_node, source)?),
                    "d" => diameter = Some(parse_f64(value_node, source)?),
                    "$fa" => fa = Some(parse_f64(value_node, source)?),
                    "$fs" => fs = Some(parse_f64(value_node, source)?),
                    "$fn" => fn_ = Some(parse_u32(value_node, source)?),
                    _ => {
                        // Ignore unknown named arguments or warn
                    }
                }
            }
        } else {
            // Positional argument (only radius/diameter is positional, effectively radius)
            if positional_index == 0 {
                radius = Some(parse_f64(&child, source)?);
            } else {
                 return Err(vec![Diagnostic::error(
                    "Too many positional arguments for circle()",
                    Span::new(child.start_byte(), child.end_byte()).unwrap(),
                )]);
            }
            positional_index += 1;
        }
    }

    // Logic: 'd' takes precedence over 'r' if both are somehow present (or if 'd' is used),
    // but typically only one is used. If 'd' is present, r = d / 2.
    // If positional is used, it sets 'radius'.

    let final_radius = if let Some(d) = diameter {
        d / 2.0
    } else if let Some(r) = radius {
        r
    } else {
         return Err(vec![Diagnostic::error(
            "Missing argument: radius (r) or diameter (d)",
            Span::new(args_node.start_byte(), args_node.end_byte()).unwrap(),
        )]);
    };

    Ok((final_radius, fa, fs, fn_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter_openscad_parser::parse_source;

    fn get_args_node(source: &str) -> tree_sitter::Tree {
        parse_source(source).unwrap()
    }

    #[test]
    fn test_circle_radius() {
        let source = "circle(10);";
        let tree = get_args_node(source);
        let root = tree.root_node();
        let args = root.child(0).unwrap().child(0).unwrap().child(1).unwrap();

        let (r, _, _, _) = parse_circle_arguments(&args, source).unwrap();
        assert_eq!(r, 10.0);
    }

    #[test]
    fn test_circle_named_r() {
        let source = "circle(r=5);";
        let tree = get_args_node(source);
        let root = tree.root_node();
        let args = root.child(0).unwrap().child(0).unwrap().child(1).unwrap();

        let (r, _, _, _) = parse_circle_arguments(&args, source).unwrap();
        assert_eq!(r, 5.0);
    }

    #[test]
    fn test_circle_named_d() {
        let source = "circle(d=20);";
        let tree = get_args_node(source);
        let root = tree.root_node();
        let args = root.child(0).unwrap().child(0).unwrap().child(1).unwrap();

        let (r, _, _, _) = parse_circle_arguments(&args, source).unwrap();
        assert_eq!(r, 10.0);
    }
}

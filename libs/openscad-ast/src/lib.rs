use tree_sitter::{Parser, Tree};
use tree_sitter_openscad;

pub mod diagnostic;
pub use diagnostic::{Diagnostic, Severity, Span};

#[derive(Debug, Clone, PartialEq)]
pub struct Cube {
    pub size: [f64; 3],
    pub center: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    Cube(Cube),
}

pub fn parse(source: &str) -> Result<Geometry, Vec<Diagnostic>> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_openscad::language())
        .expect("Error loading OpenSCAD grammar");

    let tree = parser.parse(source, None).ok_or_else(|| {
        vec![Diagnostic::error(
            "Failed to parse source code".to_string(),
            Span::new(0, source.len()),
        )]
    })?;

    let root_node = tree.root_node();
    if root_node.has_error() {
        // TODO: Traverse tree to find specific error nodes and create precise diagnostics
        return Err(vec![Diagnostic::error(
            "Syntax error in source code".to_string(),
            Span::new(0, source.len()),
        )]);
    }

    // Traverse to find the first module_call
    // cube(10); is likely parsed as a transform_chain -> module_call
    
    let mut cursor = root_node.walk();
    for child in root_node.children(&mut cursor) {
        if let Some(geometry) = find_geometry(child, source) {
            return Ok(geometry);
        }
    }

    Err(vec![Diagnostic::error(
        "No geometry found".to_string(),
        Span::new(0, source.len()),
    )])
}

fn find_geometry(node: Node, source: &str) -> Option<Geometry> {
    if node.kind() == "module_call" {
        return parse_module_call(node, source);
    }
    
    // If it's a transform_chain, it contains a module_call
    if node.kind() == "transform_chain" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(geometry) = find_geometry(child, source) {
                return Some(geometry);
            }
        }
    }

    None
}

fn parse_module_call(node: Node, source: &str) -> Option<Geometry> {
    let name_node = node.child_by_field_name("name")?;
    let name = name_node.utf8_text(source.as_bytes()).ok()?;

    if name == "cube" {
        let args_node = node.child_by_field_name("arguments")?;
        return Some(Geometry::Cube(parse_cube_args(args_node, source)));
    }

    None
}

fn parse_cube_args(node: Node, source: &str) -> Cube {
    let mut size = [1.0, 1.0, 1.0];
    let mut center = false;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // arguments can be values directly or named arguments
        // For now, let's handle simple positional args for size
        // cube(10) -> size=[10,10,10]
        // cube([1,2,3]) -> size=[1,2,3]
        
        let kind = child.kind();
        if kind == "integer" || kind == "float" || kind == "number" {
             if let Ok(val) = child.utf8_text(source.as_bytes()).unwrap_or("1").parse::<f64>() {
                 size = [val, val, val];
             }
        } else if kind == "list" {
            // parse vector [x, y, z]
            let mut vec_vals = Vec::new();
            let mut vec_cursor = child.walk();
            for vec_child in child.children(&mut vec_cursor) {
                let vec_kind = vec_child.kind();
                if vec_kind == "integer" || vec_kind == "float" || vec_kind == "number" {
                    if let Ok(val) = vec_child.utf8_text(source.as_bytes()).unwrap_or("0").parse::<f64>() {
                        vec_vals.push(val);
                    }
                }
            }
            if vec_vals.len() >= 3 {
                size = [vec_vals[0], vec_vals[1], vec_vals[2]];
            }
        }
    }

    Cube { size, center }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cube_scalar() {
        let source = "cube(10);";
        let geom = parse(source).unwrap();
        match geom {
            Geometry::Cube(cube) => {
                assert_eq!(cube.size, [10.0, 10.0, 10.0]);
                assert_eq!(cube.center, false);
            }
        }
    }

    #[test]
    fn test_parse_cube_vector() {
        let source = "cube([1, 2, 3]);";
        let geom = parse(source).unwrap();
        match geom {
            Geometry::Cube(cube) => {
                assert_eq!(cube.size, [1.0, 2.0, 3.0]);
                assert_eq!(cube.center, false);
            }
        }
    }
}

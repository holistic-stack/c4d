use tree_sitter::{Parser, Tree, Node};
use pipeline_types::{Diagnostic, Severity, Span};
use tree_sitter_openscad;
use thiserror::Error;

// diagnostics provided by pipeline-types

#[derive(Debug, Clone, PartialEq)]
pub struct Cube {
    pub size: [f64; 3],
    pub center: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Geometry {
    Cube(Cube),
}


pub struct Cst {
    pub tree: Tree,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("failed to parse source")]
    Failed { span: Span, file: Option<String> },
    #[error("syntax error in source code")]
    Syntax { span: Span, file: Option<String> },
}

#[derive(Debug, Error)]
pub enum AstError {
    #[error("AST build failed: {message}")]
    Build { message: String, span: Span, file: Option<String>, #[source] source: Option<ParseError> },
}

fn parse_cst_internal(source: &str, file: Option<&str>) -> Result<Cst, ParseError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_openscad::language())
        .expect("Error loading OpenSCAD grammar");
    let tree = parser.parse(source, None).ok_or(ParseError::Failed { span: Span { start: 0, end: source.len() }, file: file.map(|f| f.to_string()) })?;
    if tree.root_node().has_error() {
        return Err(ParseError::Syntax { span: Span { start: 0, end: source.len() }, file: file.map(|f| f.to_string()) });
    }
    Ok(Cst { tree })
}

pub fn build_ast(cst: &Cst, source: &str) -> Result<Geometry, Vec<Diagnostic>> {
    let root_node = cst.tree.root_node();
    if root_node.has_error() {
        return Err(vec![Diagnostic { severity: Severity::Error, message: "Syntax error".to_string(), span: Span { start: 0, end: source.len() }, hint: None }]);
    }

    // Traverse to find the first module_call
    // cube(10); is likely parsed as a transform_chain -> module_call
    
    let mut cursor = root_node.walk();
    for child in root_node.children(&mut cursor) {
        if let Some(geometry) = find_geometry(child, source) {
            return Ok(geometry);
        }
    }

    Err(vec![Diagnostic { severity: Severity::Error, message: "No geometry found".to_string(), span: Span { start: 0, end: source.len() }, hint: None }])
}

pub fn build_ast_from_source(source: &str, file: Option<&str>) -> Result<Geometry, AstError> {
    let cst = parse_cst_internal(source, file).map_err(|e| AstError::Build { message: "parse failed".to_string(), span: match e { ParseError::Failed { span, .. } | ParseError::Syntax { span, .. } => span }, file: file.map(|f| f.to_string()), source: Some(e) })?;
    build_ast(&cst, source).map_err(|diags| AstError::Build { message: diags.first().map(|d| d.message.clone()).unwrap_or_else(|| "ast build error".to_string()), span: diags.first().map(|d| d.span).unwrap_or(Span { start: 0, end: source.len() }), file: file.map(|f| f.to_string()), source: None })
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
    fn parse_cst(source: &str) -> Cst { parse_cst_internal(source, None).unwrap() }

    #[test]
    fn test_parse_cube_scalar() {
        let source = "cube(10);";
        let cst = parse_cst(source);
        let geom = build_ast(&cst, source).unwrap();
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
        let cst = parse_cst(source);
        let geom = build_ast(&cst, source).unwrap();
        match geom {
            Geometry::Cube(cube) => {
                assert_eq!(cube.size, [1.0, 2.0, 3.0]);
                assert_eq!(cube.center, false);
            }
        }
    }
}

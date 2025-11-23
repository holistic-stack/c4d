//! Shared argument parsing helpers.

use crate::{ast_types::CubeSize, Diagnostic, Span};
use tree_sitter::Node;

/// Parses an f64 value.
pub fn parse_f64(node: &Node, source: &str) -> Result<f64, Vec<Diagnostic>> {
    let text = &source[node.byte_range()];
    text.parse::<f64>().map_err(|_| {
        vec![Diagnostic::error(
            format!("Invalid number: {}", text),
            Span::new(node.start_byte(), node.end_byte()).unwrap(),
        )]
    })
}

/// Parses a u32 value.
pub fn parse_u32(node: &Node, source: &str) -> Result<u32, Vec<Diagnostic>> {
    let text = &source[node.byte_range()];
    // Handle float notation if used for integer context (e.g. 50.0)
    if let Ok(f) = text.parse::<f64>() {
        return Ok(f as u32);
    }
    text.parse::<u32>().map_err(|_| {
        vec![Diagnostic::error(
            format!("Invalid integer: {}", text),
            Span::new(node.start_byte(), node.end_byte()).unwrap(),
        )]
    })
}

/// Parses a boolean value.
pub fn parse_bool(node: &Node, source: &str) -> Result<bool, Vec<Diagnostic>> {
    let text = &source[node.byte_range()];
    match text {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(vec![Diagnostic::error(
            format!("Invalid boolean: {}", text),
            Span::new(node.start_byte(), node.end_byte()).unwrap(),
        )]),
    }
}

/// Parses a vector literal [x, y, z] into CubeSize::Vector.
pub fn parse_vector(node: &Node, source: &str) -> Result<CubeSize, Vec<Diagnostic>> {
    let mut cursor = node.walk();
    let mut values = Vec::new();
    
    for child in node.children(&mut cursor) {
        if child.kind() == "number" || child.kind() == "integer" || child.kind() == "float" {
             let val = parse_f64(&child, source)?;
             values.push(val);
        }
    }

    if values.len() == 3 {
        Ok(CubeSize::Vector([values[0], values[1], values[2]]))
    } else {
        Err(vec![Diagnostic::error(
            format!("Vector must have 3 elements, got {}", values.len()),
            Span::new(node.start_byte(), node.end_byte()).unwrap(),
        )])
    }
}

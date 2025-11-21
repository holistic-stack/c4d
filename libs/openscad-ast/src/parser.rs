/// Parser module for converting OpenSCAD source to AST.
///
/// This module provides the entry point for parsing OpenSCAD source code.
/// It calls the tree-sitter parser and converts the CST to a typed AST.

use crate::{ast_types::*, Diagnostic, Span};
use tree_sitter_openscad_parser::parse_source as parse_cst;

/// Parses OpenSCAD source code to AST.
///
/// This is the main entry point for the AST layer. It:
/// 1. Calls openscad-parser to get CST
/// 2. Converts CST to typed AST
/// 3. Returns diagnostics for any errors
///
/// # Arguments
/// * `source` - The OpenSCAD source code to parse
///
/// # Returns
/// * `Ok(Vec<Statement>)` - The parsed AST statements
/// * `Err(Vec<Diagnostic>)` - Diagnostics if parsing fails
///
/// # Examples
/// ```
/// use openscad_ast::parse_to_ast;
///
/// let ast = parse_to_ast("cube(10);").unwrap();
/// assert_eq!(ast.len(), 1);
/// ```
pub fn parse_to_ast(source: &str) -> Result<Vec<Statement>, Vec<Diagnostic>> {
    // Parse to CST
    let tree = parse_cst(source).map_err(|e| {
        vec![Diagnostic::error(
            format!("Parse error: {}", e),
            Span::new(0, source.len()).unwrap_or_else(|_| Span::new(0, 1).unwrap()),
        )]
    })?;

    let root = tree.root_node();
    
    // Check for syntax errors
    if root.has_error() {
        return Err(vec![Diagnostic::error(
            "Syntax error in source code",
            Span::new(0, source.len()).unwrap_or_else(|_| Span::new(0, 1).unwrap()),
        )
        .with_hint("Check for missing semicolons or parentheses")]);
    }

    // Convert CST to AST
    let mut statements = Vec::new();
    let mut cursor = root.walk();
    
    for child in root.children(&mut cursor) {
        let kind = child.kind();
        
        if kind == "module_call" {
            if let Some(stmt) = parse_module_call(&child, source)? {
                statements.push(stmt);
            }
        } else if kind == "transform_chain" {
            let mut tc_cursor = child.walk();
            for tc_child in child.children(&mut tc_cursor) {
                if tc_child.kind() == "module_call" {
                    if let Some(stmt) = parse_module_call(&tc_child, source)? {
                        statements.push(stmt);
                    }
                }
            }
        } else if kind == "var_declaration" {
            if let Some(stmt) = parse_var_declaration(&child, source)? {
                statements.push(stmt);
            }
        }
    }

    Ok(statements)
}

/// Parses a variable declaration node from the CST.
///
/// Handles `var = value;` statements.
fn parse_var_declaration(
    node: &tree_sitter::Node,
    source: &str,
) -> Result<Option<Statement>, Vec<Diagnostic>> {
    let mut cursor = node.walk();
    let assignment = node.children(&mut cursor).find(|n| n.kind() == "assignment");

    if let Some(assign) = assignment {
        let name_node = assign.child_by_field_name("name");
        let value_node = assign.child_by_field_name("value");

        if let (Some(name_n), Some(value_n)) = (name_node, value_node) {
            let name = source[name_n.byte_range()].to_string();

            // Parse value - only simple numbers for now
            // This is a simplification for Task 3.2
            match value_n.kind() {
                "number" | "integer" | "float" => {
                    let text = &source[value_n.byte_range()];
                    let value = text.parse::<f64>().map_err(|_| {
                        vec![Diagnostic::error(
                            format!("Invalid number in assignment: {}", text),
                            Span::new(value_n.start_byte(), value_n.end_byte()).unwrap(),
                        )]
                    })?;

                    return Ok(Some(Statement::Assignment {
                        name,
                        value,
                        span: Span::new(node.start_byte(), node.end_byte()).unwrap(),
                    }));
                }
                _ => {
                    // Ignore complex assignments for now
                    return Ok(None);
                }
            }
        }
    }
    Ok(None)
}

/// Parses a module call node from the CST.
///
/// Extracts cube calls with their parameters.
fn parse_module_call(
    node: &tree_sitter::Node,
    source: &str,
) -> Result<Option<Statement>, Vec<Diagnostic>> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    // Find the module name
    let name_node = children.iter().find(|n| n.kind() == "identifier");
    let name = name_node
        .map(|n| &source[n.byte_range()])
        .unwrap_or("");

    if name != "cube" {
        // Only support cube for now
        return Ok(None);
    }

    // Find the arguments
    let args_node = children.iter().find(|n| n.kind() == "arguments");
    
    if let Some(args) = args_node {
        let (size, center) = parse_cube_arguments(args, source)?;
        let span = Span::new(node.start_byte(), node.end_byte())
            .map_err(|e| vec![Diagnostic::error(format!("Invalid span: {}", e), Span::new(0, 1).unwrap())])?;
        
        Ok(Some(Statement::Cube { size, center, span }))
    } else {
        Err(vec![Diagnostic::error(
            "cube() requires arguments",
            Span::new(node.start_byte(), node.end_byte()).unwrap(),
        )
        .with_hint("Try: cube(10); or cube([1, 2, 3]);")])
    }
}

/// Parses cube arguments from the CST.
///
/// Handles both scalar and vector forms, plus named arguments.
/// Returns (size, center) where center is None if not specified.
///
/// # Supported Forms
/// - `cube(10)` → (Scalar(10), None)
/// - `cube([1,2,3])` → (Vector([1,2,3]), None)
/// - `cube(10, true)` → (Scalar(10), Some(true))
/// - `cube(size=10, center=true)` → (Scalar(10), Some(true))
/// - `cube([1,2,3], center=false)` → (Vector([1,2,3]), Some(false))
fn parse_cube_arguments(
    args_node: &tree_sitter::Node,
    source: &str,
) -> Result<(CubeSize, Option<bool>), Vec<Diagnostic>> {
    let mut cursor = args_node.walk();
    let mut size: Option<CubeSize> = None;
    let mut center: Option<bool> = None;
    let mut positional_index = 0;
    
    for child in args_node.children(&mut cursor) {
        let kind = child.kind();
        
        if kind == "assignment" {
            // Named argument: name=value
            let name_node = child.child_by_field_name("name");
            let value_node = child.child_by_field_name("value");
            
            if let (Some(name_n), Some(value_n)) = (name_node, value_node) {
                let param_name = &source[name_n.byte_range()];
                
                match param_name {
                    "size" => {
                        size = Some(parse_size_value(&value_n, source)?);
                    }
                    "center" => {
                        if value_n.kind() == "boolean" {
                            let text = &source[value_n.byte_range()];
                            center = Some(text == "true");
                        } else {
                            return Err(vec![Diagnostic::error(
                                "center parameter must be a boolean",
                                Span::new(value_n.start_byte(), value_n.end_byte()).unwrap(),
                            )]);
                        }
                    }
                    _ => {
                        return Err(vec![Diagnostic::error(
                            format!("Unknown parameter: {}", param_name),
                            Span::new(child.start_byte(), child.end_byte()).unwrap(),
                        )]);
                    }
                }
            }
        } else if kind == "number" || kind == "integer" || kind == "float" {
            // Positional scalar argument
            if positional_index == 0 {
                let text = &source[child.byte_range()];
                let value = text.parse::<f64>().map_err(|_| {
                    vec![Diagnostic::error(
                        format!("Invalid number: {}", text),
                        Span::new(child.start_byte(), child.end_byte()).unwrap(),
                    )]
                })?;
                size = Some(CubeSize::Scalar(value));
            } else {
                return Err(vec![Diagnostic::error(
                    "Unexpected positional argument",
                    Span::new(child.start_byte(), child.end_byte()).unwrap(),
                )]);
            }
            positional_index += 1;
        } else if kind == "list" {
            // Positional vector argument
            if positional_index == 0 {
                size = Some(parse_vector(&child, source)?);
            } else {
                return Err(vec![Diagnostic::error(
                    "Unexpected positional argument",
                    Span::new(child.start_byte(), child.end_byte()).unwrap(),
                )]);
            }
            positional_index += 1;
        } else if kind == "boolean" {
            // Positional boolean argument (center)
            if positional_index == 1 {
                let text = &source[child.byte_range()];
                center = Some(text == "true");
            } else {
                return Err(vec![Diagnostic::error(
                    "Unexpected boolean argument",
                    Span::new(child.start_byte(), child.end_byte()).unwrap(),
                )]);
            }
            positional_index += 1;
        }
    }

    if size.is_none() {
        return Err(vec![Diagnostic::error(
            "cube() requires a size argument",
            Span::new(args_node.start_byte(), args_node.end_byte()).unwrap(),
        )
        .with_hint("Try: cube(10); or cube([1, 2, 3]);")]);
    }

    Ok((size.unwrap(), center))
}

/// Parses a size value (number or vector) from a CST node.
fn parse_size_value(
    value_node: &tree_sitter::Node,
    source: &str,
) -> Result<CubeSize, Vec<Diagnostic>> {
    match value_node.kind() {
        "number" | "integer" | "float" => {
            let text = &source[value_node.byte_range()];
            let num = text.parse::<f64>().map_err(|_| {
                vec![Diagnostic::error(
                    format!("Invalid number: {}", text),
                    Span::new(value_node.start_byte(), value_node.end_byte()).unwrap(),
                )]
            })?;
            Ok(CubeSize::Scalar(num))
        }
        "list" => {
            parse_vector(value_node, source)
        }
        _ => {
            Err(vec![Diagnostic::error(
                format!("size parameter must be a number or vector, got {}", value_node.kind()),
                Span::new(value_node.start_byte(), value_node.end_byte()).unwrap(),
            )])
        }
    }
}

/// Parses a vector literal from the CST.
fn parse_vector(
    list_node: &tree_sitter::Node,
    source: &str,
) -> Result<CubeSize, Vec<Diagnostic>> {
    let mut cursor = list_node.walk();
    let mut values = Vec::new();

    for child in list_node.children(&mut cursor) {
        let kind = child.kind();
        if kind == "number" || kind == "integer" || kind == "float" {
            let text = &source[child.byte_range()];
            let value = text.parse::<f64>().map_err(|_| {
                vec![Diagnostic::error(
                    format!("Invalid number: {}", text),
                    Span::new(child.start_byte(), child.end_byte()).unwrap(),
                )]
            })?;
            values.push(value);
        }
    }

    if values.len() != 3 {
        return Err(vec![Diagnostic::error(
            format!("cube() vector must have 3 elements, got {}", values.len()),
            Span::new(list_node.start_byte(), list_node.end_byte()).unwrap(),
        )
        .with_hint("Try: cube([1, 2, 3]);")]);
    }

    Ok(CubeSize::Vector([values[0], values[1], values[2]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cube_scalar() {
        let ast = parse_to_ast("cube(10);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, .. } => {
                assert_eq!(*size, CubeSize::Scalar(10.0));
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_parse_cube_vector() {
        let ast = parse_to_ast("cube([1, 2, 3]);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, .. } => {
                assert_eq!(*size, CubeSize::Vector([1.0, 2.0, 3.0]));
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_parse_multiple_cubes() {
        let ast = parse_to_ast("cube(10);\ncube([1, 2, 3]);").expect("parse succeeds");
        assert_eq!(ast.len(), 2);
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let result = parse_to_ast("cube(");
        assert!(result.is_err());
        
        let diagnostics = result.unwrap_err();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity(), crate::Severity::Error);
    }

    #[test]
    fn test_parse_invalid_vector_size() {
        let result = parse_to_ast("cube([1, 2]);");
        assert!(result.is_err());
        
        let diagnostics = result.unwrap_err();
        assert!(diagnostics[0].message().contains("3 elements"));
    }

    /// Test parsing cube with named center parameter.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::parse_to_ast;
    /// let ast = parse_to_ast("cube(size=[10,10,10], center=true);").unwrap();
    /// assert_eq!(ast.len(), 1);
    /// ```
    #[test]
    fn test_parse_cube_with_center_named() {
        let ast = parse_to_ast("cube(size=[10,10,10], center=true);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, center, .. } => {
                assert_eq!(*size, CubeSize::Vector([10.0, 10.0, 10.0]));
                assert_eq!(*center, Some(true));
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test parsing cube with positional arguments including center.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::parse_to_ast;
    /// let ast = parse_to_ast("cube([10,10,10], true);").unwrap();
    /// assert_eq!(ast.len(), 1);
    /// ```
    #[test]
    fn test_parse_cube_positional_with_center() {
        let ast = parse_to_ast("cube([10,10,10], true);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, center, .. } => {
                assert_eq!(*size, CubeSize::Vector([10.0, 10.0, 10.0]));
                assert_eq!(*center, Some(true));
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test parsing cube with mixed positional and named arguments.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::parse_to_ast;
    /// let ast = parse_to_ast("cube([10,10,10], center=false);").unwrap();
    /// assert_eq!(ast.len(), 1);
    /// ```
    #[test]
    fn test_parse_cube_mixed_args() {
        let ast = parse_to_ast("cube([10,10,10], center=false);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, center, .. } => {
                assert_eq!(*size, CubeSize::Vector([10.0, 10.0, 10.0]));
                assert_eq!(*center, Some(false));
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test parsing cube with only named arguments.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::parse_to_ast;
    /// let ast = parse_to_ast("cube(size=10, center=true);").unwrap();
    /// assert_eq!(ast.len(), 1);
    /// ```
    #[test]
    fn test_parse_cube_center_only_named() {
        let ast = parse_to_ast("cube(size=10, center=true);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, center, .. } => {
                assert_eq!(*size, CubeSize::Scalar(10.0));
                assert_eq!(*center, Some(true));
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test parsing cube without center parameter defaults to None.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::parse_to_ast;
    /// let ast = parse_to_ast("cube(10);").unwrap();
    /// assert_eq!(ast.len(), 1);
    /// ```
    #[test]
    fn test_parse_cube_center_default() {
        let ast = parse_to_ast("cube(10);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);
        
        match &ast[0] {
            Statement::Cube { size, center, .. } => {
                assert_eq!(*size, CubeSize::Scalar(10.0));
                assert_eq!(*center, None);
            }
            _ => panic!("Expected Cube"),
        }
    }

    #[test]
    fn test_parse_assignment() {
        let ast = parse_to_ast("$fn = 50;").expect("parse succeeds");
        assert_eq!(ast.len(), 1);

        match &ast[0] {
            Statement::Assignment { name, value, .. } => {
                assert_eq!(name, "$fn");
                assert_eq!(*value, 50.0);
            }
            _ => panic!("expected Assignment"),
        }
    }
}

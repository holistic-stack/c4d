//! # Parser Module
//!
//! Converts OpenSCAD source code into a typed AST using tree-sitter.
//!
//! ## Tree-sitter Node Structure
//!
//! The OpenSCAD grammar produces:
//! - `source_file` → contains `transform_chain`, `var_declaration`, `union_block`, etc.
//! - `transform_chain` → `module_call` + child `statement`
//! - `module_call` → `identifier` (name) + `arguments`
//! - `union_block` → `{` + items + `}`

use crate::ast::*;
use crate::diagnostic::{Diagnostic, Severity};
use crate::span::Span;
use glam::DVec3;
use thiserror::Error;

/// Errors that can occur during parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    TreeSitter(String),
    #[error("Syntax errors in source")]
    SyntaxErrors(Vec<Diagnostic>),
    #[error("Unsupported: {0}")]
    Unsupported(String),
}

/// Parses OpenSCAD source code into a typed AST.
pub fn parse_to_ast(source: &str) -> Result<Vec<Statement>, ParseError> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_openscad_parser::LANGUAGE;
    parser
        .set_language(&language.into())
        .map_err(|e| ParseError::TreeSitter(format!("Failed to set language: {}", e)))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| ParseError::TreeSitter("Failed to parse source".to_string()))?;

    let root = tree.root_node();
    let mut diagnostics = Vec::new();
    collect_errors(&root, source, &mut diagnostics);

    if !diagnostics.is_empty() {
        return Err(ParseError::SyntaxErrors(diagnostics));
    }

    let mut statements = Vec::new();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if let Some(stmt) = parse_node(&child, source)? {
            statements.push(stmt);
        }
    }
    Ok(statements)
}

fn collect_errors(node: &tree_sitter::Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    if node.is_error() || node.is_missing() {
        let span = Span::from_ts_node(node);
        let message = if node.is_missing() {
            format!("Missing: {}", node.kind())
        } else {
            let text = &source[node.start_byte()..node.end_byte().min(source.len())];
            format!("Syntax error near '{}'", text.chars().take(20).collect::<String>())
        };
        diagnostics.push(Diagnostic::new(Severity::Error, message, span));
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_errors(&child, source, diagnostics);
    }
}

/// Parses a tree-sitter node into an AST statement.
fn parse_node(node: &tree_sitter::Node, source: &str) -> Result<Option<Statement>, ParseError> {
    let span = Span::from_ts_node(node);
    
    match node.kind() {
        // Module/Function definitions
        "module_declaration" | "module_item" => parse_module_declaration(node, source, span),
        "function_declaration" | "function_item" => parse_function_declaration(node, source, span),

        // Control flow
        "if_block" => parse_if_block(node, source, span),
        "for_block" => parse_for_block(node, source, span),
        
        // Assignments
        "assignment" => parse_assignment(node, source, span),

        // transform_chain: module_call followed by a statement (child)
        "transform_chain" => parse_transform_chain(node, source, span),
        
        // union_block: { items }
        "union_block" => parse_union_block(node, source, span),
        
        // var_declaration: assignment ;
        "var_declaration" => parse_var_declaration(node, source, span),
        
        // Skip these
        "source_file" | "comment" | "line_comment" | "block_comment" | ";" => Ok(None),
        
        // Try to recurse into children
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if let Some(stmt) = parse_node(&child, source)? {
                    return Ok(Some(stmt));
                }
            }
            Ok(None)
        }
    }
}

/// Parses a transform_chain node.
/// Structure: module_call + statement (child)
fn parse_transform_chain(node: &tree_sitter::Node, source: &str, span: Span) -> Result<Option<Statement>, ParseError> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    
    // Find the module_call
    let module_call = children.iter().find(|c| c.kind() == "module_call");
    
    // Find the child statement (another transform_chain, union_block, or ;)
    let child_stmt = children.iter().find(|c| {
        matches!(c.kind(), "transform_chain" | "union_block" | "for_block" | "if_block")
    });
    
    if let Some(mc) = module_call {
        let name = get_module_name(mc, source)?;
        let arguments = parse_module_arguments(mc, source)?;
        
        // Parse child statement if present
        let child_statements = if let Some(child) = child_stmt {
            if let Some(stmt) = parse_node(child, source)? {
                vec![stmt]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        // Convert to appropriate statement type based on module name
        convert_module_call_to_statement(&name, &arguments, child_statements, span)
    } else {
        Ok(None)
    }
}

/// Parses a union_block: { items }
fn parse_union_block(node: &tree_sitter::Node, source: &str, span: Span) -> Result<Option<Statement>, ParseError> {
    let mut children = Vec::new();
    let mut cursor = node.walk();
    
    for child in node.children(&mut cursor) {
        if let Some(stmt) = parse_node(&child, source)? {
            children.push(stmt);
        }
    }
    
    // A bare union_block is implicitly a union
    Ok(Some(Statement::Union { children, span }))
}

/// Parses a var_declaration: assignment ;
fn parse_var_declaration(node: &tree_sitter::Node, source: &str, span: Span) -> Result<Option<Statement>, ParseError> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "assignment" {
            return parse_assignment(&child, source, span);
        }
    }
    Ok(None)
}

/// Gets the module name from a module_call node.
fn get_module_name(node: &tree_sitter::Node, source: &str) -> Result<String, ParseError> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return Ok(source[child.start_byte()..child.end_byte()].to_string());
        }
    }
    Err(ParseError::TreeSitter("Module call missing name".to_string()))
}

/// Parses arguments from a module_call node.
fn parse_module_arguments(node: &tree_sitter::Node, source: &str) -> Result<Vec<Argument>, ParseError> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "arguments" {
            return parse_arguments(&child, source);
        }
    }
    Ok(Vec::new())
}

/// Converts a module call to the appropriate statement type.
fn convert_module_call_to_statement(
    name: &str,
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, ParseError> {
    // Flatten nested booleans: union() { ... } parses as Union with Union child
    // We want to extract the inner children
    let flattened_children = flatten_boolean_children(name, children);
    
    match name {
        // 3D Primitives
        "cube" => parse_cube_call(arguments, span),
        "sphere" => parse_sphere_call(arguments, span),
        "cylinder" => parse_cylinder_call(arguments, span),
        
        // Transforms
        "translate" => parse_translate_call(arguments, flattened_children, span),
        "rotate" => parse_rotate_call(arguments, flattened_children, span),
        "scale" => parse_scale_call(arguments, flattened_children, span),
        "mirror" => parse_mirror_call(arguments, flattened_children, span),
        "color" => parse_color_call(arguments, flattened_children, span),
        
        // Booleans
        "union" => Ok(Some(Statement::Union { children: flattened_children, span })),
        "difference" => Ok(Some(Statement::Difference { children: flattened_children, span })),
        "intersection" => Ok(Some(Statement::Intersection { children: flattened_children, span })),
        
        // Advanced
        "hull" => Ok(Some(Statement::Hull { children: flattened_children, span })),
        "minkowski" => Ok(Some(Statement::Minkowski { convexity: 1, children: flattened_children, span })),
        
        // Extrusions
        "linear_extrude" => parse_linear_extrude_call(arguments, flattened_children, span),
        "rotate_extrude" => parse_rotate_extrude_call(arguments, flattened_children, span),
        
        // Generic module call
        _ => Ok(Some(Statement::ModuleCall {
            name: name.to_string(),
            arguments: arguments.to_vec(),
            children: flattened_children,
            span,
        })),
    }
}

/// Flattens nested boolean children.
/// When we have union() { ... }, the parser creates Union with a single Union child.
/// This extracts the inner children.
fn flatten_boolean_children(name: &str, children: Vec<Statement>) -> Vec<Statement> {
    if children.len() == 1 {
        match (&children[0], name) {
            (Statement::Union { children: inner, .. }, "union") => inner.clone(),
            (Statement::Difference { children: inner, .. }, "difference") => inner.clone(),
            (Statement::Intersection { children: inner, .. }, "intersection") => inner.clone(),
            // Also flatten when the child is a Union (from union_block) regardless of parent name
            (Statement::Union { children: inner, .. }, _) => inner.clone(),
            _ => children,
        }
    } else {
        children
    }
}


fn parse_assignment(node: &tree_sitter::Node, source: &str, span: Span) -> Result<Option<Statement>, ParseError> {
    // Grammar uses "name" and "value" fields
    let name = get_node_text(node.child_by_field_name("name"), source)?;
    let value_node = node.child_by_field_name("value")
        .ok_or_else(|| ParseError::TreeSitter("Assignment missing value".to_string()))?;
    let value = parse_expression(&value_node, source)?;
    Ok(Some(Statement::Assignment { name, value, span }))
}

fn parse_arguments(node: &tree_sitter::Node, source: &str) -> Result<Vec<Argument>, ParseError> {
    let mut arguments = Vec::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            // Named argument: name=value
            "assignment" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = source[name_node.start_byte()..name_node.end_byte()].to_string();
                    if let Some(value_node) = child.child_by_field_name("value") {
                        let value = parse_expression(&value_node, source)?;
                        arguments.push(Argument::named(name, value));
                    }
                }
            }
            // Positional arguments
            "integer" | "decimal" | "number" | "boolean" | "string" | "list" | "identifier" => {
                let value = parse_expression(&child, source)?;
                arguments.push(Argument::positional(value));
            }
            // Skip punctuation
            "(" | ")" | "," => {}
            _ => {}
        }
    }
    Ok(arguments)
}

fn parse_expression(node: &tree_sitter::Node, source: &str) -> Result<Expression, ParseError> {
    match node.kind() {
        "number" | "integer" | "decimal" => {
            let text = &source[node.start_byte()..node.end_byte()];
            let value = text.parse::<f64>().map_err(|_| ParseError::TreeSitter(format!("Invalid number: {}", text)))?;
            Ok(Expression::Number(value))
        }
        "boolean" => {
            let text = &source[node.start_byte()..node.end_byte()];
            Ok(Expression::Boolean(text == "true"))
        }
        "string" => {
            let text = &source[node.start_byte()..node.end_byte()];
            Ok(Expression::String(text.trim_matches('"').to_string()))
        }
        "identifier" | "special_variable" => {
            let text = &source[node.start_byte()..node.end_byte()];
            Ok(Expression::Variable(text.to_string()))
        }
        "vector" | "list" => {
            let mut elements = Vec::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() != "[" && child.kind() != "]" && child.kind() != "," {
                    elements.push(parse_expression(&child, source)?);
                }
            }
            Ok(Expression::Vector(elements))
        }
        _ => {
            let text = &source[node.start_byte()..node.end_byte()];
            if let Ok(value) = text.parse::<f64>() {
                return Ok(Expression::Number(value));
            }
            Ok(Expression::Variable(text.to_string()))
        }
    }
}

fn get_node_text(node: Option<tree_sitter::Node>, source: &str) -> Result<String, ParseError> {
    let n = node.ok_or_else(|| ParseError::TreeSitter("Missing node".to_string()))?;
    Ok(source[n.start_byte()..n.end_byte()].to_string())
}

/// Parses cube() call, storing size as Expression for deferred evaluation.
fn parse_cube_call(arguments: &[Argument], span: Span) -> Result<Option<Statement>, ParseError> {
    // Default size as expression
    let mut size = Expression::Vector(vec![
        Expression::Number(1.0), Expression::Number(1.0), Expression::Number(1.0)
    ]);
    let mut center = false;
    for (i, arg) in arguments.iter().enumerate() {
        match (&arg.name, &arg.value, i) {
            (Some(name), expr, _) if name == "size" => size = expr.clone(),
            (Some(name), Expression::Boolean(b), _) if name == "center" => center = *b,
            (None, expr, 0) if !matches!(expr, Expression::Boolean(_)) => size = expr.clone(),
            (None, Expression::Boolean(b), 1) => center = *b,
            _ => {}
        }
    }
    Ok(Some(Statement::Cube { size, center, span }))
}

/// Parses sphere() call, storing radius as Expression for deferred evaluation.
/// The $fn parameter is stored as an optional override for segment calculation.
fn parse_sphere_call(arguments: &[Argument], span: Span) -> Result<Option<Statement>, ParseError> {
    let mut radius = Expression::Number(1.0);
    let mut fn_override: Option<f64> = None;
    for (i, arg) in arguments.iter().enumerate() {
        match (&arg.name, &arg.value, i) {
            (Some(name), expr, _) if name == "r" => radius = expr.clone(),
            (Some(name), expr, _) if name == "d" => {
                // d = diameter, so radius = d / 2
                radius = Expression::Binary {
                    left: Box::new(expr.clone()),
                    operator: crate::ast::BinaryOp::Divide,
                    right: Box::new(Expression::Number(2.0)),
                };
            }
            (Some(name), Expression::Number(n), _) if name == "$fn" => fn_override = Some(*n),
            (None, expr, 0) => radius = expr.clone(),
            _ => {}
        }
    }
    Ok(Some(Statement::Sphere { radius, fn_override, span }))
}

/// Parses cylinder() call, storing dimensions as Expressions for deferred evaluation.
/// The $fn parameter is stored as an optional override for segment calculation.
fn parse_cylinder_call(arguments: &[Argument], span: Span) -> Result<Option<Statement>, ParseError> {
    let mut height = Expression::Number(1.0);
    let mut r_bot = Expression::Number(1.0);
    let mut r_top = Expression::Number(1.0);
    let mut center = false;
    let mut fn_override: Option<f64> = None;
    for (i, arg) in arguments.iter().enumerate() {
        match (&arg.name, &arg.value, i) {
            (Some(n), expr, _) if n == "h" => height = expr.clone(),
            (Some(n), expr, _) if n == "r" => { r_bot = expr.clone(); r_top = expr.clone(); }
            (Some(n), expr, _) if n == "r1" => r_bot = expr.clone(),
            (Some(n), expr, _) if n == "r2" => r_top = expr.clone(),
            (Some(n), Expression::Boolean(b), _) if n == "center" => center = *b,
            (Some(n), Expression::Number(v), _) if n == "$fn" => fn_override = Some(*v),
            (None, expr, 0) => height = expr.clone(),
            (None, expr, 1) => { r_bot = expr.clone(); r_top = expr.clone(); }
            _ => {}
        }
    }
    Ok(Some(Statement::Cylinder { height, radius_bottom: r_bot, radius_top: r_top, center, fn_override, span }))
}

fn parse_translate_call(arguments: &[Argument], children: Vec<Statement>, span: Span) -> Result<Option<Statement>, ParseError> {
    let vector = arguments.first()
        .map(|a| a.value.clone())
        .unwrap_or(Expression::Vector(vec![
            Expression::Number(0.0), Expression::Number(0.0), Expression::Number(0.0)
        ]));
    Ok(Some(Statement::Translate { vector, children, span }))
}

fn parse_rotate_call(arguments: &[Argument], children: Vec<Statement>, span: Span) -> Result<Option<Statement>, ParseError> {
    let angles = arguments.first()
        .map(|a| a.value.clone())
        .unwrap_or(Expression::Vector(vec![
            Expression::Number(0.0), Expression::Number(0.0), Expression::Number(0.0)
        ]));
    Ok(Some(Statement::Rotate { angles, axis: None, children, span }))
}

fn parse_scale_call(arguments: &[Argument], children: Vec<Statement>, span: Span) -> Result<Option<Statement>, ParseError> {
    let factors = arguments.first()
        .map(|a| a.value.clone())
        .unwrap_or(Expression::Vector(vec![
            Expression::Number(1.0), Expression::Number(1.0), Expression::Number(1.0)
        ]));
    Ok(Some(Statement::Scale { factors, children, span }))
}

fn parse_mirror_call(arguments: &[Argument], children: Vec<Statement>, span: Span) -> Result<Option<Statement>, ParseError> {
    let normal = arguments.first()
        .map(|a| a.value.clone())
        .unwrap_or(Expression::Vector(vec![
            Expression::Number(1.0), Expression::Number(0.0), Expression::Number(0.0)
        ]));
    Ok(Some(Statement::Mirror { normal, children, span }))
}

fn parse_color_call(arguments: &[Argument], children: Vec<Statement>, span: Span) -> Result<Option<Statement>, ParseError> {
    let mut color = [0.8f32, 0.8, 0.8, 1.0];
    for arg in arguments {
        match &arg.value {
            Expression::Vector(v) => {
                if v.len() >= 3 {
                    if let (Some(Expression::Number(r)), Some(Expression::Number(g)), Some(Expression::Number(b))) =
                        (v.first(), v.get(1), v.get(2))
                    {
                        color[0] = *r as f32;
                        color[1] = *g as f32;
                        color[2] = *b as f32;
                        if let Some(Expression::Number(a)) = v.get(3) {
                            color[3] = *a as f32;
                        }
                    }
                }
            }
            Expression::String(name) => {
                // Handle named colors (simplified)
                color = match name.as_str() {
                    "red" => [1.0, 0.0, 0.0, 1.0],
                    "green" => [0.0, 1.0, 0.0, 1.0],
                    "blue" => [0.0, 0.0, 1.0, 1.0],
                    "yellow" => [1.0, 1.0, 0.0, 1.0],
                    "white" => [1.0, 1.0, 1.0, 1.0],
                    "black" => [0.0, 0.0, 0.0, 1.0],
                    _ => [0.8, 0.8, 0.8, 1.0],
                };
            }
            _ => {}
        }
    }
    Ok(Some(Statement::Color { color, children, span }))
}

fn parse_linear_extrude_call(arguments: &[Argument], children: Vec<Statement>, span: Span) -> Result<Option<Statement>, ParseError> {
    let mut height = 1.0;
    let mut center = false;
    let mut twist = 0.0;
    let mut slices = 1;
    let mut scale = [1.0, 1.0];

    for arg in arguments {
        match (&arg.name, &arg.value) {
            (Some(n), Expression::Number(v)) if n == "height" || n == "h" => height = *v,
            (Some(n), Expression::Boolean(b)) if n == "center" => center = *b,
            (Some(n), Expression::Number(v)) if n == "twist" => twist = *v,
            (Some(n), Expression::Number(v)) if n == "slices" => slices = *v as u32,
            (Some(n), Expression::Number(v)) if n == "scale" => scale = [*v, *v],
            (Some(n), Expression::Vector(v)) if n == "scale" => {
                if let (Some(Expression::Number(x)), Some(Expression::Number(y))) = (v.first(), v.get(1)) {
                    scale = [*x, *y];
                }
            }
            (None, Expression::Number(v)) => height = *v,
            _ => {}
        }
    }

    Ok(Some(Statement::LinearExtrude {
        height,
        center,
        twist,
        slices,
        scale,
        children,
        span,
    }))
}

fn parse_rotate_extrude_call(arguments: &[Argument], children: Vec<Statement>, span: Span) -> Result<Option<Statement>, ParseError> {
    let mut angle = 360.0;
    let mut convexity = 1;

    for arg in arguments {
        match (&arg.name, &arg.value) {
            (Some(n), Expression::Number(v)) if n == "angle" => angle = *v,
            (Some(n), Expression::Number(v)) if n == "convexity" => convexity = *v as u32,
            _ => {}
        }
    }

    Ok(Some(Statement::RotateExtrude {
        angle,
        convexity,
        children,
        span,
    }))
}

/// Parses a module declaration: module name(params) statement
fn parse_module_declaration(node: &tree_sitter::Node, source: &str, span: Span) -> Result<Option<Statement>, ParseError> {
    let mut name = String::new();
    let mut parameters = Vec::new();
    let mut body = Vec::new();
    
    // Get module name
    if let Some(name_node) = node.child_by_field_name("name") {
        name = get_node_text(Some(name_node), source)?;
    } else {
        // Fallback: look for identifier
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                name = get_node_text(Some(child), source)?;
                break;
            }
        }
    }
    
    // Parse parameters
    if let Some(params_node) = node.child_by_field_name("parameters") {
        parameters = parse_parameters(&params_node, source)?;
    }
    
    // Parse body
    if let Some(body_node) = node.child_by_field_name("body") {
        if let Some(stmt) = parse_node(&body_node, source)? {
            // If body is a block, it might return a Union or just be a list of statements
            // The AST expects Vec<Statement>.
            // If parse_node returns a Block/Union, we might want to unwrap it, or just wrap single stmt
            body.push(stmt);
        }
    } else {
        // Fallback: look for non-parameter children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let kind = child.kind();
            if kind != "identifier" && kind != "parameters" && kind != "module" && kind != "function" {
                 if let Some(stmt) = parse_node(&child, source)? {
                     body.push(stmt);
                 }
            }
        }
    }

    // Flatten body if it's a Union (common for block bodies)
    if body.len() == 1 {
        if let Statement::Union { children, .. } = &body[0] {
            body = children.clone();
        }
    }

    Ok(Some(Statement::ModuleDefinition {
        name,
        parameters,
        body,
        span,
    }))
}

/// Parses a function declaration: function name(params) = expression;
fn parse_function_declaration(node: &tree_sitter::Node, source: &str, span: Span) -> Result<Option<Statement>, ParseError> {
    let mut name = String::new();
    let mut parameters = Vec::new();
    
    // Get function name
    if let Some(name_node) = node.child_by_field_name("name") {
        name = get_node_text(Some(name_node), source)?;
    }
    
    // Parse parameters
    if let Some(params_node) = node.child_by_field_name("parameters") {
        parameters = parse_parameters(&params_node, source)?;
    }
    
    // Parse body expression
    let body_expr = if let Some(body_node) = node.child_by_field_name("body") {
        parse_expression(&body_node, source)?
    } else {
        // Look for expression after =
        let mut cursor = node.walk();
        let mut found_eq = false;
        let mut expr = Expression::Number(0.0); // Default placeholder
        for child in node.children(&mut cursor) {
            if child.kind() == "=" {
                found_eq = true;
                continue;
            }
            if found_eq {
                expr = parse_expression(&child, source)?;
                break;
            }
        }
        expr
    };

    Ok(Some(Statement::FunctionDefinition {
        name,
        parameters,
        body: body_expr,
        span,
    }))
}

/// Parses parameters for modules and functions
fn parse_parameters(node: &tree_sitter::Node, source: &str) -> Result<Vec<Parameter>, ParseError> {
    let mut parameters = Vec::new();
    let mut cursor = node.walk();
    
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                let name = get_node_text(Some(child), source)?;
                parameters.push(Parameter::required(name));
            }
            "assignment" => {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = get_node_text(Some(name_node), source)?;
                    if let Some(value_node) = child.child_by_field_name("value") {
                        let default_val = parse_expression(&value_node, source)?;
                        parameters.push(Parameter::optional(name, default_val));
                    }
                }
            }
            "parameter" => {
                 // Some grammars wrap identifier/assignment in parameter node
                 let mut inner_cursor = child.walk();
                 for inner in child.children(&mut inner_cursor) {
                     if inner.kind() == "identifier" {
                         let name = get_node_text(Some(inner), source)?;
                         parameters.push(Parameter::required(name));
                     } else if inner.kind() == "assignment" {
                        if let Some(name_node) = inner.child_by_field_name("name") {
                            let name = get_node_text(Some(name_node), source)?;
                            if let Some(value_node) = inner.child_by_field_name("value") {
                                let default_val = parse_expression(&value_node, source)?;
                                parameters.push(Parameter::optional(name, default_val));
                            }
                        }
                     }
                 }
            }
            _ => {}
        }
    }
    Ok(parameters)
}

/// Parses if block
fn parse_if_block(node: &tree_sitter::Node, source: &str, span: Span) -> Result<Option<Statement>, ParseError> {
    let mut condition = Expression::Boolean(true);
    let mut then_branch = Vec::new();
    let mut else_branch = None;

    if let Some(cond_node) = node.child_by_field_name("condition") {
        // Usually condition is parenthesized_expression
        if cond_node.kind() == "parenthesized_expression" {
            let mut cursor = cond_node.walk();
            for child in cond_node.children(&mut cursor) {
                 if child.kind() != "(" && child.kind() != ")" {
                     condition = parse_expression(&child, source)?;
                     break;
                 }
            }
        } else {
            condition = parse_expression(&cond_node, source)?;
        }
    }

    if let Some(consequence) = node.child_by_field_name("consequence") {
        if let Some(stmt) = parse_node(&consequence, source)? {
            // Flatten if block
            if let Statement::Union { children, .. } = stmt {
                then_branch = children;
            } else {
                then_branch.push(stmt);
            }
        }
    }
    
    if let Some(alternative) = node.child_by_field_name("alternative") {
        // alternative often includes "else" keyword node, we want the statement
        let mut else_stmts = Vec::new();
        // The field 'alternative' points to the statement after 'else' usually?
        // Let's check if it points to 'else' or the statement.
        // If it's the statement, parse it.
        if alternative.kind() == "else" {
            // This shouldn't happen if field points to the alternative statement, but let's be safe
        } else {
             if let Some(stmt) = parse_node(&alternative, source)? {
                if let Statement::Union { children, .. } = stmt {
                    else_stmts = children;
                } else {
                    else_stmts.push(stmt);
                }
             }
        }
        else_branch = Some(else_stmts);
    }

    Ok(Some(Statement::If {
        condition,
        then_branch,
        else_branch,
        span,
    }))
}

/// Parses for block
fn parse_for_block(node: &tree_sitter::Node, source: &str, span: Span) -> Result<Option<Statement>, ParseError> {
    // Basic support for single assignment for loops: for(i=[0:10]) ...
    let mut variable = String::new();
    let mut range = Expression::Vector(vec![]);
    let mut body = Vec::new();

    // This is simplified. Real for loops are complex in OpenSCAD (multiple assignments, etc.)
    // We assume standard `for(var = range) body` structure
    
    // Try to find assignments
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "assignment" {
             if let Some(name_node) = child.child_by_field_name("name") {
                 variable = get_node_text(Some(name_node), source)?;
             }
             if let Some(value_node) = child.child_by_field_name("value") {
                 range = parse_expression(&value_node, source)?;
             }
        } else if child.kind() == "parenthesized_expression" || child.kind() == "assignments" {
             // assignments inside parens
             let mut inner = child.walk();
             for inner_child in child.children(&mut inner) {
                 if inner_child.kind() == "assignment" {
                     if let Some(name_node) = inner_child.child_by_field_name("name") {
                         variable = get_node_text(Some(name_node), source)?;
                     }
                     if let Some(value_node) = inner_child.child_by_field_name("value") {
                         range = parse_expression(&value_node, source)?;
                     }
                 }
             }
        }
    }

    // Body
    if let Some(body_node) = node.child_by_field_name("body") {
         if let Some(stmt) = parse_node(&body_node, source)? {
            if let Statement::Union { children, .. } = stmt {
                body = children;
            } else {
                body.push(stmt);
            }
         }
    }

    Ok(Some(Statement::ForLoop {
        variable,
        range,
        body,
        span,
    }))
}

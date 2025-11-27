//! # CST Parser Module
//!
//! Converts a serialized CST (from web-tree-sitter) into a typed AST.
//! This module provides browser-safe parsing by accepting pre-parsed CST
//! from JavaScript instead of using the Rust tree-sitter crate.
//!
//! ## Architecture
//!
//! ```text
//! Browser: OpenSCAD Source → web-tree-sitter → Serialized CST (JSON)
//! WASM: Serialized CST → cst_parser → AST → Geometry IR → Mesh
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use openscad_ast::cst::SerializedNode;
//! use openscad_ast::cst_parser::parse_from_cst;
//!
//! let cst: SerializedNode = serde_json::from_str(json)?;
//! let statements = parse_from_cst(&cst)?;
//! ```

use crate::ast::{Argument, BinaryOp, Expression, Modifier, Statement};
use crate::cst::SerializedNode;
use crate::diagnostic::{Diagnostic, Severity};
use crate::span::Span;
use config::constants::DEFAULT_CONVEXITY;
use glam::DVec3;
use thiserror::Error;

/// Errors that can occur during CST parsing.
#[derive(Debug, Error)]
pub enum CstParseError {
    /// Syntax errors found in the CST
    #[error("Syntax errors in source")]
    SyntaxErrors(Vec<Diagnostic>),

    /// Unsupported syntax construct
    #[error("Unsupported: {0}")]
    Unsupported(String),

    /// Invalid value in CST
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

/// Parses a serialized CST into a typed AST.
///
/// # Arguments
///
/// * `root` - The root node of the serialized CST
///
/// # Returns
///
/// A vector of AST statements, or an error if parsing fails.
///
/// # Example
///
/// ```rust,ignore
/// use openscad_ast::cst::SerializedNode;
/// use openscad_ast::cst_parser::parse_from_cst;
///
/// let cst: SerializedNode = serde_json::from_str(json)?;
/// let statements = parse_from_cst(&cst)?;
/// ```
pub fn parse_from_cst(root: &SerializedNode) -> Result<Vec<Statement>, CstParseError> {
    // Collect syntax errors
    let mut diagnostics = Vec::new();
    collect_errors(root, &mut diagnostics);

    if !diagnostics.is_empty() {
        return Err(CstParseError::SyntaxErrors(diagnostics));
    }

    // Parse statements
    let mut statements = Vec::new();
    for child in &root.children {
        if let Some(stmt) = parse_node(child)? {
            statements.push(stmt);
        }
    }
    Ok(statements)
}

/// Collects syntax errors from the CST.
fn collect_errors(node: &SerializedNode, diagnostics: &mut Vec<Diagnostic>) {
    if node.is_error() || node.is_missing() {
        let span = Span::new(node.start_index, node.end_index);
        let message = if node.is_missing() {
            format!("Missing: {}", node.node_type)
        } else {
            format!(
                "Syntax error near '{}'",
                node.text.chars().take(20).collect::<String>()
            )
        };
        diagnostics.push(Diagnostic::new(Severity::Error, message, span));
    }

    for child in &node.children {
        collect_errors(child, diagnostics);
    }
}

/// Parses a CST node into an AST statement.
fn parse_node(node: &SerializedNode) -> Result<Option<Statement>, CstParseError> {
    let span = Span::new(node.start_index, node.end_index);

    match node.node_type.as_str() {
        // Modifier nodes: *, !, #, %
        // These wrap the following statement
        "modifier" | "modifier_chain" => parse_modifier(node, span),

        // transform_chain: module_call followed by a statement (child)
        "transform_chain" => parse_transform_chain(node, span),

        // union_block: { items }
        "union_block" => parse_union_block(node, span),

        // var_declaration: assignment ;
        "var_declaration" => parse_var_declaration(node, span),

        // for_block: for (assignments) statement
        "for_block" => {
            let mut variable = String::new();
            let mut range = Expression::Number(0.0);
            let mut body = Vec::new();
            // Look for assignments node
            for child in &node.children {
                if child.node_type == "assignments" {
                    for assign in &child.children {
                        if assign.node_type == "assignment" {
                            if let Some(n) = assign.child_by_field("name") { variable = n.text.clone(); }
                            if let Some(v) = assign.child_by_field("value") { range = parse_expression(v)?; }
                        }
                    }
                }
            }
            // Find body
            for child in &node.children {
                if matches!(child.node_type.as_str(), "transform_chain" | "union_block" | "for_block" | "if_block") {
                    if let Some(stmt) = parse_node(child)? { body.push(stmt); }
                }
            }
            Ok(Some(Statement::ForLoop { variable, range, body, span }))
        }

        // if_block: if (condition) statement [else statement]
        "if_block" => {
            let mut condition = Expression::Boolean(true);
            let mut then_branch = Vec::new();
            let mut else_branch: Option<Vec<Statement>> = None;
            // Find condition
            for child in &node.children {
                if child.node_type == "parenthesized_expression" {
                    if let Some(inner) = child.named_children.first() {
                        condition = parse_expression(inner)?;
                    }
                }
            }
            // Find then/else
            let mut in_else = false;
            for child in &node.children {
                if child.node_type == "else" { in_else = true; continue; }
                if matches!(child.node_type.as_str(), "transform_chain" | "union_block" | "for_block" | "if_block") {
                    if let Some(stmt) = parse_node(child)? {
                        if in_else { else_branch.get_or_insert_with(Vec::new).push(stmt); }
                        else { then_branch.push(stmt); }
                    }
                }
            }
            Ok(Some(Statement::If { condition, then_branch, else_branch, span }))
        }

        // module_item: module name(params) statement
        "module_item" | "module_declaration" => {
            let mut name = String::new();
            let mut parameters = Vec::new();
            let mut body = Vec::new();
            
            // Get module name from field
            if let Some(name_node) = node.child_by_field("name") {
                name = name_node.text.clone();
            } else {
                for child in &node.children {
                    if child.node_type == "identifier" && name.is_empty() {
                        name = child.text.clone();
                    }
                }
            }
            
            // Parse parameters - handle both named_children and children
            // The CST structure is: parameters -> parameter -> identifier
            // or: parameters -> assignment (for default values)
            if let Some(params_node) = node.child_by_field("parameters") {
                for child in &params_node.named_children {
                    match child.node_type.as_str() {
                        // Direct identifier (some grammar versions)
                        "identifier" => {
                            parameters.push(crate::ast::Parameter { 
                                name: child.text.clone(), 
                                default: None 
                            });
                        }
                        // Parameter node wrapping an identifier
                        "parameter" => {
                            // Look for identifier inside parameter node
                            if let Some(id_node) = child.named_children.first() {
                                if id_node.node_type == "identifier" {
                                    parameters.push(crate::ast::Parameter { 
                                        name: id_node.text.clone(), 
                                        default: None 
                                    });
                                }
                            } else {
                                // Fallback: use the parameter's text directly
                                parameters.push(crate::ast::Parameter { 
                                    name: child.text.clone(), 
                                    default: None 
                                });
                            }
                        }
                        // Assignment for default values: param = value
                        "assignment" => {
                            let param_name = child.child_by_field("name")
                                .map(|n| n.text.clone())
                                .unwrap_or_default();
                            let default_val = child.child_by_field("value")
                                .map(parse_expression)
                                .transpose()?;
                            parameters.push(crate::ast::Parameter { 
                                name: param_name, 
                                default: default_val 
                            });
                        }
                        _ => {
                            // Skip other node types
                        }
                    }
                }
            }
            
            // Parse body (can be union_block or single statement)
            if let Some(body_node) = node.child_by_field("body") {
                if body_node.node_type == "union_block" {
                    for c in &body_node.children {
                        if let Some(s) = parse_node(c)? { body.push(s); }
                    }
                } else if let Some(s) = parse_node(&body_node)? {
                    body.push(s);
                }
            } else {
                for child in &node.children {
                    if child.node_type == "union_block" {
                        for c in &child.children {
                            if let Some(s) = parse_node(c)? { body.push(s); }
                        }
                    }
                }
            }
            
            Ok(Some(Statement::ModuleDefinition { name, parameters, body, span }))
        }

        // function_item: function name(params) = expression;
        "function_item" => {
            let mut name = String::new();
            let mut parameters = Vec::new();
            let mut body = Expression::Number(0.0);
            
            // Get function name from field
            if let Some(name_node) = node.child_by_field("name") {
                name = name_node.text.clone();
            }
            
            // Parse parameters - use named_children to skip anonymous tokens like parens
            // The CST structure is: parameters -> parameter -> identifier
            // or: parameters -> assignment (for default values)
            if let Some(params_node) = node.child_by_field("parameters") {
                for child in &params_node.named_children {
                    match child.node_type.as_str() {
                        // Direct identifier (some grammar versions)
                        "identifier" => {
                            parameters.push(crate::ast::Parameter { 
                                name: child.text.clone(), 
                                default: None 
                            });
                        }
                        // Parameter node wrapping an identifier
                        "parameter" => {
                            // Look for identifier inside parameter node
                            if let Some(id_node) = child.named_children.first() {
                                if id_node.node_type == "identifier" {
                                    parameters.push(crate::ast::Parameter { 
                                        name: id_node.text.clone(), 
                                        default: None 
                                    });
                                }
                            } else {
                                // Fallback: use the parameter's text directly
                                parameters.push(crate::ast::Parameter { 
                                    name: child.text.clone(), 
                                    default: None 
                                });
                            }
                        }
                        // Assignment for default values: param = value
                        "assignment" => {
                            let param_name = child.child_by_field("name")
                                .map(|n| n.text.clone())
                                .unwrap_or_default();
                            let default_val = child.child_by_field("value")
                                .map(parse_expression)
                                .transpose()?;
                            parameters.push(crate::ast::Parameter { 
                                name: param_name, 
                                default: default_val 
                            });
                        }
                        _ => {
                            // Skip other node types
                        }
                    }
                }
            }
            
            // Parse body expression using multiple strategies
            // Strategy 1: Try field-based access (most reliable)
            if let Some(body_node) = node.child_by_field("body") {
                body = parse_expression(body_node)?;
            } 
            // Strategy 2: Find expression after parameters in named_children
            else {
                // Skip the name identifier and parameters node, take the next expression
                let mut found_params = false;
                for child in &node.named_children {
                    if child.node_type == "parameters" {
                        found_params = true;
                        continue;
                    }
                    // After parameters, any expression-like node is the body
                    if found_params && child.node_type != "identifier" {
                        body = parse_expression(child)?;
                        break;
                    }
                    // If we haven't found params yet, check if this child looks like an expression
                    // (not an identifier which would be the function name)
                    if found_params {
                        body = parse_expression(child)?;
                        break;
                    }
                }
                // Strategy 3: Last resort - use last named child that's not name or params
                if body == Expression::Number(0.0) {
                    if let Some(body_node) = node.named_children.last() {
                        if body_node.node_type != "parameters" && body_node.node_type != "identifier" {
                            body = parse_expression(body_node)?;
                        } else if body_node.node_type == "identifier" {
                            // The body might be a simple variable reference
                            body = Expression::Variable(body_node.text.clone());
                        }
                    }
                }
            }
            
            Ok(Some(Statement::FunctionDefinition { name, parameters, body, span }))
        }

        // Echo statement: echo(args) statement
        // OpenSCAD's echo() prints to console and continues with the statement
        "echo_statement" => {
            // Parse the following statement if present
            if let Some(stmt_node) = node.child_by_field("statement") {
                parse_node(&stmt_node)
            } else {
                // Echo without a following statement is a no-op
                Ok(None)
            }
        }

        // Assert statement: assert(args) statement
        // OpenSCAD's assert() checks conditions and continues with the statement
        "assert_statement" => {
            // Parse the following statement if present
            if let Some(stmt_node) = node.child_by_field("statement") {
                parse_node(&stmt_node)
            } else {
                // Assert without a following statement is a no-op
                Ok(None)
            }
        }

        // Skip these - they don't produce geometry
        "source_file" | "comment" | "line_comment" | "block_comment" | ";" => Ok(None),

        // Try to recurse into children
        _ => {
            for child in &node.children {
                if let Some(stmt) = parse_node(child)? {
                    return Ok(Some(stmt));
                }
            }
            Ok(None)
        }
    }
}

/// Parses a transform_chain node.
fn parse_transform_chain(
    node: &SerializedNode,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    // Find the module_call
    let module_call = node.find_child("module_call");

    // Find the child statement
    let child_stmt = node.children.iter().find(|c| {
        matches!(
            c.node_type.as_str(),
            "transform_chain" | "union_block" | "for_block" | "if_block"
        )
    });

    if let Some(mc) = module_call {
        let name = get_module_name(mc)?;
        let arguments = parse_module_arguments(mc)?;

        // Parse child statement if present
        let child_statements = if let Some(child) = child_stmt {
            if let Some(stmt) = parse_node(child)? {
                vec![stmt]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        convert_module_call_to_statement(&name, &arguments, child_statements, span)
    } else {
        Ok(None)
    }
}

/// Parses a union_block: { items }
fn parse_union_block(node: &SerializedNode, span: Span) -> Result<Option<Statement>, CstParseError> {
    let mut children = Vec::new();

    for child in &node.children {
        if let Some(stmt) = parse_node(child)? {
            children.push(stmt);
        }
    }

    Ok(Some(Statement::Union { children, span }))
}

/// Parses a modifier node (*, !, #, %) that wraps a statement.
///
/// OpenSCAD modifiers are prefix operators that change rendering behavior:
/// - `*` (Disable): Object is not rendered
/// - `!` (ShowOnly): Only this object is rendered
/// - `#` (Highlight): Object is rendered in magenta
/// - `%` (Transparent): Object is rendered semi-transparent
///
/// # Arguments
///
/// * `node` - The modifier CST node
/// * `span` - The source span of the modifier
///
/// # Returns
///
/// A Modified statement wrapping the child statement.
fn parse_modifier(node: &SerializedNode, span: Span) -> Result<Option<Statement>, CstParseError> {
    // Determine the modifier type from the node text or first character
    let modifier_char = node.text.chars().next().unwrap_or('*');
    let modifier = Modifier::from_char(modifier_char).unwrap_or(Modifier::Disable);

    // Find the child statement to modify
    // The child can be a transform_chain, union_block, module_call, etc.
    for child in &node.children {
        match child.node_type.as_str() {
            // Skip the modifier character itself
            "*" | "!" | "#" | "%" => continue,
            // Parse the actual statement
            _ => {
                if let Some(child_stmt) = parse_node(child)? {
                    return Ok(Some(Statement::Modified {
                        modifier,
                        child: Box::new(child_stmt),
                        span,
                    }));
                }
            }
        }
    }

    // If no child found, try named_children
    for child in &node.named_children {
        if let Some(child_stmt) = parse_node(child)? {
            return Ok(Some(Statement::Modified {
                modifier,
                child: Box::new(child_stmt),
                span,
            }));
        }
    }

    // No valid child statement found
    Ok(None)
}

/// Parses a var_declaration: assignment ;
fn parse_var_declaration(
    node: &SerializedNode,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    for child in &node.children {
        if child.node_type == "assignment" {
            return parse_assignment(child, span);
        }
    }
    Ok(None)
}

/// Gets the module name from a module_call node.
fn get_module_name(node: &SerializedNode) -> Result<String, CstParseError> {
    for child in &node.children {
        if child.node_type == "identifier" {
            return Ok(child.text.clone());
        }
    }
    Err(CstParseError::InvalidValue(
        "Module call missing name".to_string(),
    ))
}

/// Parses arguments from a module_call node.
fn parse_module_arguments(node: &SerializedNode) -> Result<Vec<Argument>, CstParseError> {
    for child in &node.children {
        if child.node_type == "arguments" {
            return parse_arguments(child);
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
) -> Result<Option<Statement>, CstParseError> {
    let flattened_children = flatten_boolean_children(name, children);

    match name {
        // 3D Primitives
        "cube" => parse_cube_call(arguments, span),
        "sphere" => parse_sphere_call(arguments, span),
        "cylinder" => parse_cylinder_call(arguments, span),
        "polyhedron" => parse_polyhedron_call(arguments, span),

        // 2D Primitives
        "circle" => parse_circle_call(arguments, span),
        "square" => parse_square_call(arguments, span),
        "polygon" => parse_polygon_call(arguments, span),

                // Transforms
        "translate" => parse_translate_call(arguments, flattened_children, span),
        "rotate" => parse_rotate_call(arguments, flattened_children, span),
        "scale" => parse_scale_call(arguments, flattened_children, span),
        "mirror" => parse_mirror_call(arguments, flattened_children, span),
        "color" => parse_color_call(arguments, flattened_children, span),
        "multmatrix" => parse_multmatrix_call(arguments, flattened_children, span),
        "resize" => parse_resize_call(arguments, flattened_children, span),
        "offset" => parse_offset_call(arguments, flattened_children, span),

        // Booleans
        "union" => Ok(Some(Statement::Union {
            children: flattened_children,
            span,
        })),
        "difference" => Ok(Some(Statement::Difference {
            children: flattened_children,
            span,
        })),
        "intersection" => Ok(Some(Statement::Intersection {
            children: flattened_children,
            span,
        })),

        // Advanced
        "hull" => Ok(Some(Statement::Hull {
            children: flattened_children,
            span,
        })),
        "minkowski" => parse_minkowski_call(arguments, flattened_children, span),

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
fn flatten_boolean_children(name: &str, children: Vec<Statement>) -> Vec<Statement> {
    if children.len() == 1 {
        match (&children[0], name) {
            (Statement::Union { children: inner, .. }, "union") => inner.clone(),
            (Statement::Difference { children: inner, .. }, "difference") => inner.clone(),
            (Statement::Intersection { children: inner, .. }, "intersection") => inner.clone(),
            (Statement::Union { children: inner, .. }, _) => inner.clone(),
            _ => children,
        }
    } else {
        children
    }
}

/// Parses an assignment node.
fn parse_assignment(
    node: &SerializedNode,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let name = node
        .child_by_field("name")
        .map(|n| n.text.clone())
        .ok_or_else(|| CstParseError::InvalidValue("Assignment missing name".to_string()))?;

    let value_node = node
        .child_by_field("value")
        .ok_or_else(|| CstParseError::InvalidValue("Assignment missing value".to_string()))?;

    let value = parse_expression(value_node)?;
    Ok(Some(Statement::Assignment { name, value, span }))
}

/// Parses arguments from an arguments node.
/// Uses named_children to skip anonymous tokens like parentheses and commas.
/// Falls back to children if named_children is empty to handle edge cases.
fn parse_arguments(node: &SerializedNode) -> Result<Vec<Argument>, CstParseError> {
    let mut arguments = Vec::new();

    // Use named_children to automatically skip "(" ")" ","
    // This handles function calls, expressions, and other named constructs
    let children_to_parse = if !node.named_children.is_empty() {
        &node.named_children
    } else {
        // Fallback: filter children to skip punctuation
        // This handles cases where named_children might be empty
        return parse_arguments_from_children(&node.children);
    };

    for child in children_to_parse {
        match child.node_type.as_str() {
            "assignment" => {
                // Named argument: name = value
                if let Some(name_node) = child.child_by_field("name") {
                    let name = name_node.text.clone();
                    if let Some(value_node) = child.child_by_field("value") {
                        let value = parse_expression(value_node)?;
                        arguments.push(Argument::named(name, value));
                    }
                }
            }
            // Skip punctuation tokens
            "(" | ")" | "," => continue,
            _ => {
                // Parse any other named child as a positional argument expression
                // This includes: function_call, call_expression, number, identifier, etc.
                let value = parse_expression(child)?;
                arguments.push(Argument::positional(value));
            }
        }
    }

    Ok(arguments)
}

/// Fallback parser for arguments when named_children is empty.
/// Filters out punctuation and parses remaining children as expressions.
fn parse_arguments_from_children(children: &[SerializedNode]) -> Result<Vec<Argument>, CstParseError> {
    let mut arguments = Vec::new();

    for child in children {
        match child.node_type.as_str() {
            // Skip punctuation and anonymous tokens
            "(" | ")" | "," | ";" => continue,
            "assignment" => {
                // Named argument: name = value
                if let Some(name_node) = child.child_by_field("name") {
                    let name = name_node.text.clone();
                    if let Some(value_node) = child.child_by_field("value") {
                        let value = parse_expression(value_node)?;
                        arguments.push(Argument::named(name, value));
                    }
                }
            }
            _ => {
                // Only parse if it looks like an expression (is_named check)
                if child.is_named {
                    let value = parse_expression(child)?;
                    arguments.push(Argument::positional(value));
                }
            }
        }
    }

    Ok(arguments)
}

/// Helper to get named argument value.
fn get_named_arg<'a>(args: &'a [Argument], name: &str) -> Option<&'a Expression> {
    args.iter()
        .find(|a| a.name.as_deref() == Some(name))
        .map(|a| &a.value)
}

/// Helper to get first positional argument.
fn get_positional_arg(args: &[Argument]) -> Option<&Expression> {
    args.iter().find(|a| a.name.is_none()).map(|a| &a.value)
}

/// Parses an expression node.
///
/// Handles all expression types from the OpenSCAD grammar including:
/// - Literals: number, boolean, string, undef
/// - Variables: identifier, special_variable
/// - Compound: list, range
/// - Operations: unary_expression, binary_expression, ternary_expression
/// - Calls: function_call
/// - Wrappers: literal, expression (supertypes that delegate to children)
fn parse_expression(node: &SerializedNode) -> Result<Expression, CstParseError> {
    match node.node_type.as_str() {
        // Wrapper types (supertypes) - delegate to first named child
        // The grammar defines these as supertypes that wrap their actual content
        "literal" | "expression" => {
            if let Some(child) = node.named_children.first() {
                parse_expression(child)
            } else {
                Err(CstParseError::InvalidValue(format!(
                    "Empty {} node",
                    node.node_type
                )))
            }
        }
        // Undef literal - OpenSCAD's null/undefined value
        "undef" => Ok(Expression::Undef),
        // Numeric literals (integer, float, or generic number)
        "number" | "integer" | "decimal" | "float" => {
            let value = node
                .text
                .parse::<f64>()
                .map_err(|_| CstParseError::InvalidValue(format!("Invalid number: {}", node.text)))?;
            Ok(Expression::Number(value))
        }
        "boolean" => {
            let value = node.text == "true";
            Ok(Expression::Boolean(value))
        }
        "string" => {
            let text = node.text.trim_matches('"').to_string();
            Ok(Expression::String(text))
        }
        "identifier" | "special_variable" => Ok(Expression::Variable(node.text.clone())),
        "list" => {
            let mut items = Vec::new();
            for child in &node.named_children {
                items.push(parse_expression(child)?);
            }
            Ok(Expression::Vector(items))
        }
        "unary_expression" => {
            if node.text.starts_with('-') {
                if let Some(operand) = node.named_children.first() {
                    let expr = parse_expression(operand)?;
                    if let Expression::Number(n) = expr {
                        return Ok(Expression::Number(-n));
                    }
                }
            }
            Err(CstParseError::Unsupported(format!("Unary: {}", node.text)))
        }
        "binary_expression" => {
            if node.named_children.len() < 2 {
                return Err(CstParseError::InvalidValue("Binary needs 2 operands".into()));
            }
            let l = parse_expression(&node.named_children[0])?;
            let r = parse_expression(&node.named_children[1])?;
            let t = &node.text;
            let op = if t.contains("||") { BinaryOp::Or }
                else if t.contains("&&") { BinaryOp::And }
                else if t.contains("==") { BinaryOp::Equal }
                else if t.contains("!=") { BinaryOp::NotEqual }
                else if t.contains("<=") { BinaryOp::LessEqual }
                else if t.contains(">=") { BinaryOp::GreaterEqual }
                else if t.contains('^') { BinaryOp::Power }
                else if t.contains('*') { BinaryOp::Multiply }
                else if t.contains('/') { BinaryOp::Divide }
                else if t.contains('%') { BinaryOp::Modulo }
                else if t.contains('<') { BinaryOp::Less }
                else if t.contains('>') { BinaryOp::Greater }
                else if t.contains('+') { BinaryOp::Add }
                else if t.contains('-') { BinaryOp::Subtract }
                else { return Err(CstParseError::Unsupported(format!("Op: {}", t))); };
            Ok(Expression::Binary { left: Box::new(l), operator: op, right: Box::new(r) })
        }
        // Range: [start:end] or [start:step:end]
        "range" => {
            let children = &node.named_children;
            let start = children.first().map(parse_expression).transpose()?.unwrap_or(Expression::Number(0.0));
            let end = children.last().map(parse_expression).transpose()?.unwrap_or(Expression::Number(0.0));
            let step = if children.len() == 3 {
                Some(Box::new(children.get(1).map(parse_expression).transpose()?.unwrap_or(Expression::Number(1.0))))
            } else { None };
            Ok(Expression::Range { start: Box::new(start), step, end: Box::new(end) })
        }
        // Parenthesized expression: (expr)
        "parenthesized_expression" => {
            // Find the inner expression and parse it
            for child in &node.named_children {
                return parse_expression(child);
            }
            Ok(Expression::Number(0.0))
        }
        // Function call: name(args)
        "function_call" | "call_expression" => {
            let mut name = String::new();
            let mut arguments = Vec::new();
            
            // Get function name from "name" field (it's an expression, usually identifier)
            if let Some(name_node) = node.child_by_field("name") {
                name = name_node.text.clone();
            } else {
                // Fallback: first identifier child in named_children or children
                for child in &node.named_children {
                    if child.node_type == "identifier" {
                        name = child.text.clone();
                        break;
                    }
                }
                if name.is_empty() {
                    for child in &node.children {
                        if child.node_type == "identifier" {
                            name = child.text.clone();
                            break;
                        }
                    }
                }
            }
            
            // Get arguments - try multiple methods to find them
            if let Some(args_node) = node.child_by_field("arguments") {
                arguments = parse_arguments(&args_node)?;
            } else {
                // Fallback: look for arguments node in named_children first
                for child in &node.named_children {
                    if child.node_type == "arguments" {
                        arguments = parse_arguments(child)?;
                        break;
                    }
                }
                // Then try children if not found
                if arguments.is_empty() {
                    for child in &node.children {
                        if child.node_type == "arguments" {
                            arguments = parse_arguments(child)?;
                            break;
                        }
                    }
                }
            }
            
            Ok(Expression::FunctionCall { name, arguments })
        }
        // Ternary expression: condition ? then : else
        "ternary_expression" => {
            let condition = node.child_by_field("condition")
                .map(parse_expression)
                .transpose()?
                .unwrap_or(Expression::Boolean(false));
            let then_expr = node.child_by_field("consequence")
                .map(parse_expression)
                .transpose()?
                .unwrap_or(Expression::Undef);
            let else_expr = node.child_by_field("alternative")
                .map(parse_expression)
                .transpose()?
                .unwrap_or(Expression::Undef);
            Ok(Expression::Ternary {
                condition: Box::new(condition),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            })
        }
        // Index expression: array[index]
        "index_expression" => {
            let array = node.child_by_field("value")
                .map(parse_expression)
                .transpose()?
                .unwrap_or(Expression::Undef);
            // The index is typically the second named child (after the value)
            let index = node.named_children.get(1)
                .map(parse_expression)
                .transpose()?
                .unwrap_or(Expression::Number(0.0));
            Ok(Expression::Index {
                array: Box::new(array),
                index: Box::new(index),
            })
        }
        // Dot index expression: object.field - treat as index with string
        "dot_index_expression" => {
            let value = node.child_by_field("value")
                .map(parse_expression)
                .transpose()?
                .unwrap_or(Expression::Undef);
            let index_name = node.child_by_field("index")
                .map(|n| n.text.clone())
                .unwrap_or_default();
            Ok(Expression::Index {
                array: Box::new(value),
                index: Box::new(Expression::String(index_name)),
            })
        }
        // Let expression: let(assignments) expression
        "let_expression" => {
            // For now, parse the inner expression - full let support would need scope handling
            if let Some(body) = node.child_by_field("body") {
                parse_expression(&body)
            } else if let Some(child) = node.named_children.last() {
                parse_expression(child)
            } else {
                Ok(Expression::Undef)
            }
        }
        // Assert/Echo expressions - return the trailing expression if present
        "assert_expression" | "echo_expression" => {
            if let Some(expr) = node.child_by_field("expression") {
                parse_expression(&expr)
            } else {
                Ok(Expression::Undef)
            }
        }
        _ => Err(CstParseError::Unsupported(format!("Expression type: {}", node.node_type))),
    }
}

// ============================================================================
// Primitive Parsers
// ============================================================================

/// Parses a cube() call.
/// Stores the size as an Expression for deferred evaluation at runtime.
/// This allows cube(size) where size is a variable or function call.
fn parse_cube_call(arguments: &[Argument], span: Span) -> Result<Option<Statement>, CstParseError> {
    // Default: cube with size [1, 1, 1]
    let mut size = Expression::Vector(vec![
        Expression::Number(1.0),
        Expression::Number(1.0),
        Expression::Number(1.0),
    ]);
    let mut center = false;

    // Check positional argument first (store expression directly)
    if let Some(expr) = get_positional_arg(arguments) {
        size = expr.clone();
    }

    // Check named arguments (expression takes priority)
    if let Some(expr) = get_named_arg(arguments, "size") {
        size = expr.clone();
    }
    if let Some(expr) = get_named_arg(arguments, "center") {
        center = parse_bool_arg(expr)?;
    }

    Ok(Some(Statement::Cube { size, center, span }))
}

/// Parses a sphere() call.
/// Stores the radius as an Expression for deferred evaluation at runtime.
/// The $fn parameter is stored as an optional override for segment calculation.
fn parse_sphere_call(
    arguments: &[Argument],
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    // Default radius of 1.0
    let mut radius = Expression::Number(1.0);
    let mut fn_override: Option<f64> = None;

    // Check positional argument first (store expression directly)
    if let Some(expr) = get_positional_arg(arguments) {
        radius = expr.clone();
    }

    // Check named arguments
    if let Some(expr) = get_named_arg(arguments, "r") {
        radius = expr.clone();
    }
    if let Some(expr) = get_named_arg(arguments, "d") {
        // d = diameter, so radius = d / 2
        radius = Expression::Binary {
            left: Box::new(expr.clone()),
            operator: BinaryOp::Divide,
            right: Box::new(Expression::Number(2.0)),
        };
    }
    // Store $fn as override - segments computed at evaluation time
    if let Some(expr) = get_named_arg(arguments, "$fn") {
        fn_override = Some(parse_number_arg(expr)?);
    }

    Ok(Some(Statement::Sphere {
        radius,
        fn_override,
        span,
    }))
}

/// Parses a cylinder() call.
/// Stores height and radii as Expressions for deferred evaluation at runtime.
/// The $fn parameter is stored as an optional override for segment calculation.
fn parse_cylinder_call(
    arguments: &[Argument],
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    // Default values as expressions
    let mut height = Expression::Number(1.0);
    let mut radius_bottom = Expression::Number(1.0);
    let mut radius_top = Expression::Number(1.0);
    let mut center = false;
    let mut fn_override: Option<f64> = None;

    // Check positional argument first (height)
    if let Some(expr) = get_positional_arg(arguments) {
        height = expr.clone();
    }

    // Check named arguments (store expressions directly)
    if let Some(expr) = get_named_arg(arguments, "h") {
        height = expr.clone();
    }
    if let Some(expr) = get_named_arg(arguments, "r") {
        radius_bottom = expr.clone();
        radius_top = expr.clone();
    }
    if let Some(expr) = get_named_arg(arguments, "r1") {
        radius_bottom = expr.clone();
    }
    if let Some(expr) = get_named_arg(arguments, "r2") {
        radius_top = expr.clone();
    }
    if let Some(expr) = get_named_arg(arguments, "d") {
        // d = diameter, so radius = d / 2
        let r = Expression::Binary {
            left: Box::new(expr.clone()),
            operator: BinaryOp::Divide,
            right: Box::new(Expression::Number(2.0)),
        };
        radius_bottom = r.clone();
        radius_top = r;
    }
    if let Some(expr) = get_named_arg(arguments, "d1") {
        radius_bottom = Expression::Binary {
            left: Box::new(expr.clone()),
            operator: BinaryOp::Divide,
            right: Box::new(Expression::Number(2.0)),
        };
    }
    if let Some(expr) = get_named_arg(arguments, "d2") {
        radius_top = Expression::Binary {
            left: Box::new(expr.clone()),
            operator: BinaryOp::Divide,
            right: Box::new(Expression::Number(2.0)),
        };
    }
    if let Some(expr) = get_named_arg(arguments, "center") {
        center = parse_bool_arg(expr)?;
    }
    // Store $fn as override - segments computed at evaluation time
    if let Some(expr) = get_named_arg(arguments, "$fn") {
        fn_override = Some(parse_number_arg(expr)?);
    }

    Ok(Some(Statement::Cylinder {
        height,
        radius_bottom,
        radius_top,
        center,
        fn_override,
        span,
    }))
}

/// Parses a polyhedron() call.
fn parse_polyhedron_call(
    arguments: &[Argument],
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut points = Vec::new();
    let mut faces: Vec<Vec<u32>> = Vec::new();
    let mut convexity = DEFAULT_CONVEXITY;

    // Check named arguments
    if let Some(expr) = get_named_arg(arguments, "points") {
        points = parse_points_arg(expr)?;
    }
    if let Some(expr) = get_named_arg(arguments, "faces") {
        faces = parse_faces_arg(expr)?;
    }
    if let Some(expr) = get_named_arg(arguments, "convexity") {
        convexity = parse_number_arg(expr)? as u32;
    }

    Ok(Some(Statement::Polyhedron {
        points,
        faces,
        convexity,
        span,
    }))
}

// ============================================================================
// Transform Parsers
// ============================================================================

/// Parses a translate() call. Stores expression for deferred evaluation.
fn parse_translate_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    // Get vector expression (deferred evaluation)
    let vector = get_positional_arg(arguments)
        .or_else(|| get_named_arg(arguments, "v"))
        .cloned()
        .unwrap_or(Expression::Vector(vec![
            Expression::Number(0.0),
            Expression::Number(0.0),
            Expression::Number(0.0),
        ]));

    Ok(Some(Statement::Translate { vector, children, span }))
}

/// Parses a rotate() call. Stores expression for deferred evaluation.
fn parse_rotate_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    // Get angles expression (deferred evaluation)
    let angles = get_positional_arg(arguments)
        .or_else(|| get_named_arg(arguments, "a"))
        .cloned()
        .unwrap_or(Expression::Vector(vec![
            Expression::Number(0.0),
            Expression::Number(0.0),
            Expression::Number(0.0),
        ]));

    // Get optional axis expression
    let axis = get_named_arg(arguments, "v").cloned();

    Ok(Some(Statement::Rotate { angles, axis, children, span }))
}

/// Parses a scale() call. Stores expression for deferred evaluation.
fn parse_scale_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    // Get factors expression (deferred evaluation)
    let factors = get_positional_arg(arguments)
        .or_else(|| get_named_arg(arguments, "v"))
        .cloned()
        .unwrap_or(Expression::Vector(vec![
            Expression::Number(1.0),
            Expression::Number(1.0),
            Expression::Number(1.0),
        ]));

    Ok(Some(Statement::Scale { factors, children, span }))
}

/// Parses a mirror() call. Stores expression for deferred evaluation.
fn parse_mirror_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    // Get normal expression (deferred evaluation)
    let normal = get_positional_arg(arguments)
        .or_else(|| get_named_arg(arguments, "v"))
        .cloned()
        .unwrap_or(Expression::Vector(vec![
            Expression::Number(1.0),
            Expression::Number(0.0),
            Expression::Number(0.0),
        ]));

    Ok(Some(Statement::Mirror { normal, children, span }))
}

/// Parses a color() call.
fn parse_color_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    if let Some(expr) = get_positional_arg(arguments) {
        if let Expression::Vector(items) = expr {
            for (i, item) in items.iter().enumerate().take(4) {
                if let Expression::Number(n) = item {
                    color[i] = *n as f32;
                }
            }
        }
    }
    if let Some(expr) = get_named_arg(arguments, "c") {
        if let Expression::Vector(items) = expr {
            for (i, item) in items.iter().enumerate().take(4) {
                if let Expression::Number(n) = item {
                    color[i] = *n as f32;
                }
            }
        }
    }

    Ok(Some(Statement::Color {
        color,
        children,
        span,
    }))
}

// ============================================================================
// Extrusion Parsers
// ============================================================================

/// Parses a linear_extrude() call.
fn parse_linear_extrude_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut height = 1.0;
    let mut center = false;
    let mut twist = 0.0;
    let mut slices = 1;
    let mut scale: [f64; 2] = [1.0, 1.0];

    // Check positional argument first (height)
    if let Some(expr) = get_positional_arg(arguments) {
        height = parse_number_arg(expr)?;
    }

    // Check named arguments
    if let Some(expr) = get_named_arg(arguments, "height") {
        height = parse_number_arg(expr)?;
    }
    if let Some(expr) = get_named_arg(arguments, "center") {
        center = parse_bool_arg(expr)?;
    }
    if let Some(expr) = get_named_arg(arguments, "twist") {
        twist = parse_number_arg(expr)?;
    }
    if let Some(expr) = get_named_arg(arguments, "slices") {
        slices = parse_number_arg(expr)? as u32;
    }
    if let Some(expr) = get_named_arg(arguments, "scale") {
        match expr {
            Expression::Number(n) => scale = [*n, *n],
            Expression::Vector(items) => {
                if items.len() >= 2 {
                    if let (Expression::Number(x), Expression::Number(y)) = (&items[0], &items[1]) {
                        scale = [*x, *y];
                    }
                }
            }
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

/// Parses a rotate_extrude() call.
fn parse_rotate_extrude_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut angle = 360.0;
    let mut convexity = DEFAULT_CONVEXITY;

    // Check named arguments
    if let Some(expr) = get_named_arg(arguments, "angle") {
        angle = parse_number_arg(expr)?;
    }
    if let Some(expr) = get_named_arg(arguments, "convexity") {
        convexity = parse_number_arg(expr)? as u32;
    }

    Ok(Some(Statement::RotateExtrude {
        angle,
        convexity,
        children,
        span,
    }))
}

// ============================================================================
// Argument Helpers
// ============================================================================

/// Parses a number argument.
fn parse_number_arg(expr: &Expression) -> Result<f64, CstParseError> {
    match expr {
        Expression::Number(n) => Ok(*n),
        _ => Err(CstParseError::InvalidValue("Expected number".to_string())),
    }
}

/// Parses a boolean argument.
fn parse_bool_arg(expr: &Expression) -> Result<bool, CstParseError> {
    match expr {
        Expression::Boolean(b) => Ok(*b),
        _ => Err(CstParseError::InvalidValue("Expected boolean".to_string())),
    }
}

/// Parses a vec3 argument.
fn parse_vec3_arg(expr: &Expression) -> Result<DVec3, CstParseError> {
    match expr {
        Expression::Vector(items) => parse_vec3_from_list(items),
        _ => Err(CstParseError::InvalidValue("Expected vector".to_string())),
    }
}

/// Parses a vec3 from a list of expressions.
fn parse_vec3_from_list(items: &[Expression]) -> Result<DVec3, CstParseError> {
    let x = items.first().map(parse_number_arg).transpose()?.unwrap_or(0.0);
    let y = items.get(1).map(parse_number_arg).transpose()?.unwrap_or(0.0);
    let z = items.get(2).map(parse_number_arg).transpose()?.unwrap_or(0.0);
    Ok(DVec3::new(x, y, z))
}

/// Parses points for polyhedron.
fn parse_points_arg(expr: &Expression) -> Result<Vec<DVec3>, CstParseError> {
    match expr {
        Expression::Vector(items) => {
            let mut points = Vec::new();
            for item in items {
                points.push(parse_vec3_arg(item)?);
            }
            Ok(points)
        }
        _ => Err(CstParseError::InvalidValue("Expected points list".to_string())),
    }
}

/// Parses faces for polyhedron.
fn parse_faces_arg(expr: &Expression) -> Result<Vec<Vec<u32>>, CstParseError> {
    match expr {
        Expression::Vector(items) => {
            let mut faces = Vec::new();
            for item in items {
                if let Expression::Vector(indices) = item {
                    let mut face = Vec::new();
                    for idx in indices {
                        if let Expression::Number(n) = idx {
                            face.push(*n as u32);
                        }
                    }
                    faces.push(face);
                }
            }
            Ok(faces)
        }
        _ => Err(CstParseError::InvalidValue("Expected faces list".to_string())),
    }
}


// ============================================================================
// 2D Primitive Parsers
// ============================================================================

/// Parses a circle() call.
/// 
/// # Arguments
/// * `arguments` - The parsed arguments from the module call
/// * `span` - Source location span for error reporting
/// 
/// # Returns
/// A Circle statement with radius and segment count
/// 
/// # Example
/// ```openscad
/// circle(r=10);      // radius = 10
/// circle(d=20);      // diameter = 20 (radius = 10)
/// circle(5, $fn=32); // radius = 5, 32 segments
/// ```
fn parse_circle_call(
    arguments: &[Argument],
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut radius = 1.0;
    let segments = 0u32;

    if let Some(expr) = get_positional_arg(arguments) {
        radius = parse_number_arg(expr)?;
    }
    if let Some(expr) = get_named_arg(arguments, "r") {
        radius = parse_number_arg(expr)?;
    }
    if let Some(expr) = get_named_arg(arguments, "d") {
        radius = parse_number_arg(expr)? / 2.0;
    }

    Ok(Some(Statement::Circle { radius, segments, span }))
}

/// Parses a square() call.
/// 
/// # Arguments
/// * `arguments` - The parsed arguments from the module call
/// * `span` - Source location span for error reporting
/// 
/// # Returns
/// A Square statement with size and center flag
/// 
/// # Example
/// ```openscad
/// square(10);                // 10x10, corner at origin
/// square([20, 30]);          // 20x30
/// square(15, center=true);   // 15x15, centered at origin
/// ```
fn parse_square_call(
    arguments: &[Argument],
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut size = [1.0, 1.0];
    let mut center = false;

    if let Some(expr) = get_positional_arg(arguments) {
        match expr {
            Expression::Number(n) => size = [*n, *n],
            Expression::Vector(items) => {
                size[0] = items.first().map(parse_number_arg).transpose()?.unwrap_or(1.0);
                size[1] = items.get(1).map(parse_number_arg).transpose()?.unwrap_or(1.0);
            }
            _ => {}
        }
    }
    if let Some(expr) = get_named_arg(arguments, "size") {
        match expr {
            Expression::Number(n) => size = [*n, *n],
            Expression::Vector(items) => {
                size[0] = items.first().map(parse_number_arg).transpose()?.unwrap_or(1.0);
                size[1] = items.get(1).map(parse_number_arg).transpose()?.unwrap_or(1.0);
            }
            _ => {}
        }
    }
    if let Some(expr) = get_named_arg(arguments, "center") {
        center = parse_bool_arg(expr).unwrap_or(false);
    }

    Ok(Some(Statement::Square { size, center, span }))
}

/// Parses a polygon() call.
/// 
/// # Arguments
/// * `arguments` - The parsed arguments from the module call
/// * `span` - Source location span for error reporting
/// 
/// # Returns
/// A Polygon statement with points and optional paths
/// 
/// # Example
/// ```openscad
/// polygon(points=[[0,0], [10,0], [5,10]]);
/// polygon([[0,0], [10,0], [10,10], [0,10]], paths=[[0,1,2,3]]);
/// ```
fn parse_polygon_call(
    arguments: &[Argument],
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut points = Vec::new();
    let paths: Option<Vec<Vec<u32>>> = None;

    if let Some(expr) = get_named_arg(arguments, "points") {
        if let Expression::Vector(items) = expr {
            for item in items {
                if let Expression::Vector(coords) = item {
                    let x = coords.first().map(parse_number_arg).transpose()?.unwrap_or(0.0);
                    let y = coords.get(1).map(parse_number_arg).transpose()?.unwrap_or(0.0);
                    points.push([x, y]);
                }
            }
        }
    } else if let Some(expr) = get_positional_arg(arguments) {
        if let Expression::Vector(items) = expr {
            for item in items {
                if let Expression::Vector(coords) = item {
                    let x = coords.first().map(parse_number_arg).transpose()?.unwrap_or(0.0);
                    let y = coords.get(1).map(parse_number_arg).transpose()?.unwrap_or(0.0);
                    points.push([x, y]);
                }
            }
        }
    }

    Ok(Some(Statement::Polygon { points, paths, span }))
}

// ============================================================================
// Additional Transform Parsers
// ============================================================================

/// Parses a multmatrix() call for arbitrary 4x4 matrix transformation.
/// 
/// # Arguments
/// * `arguments` - The 4x4 transformation matrix
/// * `children` - Child statements to transform
/// * `span` - Source location span
/// 
/// # Example
/// ```openscad
/// multmatrix([[1,0,0,10],[0,1,0,0],[0,0,1,0],[0,0,0,1]]) cube(5);
/// ```
fn parse_multmatrix_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut matrix = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    if let Some(expr) = get_positional_arg(arguments) {
        if let Expression::Vector(rows) = expr {
            for (i, row) in rows.iter().enumerate().take(4) {
                if let Expression::Vector(cols) = row {
                    for (j, col) in cols.iter().enumerate().take(4) {
                        if let Expression::Number(n) = col {
                            matrix[i][j] = *n;
                        }
                    }
                }
            }
        }
    }

    Ok(Some(Statement::Multmatrix { matrix, children, span }))
}

/// Parses a resize() call for resizing geometry to specific dimensions.
/// 
/// # Arguments
/// * `arguments` - New size and auto-scale options
/// * `children` - Child statements to resize
/// * `span` - Source location span
/// 
/// # Example
/// ```openscad
/// resize([10, 20, 30]) sphere(5);
/// resize([10, 0, 0], auto=true) cube(5);
/// ```
fn parse_resize_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut new_size = DVec3::ZERO;
    let mut auto_scale = [false, false, false];

    if let Some(expr) = get_positional_arg(arguments) {
        new_size = parse_vec3_arg(expr)?;
    }
    if let Some(expr) = get_named_arg(arguments, "newsize") {
        new_size = parse_vec3_arg(expr)?;
    }
    if let Some(expr) = get_named_arg(arguments, "auto") {
        match expr {
            Expression::Boolean(b) => auto_scale = [*b, *b, *b],
            Expression::Vector(items) => {
                auto_scale[0] = items.first().map(parse_bool_arg).transpose()?.unwrap_or(false);
                auto_scale[1] = items.get(1).map(parse_bool_arg).transpose()?.unwrap_or(false);
                auto_scale[2] = items.get(2).map(parse_bool_arg).transpose()?.unwrap_or(false);
            }
            _ => {}
        }
    }

    Ok(Some(Statement::Resize { new_size, auto_scale, children, span }))
}

/// Parses an offset() call for 2D polygon offsetting.
/// 
/// # Arguments
/// * `arguments` - Offset amount (radius or delta) and chamfer option
/// * `children` - Child 2D statements to offset
/// * `span` - Source location span
/// 
/// # Example
/// ```openscad
/// offset(r=5) square(10);    // Round corners, radius 5
/// offset(delta=2) circle(5); // Expand by 2
/// offset(delta=-1, chamfer=true) square(10); // Chamfered shrink
/// ```
fn parse_offset_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut amount = crate::ast::OffsetAmount::Radius(1.0);
    let mut chamfer = false;

    if let Some(expr) = get_positional_arg(arguments) {
        amount = crate::ast::OffsetAmount::Radius(parse_number_arg(expr)?);
    }
    if let Some(expr) = get_named_arg(arguments, "r") {
        amount = crate::ast::OffsetAmount::Radius(parse_number_arg(expr)?);
    }
    if let Some(expr) = get_named_arg(arguments, "delta") {
        amount = crate::ast::OffsetAmount::Delta(parse_number_arg(expr)?);
    }
    if let Some(expr) = get_named_arg(arguments, "chamfer") {
        chamfer = parse_bool_arg(expr).unwrap_or(false);
    }

    Ok(Some(Statement::Offset { amount, chamfer, children, span }))
}

/// Parses a minkowski() call with optional convexity.
/// 
/// # Arguments
/// * `arguments` - Optional convexity parameter
/// * `children` - Child statements for Minkowski sum
/// * `span` - Source location span
/// 
/// # Example
/// ```openscad
/// minkowski() { cube(10); sphere(2); }
/// minkowski(convexity=4) { cylinder(10, 5); sphere(1); }
/// ```
fn parse_minkowski_call(
    arguments: &[Argument],
    children: Vec<Statement>,
    span: Span,
) -> Result<Option<Statement>, CstParseError> {
    let mut convexity = 1u32;

    if let Some(expr) = get_named_arg(arguments, "convexity") {
        if let Ok(n) = parse_number_arg(expr) {
            convexity = n as u32;
        }
    }

    Ok(Some(Statement::Minkowski { convexity, children, span }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cst::Position;

    fn test_node(node_type: &str, text: &str) -> SerializedNode {
        SerializedNode {
            node_type: node_type.to_string(),
            text: text.to_string(),
            start_index: 0,
            end_index: text.len(),
            start_position: Position { row: 0, column: 0 },
            end_position: Position { row: 0, column: text.len() },
            children: Vec::new(),
            named_children: Vec::new(),
            is_named: true,
            field_name: None,
        }
    }

    #[test]
    fn test_parse_number_expression() {
        let node = test_node("number", "42.5");
        let expr = parse_expression(&node).unwrap();
        assert!(matches!(expr, Expression::Number(n) if (n - 42.5).abs() < 0.001));
    }

    #[test]
    fn test_parse_float_expression() {
        let node = test_node("float", "3.14159");
        let expr = parse_expression(&node).unwrap();
        assert!(matches!(expr, Expression::Number(n) if (n - 3.14159).abs() < 0.00001));
    }

    #[test]
    fn test_parse_boolean_expression() {
        let node = test_node("boolean", "true");
        let expr = parse_expression(&node).unwrap();
        assert!(matches!(expr, Expression::Boolean(true)));
    }

    #[test]
    fn test_parse_string_expression() {
        let node = test_node("string", "\"hello\"");
        let expr = parse_expression(&node).unwrap();
        assert!(matches!(expr, Expression::String(s) if s == "hello"));
    }

    /// Test that `literal` wrapper type correctly delegates to child nodes.
    /// The grammar defines `literal` as a supertype containing number, boolean, string, etc.
    #[test]
    fn test_parse_literal_wrapper() {
        let mut literal_node = test_node("literal", "0.1");
        let float_child = test_node("float", "0.1");
        literal_node.named_children.push(float_child);
        
        let expr = parse_expression(&literal_node).unwrap();
        assert!(matches!(expr, Expression::Number(n) if (n - 0.1).abs() < 0.001));
    }

    /// Test that `expression` wrapper type correctly delegates to child nodes.
    /// The grammar defines `expression` as a supertype for all expression variants.
    #[test]
    fn test_parse_expression_wrapper() {
        let mut expr_node = test_node("expression", "42");
        let num_child = test_node("integer", "42");
        expr_node.named_children.push(num_child);
        
        let expr = parse_expression(&expr_node).unwrap();
        assert!(matches!(expr, Expression::Number(n) if (n - 42.0).abs() < 0.001));
    }

    /// Test undef literal parsing.
    #[test]
    fn test_parse_undef_expression() {
        let node = test_node("undef", "undef");
        let expr = parse_expression(&node).unwrap();
        assert!(matches!(expr, Expression::Undef));
    }

    /// Test nested literal > number > float structure (as produced by tree-sitter).
    #[test]
    fn test_parse_nested_literal_float() {
        let mut literal_node = test_node("literal", "0.1");
        let mut number_node = test_node("number", "0.1");
        let float_node = test_node("float", "0.1");
        number_node.named_children.push(float_node);
        literal_node.named_children.push(number_node);
        
        let expr = parse_expression(&literal_node).unwrap();
        assert!(matches!(expr, Expression::Number(n) if (n - 0.1).abs() < 0.001));
    }
}


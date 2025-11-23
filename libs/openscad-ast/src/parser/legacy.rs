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
        if let Some(stmt) = parse_statement(&child, source)? {
            statements.push(stmt);
        }
    }

    Ok(statements)
}

fn parse_statement(
    node: &tree_sitter::Node,
    source: &str,
) -> Result<Option<Statement>, Vec<Diagnostic>> {
    let kind = node.kind();

    if kind == "module_call" {
        parse_module_call(node, source)
    } else if kind == "transform_chain" {
        parse_transform_chain(node, source)
    } else if kind == "var_declaration" {
        parse_var_declaration(node, source)
    } else {
        // Ignore other nodes (comments, etc)
        Ok(None)
    }
}

fn parse_transform_chain(
    node: &tree_sitter::Node,
    source: &str,
) -> Result<Option<Statement>, Vec<Diagnostic>> {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    // transform_chain = modifier* module_call statement
    // We need to find the module_call (the transform) and the statement (the body).

    let transform_node = children.iter().find(|n| n.kind() == "module_call");
    // In grammar, statement is the last element.
    // Note: Tree-sitter children are ordered. The statement should be after the module_call.
    // Let's rely on order if possible, or find the last child that looks like a statement.

    // The structure is [modifiers..., module_call, statement]
    // So the body node is the last child.
    let body_node = children.last();
    // Note: The grammar says the last element is "statement". But "statement" rule in grammar includes transform_chain.
    // However, tree-sitter might wrap it in a node called "statement" or directly expose "transform_chain".
    // Checking grammar.json: "statement" rule HAS "transform_chain".
    // So body_node is likely kind "statement".
    // But let's be robust and look for the last significant child.
    // Actually, strictly following grammar: module_call is the transform, and the next sibling is the statement.

    if let (Some(transform), Some(body)) = (transform_node, body_node) {
        // Parse the child statement
        // The body node might be a "statement" wrapper node, we need to unwrap or handle it.
        // If body.kind() == "statement", we need to look inside it.
        // "statement" rule choices: for_block, if_block, ... transform_chain, module_call (indirectly via _item?? No, statement includes transform_chain directly).
        // Wait, module_call is NOT in statement directly in grammar?
        // Grammar: statement -> for_block | ... | transform_chain | ... | ";"
        // Grammar: _item -> var_declaration | statement | module_item | function_item
        // Grammar: module_call is part of transform_chain.
        // But `cube(10);` is valid. How is `cube(10);` parsed?
        // `source_file` -> `_item`*
        // `_item` -> `statement`.
        // `statement` has `transform_chain`.
        // `transform_chain` has `module_call` then `statement`.
        // Wait, if `cube(10);` is just `module_call` followed by nothing?
        // Ah, `transform_chain` definition: `modifier* module_call statement`.
        // And `module_call` is NOT in `statement` directly.
        // So a bare `cube(10);` must be... where?
        // Looking at grammar again:
        // `statement` -> `transform_chain`
        // `transform_chain` -> `module_call` `statement` ??
        // No, that would be infinite recursion if `statement` requires `transform_chain` which requires `statement`.
        // A `cube(10);` MUST be a `module_call` but `module_call` is NOT a `statement`?
        // Wait, looking at grammar.json provided earlier:
        // "statement": { "type": "CHOICE", "members": [ ... "transform_chain", ... ";" ] }
        // "transform_chain": { "type": "SEQ", "members": [ ... "module_call", "statement" ] }
        // This implies `translate(...) cube(...);`
        // But where is `cube(...);`?
        // If `cube(...)` is a `module_call`, and `module_call` is ONLY in `transform_chain`...
        // Then `cube(10);` is NOT a valid statement unless `transform_chain` allows empty transform?
        // No, `transform_chain` requires `module_call`.
        // Maybe `cube(10);` is parsed as `transform_chain` where `module_call` is `cube` and `statement` is `;`?
        // "statement" -> ";" is valid.
        // So `cube(10);` -> `transform_chain` -> `module_call`="cube(10)" + `statement`=";"
        
        // If this is true, then `translate(...) cube(...);` is:
        // `transform_chain` (translate)
        //   `module_call` (translate)
        //   `statement` -> `transform_chain` (cube)
        //     `module_call` (cube)
        //     `statement` (;)

        // So, when parsing `transform_chain`:
        // 1. Identify `module_call`. Parse it.
        //    - If it is `cube` or `sphere`, it returns a Geometry Primitive statement.
        //      BUT it requires `statement` sibling which is `;`.
        //      So `cube` is a leaf.
        //    - If it is `translate`, `rotate`, `scale`, it is a Transform.
        // 2. Identify `statement` sibling. Parse it recursively.

        // Let's verify `parse_module_call`. It handles `cube`.
        // If `parse_module_call` returns `Some(Cube)`, and then we see `statement` is `;`, we return `Cube`.
        // If `parse_module_call` returns `None` (because it is `translate`), we assume it is a transform and look at `statement`.

        let inner_stmt = if body.kind() == "statement" {
            // We need to look inside "statement" node because it's a wrapper in CST
            // Or maybe parse_statement handles it.
            // Let's look at the child of "statement".
            // "statement" is a choice, so it has 1 child.
            if let Some(child) = body.named_child(0) {
                parse_statement(&child, source)?
            } else {
                // Semicolon case? "statement" -> ";" is anonymous string, so no named child?
                // If child count is 0 or named child count is 0.
                if body.child_count() > 0 && body.child(0).unwrap().kind() == ";" {
                    None
                } else {
                    None
                }
            }
        } else {
            // Direct child (e.g. ; )
            if body.kind() == ";" { None } else { parse_statement(body, source)? }
        };

        // If module_call was a primitive (cube), return it. (Ignoring inner statement if it is just semicolon)
        // But wait, `parse_module_call` returns `Statement::Cube`.
        if let Some(primitive) = parse_module_call(transform, source)? {
            // It was a primitive like cube.
            // We expect the following statement to be empty (semicolon).
            // If inner_stmt is Some, that means `cube(...) { ... }` or similar?
            // OpenSCAD `cube` cannot have children.
            // So if we got a primitive, return it.
            return Ok(Some(primitive));
        }

        // It was not a primitive, so it must be a transform (translate/rotate/scale)
        // Parse transform args
        let transform_name_node = transform.child_by_field_name("name");
        let args_node = transform.child_by_field_name("arguments");

        if let (Some(name_node), Some(args), Some(child_stmt)) = (transform_name_node, args_node, inner_stmt) {
            let name = &source[name_node.byte_range()];
            let vector = parse_vector_arg(&args, source)?;
            let span = Span::new(node.start_byte(), node.end_byte()).unwrap();

            match name {
                "translate" => return Ok(Some(Statement::Translate { vector, child: Box::new(child_stmt), span })),
                "rotate" => return Ok(Some(Statement::Rotate { vector, child: Box::new(child_stmt), span })),
                "scale" => return Ok(Some(Statement::Scale { vector, child: Box::new(child_stmt), span })),
                _ => return Ok(None), // Unknown transform or module
            }
        }
    }

    Ok(None)
}

fn parse_vector_arg(args_node: &tree_sitter::Node, source: &str) -> Result<[f64; 3], Vec<Diagnostic>> {
    // Extract the first argument, ensure it is a vector
    let mut cursor = args_node.walk();
    for child in args_node.children(&mut cursor) {
        if child.kind() == "list" {
             match parse_vector(&child, source) {
                 Ok(CubeSize::Vector(v)) => return Ok(v),
                 Ok(_) => return Err(vec![Diagnostic::error("Expected vector", Span::new(child.start_byte(), child.end_byte()).unwrap())]),
                 Err(e) => return Err(e),
             }
        }
    }
    Err(vec![Diagnostic::error("Transform requires a vector argument", Span::new(args_node.start_byte(), args_node.end_byte()).unwrap())])
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

    match name {
        "cube" => {
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
        "sphere" => {
            let args_node = children.iter().find(|n| n.kind() == "arguments");
            if let Some(args) = args_node {
                let (radius, fa, fs, fn_) = parse_sphere_arguments(args, source)?;
                let span = Span::new(node.start_byte(), node.end_byte())
                    .map_err(|e| vec![Diagnostic::error(format!("Invalid span: {}", e), Span::new(0, 1).unwrap())])?;

                Ok(Some(Statement::Sphere { radius, fa, fs, fn_, span }))
            } else {
                Err(vec![Diagnostic::error(
                    "sphere() requires arguments",
                    Span::new(node.start_byte(), node.end_byte()).unwrap(),
                )
                .with_hint("Try: sphere(10); or sphere(r=10);")])
            }
        }
        "cylinder" => {
            let args_node = children.iter().find(|n| n.kind() == "arguments");
            if let Some(args) = args_node {
                let (height, r1, r2, center, fa, fs, fn_) = parse_cylinder_arguments(args, source)?;
                let span = Span::new(node.start_byte(), node.end_byte())
                    .map_err(|e| vec![Diagnostic::error(format!("Invalid span: {}", e), Span::new(0, 1).unwrap())])?;

                Ok(Some(Statement::Cylinder {
                    height,
                    r1,
                    r2,
                    center,
                    fa,
                    fs,
                    fn_,
                    span,
                }))
            } else {
                Err(vec![Diagnostic::error(
                    "cylinder() requires arguments",
                    Span::new(node.start_byte(), node.end_byte()).unwrap(),
                )
                .with_hint("Try: cylinder(h=20, r=5);")])
            }
        }
        _ => Ok(None),
    }
}

/// Parses sphere arguments from the CST.
fn parse_sphere_arguments(
    args_node: &tree_sitter::Node,
    source: &str,
) -> Result<(f64, Option<f64>, Option<f64>, Option<u32>), Vec<Diagnostic>> {
    let mut cursor = args_node.walk();
    let mut radius: Option<f64> = None;
    let mut fa: Option<f64> = None;
    let mut fs: Option<f64> = None;
    let mut fn_: Option<u32> = None;
    let mut positional_index = 0;
    
    for child in args_node.children(&mut cursor) {
        let kind = child.kind();
        
        if kind == "assignment" {
            let name_node = child.child_by_field_name("name");
            let value_node = child.child_by_field_name("value");

            if let (Some(name_n), Some(value_n)) = (name_node, value_node) {
                let param_name = &source[name_n.byte_range()];

                match param_name {
                    "r" | "d" => { // 'd' is diameter, r = d/2. For now treat as radius or handle d.
                        // OpenSCAD supports r=radius or d=diameter.
                        // If d is used, radius = d/2.
                        let val = parse_f64(&value_n, source)?;
                        if param_name == "d" {
                            radius = Some(val / 2.0);
                        } else {
                            radius = Some(val);
                        }
                    }
                    "$fn" => {
                        fn_ = Some(parse_u32(&value_n, source)?);
                    }
                    "$fa" => {
                        fa = Some(parse_f64(&value_n, source)?);
                    }
                    "$fs" => {
                        fs = Some(parse_f64(&value_n, source)?);
                    }
                    _ => {
                        // Ignore unknown parameters or error?
                        // For sphere, only these matter for resolution.
                    }
                }
            }
        } else if kind == "number" || kind == "integer" || kind == "float" {
            if positional_index == 0 {
                radius = Some(parse_f64(&child, source)?);
            }
            positional_index += 1;
        }
    }

    if radius.is_none() {
        return Err(vec![Diagnostic::error(
            "sphere() requires a radius argument (r or d)",
            Span::new(args_node.start_byte(), args_node.end_byte()).unwrap(),
        )]);
    }

    Ok((radius.unwrap(), fa, fs, fn_))
}

fn parse_f64(node: &tree_sitter::Node, source: &str) -> Result<f64, Vec<Diagnostic>> {
    let text = &source[node.byte_range()];
    text.parse::<f64>().map_err(|_| {
        vec![Diagnostic::error(
            format!("Invalid number: {}", text),
            Span::new(node.start_byte(), node.end_byte()).unwrap(),
        )]
    })
}

fn parse_u32(node: &tree_sitter::Node, source: &str) -> Result<u32, Vec<Diagnostic>> {
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

/// Parses cylinder arguments from the CST into normalized values.
///
/// # Examples
/// ```
/// use openscad_ast::parse_to_ast;
/// let ast = parse_to_ast("cylinder(h=10, r=5);").unwrap();
/// assert!(matches!(ast[0], openscad_ast::Statement::Cylinder { .. }));
/// ```
fn parse_cylinder_arguments(
    args_node: &tree_sitter::Node,
    source: &str,
) -> Result<(f64, f64, f64, bool, Option<f64>, Option<f64>, Option<u32>), Vec<Diagnostic>> {
    let mut cursor = args_node.walk();
    let mut height: Option<f64> = None;
    let mut radius: Option<f64> = None;
    let mut diameter: Option<f64> = None;
    let mut r1: Option<f64> = None;
    let mut r2: Option<f64> = None;
    let mut d1: Option<f64> = None;
    let mut d2: Option<f64> = None;
    let mut center: Option<bool> = None;
    let mut fa: Option<f64> = None;
    let mut fs: Option<f64> = None;
    let mut fn_: Option<u32> = None;
    let mut positional_index = 0;

    for child in args_node.children(&mut cursor) {
        let kind = child.kind();

        if kind == "assignment" {
            let name_node = child.child_by_field_name("name");
            let value_node = child.child_by_field_name("value");

            if let (Some(name_n), Some(value_n)) = (name_node, value_node) {
                let param_name = &source[name_n.byte_range()];
                match param_name {
                    "h" => height = Some(parse_f64(&value_n, source)?),
                    "r" => radius = Some(parse_f64(&value_n, source)?),
                    "d" => diameter = Some(parse_f64(&value_n, source)?),
                    "r1" => r1 = Some(parse_f64(&value_n, source)?),
                    "r2" => r2 = Some(parse_f64(&value_n, source)?),
                    "d1" => d1 = Some(parse_f64(&value_n, source)?),
                    "d2" => d2 = Some(parse_f64(&value_n, source)?),
                    "center" => center = Some(parse_bool(&value_n, source)?),
                    "$fn" => fn_ = Some(parse_u32(&value_n, source)?),
                    "$fa" => fa = Some(parse_f64(&value_n, source)?),
                    "$fs" => fs = Some(parse_f64(&value_n, source)?),
                    _ => {
                        return Err(vec![Diagnostic::error(
                            format!("Unknown parameter: {}", param_name),
                            Span::new(child.start_byte(), child.end_byte()).unwrap(),
                        )])
                    }
                }
            }
        } else if kind == "number" || kind == "integer" || kind == "float" {
            let value = parse_f64(&child, source)?;
            match positional_index {
                0 => height = Some(value),
                1 => radius = Some(value),
                _ => {
                    return Err(vec![Diagnostic::error(
                        "Unexpected positional argument",
                        Span::new(child.start_byte(), child.end_byte()).unwrap(),
                    )])
                }
            }
            positional_index += 1;
        } else if kind == "boolean" {
            if positional_index == 2 {
                center = Some(parse_bool(&child, source)?);
            } else {
                return Err(vec![Diagnostic::error(
                    "Unexpected boolean argument",
                    Span::new(child.start_byte(), child.end_byte()).unwrap(),
                )]);
            }
            positional_index += 1;
        }
    }

    let height_val = height.unwrap_or(1.0);
    let base_radius = radius.or_else(|| diameter.map(|d| d / 2.0));
    let bottom_radius = r1
        .or_else(|| d1.map(|d| d / 2.0))
        .or(base_radius)
        .unwrap_or(1.0);
    let top_radius = r2
        .or_else(|| d2.map(|d| d / 2.0))
        .or(base_radius)
        .unwrap_or(1.0);
    let center_val = center.unwrap_or(false);

    Ok((height_val, bottom_radius, top_radius, center_val, fa, fs, fn_))
}

fn parse_bool(node: &tree_sitter::Node, source: &str) -> Result<bool, Vec<Diagnostic>> {
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

    #[test]
    fn test_parse_translate() {
        let ast = parse_to_ast("translate([10, 20, 30]) cube(10);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);

        match &ast[0] {
            Statement::Translate { vector, child, .. } => {
                assert_eq!(*vector, [10.0, 20.0, 30.0]);
                match &**child {
                    Statement::Cube { size, .. } => {
                        assert_eq!(*size, CubeSize::Scalar(10.0));
                    }
                    _ => panic!("Expected Cube child"),
                }
            }
            _ => panic!("Expected Translate"),
        }
    }

    #[test]
    fn test_parse_nested_transforms() {
        let ast = parse_to_ast("translate([1,0,0]) rotate([0,90,0]) cube(10);").expect("parse succeeds");
        assert_eq!(ast.len(), 1);

        match &ast[0] {
            Statement::Translate { vector: v1, child: c1, .. } => {
                assert_eq!(*v1, [1.0, 0.0, 0.0]);
                match &**c1 {
                    Statement::Rotate { vector: v2, child: c2, .. } => {
                        assert_eq!(*v2, [0.0, 90.0, 0.0]);
                        match &**c2 {
                            Statement::Cube { .. } => {},
                            _ => panic!("Expected Cube"),
                        }
                    }
                    _ => panic!("Expected Rotate"),
                }
            }
            _ => panic!("Expected Translate"),
        }
    }
}

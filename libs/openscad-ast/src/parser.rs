use crate::ast::*;
use crate::error::*;
use crate::span::Span;
use tree_sitter::Parser as TsParser;

pub fn parse(source: &str) -> Result<Ast, ParseError> {
    let mut parser = TsParser::new();
    parser
        .set_language(&tree_sitter_openscad::language())
        .map_err(|e| ParseError::Syntax { message: format!("{}", e), span: Span { start_byte: 0, end_byte: 0, start: crate::span::Position { row: 0, col: 0 }, end: crate::span::Position { row: 0, col: 0 }, text: None } })?;
    let tree = parser.parse(source, None).ok_or(ParseError::Syntax {
        message: "parse failed".to_string(),
        span: Span { start_byte: 0, end_byte: 0, start: crate::span::Position { row: 0, col: 0 }, end: crate::span::Position { row: 0, col: 0 }, text: None },
    })?;
    let root = tree.root_node();
    if root.has_error() {
        return Err(ParseError::Syntax { message: "syntax error".to_string(), span: Span::from_node(source, root) });
    }
    let mut cursor = root.walk();
    let mut items = Vec::new();
    let mut ctx = Context { scopes: vec![Scope { vars: Vec::new() }], specials: SpecialRegistry { fn_: None, fa: None, fs: None, t: None, preview: None, children: None }, modules: Vec::new(), functions: Vec::new() };
    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            if let Some(item) = read_item(source, child, &mut ctx) {
                items.push(item);
            }
        }
    }
    let ast = Ast { items, span: Span::from_node(source, root), context: ctx };
    Ok(ast)
}

fn read_item(source: &str, node: tree_sitter::Node, ctx: &mut Context) -> Option<Item> {
    match node.kind() {
        "var_declaration" => Some(Item::VarDecl(read_var_decl(source, node, ctx))),
        "module_item" => Some(Item::ModuleDef(read_module_def(source, node, ctx))),
        "function_item" => Some(Item::FunctionDef(read_function_def(source, node, ctx))),
        k if k == "statement" || k == "union_block" || k == "transform_chain" || k.ends_with("_block") || k.ends_with("_statement") => Some(Item::Stmt(read_stmt(source, node, ctx))),
        _ => None,
    }
}

fn read_var_decl(source: &str, node: tree_sitter::Node, _ctx: &mut Context) -> VarDecl {
    let span = Span::from_node(source, node);
    let assign = node.child_by_field_name("assignment").or_else(|| {
        for i in 0..node.child_count() { if let Some(ch) = node.child(i) { if ch.kind()=="assignment" { return Some(ch) } } }
        None
    }).unwrap();
    let name_node = assign.child_by_field_name("name").unwrap();
    let value_node = assign.child_by_field_name("value").unwrap();
    let name = read_name(source, name_node);
    let value = read_expr(source, value_node);
    let decl = VarDecl { name: name.clone(), value: value.clone(), span };
    if let Name::Special(s) = &name {
        match s.name.as_str() {
            "$fn" => _ctx.specials.fn_ = Some(value.clone()),
            "$fa" => _ctx.specials.fa = Some(value.clone()),
            "$fs" => _ctx.specials.fs = Some(value.clone()),
            "$t" => _ctx.specials.t = Some(value.clone()),
            "$preview" => _ctx.specials.preview = Some(value.clone()),
            "$children" => _ctx.specials.children = Some(value.clone()),
            _ => {}
        }
    }
    decl
}

fn read_module_def(source: &str, node: tree_sitter::Node, ctx: &mut Context) -> ModuleDef {
    let span = Span::from_node(source, node);
    let name = read_ident(source, node.child_by_field_name("name").unwrap());
    let params = read_params(source, node.child_by_field_name("parameters"));
    let body_stmt = read_stmt(source, node.child_by_field_name("body").unwrap(), ctx);
    let sig = ModuleSig { name: name.name.clone(), params: params.clone(), span: span.clone() };
    ctx.modules.push(sig);
    ModuleDef { name, params, body: body_stmt, span }
}

fn read_function_def(source: &str, node: tree_sitter::Node, ctx: &mut Context) -> FunctionDef {
    let span = Span::from_node(source, node);
    let name = read_ident(source, node.child_by_field_name("name").unwrap());
    let params = read_params(source, node.child_by_field_name("parameters"));
    let expr_node = node.child_by_field_name("expression").unwrap_or_else(|| node.child(node.child_count() - 1).unwrap());
    let body = read_expr(source, expr_node);
    let sig = FunctionSig { name: name.name.clone(), params: params.clone(), span: span.clone() };
    ctx.functions.push(sig);
    FunctionDef { name, params, body, span }
}

fn read_stmt(source: &str, node: tree_sitter::Node, ctx: &mut Context) -> Stmt {
    match node.kind() {
        "statement" => {
            for i in 0..node.child_count() {
                if let Some(ch) = node.child(i) { if ch.is_named() { return read_stmt(source, ch, ctx) } }
            }
            Stmt::Empty
        }
        "module_call" => {
            let call = read_module_call(source, node);
            Stmt::TransformChain(TransformChain { call, tail: Box::new(Stmt::Empty), span: Span::from_node(source, node) })
        }
        "union_block" => {
            let mut items = Vec::new();
            for i in 0..node.child_count() {
                if let Some(ch) = node.child(i) {
                    if let Some(it) = read_item(source, ch, ctx) { items.push(it); }
                }
            }
            Stmt::UnionBlock(Block { items, span: Span::from_node(source, node) })
        }
        "difference_block" => {
            let mut items = Vec::new();
            for i in 0..node.child_count() {
                if let Some(ch) = node.child(i) {
                    if let Some(it) = read_item(source, ch, ctx) { items.push(it); }
                }
            }
            let block = Stmt::UnionBlock(Block { items, span: Span::from_node(source, node) });
            let call = ModuleCall { name: Ident { name: "difference".to_string(), span: Span::from_node(source, node) }, args: Vec::new(), span: Span::from_node(source, node) };
            Stmt::TransformChain(TransformChain { call, tail: Box::new(block), span: Span::from_node(source, node) })
        }
        "intersection_block" => {
            let mut items = Vec::new();
            for i in 0..node.child_count() {
                if let Some(ch) = node.child(i) {
                    if let Some(it) = read_item(source, ch, ctx) { items.push(it); }
                }
            }
            let block = Stmt::UnionBlock(Block { items, span: Span::from_node(source, node) });
            let call = ModuleCall { name: Ident { name: "intersection".to_string(), span: Span::from_node(source, node) }, args: Vec::new(), span: Span::from_node(source, node) };
            Stmt::TransformChain(TransformChain { call, tail: Box::new(block), span: Span::from_node(source, node) })
        }
        "transform_chain" => {
            let mut call_node = None;
            let mut tail_node = None;
            for i in 0..node.child_count() { if let Some(ch) = node.child(i) { match ch.kind() { "module_call" => call_node = Some(ch), "statement" => tail_node = Some(ch), _ => {} } } }
            let call = read_module_call(source, call_node.unwrap());
            let tail_stmt = tail_node.map(|n| read_stmt(source, n, ctx)).unwrap_or(Stmt::Empty);
            Stmt::TransformChain(TransformChain { call, tail: Box::new(tail_stmt), span: Span::from_node(source, node) })
        }
        "include_statement" => {
            let p = node.child_by_field_name("include_path").map(|n| n.utf8_text(source.as_bytes()).unwrap().to_string()).unwrap_or_default();
            Stmt::Include(Include { path: p, span: Span::from_node(source, node) })
        }
        "use_statement" => {
            let p = node.child_by_field_name("include_path").map(|n| n.utf8_text(source.as_bytes()).unwrap().to_string()).unwrap_or_default();
            Stmt::Use(Include { path: p, span: Span::from_node(source, node) })
        }
        "assert_statement" => {
            let args = read_args(source, node.child_by_field_name("arguments").unwrap());
            let s = read_stmt(source, node.child_by_field_name("statement").unwrap(), ctx);
            Stmt::AssertStmt(AssertStmt { args, stmt: Box::new(s), span: Span::from_node(source, node) })
        }
        "for_block" => {
            let binds = read_assigns_or_cond(source, node.child(1).unwrap());
            let body = read_stmt(source, node.child_by_field_name("body").unwrap(), ctx);
            Stmt::ForBlock(ForBlock { binds, body: Box::new(body), span: Span::from_node(source, node) })
        }
        "intersection_for_block" => {
            let binds = read_assigns_or_cond(source, node.child(1).unwrap());
            let body = read_stmt(source, node.child_by_field_name("body").unwrap(), ctx);
            Stmt::IntersectionForBlock(ForBlock { binds, body: Box::new(body), span: Span::from_node(source, node) })
        }
        "if_block" => {
            let cond_expr = read_expr(source, node.child_by_field_name("condition").unwrap().child(1).unwrap());
            let cons = read_stmt(source, node.child_by_field_name("consequence").unwrap(), ctx);
            let alt = node.child_by_field_name("alternative").map(|_| read_stmt(source, node.child(node.child_count() - 1).unwrap(), ctx));
            Stmt::IfBlock(IfBlock { condition: cond_expr, consequence: Box::new(cons), alternative: alt.map(Box::new), span: Span::from_node(source, node) })
        }
        "let_block" | "assign_block" => {
            let assigns = read_assigns(source, node.child(1).unwrap());
            let body = read_stmt(source, node.child_by_field_name("body").unwrap(), ctx);
            Stmt::LetBlock(BodiedBlock { assigns, body: Box::new(body), span: Span::from_node(source, node) })
        }
        _ => Stmt::Empty,
    }
}

fn read_module_call(source: &str, node: tree_sitter::Node) -> ModuleCall {
    let name_node = node.child_by_field_name("name").or_else(|| node.child(0)).unwrap();
    let args = node.child_by_field_name("arguments").map(|n| read_args(source, n)).unwrap_or_default();
    ModuleCall { name: read_ident(source, name_node), args, span: Span::from_node(source, node) }
}

fn read_args(source: &str, node: tree_sitter::Node) -> Vec<Arg> {
    let mut out = Vec::new();
    for i in 0..node.child_count() {
        if let Some(ch) = node.child(i) {
            match ch.kind() {
                "assignment" => {
                    let name = read_name(source, ch.child_by_field_name("name").unwrap());
                    let value = read_expr(source, ch.child_by_field_name("value").unwrap());
                    out.push(Arg::Named { name, value })
                }
                _ => {
                    if ch.is_named() {
                        out.push(Arg::Positional(read_expr(source, ch)))
                    }
                }
            }
        }
    }
    out
}

fn read_params(source: &str, node: Option<tree_sitter::Node>) -> Vec<Param> {
    let mut params = Vec::new();
    if let Some(n) = node {
        for i in 0..n.child_count() {
            if let Some(ch) = n.child(i) {
                match ch.kind() {
                    "parameter" => {
                        if let Some(a) = ch.child_by_field_name("assignment") {
                            let name = read_name(source, a.child_by_field_name("name").unwrap());
                            let value = read_expr(source, a.child_by_field_name("value").unwrap());
                            params.push(Param { name, default: Some(value), span: Span::from_node(source, ch) })
                        } else if let Some(id) = ch.child(0) { params.push(Param { name: read_name(source, id), default: None, span: Span::from_node(source, ch) }) }
                    }
                    _ => {}
                }
            }
        }
    }
    params
}

fn read_assigns(source: &str, node: tree_sitter::Node) -> Vec<Assignment> {
    let mut out = Vec::new();
    for i in 0..node.child_count() {
        if let Some(ch) = node.child(i) {
            if ch.kind() == "assignment" {
                let name = read_name(source, ch.child_by_field_name("name").unwrap());
                let value = read_expr(source, ch.child_by_field_name("value").unwrap());
                out.push(Assignment { name, value, span: Span::from_node(source, ch) })
            }
        }
    }
    out
}

fn read_assigns_or_cond(source: &str, node: tree_sitter::Node) -> ForBinds {
    match node.kind() {
        "assignments" => ForBinds::Assigns(read_assigns(source, node)),
        "condition_update_clause" => {
            let init = read_assigns(source, node.child_by_field_name("initializer").unwrap());
            let cond = read_expr(source, node.child_by_field_name("condition").unwrap());
            let update = read_assigns(source, node.child_by_field_name("update").unwrap());
            ForBinds::CondUpdate(Box::new(CondUpdate { init, cond: Box::new(cond), update }))
        }
        _ => ForBinds::Assigns(Vec::new()),
    }
}

fn read_expr(source: &str, node: tree_sitter::Node) -> Expr {
    match node.kind() {
        "string" => {
            let s = node.utf8_text(source.as_bytes()).unwrap().to_string();
            Expr::Literal(Literal::String(s, Span::from_node(source, node)))
        }
        "integer" => {
            let v = node.utf8_text(source.as_bytes()).unwrap().parse::<i64>().unwrap_or(0);
            Expr::Literal(Literal::Integer(v, Span::from_node(source, node)))
        }
        "float" => {
            let v = node.utf8_text(source.as_bytes()).unwrap().parse::<f64>().unwrap_or(0.0);
            Expr::Literal(Literal::Float(v, Span::from_node(source, node)))
        }
        "boolean" => {
            let v = node.utf8_text(source.as_bytes()).unwrap() == "true";
            Expr::Literal(Literal::Boolean(v, Span::from_node(source, node)))
        }
        "undef" => Expr::Literal(Literal::Undef(Span::from_node(source, node))),
        "identifier" => Expr::Ident(read_ident(source, node)),
        "special_variable" => Expr::SpecialIdent(read_special(source, node)),
        "function_call" => {
            let callee = read_expr(source, node.child_by_field_name("name").unwrap());
            let args = read_args(source, node.child_by_field_name("arguments").unwrap());
            Expr::Call { callee: Box::new(callee), args, span: Span::from_node(source, node) }
        }
        "index_expression" => {
            let value = read_expr(source, node.child_by_field_name("value").unwrap());
            let idx_expr = node.child(node.child_count() - 1).unwrap();
            let index = read_expr(source, idx_expr);
            Expr::Index { value: Box::new(value), index: Box::new(index), span: Span::from_node(source, node) }
        }
        "dot_index_expression" => {
            let value = read_expr(source, node.child_by_field_name("value").unwrap());
            let field = read_ident(source, node.child_by_field_name("index").unwrap());
            Expr::DotIndex { value: Box::new(value), field, span: Span::from_node(source, node) }
        }
        "unary_expression" => {
            let op = read_unary_op(node.child(0).unwrap().utf8_text(source.as_bytes()).unwrap());
            let expr = read_expr(source, node.child(1).unwrap());
            Expr::Unary { op, expr: Box::new(expr), span: Span::from_node(source, node) }
        }
        "binary_expression" => {
            let left = read_expr(source, node.child_by_field_name("left").unwrap());
            let right = read_expr(source, node.child_by_field_name("right").unwrap());
            let op_text = find_operator_text(source, node);
            let op = read_binary_op(&op_text);
            Expr::Binary { op, left: Box::new(left), right: Box::new(right), span: Span::from_node(source, node) }
        }
        "ternary_expression" => {
            let cond = read_expr(source, node.child_by_field_name("condition").unwrap());
            let cons = read_expr(source, node.child_by_field_name("consequence").unwrap());
            let alt = read_expr(source, node.child_by_field_name("alternative").unwrap());
            Expr::Ternary { cond: Box::new(cond), cons: Box::new(cons), alt: Box::new(alt), span: Span::from_node(source, node) }
        }
        "parenthesized_expression" => {
            let inner = read_expr(source, node.child(1).unwrap());
            Expr::Paren { inner: Box::new(inner), span: Span::from_node(source, node) }
        }
        "let_expression" => {
            let assigns = read_assigns(source, node.child(1).unwrap());
            let body = read_expr(source, node.child_by_field_name("body").unwrap());
            Expr::LetExpr { assigns, body: Box::new(body), span: Span::from_node(source, node) }
        }
        "assert_expression" => {
            let args = read_args(source, node.child_by_field_name("arguments").unwrap());
            let tail = node.child_by_field_name("expression").map(|e| read_expr(source, e));
            Expr::AssertExpr { args, tail: tail.map(Box::new), span: Span::from_node(source, node) }
        }
        "echo_expression" => {
            let args = read_args(source, node.child_by_field_name("arguments").unwrap());
            let tail = node.child_by_field_name("expression").map(|e| read_expr(source, e));
            Expr::EchoExpr { args, tail: tail.map(Box::new), span: Span::from_node(source, node) }
        }
        "list" => {
            let mut cells = Vec::new();
            for i in 0..node.child_count() {
                if let Some(ch) = node.child(i) {
                    if ch.is_named() {
                        cells.push(ListCell::Expr(read_expr(source, ch)))
                    }
                }
            }
            Expr::List { cells, span: Span::from_node(source, node) }
        }
        "each" => {
            let target = node.child(node.child_count() - 1).map(|n| read_expr(source, n)).unwrap_or(Expr::Literal(Literal::Undef(Span::from_node(source, node))));
            Expr::Each { value: Box::new(target), span: Span::from_node(source, node) }
        }
        "list_comprehension" => {
            let inner = node.child(0).unwrap();
            match inner.kind() {
                "for_clause" => {
                    let binds = read_assigns_or_cond(source, inner.child(1).unwrap());
                    let cell = read_expr(source, inner.child(inner.child_count() - 1).unwrap());
                    Expr::ListComp { kind: Box::new(ListCompKind::For { binds: Box::new(binds), cell: Box::new(cell) }), span: Span::from_node(source, node) }
                }
                "if_clause" => {
                    let condition = read_expr(source, inner.child_by_field_name("condition").unwrap().child(1).unwrap());
                    let consequence = read_expr(source, inner.child_by_field_name("consequence").unwrap());
                    let alternative = inner.child_by_field_name("alternative").map(|_| read_expr(source, inner.child(inner.child_count() - 1).unwrap()));
                    Expr::ListComp { kind: Box::new(ListCompKind::If { condition: Box::new(condition), consequence: Box::new(consequence), alternative: alternative.map(Box::new) }), span: Span::from_node(source, node) }
                }
                _ => Expr::Literal(Literal::Undef(Span::from_node(source, node))),
            }
        }
        "range" => {
            let start = read_expr(source, node.child_by_field_name("start").unwrap());
            let inc = node.child_by_field_name("increment").map(|e| read_expr(source, e));
            let end = read_expr(source, node.child_by_field_name("end").unwrap());
            Expr::Range { start: Box::new(start), inc: inc.map(Box::new), end: Box::new(end), span: Span::from_node(source, node) }
        }
        "function_lit" => {
            let params = read_params(source, node.child_by_field_name("parameters"));
            let body = read_expr(source, node.child_by_field_name("body").unwrap());
            Expr::FunctionLit { params, body: Box::new(body), span: Span::from_node(source, node) }
        }
        _ => Expr::Literal(Literal::Undef(Span::from_node(source, node))),
    }
}

fn read_ident(source: &str, node: tree_sitter::Node) -> Ident { Ident { name: node.utf8_text(source.as_bytes()).unwrap().to_string(), span: Span::from_node(source, node) } }
fn read_special(source: &str, node: tree_sitter::Node) -> SpecialIdent { let name = node.child(1).unwrap().utf8_text(source.as_bytes()).unwrap().to_string(); SpecialIdent { name: format!("${}", name), span: Span::from_node(source, node) } }
fn read_name(source: &str, node: tree_sitter::Node) -> Name { match node.kind() { "special_variable" => Name::Special(read_special(source, node)), _ => Name::Ident(read_ident(source, node)) } }

fn read_unary_op(s: &str) -> UnaryOp { match s { "!" => UnaryOp::Not, "+" => UnaryOp::Pos, "-" => UnaryOp::Neg, _ => UnaryOp::Pos } }
fn read_binary_op(s: &str) -> BinaryOp { match s { "||" => BinaryOp::Or, "&&" => BinaryOp::And, "==" => BinaryOp::Eq, "!=" => BinaryOp::Ne, "<" => BinaryOp::Lt, ">" => BinaryOp::Gt, "<=" => BinaryOp::Le, ">=" => BinaryOp::Ge, "+" => BinaryOp::Add, "-" => BinaryOp::Sub, "*" => BinaryOp::Mul, "/" => BinaryOp::Div, "%" => BinaryOp::Mod, "^" => BinaryOp::Pow, _ => BinaryOp::Add } }

fn find_operator_text(source: &str, node: tree_sitter::Node) -> String {
    for i in 0..node.child_count() {
        if let Some(ch) = node.child(i) { if !ch.is_named() { return ch.utf8_text(source.as_bytes()).unwrap().to_string() } }
    }
    "".to_string()
}
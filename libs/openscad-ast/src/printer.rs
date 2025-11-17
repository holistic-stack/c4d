use crate::ast::*;

pub fn print(ast: &Ast) -> String { ast.items.iter().map(print_item).collect::<Vec<_>>().join("\n") }

fn print_item(item: &Item) -> String {
    match item {
        Item::VarDecl(v) => format!("{} = {};", print_name(&v.name), print_expr(&v.value)),
        Item::Stmt(s) => print_stmt(s),
        Item::ModuleDef(m) => format!("module {}() {{ {} }}", m.name.name, print_stmt(&m.body)),
        Item::FunctionDef(f) => format!("function {}() = {};", f.name.name, print_expr(&f.body)),
    }
}

fn print_stmt(s: &Stmt) -> String {
    match s {
        Stmt::TransformChain(t) => format!("{}({}) {};", t.call.name.name, print_args(&t.call.args), match *t.tail.clone() { Stmt::Empty => String::new(), _ => print_stmt(&t.tail) }),
        Stmt::UnionBlock(b) => format!("{{ {} }}", b.items.iter().map(print_item).collect::<Vec<_>>().join(" ")),
        Stmt::Include(i) => format!("include {};", i.path),
        Stmt::Use(i) => format!("use {};", i.path),
        Stmt::IfBlock(i) => {
            let mut s = format!("if ({}) {}", print_expr(&i.condition), print_stmt(&i.consequence));
            if let Some(a) = &i.alternative { s.push_str(&format!(" else {}", print_stmt(a))) }
            s
        }
        Stmt::LetBlock(b) => format!("let ({}) {}", print_assigns(&b.assigns), print_stmt(&b.body)),
        Stmt::AssignBlock(b) => format!("assign ({}) {}", print_assigns(&b.assigns), print_stmt(&b.body)),
        Stmt::ForBlock(f) => format!("for ({}) {}", print_for_binds(&f.binds), print_stmt(&f.body)),
        Stmt::IntersectionForBlock(f) => format!("intersection_for ({}) {}", print_for_binds(&f.binds), print_stmt(&f.body)),
        Stmt::AssertStmt(a) => format!("assert({}) {}", print_args(&a.args), print_stmt(&a.stmt)),
        Stmt::Empty => String::new(),
    }
}

fn print_args(args: &Vec<Arg>) -> String {
    args.iter().map(|a| match a { Arg::Positional(e) => print_expr(e), Arg::Named { name, value } => format!("{}={}", print_name(name), print_expr(value)) }).collect::<Vec<_>>().join(", ")
}

fn print_name(n: &Name) -> String { match n { Name::Ident(i) => i.name.clone(), Name::Special(s) => s.name.clone() } }

fn print_expr(e: &Expr) -> String {
    match e {
        Expr::Literal(Literal::Integer(v, _)) => v.to_string(),
        Expr::Literal(Literal::Float(v, _)) => v.to_string(),
        Expr::Literal(Literal::Boolean(v, _)) => v.to_string(),
        Expr::Literal(Literal::String(s, _)) => s.clone(),
        Expr::Ident(i) => i.name.clone(),
        Expr::SpecialIdent(i) => i.name.clone(),
        Expr::List { cells, .. } => format!("[{}]", cells.iter().map(|c| match c { ListCell::Expr(e) => print_expr(e) }).collect::<Vec<_>>().join(",")),
        Expr::Range { start, inc, end, .. } => match inc { Some(i) => format!("[{}:{}:{}]", print_expr(start), print_expr(i), print_expr(end)), None => format!("[{}:{}]", print_expr(start), print_expr(end)) },
        Expr::Binary { op, left, right, .. } => format!("{} {} {}", print_expr(left), print_op(op), print_expr(right)),
        Expr::Call { callee, args, .. } => format!("{}({})", print_expr(callee), print_args(args)),
        Expr::Unary { op, expr, .. } => format!("{}{}", print_unary_op(op), print_expr(expr)),
        Expr::Paren { inner, .. } => format!("({})", print_expr(inner)),
        Expr::Index { value, index, .. } => format!("{}[{}]", print_expr(value), print_expr(index)),
        Expr::DotIndex { value, field, .. } => format!("{}.{}", print_expr(value), field.name),
        Expr::Ternary { cond, cons, alt, .. } => format!("{} ? {} : {}", print_expr(cond), print_expr(cons), print_expr(alt)),
        Expr::LetExpr { assigns, body, .. } => format!("let ({}) {}", print_assigns(assigns), print_expr(body)),
        Expr::AssertExpr { args, tail, .. } => match tail { Some(t) => format!("assert({}) {}", print_args(args), print_expr(t)), None => format!("assert({})", print_args(args)) },
        Expr::EchoExpr { args, tail, .. } => match tail { Some(t) => format!("echo({}) {}", print_args(args), print_expr(t)), None => format!("echo({})", print_args(args)) },
        Expr::Each { value, .. } => format!("each {}", print_expr(value)),
        Expr::ListComp { .. } => String::new(),
        _ => String::new(),
    }
}

fn print_op(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Or => "||",
        BinaryOp::And => "&&",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::Le => "<=",
        BinaryOp::Ge => ">=",
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Pow => "^",
    }
}

fn print_unary_op(op: &UnaryOp) -> &'static str {
    match op { UnaryOp::Not => "!", UnaryOp::Pos => "+", UnaryOp::Neg => "-" }
}

fn print_assigns(a: &Vec<Assignment>) -> String {
    a.iter().map(|x| format!("{}={}", print_name(&x.name), print_expr(&x.value))).collect::<Vec<_>>().join(", ")
}

fn print_for_binds(b: &ForBinds) -> String {
    match b { ForBinds::Assigns(v) => print_assigns(v), ForBinds::CondUpdate(c) => format!("{}; {}; {}", print_assigns(&c.init), print_expr(&c.cond), print_assigns(&c.update)) }
}
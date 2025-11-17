use crate::{ast::*, error::*};

pub fn validate(ast: &Ast) -> Result<(), ParseError> {
    for item in &ast.items {
        if let Item::Stmt(s) = item { validate_stmt(s)?; }
    }
    Ok(())
}

fn validate_stmt(s: &Stmt) -> Result<(), ParseError> {
    match s {
        Stmt::LetBlock(b) | Stmt::AssignBlock(b) => {
            if b.assigns.is_empty() {
                return Err(ParseError::Semantic { kind: SemanticKind::MalformedBlock, span: b.span.clone(), details: Some("empty assignments".into()) })
            }
            validate_stmt(&b.body)
        }
        Stmt::ForBlock(f) | Stmt::IntersectionForBlock(f) => {
            match &f.binds {
                ForBinds::Assigns(v) => if v.is_empty() { return Err(ParseError::Semantic { kind: SemanticKind::MalformedBlock, span: f.span.clone(), details: Some("empty for binds".into()) }) },
                ForBinds::CondUpdate(_) => {}
            }
            validate_stmt(&f.body)
        }
        Stmt::UnionBlock(b) => {
            for it in &b.items { if let Item::Stmt(st) = it { validate_stmt(st)? } }
            Ok(())
        }
        Stmt::TransformChain(t) => {
            if t.call.name.name == "multmatrix" {
                if !valid_multmatrix_args(&t.call.args) {
                    return Err(ParseError::Semantic { kind: SemanticKind::InvalidMatrix, span: t.span.clone(), details: Some("multmatrix expects 4x4 matrix".into()) })
                }
            }
            validate_stmt(&t.tail)
        }
        Stmt::IfBlock(i) => {
            validate_stmt(&i.consequence)?;
            if let Some(a) = &i.alternative { validate_stmt(a)? }
            Ok(())
        }
        _ => Ok(())
    }
}

fn valid_multmatrix_args(args: &Vec<Arg>) -> bool {
    let target = args.iter().find_map(|a| match a { Arg::Positional(e) => Some(e), Arg::Named { name, value } => match name { Name::Ident(i) if i.name == "m" => Some(value), _ => None } }).cloned();
    if let Some(e) = target {
        is_4x4_list(&e)
    } else {
        false
    }
}

fn is_4x4_list(e: &Expr) -> bool {
    match e {
        Expr::List { cells, .. } if cells.len() == 4 => cells.iter().all(|c| match c { ListCell::Expr(Expr::List { cells: inner, .. }) if inner.len() == 4 => inner.iter().all(|x| match x { ListCell::Expr(Expr::Literal(Literal::Integer(_, _))) | ListCell::Expr(Expr::Literal(Literal::Float(_, _))) => true, _ => false }), _ => false }),
        _ => false,
    }
}
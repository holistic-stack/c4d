use openscad_ast::{parse, parse_strict, ParseError};

#[test]
fn invalid_dot_index_is_syntax_error() {
    let src = "echo(object.1);";
    let err = parse(src).err().unwrap();
    match err { ParseError::Syntax { .. } => {}, _ => panic!("expected syntax error") }
}

#[test]
fn empty_let_block_is_semantic_error() {
    let src = "let () cube(1);";
    let err = parse_strict(src).err().unwrap();
    match err { ParseError::Semantic { .. } => {}, _ => panic!("expected semantic error") }
}

#[test]
fn empty_for_parens_is_semantic_error() {
    let src = "for () cube(1);";
    let err = parse_strict(src).err().unwrap();
    match err { ParseError::Semantic { .. } => {}, _ => panic!("expected semantic error") }
}
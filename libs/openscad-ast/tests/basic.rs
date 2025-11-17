use openscad_ast::parse;
use openscad_ast::ParseError;

#[test]
fn parses_var_declaration_and_specials() {
    let src = "$fn = 32; a = 1;";
    let ast = parse(src).unwrap();
    assert!(ast.items.len() >= 2);
}

#[test]
fn parses_transform_chain() {
    let src = "translate([1,2,3]) cube(1);";
    let ast = parse(src).unwrap();
    assert!(!ast.items.is_empty());
}

#[test]
fn syntax_error_on_unterminated_list() {
    let src = "[1,2,3";
    let err = parse(src).err().unwrap();
    match err { ParseError::Syntax { .. } => {}, _ => panic!("expected syntax error") }
}
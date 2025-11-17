use openscad_ast::parse;

#[test]
fn specials_registry_populated() {
    let src = "$fn = 32; $fa = 12; $fs = 0.5; $t = 0; $preview = true; $children = 1;";
    let ast = parse(src).unwrap();
    assert!(ast.context.specials.fn_.is_some());
    assert!(ast.context.specials.fa.is_some());
    assert!(ast.context.specials.fs.is_some());
    assert!(ast.context.specials.t.is_some());
    assert!(ast.context.specials.preview.is_some());
    assert!(ast.context.specials.children.is_some());
}
use openscad_ast::{parse, print};

#[test]
fn roundtrip_basic() {
    let src = "a = 1; translate([1,2,3]) cube(1);";
    let ast1 = parse(src).unwrap();
    let out = print(&ast1);
    let ast2 = parse(&out).unwrap();
    assert!(ast2.items.len() >= 2);
}
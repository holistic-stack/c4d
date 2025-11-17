use openscad_ast::{parse_strict, ParseError};

#[test]
fn invalid_multmatrix_shape_is_semantic_error() {
    let src = "multmatrix([[1,0],[0,1]]) cube(1);";
    let err = parse_strict(src).err().unwrap();
    match err { ParseError::Semantic { .. } => {}, _ => panic!("expected semantic error") }
}
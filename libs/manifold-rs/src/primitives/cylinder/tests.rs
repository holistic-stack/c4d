use super::*;

#[test]
fn test_cylinder_simple() {
    let cyl = cylinder(10.0, 5.0, 5.0, false, 4).expect("cylinder succeeds");
    // 8 vertices.
    // Sides: 4 * 2 = 8 tris.
    // Caps: 2 * (4-2) = 4 tris.
    // Total 12 tris.
    assert_eq!(cyl.vertex_count(), 8);
    assert_eq!(cyl.face_count(), 12);
    cyl.validate().expect("valid topology");
}

#[test]
fn test_cone() {
    // Top radius 0
    let cone = cylinder(10.0, 5.0, 0.0, false, 4).expect("cone succeeds");
    // Vertices: 4 bottom + 1 apex = 5.
    assert_eq!(cone.vertex_count(), 5);
    // Faces: 4 sides (triangles) + 2 bottom cap = 6.
    assert_eq!(cone.face_count(), 6);
    cone.validate().expect("valid topology");
}

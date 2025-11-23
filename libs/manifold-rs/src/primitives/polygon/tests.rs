use super::*;

#[test]
fn test_polygon_triangle() {
    let points = vec![
        DVec2::new(0.0, 0.0),
        DVec2::new(10.0, 0.0),
        DVec2::new(0.0, 10.0),
    ];
    let paths = vec![vec![0, 1, 2]];

    let m = polygon(points, paths, 1).unwrap();
    assert!(m.validate().is_ok());
    assert_eq!(m.face_count(), 2); // Front + Back
}

#[test]
fn test_polygon_square() {
    let points = vec![
        DVec2::new(0.0, 0.0),
        DVec2::new(10.0, 0.0),
        DVec2::new(10.0, 10.0),
        DVec2::new(0.0, 10.0),
    ];
    let paths = vec![vec![0, 1, 2, 3]];

    let m = polygon(points, paths, 1).unwrap();
    assert!(m.validate().is_ok());
    assert_eq!(m.face_count(), 4); // 2 triangles front + 2 back
}

#[test]
fn test_polygon_too_few_points() {
    let points = vec![
        DVec2::new(0.0, 0.0),
        DVec2::new(10.0, 0.0),
    ];
    let paths = vec![vec![0, 1]];

    assert!(polygon(points, paths, 1).is_err());
}

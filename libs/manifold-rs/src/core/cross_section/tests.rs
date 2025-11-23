use super::*;
use crate::core::vec2::Vec2;

#[test]
fn test_square_creation() {
    let sq = CrossSection::square(Vec2::new(10.0, 20.0), false);
    assert_eq!(sq.contours.len(), 1);
    let contour = &sq.contours[0];
    assert_eq!(contour.len(), 4);
    assert_eq!(contour[0], Vec2::new(0.0, 0.0));
    assert_eq!(contour[1], Vec2::new(10.0, 0.0));
    assert_eq!(contour[2], Vec2::new(10.0, 20.0));
    assert_eq!(contour[3], Vec2::new(0.0, 20.0));
}

#[test]
fn test_square_centered() {
    let sq = CrossSection::square(Vec2::new(10.0, 10.0), true);
    let contour = &sq.contours[0];
    assert_eq!(contour[0], Vec2::new(-5.0, -5.0));
    assert_eq!(contour[2], Vec2::new(5.0, 5.0));
}

#[test]
fn test_circle_creation() {
    let circle = CrossSection::circle(10.0, 4);
    assert_eq!(circle.contours.len(), 1);
    let contour = &circle.contours[0];
    assert_eq!(contour.len(), 4);
    // Square rotated by 0 degrees (points at (10,0), (0,10), (-10,0), (0,-10) approx)
    // Actually, theta=0 -> (10, 0). theta=90 -> (0, 10).
    // cos(0)=1, sin(0)=0.
    assert!((contour[0].x - 10.0).abs() < 1e-10);
    assert!((contour[0].y - 0.0).abs() < 1e-10);
}

#[test]
fn test_polygon_simple() {
    let points = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
        Vec2::new(5.0, 10.0),
    ];
    let poly = CrossSection::polygon(&points, None).unwrap();
    assert_eq!(poly.contours.len(), 1);
    assert_eq!(poly.contours[0], points);
}

#[test]
fn test_polygon_paths() {
    let points = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
        Vec2::new(10.0, 10.0),
        Vec2::new(0.0, 10.0),
    ];
    // Define two triangles from the square vertices
    let paths = vec![
        vec![0, 1, 2],
        vec![0, 2, 3],
    ];
    let poly = CrossSection::polygon(&points, Some(&paths)).unwrap();
    assert_eq!(poly.contours.len(), 2);
    assert_eq!(poly.contours[0].len(), 3);
    assert_eq!(poly.contours[1].len(), 3);
}

#[test]
fn test_polygon_out_of_bounds() {
    let points = vec![Vec2::new(0.0, 0.0)];
    let paths = vec![vec![1]]; // Index 1 is out of bounds
    let res = CrossSection::polygon(&points, Some(&paths));
    assert!(res.is_err());
}

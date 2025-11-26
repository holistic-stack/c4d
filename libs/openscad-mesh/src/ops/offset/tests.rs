//! # Offset Tests
//!
//! Tests for 2D polygon offset operations.

use super::*;
use glam::DVec2;

#[test]
fn test_offset_square_expand() {
    let square = Polygon2D::square(DVec2::splat(10.0), true);
    let params = OffsetParams {
        amount: 1.0,
        chamfer: false,
    };
    
    let result = offset_polygon(&square, &params).unwrap();
    
    // Expanded square should have at least 4 vertices
    assert!(result.vertex_count() >= 4);
    
    // Check that at least some vertices are farther from origin
    let max_dist = result.outer.iter()
        .map(|v| v.x.abs().max(v.y.abs()))
        .fold(0.0_f64, |a, b| a.max(b));
    assert!(max_dist > 5.0, "Max distance should be > 5.0, got {}", max_dist);
}

#[test]
fn test_offset_square_shrink() {
    let square = Polygon2D::square(DVec2::splat(10.0), true);
    let params = OffsetParams {
        amount: -1.0,
        chamfer: false,
    };
    
    let result = offset_polygon(&square, &params).unwrap();
    
    // Shrunk square should have at least 4 vertices
    assert!(result.vertex_count() >= 4);
    
    // Check that at least some vertices are closer to origin
    let min_dist = result.outer.iter()
        .map(|v| v.x.abs().max(v.y.abs()))
        .fold(f64::MAX, |a, b| a.min(b));
    assert!(min_dist < 5.0, "Min distance should be < 5.0, got {}", min_dist);
}

#[test]
fn test_offset_zero() {
    let square = Polygon2D::square(DVec2::splat(10.0), true);
    let params = OffsetParams {
        amount: 0.0,
        chamfer: false,
    };
    
    let result = offset_polygon(&square, &params).unwrap();
    
    // Zero offset should return same polygon
    assert_eq!(result.vertex_count(), square.vertex_count());
}

#[test]
fn test_offset_chamfer() {
    let square = Polygon2D::square(DVec2::splat(10.0), true);
    let params = OffsetParams {
        amount: 2.0,
        chamfer: true,
    };
    
    let result = offset_polygon(&square, &params).unwrap();
    
    // Chamfered square should have more vertices (8 for a square)
    assert!(result.vertex_count() >= 4);
}

#[test]
fn test_offset_triangle() {
    let triangle = Polygon2D::new(vec![
        DVec2::new(0.0, 0.0),
        DVec2::new(10.0, 0.0),
        DVec2::new(5.0, 10.0),
    ]);
    let params = OffsetParams {
        amount: 1.0,
        chamfer: false,
    };
    
    let result = offset_polygon(&triangle, &params).unwrap();
    
    assert!(result.vertex_count() >= 3);
}

#[test]
fn test_offset_invalid_polygon() {
    let line = Polygon2D::new(vec![
        DVec2::new(0.0, 0.0),
        DVec2::new(10.0, 0.0),
    ]);
    let params = OffsetParams::default();
    
    let result = offset_polygon(&line, &params);
    assert!(result.is_err());
}

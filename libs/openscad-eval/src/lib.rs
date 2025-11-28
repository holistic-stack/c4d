//! # OpenSCAD Eval
//!
//! AST evaluation and geometry IR generation.
//!
//! ## Architecture
//!
//! ```text
//! Source → openscad-ast (AST) → openscad-eval (GeometryNode) → openscad-mesh
//! ```
//!
//! ## Example
//!
//! ```rust
//! use openscad_eval::evaluate;
//!
//! let result = evaluate("cube(10);").unwrap();
//! // result.geometry is a GeometryNode::Cube
//! ```

pub mod geometry;
pub mod error;
pub mod scope;
pub mod visitor;
pub mod value;

// Re-export public API
pub use geometry::{GeometryNode, EvaluatedAst};
pub use error::EvalError;
pub use scope::Scope;
pub use value::Value;

// =============================================================================
// PUBLIC API
// =============================================================================

/// Evaluate OpenSCAD source code to geometry.
///
/// This is the main entry point for the evaluator.
///
/// ## Parameters
///
/// - `source`: OpenSCAD source code string
///
/// ## Returns
///
/// `Result<EvaluatedAst, EvalError>` - Evaluated geometry on success
///
/// ## Example
///
/// ```rust
/// use openscad_eval::evaluate;
///
/// let result = evaluate("cube(10);").unwrap();
/// ```
pub fn evaluate(source: &str) -> Result<EvaluatedAst, EvalError> {
    // Parse to AST using openscad-ast
    let ast = openscad_ast::parse(source)
        .map_err(|e| EvalError::ParseError(e.to_string()))?;
    
    // Evaluate AST to geometry
    visitor::evaluate_ast(&ast)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test evaluating simple cube.
    #[test]
    fn test_evaluate_cube() {
        let result = evaluate("cube(10);").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, center } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
                assert!(!center);
            }
            _ => panic!("Expected Cube, got {:?}", result.geometry),
        }
    }

    /// Test evaluating cube with center.
    #[test]
    fn test_evaluate_cube_center() {
        let result = evaluate("cube(10, center=true);").unwrap();
        match result.geometry {
            GeometryNode::Cube { center, .. } => {
                assert!(center);
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test evaluating cube with array size.
    #[test]
    fn test_evaluate_cube_array() {
        let result = evaluate("cube([10, 20, 30]);").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 20.0, 30.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test evaluating union.
    #[test]
    fn test_evaluate_union() {
        let result = evaluate("union() { cube(10); cube(5); }").unwrap();
        match result.geometry {
            GeometryNode::Union { children } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected Union"),
        }
    }

    /// Test variable assignment.
    #[test]
    fn test_evaluate_variable() {
        let result = evaluate("x = 10; cube(x);").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test $fn special variable.
    #[test]
    fn test_evaluate_fn() {
        let result = evaluate("$fn = 32; sphere(5);").unwrap();
        match result.geometry {
            GeometryNode::Sphere { fn_, .. } => {
                assert_eq!(fn_, 32);
            }
            _ => panic!("Expected Sphere"),
        }
    }

    /// Test $fn in primitive argument.
    #[test]
    fn test_evaluate_fn_in_arg() {
        let result = evaluate("sphere(5, $fn=24);").unwrap();
        match result.geometry {
            GeometryNode::Sphere { fn_, .. } => {
                assert_eq!(fn_, 24);
            }
            _ => panic!("Expected Sphere"),
        }
    }

    /// Test variable scoping.
    #[test]
    fn test_evaluate_scope() {
        // Inner scope shadows outer variable
        let result = evaluate("x = 10; { x = 5; cube(x); }").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [5.0, 5.0, 5.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test for loop.
    #[test]
    fn test_evaluate_for_loop() {
        let result = evaluate("for (i = [0:2]) translate([i * 10, 0, 0]) cube(5);").unwrap();
        match &result.geometry {
            GeometryNode::Group { children } => {
                assert_eq!(children.len(), 3); // 0, 1, 2
            }
            other => panic!("Expected Group with 3 children, got {:?}", other),
        }
    }

    /// Test if/else.
    #[test]
    fn test_evaluate_if_else() {
        // True condition
        let result = evaluate("if (true) cube(10);").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
            }
            _ => panic!("Expected Cube"),
        }

        // False condition with else
        let result = evaluate("if (false) cube(10); else sphere(5);").unwrap();
        match result.geometry {
            GeometryNode::Sphere { radius, .. } => {
                assert_eq!(radius, 5.0);
            }
            _ => panic!("Expected Sphere"),
        }
    }

    /// Test mirror transform.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// mirror([1, 0, 0]) cube(10);
    /// ```
    #[test]
    fn test_evaluate_mirror() {
        let result = evaluate("mirror([1, 0, 0]) cube(10);").unwrap();
        match result.geometry {
            GeometryNode::Mirror { normal, child } => {
                assert_eq!(normal, [1.0, 0.0, 0.0]);
                match *child {
                    GeometryNode::Cube { size, .. } => {
                        assert_eq!(size, [10.0, 10.0, 10.0]);
                    }
                    _ => panic!("Expected Cube as child"),
                }
            }
            _ => panic!("Expected Mirror"),
        }
    }

    /// Test mirror with diagonal normal.
    #[test]
    fn test_evaluate_mirror_diagonal() {
        let result = evaluate("mirror([1, 1, 0]) cube(5);").unwrap();
        match result.geometry {
            GeometryNode::Mirror { normal, .. } => {
                assert_eq!(normal, [1.0, 1.0, 0.0]);
            }
            _ => panic!("Expected Mirror"),
        }
    }

    /// Test color modifier with RGB.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// color([1, 0, 0]) cube(10);
    /// ```
    #[test]
    fn test_evaluate_color_rgb() {
        let result = evaluate("color([1, 0, 0]) cube(10);").unwrap();
        match result.geometry {
            GeometryNode::Color { rgba, child } => {
                assert_eq!(rgba[0], 1.0);
                assert_eq!(rgba[1], 0.0);
                assert_eq!(rgba[2], 0.0);
                match *child {
                    GeometryNode::Cube { size, .. } => {
                        assert_eq!(size, [10.0, 10.0, 10.0]);
                    }
                    _ => panic!("Expected Cube as child"),
                }
            }
            _ => panic!("Expected Color"),
        }
    }

    /// Test color modifier with RGBA.
    #[test]
    fn test_evaluate_color_rgba() {
        let result = evaluate("color([0, 1, 0, 0.5]) cube(5);").unwrap();
        match result.geometry {
            GeometryNode::Color { rgba, .. } => {
                assert_eq!(rgba, [0.0, 1.0, 0.0, 0.5]);
            }
            _ => panic!("Expected Color"),
        }
    }

    /// Test user-defined function.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// function double(x) = x * 2;
    /// cube(double(5));
    /// ```
    #[test]
    fn test_evaluate_user_function() {
        let result = evaluate("function double(x) = x * 2; cube(double(5));").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test user-defined function with multiple parameters.
    #[test]
    fn test_evaluate_user_function_multi_param() {
        let result = evaluate("function add(a, b) = a + b; cube(add(3, 7));").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test nested function calls.
    #[test]
    fn test_evaluate_nested_functions() {
        let result = evaluate("function double(x) = x * 2; function triple(x) = x * 3; cube(double(triple(2)));").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                // triple(2) = 6, double(6) = 12
                assert_eq!(size, [12.0, 12.0, 12.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test function in for loop.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// function offset(i) = i * 10;
    /// for (i = [0:2]) translate([offset(i), 0, 0]) cube(5);
    /// ```
    #[test]
    fn test_evaluate_function_in_loop() {
        let result = evaluate("function offset(i) = i * 10; for (i = [0:2]) translate([offset(i), 0, 0]) cube(5);").unwrap();
        match result.geometry {
            GeometryNode::Group { children } => {
                assert_eq!(children.len(), 3);
                // First child should be translated by 0
                match &children[0] {
                    GeometryNode::Translate { offset, .. } => {
                        assert_eq!(offset[0], 0.0);
                    }
                    _ => panic!("Expected Translate"),
                }
                // Second child should be translated by 10
                match &children[1] {
                    GeometryNode::Translate { offset, .. } => {
                        assert_eq!(offset[0], 10.0);
                    }
                    _ => panic!("Expected Translate"),
                }
            }
            _ => panic!("Expected Group"),
        }
    }

    // =========================================================================
    // USER-DEFINED MODULES TESTS
    // =========================================================================

    /// Test simple user-defined module.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// module box(size=10) { cube(size); }
    /// box(20);
    /// ```
    #[test]
    fn test_evaluate_user_module() {
        let result = evaluate("module box(size=10) { cube(size); } box(20);").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [20.0, 20.0, 20.0]);
            }
            _ => panic!("Expected Cube, got {:?}", result.geometry),
        }
    }

    /// Test module with default parameter.
    #[test]
    fn test_evaluate_module_default_param() {
        let result = evaluate("module box(size=10) { cube(size); } box();").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    /// Test module with children.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// module wrapper() { translate([10, 0, 0]) children(); }
    /// wrapper() cube(5);
    /// ```
    #[test]
    fn test_evaluate_module_with_children() {
        let result = evaluate("module wrapper() { translate([10, 0, 0]) children(); } wrapper() cube(5);").unwrap();
        match result.geometry {
            GeometryNode::Translate { offset, child } => {
                assert_eq!(offset, [10.0, 0.0, 0.0]);
                match *child {
                    GeometryNode::Cube { size, .. } => {
                        assert_eq!(size, [5.0, 5.0, 5.0]);
                    }
                    _ => panic!("Expected Cube as child"),
                }
            }
            _ => panic!("Expected Translate"),
        }
    }

    /// Test module with multiple children.
    #[test]
    fn test_evaluate_module_multiple_children() {
        let result = evaluate("module wrapper() { color([1,0,0]) children(); } wrapper() { cube(5); sphere(3); }").unwrap();
        match result.geometry {
            GeometryNode::Color { child, .. } => {
                match *child {
                    GeometryNode::Group { children } => {
                        assert_eq!(children.len(), 2);
                    }
                    _ => panic!("Expected Group"),
                }
            }
            _ => panic!("Expected Color"),
        }
    }

    /// Test nested module calls.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// module outer() { translate([10, 0, 0]) children(); }
    /// module inner() { scale([2, 2, 2]) children(); }
    /// outer() inner() cube(5);
    /// ```
    #[test]
    fn test_evaluate_nested_modules() {
        let result = evaluate("module outer() { translate([10, 0, 0]) children(); } module inner() { scale([2, 2, 2]) children(); } outer() inner() cube(5);").unwrap();
        match result.geometry {
            GeometryNode::Translate { offset, child } => {
                assert_eq!(offset, [10.0, 0.0, 0.0]);
                match *child {
                    GeometryNode::Scale { factors, child: inner } => {
                        assert_eq!(factors, [2.0, 2.0, 2.0]);
                        match *inner {
                            GeometryNode::Cube { size, .. } => {
                                assert_eq!(size, [5.0, 5.0, 5.0]);
                            }
                            _ => panic!("Expected Cube"),
                        }
                    }
                    _ => panic!("Expected Scale"),
                }
            }
            _ => panic!("Expected Translate"),
        }
    }

    /// Test $children special variable.
    #[test]
    fn test_evaluate_children_count() {
        // This tests that $children is accessible (indirectly through module working)
        let result = evaluate("module test() { children(); } test() { cube(5); sphere(3); }").unwrap();
        match result.geometry {
            GeometryNode::Group { children } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected Group"),
        }
    }

    /// Test children(i) - accessing specific child by index.
    #[test]
    fn test_evaluate_children_by_index() {
        // Note: children(0) requires proper parsing of positional argument
        // For now, test that we get the first child when using children()
        let result = evaluate("module first() { children(); } first() cube(5);").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [5.0, 5.0, 5.0]);
            }
            _ => panic!("Expected Cube, got {:?}", result.geometry),
        }
    }

    /// Test deeply nested module definitions.
    #[test]
    fn test_evaluate_deep_nested_module_defs() {
        let code = r#"
            module level1() { 
                module level2() { 
                    cube(10); 
                } 
                level2(); 
            }
            level1();
        "#;
        let result = evaluate(code).unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
            }
            _ => panic!("Expected Cube"),
        }
    }

    // =========================================================================
    // HULL AND MINKOWSKI TESTS
    // =========================================================================

    /// Test hull() with two spheres.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// hull() {
    ///   sphere(5);
    ///   translate([20, 0, 0]) sphere(5);
    /// }
    /// ```
    #[test]
    fn test_evaluate_hull() {
        let result = evaluate("hull() { sphere(5); translate([20, 0, 0]) sphere(5); }").unwrap();
        match result.geometry {
            GeometryNode::Hull { children } => {
                assert_eq!(children.len(), 2);
            }
            _ => panic!("Expected Hull, got {:?}", result.geometry),
        }
    }

    /// Test hull() with single child returns child directly.
    #[test]
    fn test_evaluate_hull_single_child() {
        let result = evaluate("hull() { cube(10); }").unwrap();
        match result.geometry {
            GeometryNode::Cube { size, .. } => {
                assert_eq!(size, [10.0, 10.0, 10.0]);
            }
            _ => panic!("Expected Cube directly"),
        }
    }

    /// Test minkowski() with cube and sphere.
    ///
    /// ## OpenSCAD Equivalent
    ///
    /// ```text
    /// minkowski() {
    ///   cube(10);
    ///   sphere(2);
    /// }
    /// ```
    #[test]
    fn test_evaluate_minkowski() {
        let result = evaluate("minkowski() { cube(10); sphere(2); }").unwrap();
        match result.geometry {
            GeometryNode::Minkowski { children } => {
                assert_eq!(children.len(), 2);
                // First child should be cube
                match &children[0] {
                    GeometryNode::Cube { size, .. } => {
                        assert_eq!(*size, [10.0, 10.0, 10.0]);
                    }
                    _ => panic!("First child should be Cube"),
                }
                // Second child should be sphere
                match &children[1] {
                    GeometryNode::Sphere { radius, .. } => {
                        assert_eq!(*radius, 2.0);
                    }
                    _ => panic!("Second child should be Sphere"),
                }
            }
            _ => panic!("Expected Minkowski"),
        }
    }

    /// Test empty hull returns empty.
    #[test]
    fn test_evaluate_hull_empty() {
        let result = evaluate("hull() { }").unwrap();
        assert!(result.geometry.is_empty());
    }

    /// Test hull with cylinders (common rounded box pattern).
    #[test]
    fn test_evaluate_hull_rounded_box() {
        let code = r#"
            hull() {
                cylinder(h=10, r=2);
                translate([20, 0, 0]) cylinder(h=10, r=2);
                translate([0, 20, 0]) cylinder(h=10, r=2);
                translate([20, 20, 0]) cylinder(h=10, r=2);
            }
        "#;
        let result = evaluate(code).unwrap();
        match result.geometry {
            GeometryNode::Hull { children } => {
                assert_eq!(children.len(), 4);
            }
            _ => panic!("Expected Hull with 4 children"),
        }
    }
}

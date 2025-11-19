use openscad_ast::{self, Geometry, Diagnostic};

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluatedCube {
    pub size: [f64; 3],
    pub center: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvaluatedGeometry {
    Cube(EvaluatedCube),
}

pub fn evaluate(source: &str) -> Result<EvaluatedGeometry, Vec<Diagnostic>> {
    let ast = openscad_ast::parse(source)?;
    
    match ast {
        Geometry::Cube(cube) => Ok(EvaluatedGeometry::Cube(EvaluatedCube {
            size: cube.size,
            center: cube.center,
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_cube() {
        let source = "cube(10);";
        let result = evaluate(source);
        assert!(result.is_ok());
        match result.unwrap() {
            EvaluatedGeometry::Cube(cube) => {
                assert_eq!(cube.size, [10.0, 10.0, 10.0]);
                assert_eq!(cube.center, false);
            }
        }
    }
}

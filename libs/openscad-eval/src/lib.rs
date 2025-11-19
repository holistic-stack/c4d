use openscad_ast::{self, Geometry, build_ast_from_source, AstError};
use pipeline_types::{Diagnostic, Span, Severity};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct EvaluatedCube {
    pub size: [f64; 3],
    pub center: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvaluatedGeometry {
    Cube(EvaluatedCube),
}

pub struct EvaluationContext {
    pub fn_: Option<u32>,
    pub fa: Option<f64>,
    pub fs: Option<f64>,
}

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("evaluation failed: {message}")]
    Eval { message: String, span: Option<Span>, file: Option<String>, #[source] source: Option<AstError> },
}

pub fn evaluate_ast(ast: &Geometry, _ctx: &EvaluationContext) -> Result<EvaluatedGeometry, Vec<Diagnostic>> {
    match ast {
        Geometry::Cube(cube) => Ok(EvaluatedGeometry::Cube(EvaluatedCube {
            size: cube.size,
            center: cube.center,
        })),
    }
}

pub fn evaluate(source: &str) -> Result<EvaluatedGeometry, Vec<Diagnostic>> {
    let ast = build_ast_from_source(source, None).map_err(|e| vec![Diagnostic { severity: Severity::Error, message: format!("{e}"), span: Span { start: 0, end: source.len() }, hint: None }])?;
    let ctx = EvaluationContext { fn_: None, fa: None, fs: None };
    evaluate_ast(&ast, &ctx)
}

pub fn parse_only(source: &str) -> Result<(), EvalError> {
    match build_ast_from_source(source, None) {
        Ok(_) => Ok(()),
        Err(e) => Err(EvalError::Eval { message: "parse/ast failed".to_string(), span: Some(Span { start: 0, end: source.len() }), file: None, source: Some(e) }),
    }
}

pub fn parse_and_evaluate(source: &str, file: Option<&str>, ctx: &EvaluationContext) -> Result<EvaluatedGeometry, EvalError> {
    let ast = build_ast_from_source(source, file).map_err(|e| EvalError::Eval { message: "ast build failed".to_string(), span: Some(Span { start: 0, end: source.len() }), file: file.map(|f| f.to_string()), source: Some(e) })?;
    match evaluate_ast(&ast, ctx) {
        Ok(geom) => Ok(geom),
        Err(diags) => Err(EvalError::Eval { message: diags.first().map(|d| d.message.clone()).unwrap_or_else(|| "evaluation error".to_string()), span: diags.first().map(|d| d.span), file: None, source: None })
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

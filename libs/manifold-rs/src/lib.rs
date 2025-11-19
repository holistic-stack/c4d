//! Manifold geometry kernel for Rust OpenSCAD pipeline
//!
//! This crate provides the core geometry kernel with index-based half-edge
//! mesh representation, following the C++ Manifold library architecture.

pub mod config;
pub mod core;
pub mod primitives;

pub use config::*;
pub use core::*;
pub use primitives::*;

use openscad_eval::{evaluate, EvaluatedGeometry, EvalError, parse_and_evaluate, EvaluationContext};
use pipeline_types::{Diagnostic, TraceDiagnostic, Stage, Span};
use openscad_ast::{AstError, ParseError};

/// Processes OpenSCAD source code and returns a triangulated mesh as a flat vector of coordinates.
pub fn process_openscad(source: &str) -> Result<Vec<f64>, String> {
    let evaluated_geometry = evaluate(source).map_err(|diags| format!("Parse/Eval failed: {:?}", diags))?;

    match evaluated_geometry {
        EvaluatedGeometry::Cube(cube_params) => {
            let center = if cube_params.center {
                Vec3::new(0.0, 0.0, 0.0)
            } else {
                Vec3::new(
                    cube_params.size[0] / 2.0,
                    cube_params.size[1] / 2.0,
                    cube_params.size[2] / 2.0,
                )
            };
            let size = Vec3::new(
                cube_params.size[0],
                cube_params.size[1],
                cube_params.size[2],
            );

            let cube = Cube::from_center_size(center, size);
            
            match cube.to_mesh() {
                Ok(mesh) => Ok(mesh.triangulate()),
                Err(e) => Err(format!("Error creating cube mesh: {:?}", e)),
            }
        }
    }
}

pub fn to_mesh(evaluated_geometry: &EvaluatedGeometry) -> Result<Vec<f64>, String> {
    match evaluated_geometry {
        EvaluatedGeometry::Cube(cube_params) => {
            let center = Vec3::new(0.0, 0.0, 0.0);
            let size = Vec3::new(
                cube_params.size[0],
                cube_params.size[1],
                cube_params.size[2],
            );
            let cube = Cube::from_center_size(center, size);
            match cube.to_mesh() {
                Ok(mesh) => Ok(mesh.triangulate()),
                Err(e) => Err(format!("Error creating cube mesh: {:?}", e)),
            }
        }
    }
}

// No parse-only public API; compilation errors surface diagnostics via error wrappers

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("kernel failed: {message}")]
    Kernel { message: String, span: Option<pipeline_types::Span>, file: Option<String>, #[source] source: Option<EvalError> },
}

pub fn compile(source: &str) -> Result<Vec<f64>, KernelError> {
    let ctx = EvaluationContext { fn_: None, fa: None, fs: None };
    let evaluated_geometry = parse_and_evaluate(source, None, &ctx).map_err(|e| KernelError::Kernel { message: "parse/eval failed".to_string(), span: Some(pipeline_types::Span { start: 0, end: source.len() }), file: None, source: Some(e) })?;
    to_mesh(&evaluated_geometry).map_err(|e| KernelError::Kernel { message: e, span: Some(pipeline_types::Span { start: 0, end: source.len() }), file: None, source: None })
}

pub fn error_to_trace(err: &KernelError, source_text: &str) -> Vec<TraceDiagnostic> {
    let mut chain: Vec<TraceDiagnostic> = Vec::new();
    match err {
        KernelError::Kernel { message, source, span: _, file: _ } => {
            let causes = source.as_ref().map(|e| vec![trace_eval(e, source_text)]);
            chain.push(TraceDiagnostic {
                stage: Stage::Kernel,
                message: message.clone(),
                span: Span { start: 0, end: source_text.len() },
                file: None,
                hint: None,
                causes,
            });
        }
    }
    chain
}

fn trace_eval(eval: &EvalError, source_text: &str) -> TraceDiagnostic {
    match eval {
        EvalError::Eval { message, source, span: _, file: _ } => {
            let causes = source.as_ref().map(|a| vec![trace_ast(a, source_text)]);
            TraceDiagnostic {
                stage: Stage::Eval,
                message: message.clone(),
                span: Span { start: 0, end: source_text.len() },
                file: None,
                hint: None,
                causes,
            }
        }
    }
}

fn trace_ast(ast: &AstError, _source_text: &str) -> TraceDiagnostic {
    match ast {
        AstError::Build { message, span, file, source } => {
            let causes = source.as_ref().map(|p| vec![trace_parse(p)]);
            TraceDiagnostic {
                stage: Stage::Ast,
                message: message.clone(),
                span: *span,
                file: file.clone(),
                hint: None,
                causes,
            }
        }
    }
}

fn trace_parse(parse: &ParseError) -> TraceDiagnostic {
    match parse {
        ParseError::Failed { span, file } => TraceDiagnostic {
            stage: Stage::Parse,
            message: "failed to parse source".to_string(),
            span: *span,
            file: file.clone(),
            hint: None,
            causes: None,
        },
        ParseError::Syntax { span, file } => TraceDiagnostic {
            stage: Stage::Parse,
            message: "syntax error in source code".to_string(),
            span: *span,
            file: file.clone(),
            hint: None,
            causes: None,
        },
    }
}

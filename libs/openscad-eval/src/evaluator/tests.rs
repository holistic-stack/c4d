//! Evaluator tests guaranteeing deterministic behavior.

use super::*;
use crate::filesystem::InMemoryFilesystem;
use glam::DVec3;

/// Confirms `evaluate_source` parses cube primitives.
///
/// # Examples
/// ```
/// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem};
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// assert_eq!(evaluator.evaluate_source("cube(1);").unwrap().len(), 1);
/// ```
#[test]
fn evaluate_cube_literal() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator.evaluate_source("cube(2);").expect("cube parsed");
    assert_eq!(nodes.len(), 1);
}

/// Confirms `evaluate_source` correctly handles vector cube arguments.
///
/// # Examples
/// ```rust
/// use glam::DVec3;
/// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem};
///
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// let nodes = evaluator.evaluate_source("cube([1, 2, 3]);").unwrap();
/// assert_eq!(nodes[0].size(), DVec3::new(1.0, 2.0, 3.0));
/// ```
#[test]
fn evaluate_cube_vector_literal() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let nodes = evaluator
        .evaluate_source("cube([1, 2, 3]);")
        .expect("cube parsed");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].size(), DVec3::new(1.0, 2.0, 3.0));
}

/// Ensures unsupported snippets raise explicit errors.
///
/// # Examples
/// ```
/// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem};
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// assert!(evaluator.evaluate_source("sphere(1);").is_err());
/// ```
#[test]
fn evaluate_unsupported_source() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let err = evaluator.evaluate_source("sphere(1);").unwrap_err();
    assert!(matches!(err, EvaluationError::UnsupportedSource { .. }));
}

/// Ensures AST diagnostics are surfaced as explicit evaluator errors.
///
/// # Examples
/// ```rust
/// use openscad_eval::{evaluator::Evaluator, filesystem::InMemoryFilesystem, EvaluationError};
///
/// let evaluator = Evaluator::new(InMemoryFilesystem::default());
/// let result = evaluator.evaluate_source("cube([1, 2]);");
/// assert!(matches!(result, Err(EvaluationError::AstDiagnostics(_))));
/// ```
#[test]
fn evaluate_invalid_cube_vector_reports_ast_error() {
    let evaluator = Evaluator::new(InMemoryFilesystem::default());
    let err = evaluator
        .evaluate_source("cube([1, 2]);")
        .expect_err("evaluation should fail");

    if let EvaluationError::AstDiagnostics(diagnostics) = err {
        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message().contains("3 elements"));
    } else {
        panic!("expected AST diagnostics error");
    }
}

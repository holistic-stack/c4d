//! Tests for EvaluationContext.

use super::*;
use config::constants::{DEFAULT_FA, DEFAULT_FN, DEFAULT_FS};

#[test]
fn context_initialization_defaults() {
    let ctx = EvaluationContext::default();
    assert_eq!(ctx.get_fn(), DEFAULT_FN);
    assert_eq!(ctx.get_fa(), DEFAULT_FA);
    assert_eq!(ctx.get_fs(), DEFAULT_FS);
}

#[test]
fn context_set_special_variables() {
    let mut ctx = EvaluationContext::default();
    ctx.set_fn(100);
    ctx.set_fa(5.0);
    ctx.set_fs(1.0);

    assert_eq!(ctx.get_fn(), 100);
    assert_eq!(ctx.get_fa(), 5.0);
    assert_eq!(ctx.get_fs(), 1.0);
}

#[test]
fn context_set_variable_generic() {
    let mut ctx = EvaluationContext::default();
    ctx.set_variable("$fn", 50.0);
    ctx.set_variable("$fa", 10.0);
    ctx.set_variable("$fs", 0.5);
    ctx.set_variable("my_var", 42.0);

    assert_eq!(ctx.get_fn(), 50);
    assert_eq!(ctx.get_fa(), 10.0);
    assert_eq!(ctx.get_fs(), 0.5);
    assert_eq!(ctx.get_variable("my_var"), Some(42.0));
}

#[test]
fn context_get_variable_generic() {
    let mut ctx = EvaluationContext::default();
    ctx.set_fn(100);

    assert_eq!(ctx.get_variable("$fn"), Some(100.0));
}

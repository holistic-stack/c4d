//! Evaluation context holding variable states and resolution settings.
//!
//! This module defines `EvaluationContext`, which tracks global variables like
//! `$fn`, `$fa`, and `$fs` used for controlling tessellation quality.

use config::constants::{DEFAULT_FA, DEFAULT_FN, DEFAULT_FS};
use std::collections::HashMap;

/// Holds the current state of variables during evaluation.
///
/// # Examples
/// ```
/// use openscad_eval::evaluator::context::EvaluationContext;
/// let mut ctx = EvaluationContext::default();
/// assert_eq!(ctx.get_fn(), 0);
/// ctx.set_fn(50);
/// assert_eq!(ctx.get_fn(), 50);
/// ```
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    variables: HashMap<String, f64>,
    // Special resolution variables
    fn_val: u32,
    fa_val: f64,
    fs_val: f64,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            fn_val: DEFAULT_FN,
            fa_val: DEFAULT_FA,
            fs_val: DEFAULT_FS,
        }
    }
}

impl EvaluationContext {
    /// Creates a new empty evaluation context with default special variables.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current value of `$fn` (number of fragments).
    pub fn get_fn(&self) -> u32 {
        self.fn_val
    }

    /// Returns the current value of `$fa` (minimum angle).
    pub fn get_fa(&self) -> f64 {
        self.fa_val
    }

    /// Returns the current value of `$fs` (minimum size).
    pub fn get_fs(&self) -> f64 {
        self.fs_val
    }

    /// Sets the value of `$fn`.
    pub fn set_fn(&mut self, value: u32) {
        self.fn_val = value;
    }

    /// Sets the value of `$fa`.
    pub fn set_fa(&mut self, value: f64) {
        self.fa_val = value;
    }

    /// Sets the value of `$fs`.
    pub fn set_fs(&mut self, value: f64) {
        self.fs_val = value;
    }

    // Placeholder for generic variable handling
    pub fn set_variable(&mut self, name: &str, value: f64) {
        match name {
            "$fn" => self.fn_val = value as u32,
            "$fa" => self.fa_val = value,
            "$fs" => self.fs_val = value,
            _ => {
                self.variables.insert(name.to_string(), value);
            }
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<f64> {
        match name {
            "$fn" => Some(self.fn_val as f64),
            "$fa" => Some(self.fa_val),
            "$fs" => Some(self.fs_val),
            _ => self.variables.get(name).copied(),
        }
    }
}

#[cfg(test)]
#[path = "tests_context.rs"]
mod tests;

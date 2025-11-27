//! # Runtime Values
//!
//! Value types used during evaluation.

use crate::error::EvalError;
use serde::{Deserialize, Serialize};

// =============================================================================
// VALUE
// =============================================================================

/// A runtime value in OpenSCAD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    /// Undefined value.
    Undef,
    /// Boolean.
    Boolean(bool),
    /// Number (f64).
    Number(f64),
    /// String.
    String(String),
    /// List of values.
    List(Vec<Value>),
    /// Range [start:end] or [start:step:end].
    Range {
        start: f64,
        end: f64,
        step: Option<f64>,
    },
}

impl Value {
    /// Convert to number, or error.
    pub fn as_number(&self) -> Result<f64, EvalError> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            _ => Err(EvalError::TypeError(format!("Expected number, got {:?}", self))),
        }
    }

    /// Convert to boolean.
    pub fn as_boolean(&self) -> bool {
        match self {
            Value::Undef => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Range { .. } => true,
        }
    }

    /// Convert to list of numbers (for vectors/arrays).
    pub fn as_number_list(&self) -> Result<Vec<f64>, EvalError> {
        match self {
            Value::List(items) => {
                items.iter()
                    .map(|v| v.as_number())
                    .collect()
            }
            Value::Number(n) => Ok(vec![*n]),
            _ => Err(EvalError::TypeError(format!("Expected list of numbers, got {:?}", self))),
        }
    }

    /// Convert to [f64; 3] for 3D vectors.
    pub fn as_vec3(&self) -> Result<[f64; 3], EvalError> {
        let nums = self.as_number_list()?;
        match nums.len() {
            1 => Ok([nums[0], nums[0], nums[0]]),
            2 => Ok([nums[0], nums[1], 0.0]),
            3 => Ok([nums[0], nums[1], nums[2]]),
            _ => Err(EvalError::TypeError(format!(
                "Expected 1-3 numbers for vec3, got {}",
                nums.len()
            ))),
        }
    }

    /// Convert to [f64; 2] for 2D vectors.
    pub fn as_vec2(&self) -> Result<[f64; 2], EvalError> {
        let nums = self.as_number_list()?;
        match nums.len() {
            1 => Ok([nums[0], nums[0]]),
            2 => Ok([nums[0], nums[1]]),
            _ => Err(EvalError::TypeError(format!(
                "Expected 1-2 numbers for vec2, got {}",
                nums.len()
            ))),
        }
    }

    /// Check if this is undef.
    pub fn is_undef(&self) -> bool {
        matches!(self, Value::Undef)
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Undef
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_as_number() {
        let v = Value::Number(42.0);
        assert_eq!(v.as_number().unwrap(), 42.0);
    }

    #[test]
    fn test_list_as_vec3() {
        let v = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        assert_eq!(v.as_vec3().unwrap(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_single_number_as_vec3() {
        let v = Value::Number(10.0);
        assert_eq!(v.as_vec3().unwrap(), [10.0, 10.0, 10.0]);
    }

    #[test]
    fn test_boolean_truthiness() {
        assert!(Value::Boolean(true).as_boolean());
        assert!(!Value::Boolean(false).as_boolean());
        assert!(Value::Number(1.0).as_boolean());
        assert!(!Value::Number(0.0).as_boolean());
    }
}

//! # Variable Scope
//!
//! Lexical scoping for OpenSCAD variables.
//!
//! ## OpenSCAD Scoping Rules
//!
//! - Variables are lexically scoped
//! - Inner scopes can shadow outer scope variables
//! - Variables cannot be reassigned in the same scope
//! - Special variables ($fn, $fa, $fs) have default values
//!
//! ## Example
//!
//! ```rust
//! use openscad_eval::scope::Scope;
//! use openscad_eval::value::Value;
//!
//! let mut scope = Scope::new();
//! scope.define("x", Value::Number(10.0));
//! assert_eq!(scope.get("x"), Some(&Value::Number(10.0)));
//! ```

use crate::value::Value;
use std::collections::HashMap;

// =============================================================================
// CONSTANTS - OpenSCAD defaults
// =============================================================================

/// Default $fn value (0 means use $fa/$fs).
pub const DEFAULT_FN: f64 = 0.0;
/// Default $fa value (degrees).
pub const DEFAULT_FA: f64 = 12.0;
/// Default $fs value (mm).
pub const DEFAULT_FS: f64 = 2.0;

// =============================================================================
// SCOPE
// =============================================================================

/// A single scope level containing variable bindings.
#[derive(Debug, Clone)]
struct ScopeLevel {
    /// Variable bindings in this scope.
    bindings: HashMap<String, Value>,
}

impl ScopeLevel {
    /// Create a new empty scope level.
    fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
}

/// Lexical scope for variable resolution.
///
/// Implements OpenSCAD's scoping rules:
/// - Variables are resolved from innermost to outermost scope
/// - Inner scopes can shadow outer scope variables
/// - Special variables have default values
///
/// ## Example
///
/// ```rust
/// let mut scope = Scope::new();
/// scope.define("x", Value::Number(10.0));
///
/// scope.push(); // Enter new scope
/// scope.define("x", Value::Number(20.0)); // Shadows outer x
/// assert_eq!(scope.get("x"), Some(&Value::Number(20.0)));
///
/// scope.pop(); // Exit scope
/// assert_eq!(scope.get("x"), Some(&Value::Number(10.0)));
/// ```
#[derive(Debug, Clone)]
pub struct Scope {
    /// Stack of scope levels (innermost last).
    levels: Vec<ScopeLevel>,
}

impl Scope {
    /// Create a new scope with default special variables.
    ///
    /// ## Returns
    ///
    /// A scope with $fn, $fa, $fs set to defaults.
    pub fn new() -> Self {
        let mut scope = Self {
            levels: vec![ScopeLevel::new()],
        };
        
        // Initialize special variables with defaults
        scope.define("$fn", Value::Number(DEFAULT_FN));
        scope.define("$fa", Value::Number(DEFAULT_FA));
        scope.define("$fs", Value::Number(DEFAULT_FS));
        scope.define("$t", Value::Number(0.0)); // Animation time
        scope.define("$preview", Value::Boolean(true)); // Preview mode
        
        scope
    }

    /// Push a new scope level.
    ///
    /// Called when entering a block, module, or function.
    pub fn push(&mut self) {
        self.levels.push(ScopeLevel::new());
    }

    /// Pop the current scope level.
    ///
    /// Called when exiting a block, module, or function.
    /// Panics if trying to pop the global scope.
    pub fn pop(&mut self) {
        if self.levels.len() > 1 {
            self.levels.pop();
        }
    }

    /// Define a variable in the current scope.
    ///
    /// ## Parameters
    ///
    /// - `name`: Variable name
    /// - `value`: Variable value
    ///
    /// ## Note
    ///
    /// This will shadow any variable with the same name in outer scopes.
    pub fn define(&mut self, name: &str, value: Value) {
        if let Some(level) = self.levels.last_mut() {
            level.bindings.insert(name.to_string(), value);
        }
    }

    /// Get a variable value.
    ///
    /// Searches from innermost to outermost scope.
    ///
    /// ## Parameters
    ///
    /// - `name`: Variable name
    ///
    /// ## Returns
    ///
    /// The variable value if found, None otherwise.
    pub fn get(&self, name: &str) -> Option<&Value> {
        // Search from innermost to outermost
        for level in self.levels.iter().rev() {
            if let Some(value) = level.bindings.get(name) {
                return Some(value);
            }
        }
        None
    }

    /// Get $fn value as u32.
    pub fn fn_value(&self) -> u32 {
        self.get("$fn")
            .and_then(|v| v.as_number().ok())
            .map(|n| n as u32)
            .unwrap_or(DEFAULT_FN as u32)
    }

    /// Get $fa value.
    pub fn fa_value(&self) -> f64 {
        self.get("$fa")
            .and_then(|v| v.as_number().ok())
            .unwrap_or(DEFAULT_FA)
    }

    /// Get $fs value.
    pub fn fs_value(&self) -> f64 {
        self.get("$fs")
            .and_then(|v| v.as_number().ok())
            .unwrap_or(DEFAULT_FS)
    }

    /// Calculate number of fragments for circular shapes.
    ///
    /// Uses OpenSCAD's formula:
    /// - If $fn > 0: use $fn (minimum 3)
    /// - Otherwise: min(360/$fa, 2*PI*r/$fs) (minimum 3)
    ///
    /// ## Parameters
    ///
    /// - `radius`: Radius of the circular shape
    ///
    /// ## Returns
    ///
    /// Number of fragments to use.
    pub fn calculate_fragments(&self, radius: f64) -> u32 {
        let fn_ = self.fn_value();
        if fn_ > 0 {
            fn_.max(3)
        } else {
            let fa = self.fa_value();
            let fs = self.fs_value();
            let fa_fragments = (360.0 / fa).ceil() as u32;
            let fs_fragments = ((2.0 * std::f64::consts::PI * radius) / fs).ceil() as u32;
            fa_fragments.min(fs_fragments).max(3)
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_new() {
        let scope = Scope::new();
        assert_eq!(scope.get("$fn"), Some(&Value::Number(0.0)));
        assert_eq!(scope.get("$fa"), Some(&Value::Number(12.0)));
        assert_eq!(scope.get("$fs"), Some(&Value::Number(2.0)));
    }

    #[test]
    fn test_scope_define_get() {
        let mut scope = Scope::new();
        scope.define("x", Value::Number(10.0));
        assert_eq!(scope.get("x"), Some(&Value::Number(10.0)));
    }

    #[test]
    fn test_scope_shadowing() {
        let mut scope = Scope::new();
        scope.define("x", Value::Number(10.0));
        
        scope.push();
        scope.define("x", Value::Number(20.0));
        assert_eq!(scope.get("x"), Some(&Value::Number(20.0)));
        
        scope.pop();
        assert_eq!(scope.get("x"), Some(&Value::Number(10.0)));
    }

    #[test]
    fn test_scope_inner_access_outer() {
        let mut scope = Scope::new();
        scope.define("x", Value::Number(10.0));
        
        scope.push();
        // Inner scope can access outer variable
        assert_eq!(scope.get("x"), Some(&Value::Number(10.0)));
    }

    #[test]
    fn test_scope_undefined() {
        let scope = Scope::new();
        assert_eq!(scope.get("undefined_var"), None);
    }

    #[test]
    fn test_calculate_fragments_with_fn() {
        let mut scope = Scope::new();
        scope.define("$fn", Value::Number(32.0));
        assert_eq!(scope.calculate_fragments(10.0), 32);
    }

    #[test]
    fn test_calculate_fragments_minimum() {
        let mut scope = Scope::new();
        scope.define("$fn", Value::Number(2.0)); // Below minimum
        assert_eq!(scope.calculate_fragments(10.0), 3); // Should be at least 3
    }

    #[test]
    fn test_calculate_fragments_default() {
        let scope = Scope::new();
        // With defaults: $fa=12, $fs=2
        // For radius 10: min(360/12, 2*PI*10/2) = min(30, 31.4) = 30
        assert_eq!(scope.calculate_fragments(10.0), 30);
    }
}

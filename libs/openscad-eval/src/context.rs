//! # Evaluation Context
//!
//! Tracks variables, scopes, and special variables ($fn, $fa, $fs, $vpr, etc.) during evaluation.
//!
//! ## OpenSCAD Special Variables
//!
//! | Variable | Type | Description | Default |
//! |----------|------|-------------|---------|
//! | `$fn` | f64 | Fragment count override | 0.0 |
//! | `$fa` | f64 | Minimum fragment angle (degrees) | 12.0 |
//! | `$fs` | f64 | Minimum fragment size | 2.0 |
//! | `$t` | f64 | Animation time (0.0 to 1.0) | 0.0 |
//! | `$children` | usize | Number of children in module scope | 0 |
//! | `$preview` | bool | True in preview mode (F5), false in render (F6) | true |
//! | `$vpr` | [f64; 3] | Viewport rotation (Euler angles) | [55, 0, 25] |
//! | `$vpt` | [f64; 3] | Viewport translation | [0, 0, 0] |
//! | `$vpd` | f64 | Viewport camera distance | 140.0 |
//! | `$vpf` | f64 | Viewport field of view (degrees) | 22.5 |

use crate::value::Value;
use config::constants::{DEFAULT_FA, DEFAULT_FN, DEFAULT_FS};
use openscad_ast::{Expression, Statement};
use std::collections::HashMap;

/// Default viewport rotation: [55, 0, 25] degrees (OpenSCAD standard)
pub const DEFAULT_VPR: [f64; 3] = [55.0, 0.0, 25.0];

/// Default viewport translation: origin
pub const DEFAULT_VPT: [f64; 3] = [0.0, 0.0, 0.0];

/// Default viewport camera distance
pub const DEFAULT_VPD: f64 = 140.0;

/// Default viewport field of view (degrees)
pub const DEFAULT_VPF: f64 = 22.5;

/// Default preview mode (true = preview/F5, false = render/F6)
/// In browser playground, we're always in preview mode.
pub const DEFAULT_PREVIEW: bool = true;

/// Maximum recursion depth for module calls to prevent stack overflow
pub const MAX_RECURSION_DEPTH: usize = 2000;

/// Evaluation context for OpenSCAD programs.
///
/// Tracks:
/// - User-defined variables
/// - Special variables ($fn, $fa, $fs)
/// - Scope stack for nested blocks
/// - Recursion depth
///
/// # Example
///
/// ```rust
/// use openscad_eval::EvaluationContext;
///
/// let mut ctx = EvaluationContext::new();
/// ctx.set_fn(32.0);
/// assert_eq!(ctx.fn_value(), 32.0);
/// ```
/// Stored module definition for user-defined modules.
#[derive(Debug, Clone)]
pub struct ModuleDefinition {
    /// Parameter names with optional default values (expressions to be evaluated at call time)
    pub parameters: Vec<(String, Option<Expression>)>,
    /// Module body statements
    pub body: Vec<Statement>,
}

/// Stored function definition for user-defined functions.
/// Functions return a value computed from an expression.
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    /// Parameter names with optional default values (expressions to be evaluated at call time)
    pub parameters: Vec<(String, Option<Expression>)>,
    /// The expression that computes the return value
    pub body: Expression,
}

/// A single scope frame containing variables and optional children reference.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub variables: HashMap<String, Value>,
    /// Index into children_stack for the module associated with this scope.
    /// None if this scope is not a module scope (e.g. for loop, or global).
    pub children_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Stack of variable scopes
    scopes: Vec<Scope>,
    /// User-defined modules
    modules: HashMap<String, ModuleDefinition>,
    /// User-defined functions
    functions: HashMap<String, FunctionDefinition>,
    /// Current recursion depth
    pub recursion_depth: usize,
    /// Maximum recursion depth allowed
    pub max_recursion_depth: usize,
    /// Stack of module names being called for error reporting
    pub call_stack: Vec<String>,
    /// Stack of children for module calls: (Statements, ScopeDepth)
    pub children_stack: Vec<(Vec<Statement>, usize)>,
    /// $fn - fragment count override
    fn_value: f64,
    /// $fa - minimum fragment angle (degrees)
    fa_value: f64,
    /// $fs - minimum fragment size
    fs_value: f64,
    /// $t - animation time (0.0 to 1.0)
    t_value: f64,
    /// $children - number of child modules in current scope
    children_count: usize,
    /// $vpr - viewport rotation (Euler angles in degrees)
    vpr_value: [f64; 3],
    /// $vpt - viewport translation
    vpt_value: [f64; 3],
    /// $vpd - viewport camera distance (zoom level)
    vpd_value: f64,
    /// $vpf - viewport field of view (degrees)
    vpf_value: f64,
    /// $preview - true in preview mode (F5), false in render mode (F6)
    preview_value: bool,
}

impl Default for EvaluationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl EvaluationContext {
    /// Creates a new evaluation context with default values.
    ///
    /// Default values match OpenSCAD:
    /// - $fn = 0 (use $fa/$fs calculation)
    /// - $fa = 12 degrees
    /// - $fs = 2 units
    /// - $vpr = [55, 0, 25] (viewport rotation)
    /// - $vpt = [0, 0, 0] (viewport translation)
    /// - $vpd = 140 (viewport distance)
    /// - $vpf = 22.5 (viewport field of view)
    /// - $preview = true (preview mode)
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
            modules: HashMap::new(),
            functions: HashMap::new(),
            recursion_depth: 0,
            max_recursion_depth: MAX_RECURSION_DEPTH,
            call_stack: Vec::new(),
            children_stack: Vec::new(),
            fn_value: DEFAULT_FN,
            fa_value: DEFAULT_FA,
            fs_value: DEFAULT_FS,
            t_value: 0.0,
            children_count: 0,
            vpr_value: DEFAULT_VPR,
            vpt_value: DEFAULT_VPT,
            vpd_value: DEFAULT_VPD,
            vpf_value: DEFAULT_VPF,
            preview_value: DEFAULT_PREVIEW,
        }
    }

    /// Registers a user-defined module.
    pub fn register_module(&mut self, name: String, definition: ModuleDefinition) {
        self.modules.insert(name, definition);
    }

    /// Gets a user-defined module by name.
    pub fn get_module(&self, name: &str) -> Option<&ModuleDefinition> {
        self.modules.get(name)
    }

    /// Returns a list of registered module names.
    pub fn module_names(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    /// Registers a user-defined function.
    pub fn register_function(&mut self, name: String, definition: FunctionDefinition) {
        self.functions.insert(name, definition);
    }

    /// Gets a user-defined function by name.
    pub fn get_function(&self, name: &str) -> Option<&FunctionDefinition> {
        self.functions.get(name)
    }

    /// Returns the current $t value (animation time 0.0 to 1.0).
    #[inline]
    pub fn t_value(&self) -> f64 {
        self.t_value
    }

    /// Sets the $t value for animation.
    pub fn set_t(&mut self, value: f64) {
        self.t_value = value.clamp(0.0, 1.0);
    }

    /// Returns the current $children count.
    #[inline]
    pub fn children_count(&self) -> usize {
        self.children_count
    }

    /// Sets the $children count for module invocation.
    pub fn set_children_count(&mut self, count: usize) {
        self.children_count = count;
    }

    /// Returns the current $fn value.
    #[inline]
    pub fn fn_value(&self) -> f64 {
        self.fn_value
    }

    /// Returns the current $fa value.
    #[inline]
    pub fn fa_value(&self) -> f64 {
        self.fa_value
    }

    /// Returns the current $fs value.
    #[inline]
    pub fn fs_value(&self) -> f64 {
        self.fs_value
    }

    /// Sets the $fn value.
    pub fn set_fn(&mut self, value: f64) {
        self.fn_value = value;
    }

    /// Sets the $fa value.
    pub fn set_fa(&mut self, value: f64) {
        self.fa_value = value;
    }

    /// Sets the $fs value.
    pub fn set_fs(&mut self, value: f64) {
        self.fs_value = value;
    }

    /// Returns the current $vpr value (viewport rotation).
    /// 
    /// # Returns
    /// 
    /// A 3-element array of Euler angles in degrees [rx, ry, rz].
    #[inline]
    pub fn vpr_value(&self) -> [f64; 3] {
        self.vpr_value
    }

    /// Returns the current $vpt value (viewport translation).
    /// 
    /// # Returns
    /// 
    /// A 3-element array [x, y, z] representing camera position offset.
    #[inline]
    pub fn vpt_value(&self) -> [f64; 3] {
        self.vpt_value
    }

    /// Returns the current $vpd value (viewport camera distance).
    #[inline]
    pub fn vpd_value(&self) -> f64 {
        self.vpd_value
    }

    /// Returns the current $vpf value (viewport field of view in degrees).
    #[inline]
    pub fn vpf_value(&self) -> f64 {
        self.vpf_value
    }

    /// Sets the $vpr value (viewport rotation).
    pub fn set_vpr(&mut self, value: [f64; 3]) {
        self.vpr_value = value;
    }

    /// Sets the $vpt value (viewport translation).
    pub fn set_vpt(&mut self, value: [f64; 3]) {
        self.vpt_value = value;
    }

    /// Sets the $vpd value (viewport camera distance).
    pub fn set_vpd(&mut self, value: f64) {
        self.vpd_value = value;
    }

    /// Sets the $vpf value (viewport field of view).
    pub fn set_vpf(&mut self, value: f64) {
        self.vpf_value = value;
    }

    /// Returns the current $preview value.
    /// 
    /// # Returns
    /// 
    /// `true` if in preview mode (F5), `false` if in render mode (F6).
    #[inline]
    pub fn preview_value(&self) -> bool {
        self.preview_value
    }

    /// Sets the $preview value.
    /// 
    /// # Arguments
    /// 
    /// * `value` - `true` for preview mode, `false` for render mode
    pub fn set_preview(&mut self, value: bool) {
        self.preview_value = value;
    }

    /// Sets a variable in the current scope.
    /// Special variables ($fn, $fa, $fs, $vpd, $vpf) are stored separately if passed as numbers.
    /// But general variables are stored as Value.
    pub fn set_variable(&mut self, name: &str, value: Value) {
        // Handle scalar special variables
        // We try to extract f64 for special variables that require it
        match name {
            "$fn" => if let Some(v) = value.as_f64() { self.fn_value = v; },
            "$fa" => if let Some(v) = value.as_f64() { self.fa_value = v; },
            "$fs" => if let Some(v) = value.as_f64() { self.fs_value = v; },
            "$vpd" => if let Some(v) = value.as_f64() { self.vpd_value = v; },
            "$vpf" => if let Some(v) = value.as_f64() { self.vpf_value = v; },
            _ => {
                if let Some(scope) = self.scopes.last_mut() {
                    scope.variables.insert(name.to_string(), value);
                }
            }
        }
    }

    /// Gets a variable value, searching from innermost to outermost scope.
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        // Handle special variables
        match name {
            "$fn" => return Some(Value::Number(self.fn_value)),
            "$fa" => return Some(Value::Number(self.fa_value)),
            "$fs" => return Some(Value::Number(self.fs_value)),
            "$t" => return Some(Value::Number(self.t_value)),
            "$children" => return Some(Value::Number(self.get_children_count() as f64)),
            "$vpd" => return Some(Value::Number(self.vpd_value)),
            "$vpf" => return Some(Value::Number(self.vpf_value)),
            "$preview" => return Some(Value::Boolean(self.preview_value)),
            "$vpr" => return Some(Value::Vector(vec![
                Value::Number(self.vpr_value[0]),
                Value::Number(self.vpr_value[1]),
                Value::Number(self.vpr_value[2])
            ])),
            "$vpt" => return Some(Value::Vector(vec![
                Value::Number(self.vpt_value[0]),
                Value::Number(self.vpt_value[1]),
                Value::Number(self.vpt_value[2])
            ])),
            _ => {}
        }

        // Search scopes from innermost to outermost
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.variables.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Pushes a new scope onto the stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    /// Pushes a new module scope with associated children index.
    pub fn push_module_scope(&mut self, children_index: Option<usize>) {
        self.scopes.push(Scope {
            variables: HashMap::new(),
            children_index,
        });
    }

    /// Temporarily removes scopes above a certain depth.
    /// Used for children evaluation where we want to simulate the call site scope.
    /// Returns the removed scopes.
    pub fn temporary_pop_scopes(&mut self, keep_depth: usize) -> Vec<Scope> {
        if keep_depth >= self.scopes.len() {
            return Vec::new();
        }
        self.scopes.split_off(keep_depth)
    }

    /// Restores previously popped scopes.
    pub fn restore_scopes(&mut self, mut scopes: Vec<Scope>) {
        self.scopes.append(&mut scopes);
    }

    /// Helper to get children count for the current context
    fn get_children_count(&self) -> usize {
        if let Some(idx) = self.get_current_children_index() {
             if let Some((children, _)) = self.children_stack.get(idx) {
                 return children.len();
             }
        }
        0
    }
    
    /// Finds the children index for the current scope context
    pub fn get_current_children_index(&self) -> Option<usize> {
        for scope in self.scopes.iter().rev() {
            if let Some(idx) = scope.children_index {
                return Some(idx);
            }
        }
        None
    }

    /// Pops the current scope from the stack.
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Returns the current scope depth.
    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let ctx = EvaluationContext::new();
        assert_eq!(ctx.fn_value(), DEFAULT_FN);
        assert_eq!(ctx.fa_value(), DEFAULT_FA);
        assert_eq!(ctx.fs_value(), DEFAULT_FS);
    }

    #[test]
    fn test_set_fn() {
        let mut ctx = EvaluationContext::new();
        ctx.set_fn(32.0);
        assert_eq!(ctx.fn_value(), 32.0);
    }

    #[test]
    fn test_set_variable_special() {
        let mut ctx = EvaluationContext::new();
        ctx.set_variable("$fn", Value::Number(64.0));
        assert_eq!(ctx.fn_value(), 64.0);
        assert_eq!(ctx.get_variable("$fn"), Some(Value::Number(64.0)));
    }

    #[test]
    fn test_set_variable_regular() {
        let mut ctx = EvaluationContext::new();
        ctx.set_variable("x", Value::Number(10.0));
        assert_eq!(ctx.get_variable("x"), Some(Value::Number(10.0)));
    }

    #[test]
    fn test_scope_push_pop() {
        let mut ctx = EvaluationContext::new();
        assert_eq!(ctx.scope_depth(), 1);
        
        ctx.push_scope();
        assert_eq!(ctx.scope_depth(), 2);
        
        ctx.pop_scope();
        assert_eq!(ctx.scope_depth(), 1);
    }

    #[test]
    fn test_scope_shadowing() {
        let mut ctx = EvaluationContext::new();
        ctx.set_variable("x", Value::Number(10.0));
        
        ctx.push_scope();
        ctx.set_variable("x", Value::Number(20.0));
        assert_eq!(ctx.get_variable("x"), Some(Value::Number(20.0)));
        
        ctx.pop_scope();
        assert_eq!(ctx.get_variable("x"), Some(Value::Number(10.0)));
    }

    #[test]
    fn test_scope_inheritance() {
        let mut ctx = EvaluationContext::new();
        ctx.set_variable("x", Value::Number(10.0));
        
        ctx.push_scope();
        // Inner scope can see outer scope variables
        assert_eq!(ctx.get_variable("x"), Some(Value::Number(10.0)));
    }
}

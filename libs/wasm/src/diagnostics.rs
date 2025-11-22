/// WASM-compatible diagnostic types.
///
/// This module provides JavaScript-compatible wrappers for Rust diagnostics.

use openscad_ast::{Diagnostic as RustDiagnostic, Severity as RustSeverity};
use wasm_bindgen::prelude::*;

/// Diagnostic severity for JavaScript.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl From<RustSeverity> for Severity {
    fn from(severity: RustSeverity) -> Self {
        match severity {
            RustSeverity::Error => Severity::Error,
            RustSeverity::Warning => Severity::Warning,
            RustSeverity::Info => Severity::Info,
        }
    }
}

/// A diagnostic message for JavaScript.
///
/// # Examples
/// ```no_run
/// // In JavaScript:
/// // const diag = result.diagnostics[0];
/// // console.log(diag.message());
/// // console.log(diag.start(), diag.end());
/// ```
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Diagnostic {
    severity: Severity,
    message: String,
    start: usize,
    end: usize,
    hint: Option<String>,
}

#[wasm_bindgen]
impl Diagnostic {
    /// Returns the severity of the diagnostic.
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the diagnostic message.
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Returns the start position in the source.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the end position in the source.
    pub fn end(&self) -> usize {
        self.end
    }

    /// Returns the hint, if any.
    pub fn hint(&self) -> Option<String> {
        self.hint.clone()
    }

    /// Converts this diagnostic to a plain JavaScript object.
    ///
    /// This is useful for passing data between the worker and main thread,
    /// as wasm-bindgen wrappers cannot be transferred.
    pub fn to_js_object(&self) -> JsValue {
        use js_sys::{Object, Reflect};

        let obj = Object::new();
        Reflect::set(&obj, &JsValue::from_str("severity"), &JsValue::from(self.severity as i32)).unwrap();
        Reflect::set(&obj, &JsValue::from_str("message"), &JsValue::from_str(&self.message)).unwrap();
        Reflect::set(&obj, &JsValue::from_str("start"), &JsValue::from(self.start as f64)).unwrap();
        Reflect::set(&obj, &JsValue::from_str("end"), &JsValue::from(self.end as f64)).unwrap();

        if let Some(hint) = &self.hint {
            Reflect::set(&obj, &JsValue::from_str("hint"), &JsValue::from_str(hint)).unwrap();
        }

        JsValue::from(obj)
    }
}

impl From<RustDiagnostic> for Diagnostic {
    fn from(diag: RustDiagnostic) -> Self {
        Self {
            severity: diag.severity().into(),
            message: diag.message().to_string(),
            start: diag.span().start(),
            end: diag.span().end(),
            hint: diag.hint().map(|s| s.to_string()),
        }
    }
}

/// A collection of diagnostics.
#[wasm_bindgen]
pub struct DiagnosticList {
    diagnostics: Vec<Diagnostic>,
}

#[wasm_bindgen]
impl DiagnosticList {
    /// Returns the number of diagnostics.
    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    /// Returns true if there are no diagnostics.
    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Returns a diagnostic by index.
    pub fn get(&self, index: usize) -> Option<Diagnostic> {
        self.diagnostics.get(index).cloned()
    }
}

impl DiagnosticList {
    pub fn new(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }
}

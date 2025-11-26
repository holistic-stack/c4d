//! # Diagnostics
//!
//! WASM-compatible diagnostic types for error reporting.

use wasm_bindgen::prelude::*;

/// A diagnostic message that can be accessed from JavaScript.
///
/// # Example (JavaScript)
///
/// ```javascript
/// try {
///     compile_and_render("invalid code");
/// } catch (error) {
///     for (const diag of error.diagnostics) {
///         console.error(`${diag.severity}: ${diag.message}`);
///         console.error(`  at ${diag.start}..${diag.end}`);
///         if (diag.hint) {
///             console.error(`  hint: ${diag.hint}`);
///         }
///     }
/// }
/// ```
#[wasm_bindgen]
pub struct Diagnostic {
    severity: String,
    message: String,
    start: u32,
    end: u32,
    hint: Option<String>,
}

#[wasm_bindgen]
impl Diagnostic {
    /// Returns the severity level ("error" or "warning").
    #[wasm_bindgen(getter)]
    pub fn severity(&self) -> String {
        self.severity.clone()
    }

    /// Returns the diagnostic message.
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// Returns the start byte offset in the source.
    #[wasm_bindgen(getter)]
    pub fn start(&self) -> u32 {
        self.start
    }

    /// Returns the end byte offset in the source.
    #[wasm_bindgen(getter)]
    pub fn end(&self) -> u32 {
        self.end
    }

    /// Returns the optional hint for fixing the issue.
    #[wasm_bindgen(getter)]
    pub fn hint(&self) -> Option<String> {
        self.hint.clone()
    }
}

impl Diagnostic {
    /// Creates a new diagnostic.
    pub fn new(
        severity: impl Into<String>,
        message: impl Into<String>,
        start: u32,
        end: u32,
        hint: Option<String>,
    ) -> Self {
        Self {
            severity: severity.into(),
            message: message.into(),
            start,
            end,
            hint,
        }
    }

    /// Creates an error diagnostic.
    pub fn error(message: impl Into<String>, start: u32, end: u32) -> Self {
        Self::new("error", message, start, end, None)
    }

    /// Creates a warning diagnostic.
    pub fn warning(message: impl Into<String>, start: u32, end: u32) -> Self {
        Self::new("warning", message, start, end, None)
    }

    /// Converts to a plain JavaScript object.
    pub fn to_js_object(&self) -> JsValue {
        let obj = js_sys::Object::new();
        
        js_sys::Reflect::set(&obj, &"severity".into(), &self.severity.clone().into())
            .expect("set severity");
        js_sys::Reflect::set(&obj, &"message".into(), &self.message.clone().into())
            .expect("set message");
        js_sys::Reflect::set(&obj, &"start".into(), &self.start.into())
            .expect("set start");
        js_sys::Reflect::set(&obj, &"end".into(), &self.end.into())
            .expect("set end");
        
        if let Some(hint) = &self.hint {
            js_sys::Reflect::set(&obj, &"hint".into(), &hint.clone().into())
                .expect("set hint");
        }
        
        obj.into()
    }
}

/// Converts a mesh error to a diagnostic.
pub fn from_mesh_error(error: &openscad_mesh::MeshError) -> Diagnostic {
    let message = error.to_string();
    
    // Extract span if available
    let (start, end) = match error {
        openscad_mesh::MeshError::EvalError(e) => {
            if let Some(span) = e.span() {
                (span.start() as u32, span.end() as u32)
            } else {
                (0, 0)
            }
        }
        openscad_mesh::MeshError::InvalidTopology { span, .. }
        | openscad_mesh::MeshError::DegenerateGeometry { span, .. }
        | openscad_mesh::MeshError::BooleanFailed { span, .. }
        | openscad_mesh::MeshError::Unsupported { span, .. } => {
            span.map(|s| (s.start() as u32, s.end() as u32)).unwrap_or((0, 0))
        }
        _ => (0, 0),
    };
    
    Diagnostic::error(message, start, end)
}

/// Converts an eval error to a diagnostic.
pub fn from_eval_error(error: &openscad_eval::EvalError) -> Diagnostic {
    let message = error.to_string();
    let (start, end) = error
        .span()
        .map(|s| (s.start() as u32, s.end() as u32))
        .unwrap_or((0, 0));
    
    Diagnostic::error(message, start, end)
}

/// Builds a JavaScript error payload with diagnostics array.
pub fn build_error_payload(diagnostics: Vec<Diagnostic>) -> JsValue {
    let array = js_sys::Array::new();
    for diag in diagnostics {
        array.push(&diag.to_js_object());
    }
    
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"diagnostics".into(), &array)
        .expect("set diagnostics");
    
    obj.into()
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    Parse,
    Ast,
    Eval,
    Kernel,
    Wasm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceDiagnostic {
    pub stage: Stage,
    pub message: String,
    pub span: Span,
    pub file: Option<String>,
    pub hint: Option<String>,
    pub causes: Option<Vec<TraceDiagnostic>>,
}
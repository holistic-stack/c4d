//! # Diagnostics
//!
//! Structured error and warning reporting for the OpenSCAD pipeline.
//! Diagnostics carry source spans for precise error location reporting.
//!
//! ## Usage
//!
//! ```rust
//! use openscad_ast::{Diagnostic, Severity, Span};
//!
//! let diagnostic = Diagnostic::error(
//!     "Unknown function 'foo'",
//!     Span::new(0, 3),
//! ).with_hint("Did you mean 'for'?");
//! ```

use crate::span::Span;

/// Severity level of a diagnostic message.
///
/// # Variants
///
/// - `Error`: A problem that prevents successful compilation
/// - `Warning`: A potential issue that doesn't prevent compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// A problem that prevents successful compilation
    Error,
    /// A potential issue that doesn't prevent compilation
    Warning,
}

impl Severity {
    /// Returns the string representation of the severity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use openscad_ast::Severity;
    ///
    /// assert_eq!(Severity::Error.as_str(), "error");
    /// assert_eq!(Severity::Warning.as_str(), "warning");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

/// A diagnostic message with source location and optional hint.
///
/// Diagnostics are the canonical error type used throughout the pipeline.
/// They carry enough information for IDE integration (squiggles, tooltips).
///
/// # Example
///
/// ```rust
/// use openscad_ast::{Diagnostic, Severity, Span};
///
/// let diag = Diagnostic::error("Syntax error", Span::new(0, 5));
/// assert_eq!(diag.severity(), Severity::Error);
/// assert_eq!(diag.message(), "Syntax error");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The severity level (Error or Warning)
    severity: Severity,
    /// The diagnostic message
    message: String,
    /// The source location of the issue
    span: Span,
    /// Optional hint for fixing the issue
    hint: Option<String>,
}

impl Diagnostic {
    /// Creates a new diagnostic with the given severity, message, and span.
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level
    /// * `message` - The diagnostic message
    /// * `span` - The source location
    ///
    /// # Example
    ///
    /// ```rust
    /// use openscad_ast::{Diagnostic, Severity, Span};
    ///
    /// let diag = Diagnostic::new(
    ///     Severity::Warning,
    ///     "Unused variable",
    ///     Span::new(10, 15),
    /// );
    /// ```
    pub fn new(severity: Severity, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity,
            message: message.into(),
            span,
            hint: None,
        }
    }

    /// Creates an error diagnostic.
    ///
    /// # Example
    ///
    /// ```rust
    /// use openscad_ast::{Diagnostic, Span};
    ///
    /// let diag = Diagnostic::error("Parse error", Span::new(0, 5));
    /// ```
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self::new(Severity::Error, message, span)
    }

    /// Creates a warning diagnostic.
    ///
    /// # Example
    ///
    /// ```rust
    /// use openscad_ast::{Diagnostic, Span};
    ///
    /// let diag = Diagnostic::warning("Deprecated syntax", Span::new(0, 5));
    /// ```
    pub fn warning(message: impl Into<String>, span: Span) -> Self {
        Self::new(Severity::Warning, message, span)
    }

    /// Adds a hint to the diagnostic.
    ///
    /// # Example
    ///
    /// ```rust
    /// use openscad_ast::{Diagnostic, Span};
    ///
    /// let diag = Diagnostic::error("Unknown function", Span::new(0, 5))
    ///     .with_hint("Check spelling or import the module");
    /// ```
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Returns the severity level.
    #[inline]
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the diagnostic message.
    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the source span.
    #[inline]
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns the start byte offset.
    #[inline]
    pub fn start(&self) -> usize {
        self.span.start()
    }

    /// Returns the end byte offset.
    #[inline]
    pub fn end(&self) -> usize {
        self.span.end()
    }

    /// Returns the optional hint.
    #[inline]
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    /// Returns true if this is an error diagnostic.
    #[inline]
    pub fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }

    /// Returns true if this is a warning diagnostic.
    #[inline]
    pub fn is_warning(&self) -> bool {
        matches!(self.severity, Severity::Warning)
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} (at {}..{})",
            self.severity.as_str(),
            self.message,
            self.span.start(),
            self.span.end()
        )?;
        if let Some(hint) = &self.hint {
            write!(f, "\n  hint: {}", hint)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_as_str() {
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Warning.as_str(), "warning");
    }

    #[test]
    fn test_diagnostic_error() {
        let diag = Diagnostic::error("Test error", Span::new(0, 10));
        assert_eq!(diag.severity(), Severity::Error);
        assert_eq!(diag.message(), "Test error");
        assert_eq!(diag.start(), 0);
        assert_eq!(diag.end(), 10);
        assert!(diag.hint().is_none());
        assert!(diag.is_error());
        assert!(!diag.is_warning());
    }

    #[test]
    fn test_diagnostic_warning() {
        let diag = Diagnostic::warning("Test warning", Span::new(5, 15));
        assert_eq!(diag.severity(), Severity::Warning);
        assert!(diag.is_warning());
        assert!(!diag.is_error());
    }

    #[test]
    fn test_diagnostic_with_hint() {
        let diag = Diagnostic::error("Error", Span::new(0, 5))
            .with_hint("Try this instead");
        assert_eq!(diag.hint(), Some("Try this instead"));
    }

    #[test]
    fn test_diagnostic_display() {
        let diag = Diagnostic::error("Parse error", Span::new(0, 10));
        let display = format!("{}", diag);
        assert!(display.contains("error"));
        assert!(display.contains("Parse error"));
        assert!(display.contains("0..10"));
    }

    #[test]
    fn test_diagnostic_display_with_hint() {
        let diag = Diagnostic::error("Error", Span::new(0, 5))
            .with_hint("Fix it");
        let display = format!("{}", diag);
        assert!(display.contains("hint: Fix it"));
    }
}

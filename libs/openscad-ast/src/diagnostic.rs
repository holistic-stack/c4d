/// Diagnostic severity levels.
///
/// Represents the severity of a diagnostic message, from informational
/// to critical errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// An error that prevents compilation.
    Error,
    /// A warning about potentially problematic code.
    Warning,
    /// Informational message.
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// A diagnostic message with source location.
///
/// Diagnostics are used throughout the pipeline to report errors, warnings,
/// and informational messages with precise source locations.
///
/// # Examples
/// ```
/// use openscad_ast::{Diagnostic, Severity, Span};
///
/// let span = Span::new(10, 15).unwrap();
/// let diag = Diagnostic::error("Unexpected token", span)
///     .with_hint("Expected ';' after statement");
///
/// assert_eq!(diag.severity(), Severity::Error);
/// assert_eq!(diag.message(), "Unexpected token");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    severity: Severity,
    message: String,
    span: crate::Span,
    hint: Option<String>,
}

impl Diagnostic {
    /// Creates a new diagnostic.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::{Diagnostic, Severity, Span};
    ///
    /// let span = Span::new(0, 5).unwrap();
    /// let diag = Diagnostic::new(Severity::Error, "Parse error", span);
    /// ```
    pub fn new(severity: Severity, message: impl Into<String>, span: crate::Span) -> Self {
        Self {
            severity,
            message: message.into(),
            span,
            hint: None,
        }
    }

    /// Creates an error diagnostic.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::{Diagnostic, Span};
    ///
    /// let span = Span::new(0, 5).unwrap();
    /// let diag = Diagnostic::error("Syntax error", span);
    /// ```
    pub fn error(message: impl Into<String>, span: crate::Span) -> Self {
        Self::new(Severity::Error, message, span)
    }

    /// Creates a warning diagnostic.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::{Diagnostic, Span};
    ///
    /// let span = Span::new(0, 5).unwrap();
    /// let diag = Diagnostic::warning("Deprecated syntax", span);
    /// ```
    pub fn warning(message: impl Into<String>, span: crate::Span) -> Self {
        Self::new(Severity::Warning, message, span)
    }

    /// Creates an info diagnostic.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::{Diagnostic, Span};
    ///
    /// let span = Span::new(0, 5).unwrap();
    /// let diag = Diagnostic::info("Optimization applied", span);
    /// ```
    pub fn info(message: impl Into<String>, span: crate::Span) -> Self {
        Self::new(Severity::Info, message, span)
    }

    /// Adds a hint to the diagnostic.
    ///
    /// # Examples
    /// ```
    /// use openscad_ast::{Diagnostic, Span};
    ///
    /// let span = Span::new(0, 5).unwrap();
    /// let diag = Diagnostic::error("Missing semicolon", span)
    ///     .with_hint("Add ';' at the end of the statement");
    /// ```
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Returns the severity of the diagnostic.
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the message of the diagnostic.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the span of the diagnostic.
    pub fn span(&self) -> crate::Span {
        self.span
    }

    /// Returns the hint, if any.
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} [{}:{}]",
            self.severity,
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
    fn test_diagnostic_creation() {
        let span = crate::Span::new(10, 20).unwrap();
        let diag = Diagnostic::error("Test error", span);
        
        assert_eq!(diag.severity(), Severity::Error);
        assert_eq!(diag.message(), "Test error");
        assert_eq!(diag.span(), span);
        assert_eq!(diag.hint(), None);
    }

    #[test]
    fn test_diagnostic_with_hint() {
        let span = crate::Span::new(0, 5).unwrap();
        let diag = Diagnostic::warning("Deprecated", span)
            .with_hint("Use new syntax instead");
        
        assert_eq!(diag.hint(), Some("Use new syntax instead"));
    }

    #[test]
    fn test_diagnostic_display() {
        let span = crate::Span::new(5, 10).unwrap();
        let diag = Diagnostic::error("Parse error", span);
        
        let output = format!("{}", diag);
        assert!(output.contains("error"));
        assert!(output.contains("Parse error"));
        assert!(output.contains("[5:10]"));
    }

    #[test]
    fn test_severity_levels() {
        let span = crate::Span::new(0, 1).unwrap();
        
        let error = Diagnostic::error("error", span);
        let warning = Diagnostic::warning("warning", span);
        let info = Diagnostic::info("info", span);
        
        assert_eq!(error.severity(), Severity::Error);
        assert_eq!(warning.severity(), Severity::Warning);
        assert_eq!(info.severity(), Severity::Info);
    }
}

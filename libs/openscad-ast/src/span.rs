//! # Source Span
//!
//! Represents a range in the source code for error reporting and source mapping.
//! Spans are preserved through the entire pipeline from parsing to mesh generation.
//!
//! ## Usage
//!
//! ```rust
//! use openscad_ast::Span;
//!
//! let span = Span::new(0, 10);
//! assert_eq!(span.start(), 0);
//! assert_eq!(span.end(), 10);
//! assert_eq!(span.len(), 10);
//! ```

/// A range in the source code, represented as byte offsets.
///
/// Used for error reporting, diagnostics, and mapping geometry back to source.
///
/// # Fields
///
/// - `start`: Starting byte offset (inclusive)
/// - `end`: Ending byte offset (exclusive)
///
/// # Example
///
/// ```rust
/// use openscad_ast::Span;
///
/// // For source "cube(10);" the span of "cube" would be:
/// let span = Span::new(0, 4);
/// assert_eq!(span.len(), 4);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Starting byte offset (inclusive)
    start: usize,
    /// Ending byte offset (exclusive)
    end: usize,
}

impl Span {
    /// Creates a new span from start and end byte offsets.
    ///
    /// # Arguments
    ///
    /// * `start` - Starting byte offset (inclusive)
    /// * `end` - Ending byte offset (exclusive)
    ///
    /// # Example
    ///
    /// ```rust
    /// use openscad_ast::Span;
    ///
    /// let span = Span::new(5, 15);
    /// assert_eq!(span.start(), 5);
    /// assert_eq!(span.end(), 15);
    /// ```
    #[inline]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Returns the starting byte offset.
    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the ending byte offset.
    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    /// Returns the length of the span in bytes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use openscad_ast::Span;
    ///
    /// let span = Span::new(10, 25);
    /// assert_eq!(span.len(), 15);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if the span has zero length.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Creates a span that encompasses both this span and another.
    ///
    /// # Example
    ///
    /// ```rust
    /// use openscad_ast::Span;
    ///
    /// let span1 = Span::new(0, 5);
    /// let span2 = Span::new(10, 15);
    /// let merged = span1.merge(&span2);
    /// assert_eq!(merged.start(), 0);
    /// assert_eq!(merged.end(), 15);
    /// ```
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Checks if this span contains a byte offset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use openscad_ast::Span;
    ///
    /// let span = Span::new(5, 10);
    /// assert!(span.contains(7));
    /// assert!(!span.contains(10)); // end is exclusive
    /// ```
    #[inline]
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }

    /// Creates a span from a tree-sitter node.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use openscad_ast::Span;
    ///
    /// let span = Span::from_ts_node(&node);
    /// ```
    #[cfg(feature = "native-parser")]
    pub fn from_ts_node(node: &tree_sitter::Node) -> Self {
        Self {
            start: node.start_byte(),
            end: node.end_byte(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let span = Span::new(10, 20);
        assert_eq!(span.start(), 10);
        assert_eq!(span.end(), 20);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(5, 15);
        assert_eq!(span.len(), 10);
    }

    #[test]
    fn test_span_is_empty() {
        assert!(Span::new(5, 5).is_empty());
        assert!(Span::new(10, 5).is_empty()); // Invalid span is empty
        assert!(!Span::new(0, 1).is_empty());
    }

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(0, 10);
        let span2 = Span::new(5, 20);
        let merged = span1.merge(&span2);
        assert_eq!(merged.start(), 0);
        assert_eq!(merged.end(), 20);
    }

    #[test]
    fn test_span_merge_disjoint() {
        let span1 = Span::new(0, 5);
        let span2 = Span::new(10, 15);
        let merged = span1.merge(&span2);
        assert_eq!(merged.start(), 0);
        assert_eq!(merged.end(), 15);
    }

    #[test]
    fn test_span_contains() {
        let span = Span::new(5, 10);
        assert!(!span.contains(4));
        assert!(span.contains(5));
        assert!(span.contains(7));
        assert!(span.contains(9));
        assert!(!span.contains(10)); // end is exclusive
    }

    #[test]
    fn test_span_default() {
        let span = Span::default();
        assert_eq!(span.start(), 0);
        assert_eq!(span.end(), 0);
        assert!(span.is_empty());
    }
}

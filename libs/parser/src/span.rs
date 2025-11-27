//! # Source Span and Position
//!
//! Types for tracking source locations in parsed code.
//! Inspired by tree-sitter's TSPoint and span tracking.
//!
//! ## Example
//!
//! ```rust
//! use openscad_parser::span::{Position, Span};
//!
//! let start = Position::new(0, 0, 0);
//! let end = Position::new(10, 0, 10);
//! let span = Span::new(start, end);
//! assert_eq!(span.len(), 10);
//! ```

use serde::{Deserialize, Serialize};

// =============================================================================
// POSITION
// =============================================================================

/// A position in source code.
///
/// Tracks byte offset, line number, and column number.
/// All values are 0-indexed.
///
/// ## Example
///
/// ```rust
/// let pos = Position::new(0, 0, 0); // Start of file
/// let pos2 = Position::new(10, 0, 10); // Byte 10, line 0, column 10
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Position {
    /// Byte offset in source (0-indexed).
    pub byte: usize,
    /// Line number (0-indexed).
    pub line: usize,
    /// Column number in bytes (0-indexed).
    pub column: usize,
}

impl Position {
    /// Create a new position.
    ///
    /// ## Parameters
    ///
    /// - `byte`: Byte offset in source
    /// - `line`: Line number (0-indexed)
    /// - `column`: Column number (0-indexed)
    ///
    /// ## Example
    ///
    /// ```rust
    /// let pos = Position::new(5, 0, 5);
    /// assert_eq!(pos.byte, 5);
    /// ```
    pub const fn new(byte: usize, line: usize, column: usize) -> Self {
        Self { byte, line, column }
    }

    /// Create position at start of source.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let pos = Position::zero();
    /// assert_eq!(pos.byte, 0);
    /// assert_eq!(pos.line, 0);
    /// assert_eq!(pos.column, 0);
    /// ```
    pub const fn zero() -> Self {
        Self::new(0, 0, 0)
    }
}

// =============================================================================
// SPAN
// =============================================================================

/// A span in source code (start to end position).
///
/// ## Example
///
/// ```rust
/// let span = Span::new(
///     Position::new(0, 0, 0),
///     Position::new(10, 0, 10),
/// );
/// assert_eq!(span.len(), 10);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Span {
    /// Start position (inclusive).
    pub start: Position,
    /// End position (exclusive).
    pub end: Position,
}

impl Span {
    /// Create a new span.
    ///
    /// ## Parameters
    ///
    /// - `start`: Start position (inclusive)
    /// - `end`: End position (exclusive)
    ///
    /// ## Example
    ///
    /// ```rust
    /// let span = Span::new(Position::zero(), Position::new(5, 0, 5));
    /// ```
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create empty span at position zero.
    pub const fn zero() -> Self {
        Self::new(Position::zero(), Position::zero())
    }

    /// Get byte length of span.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let span = Span::new(Position::new(0, 0, 0), Position::new(10, 0, 10));
    /// assert_eq!(span.len(), 10);
    /// ```
    pub const fn len(&self) -> usize {
        self.end.byte - self.start.byte
    }

    /// Check if span is empty.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create span from byte range.
    ///
    /// Note: Line and column will be 0. Use for simple cases only.
    pub const fn from_bytes(start: usize, end: usize) -> Self {
        Self::new(Position::new(start, 0, 0), Position::new(end, 0, 0))
    }
}

// =============================================================================
// TRAIT: SPANNED
// =============================================================================

/// Trait for types that have a source span.
pub trait Spanned {
    /// Get the source span of this item.
    fn span(&self) -> Span;
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(10, 2, 5);
        assert_eq!(pos.byte, 10);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 5);
    }

    #[test]
    fn test_position_zero() {
        let pos = Position::zero();
        assert_eq!(pos.byte, 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(Position::new(5, 0, 5), Position::new(15, 0, 15));
        assert_eq!(span.len(), 10);
    }

    #[test]
    fn test_span_is_empty() {
        let empty = Span::zero();
        assert!(empty.is_empty());

        let non_empty = Span::from_bytes(0, 5);
        assert!(!non_empty.is_empty());
    }
}

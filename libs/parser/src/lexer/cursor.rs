//! # Character Cursor
//!
//! Peekable character cursor for the lexer.
//! Tracks position (byte, line, column) as it advances.
//!
//! ## Example
//!
//! ```rust
//! use openscad_parser::lexer::Cursor;
//!
//! let mut cursor = Cursor::new("hello");
//! assert_eq!(cursor.peek(), Some('h'));
//! cursor.advance();
//! assert_eq!(cursor.peek(), Some('e'));
//! ```

use crate::span::Position;
use std::str::Chars;

// =============================================================================
// CURSOR
// =============================================================================

/// Character cursor with position tracking.
///
/// Provides peekable iteration over source characters
/// while tracking byte offset, line, and column.
///
/// ## Example
///
/// ```rust
/// let mut cursor = Cursor::new("cube");
/// assert_eq!(cursor.advance(), Some('c'));
/// assert_eq!(cursor.position().byte, 1);
/// ```
pub struct Cursor<'a> {
    /// Source text.
    source: &'a str,
    /// Character iterator.
    chars: Chars<'a>,
    /// Current byte offset.
    byte: usize,
    /// Current line (0-indexed).
    line: usize,
    /// Current column (0-indexed).
    column: usize,
}

impl<'a> Cursor<'a> {
    /// Create a new cursor for source text.
    ///
    /// ## Parameters
    ///
    /// - `source`: Source text to iterate over
    ///
    /// ## Example
    ///
    /// ```rust
    /// let cursor = Cursor::new("cube(10);");
    /// assert!(!cursor.is_eof());
    /// ```
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars(),
            byte: 0,
            line: 0,
            column: 0,
        }
    }

    /// Get current position.
    ///
    /// ## Returns
    ///
    /// Current position (byte, line, column)
    ///
    /// ## Example
    ///
    /// ```rust
    /// let cursor = Cursor::new("hello");
    /// let pos = cursor.position();
    /// assert_eq!(pos.byte, 0);
    /// ```
    pub fn position(&self) -> Position {
        Position::new(self.byte, self.line, self.column)
    }

    /// Check if at end of file.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let cursor = Cursor::new("");
    /// assert!(cursor.is_eof());
    /// ```
    pub fn is_eof(&self) -> bool {
        self.byte >= self.source.len()
    }

    /// Peek at current character without consuming it.
    ///
    /// ## Returns
    ///
    /// Current character or None if at EOF
    ///
    /// ## Example
    ///
    /// ```rust
    /// let cursor = Cursor::new("abc");
    /// assert_eq!(cursor.peek(), Some('a'));
    /// assert_eq!(cursor.peek(), Some('a')); // Still 'a'
    /// ```
    pub fn peek(&self) -> Option<char> {
        self.source[self.byte..].chars().next()
    }

    /// Peek at next character (one ahead of current).
    ///
    /// ## Returns
    ///
    /// Next character or None if not available
    ///
    /// ## Example
    ///
    /// ```rust
    /// let cursor = Cursor::new("ab");
    /// assert_eq!(cursor.peek_next(), Some('b'));
    /// ```
    pub fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.byte..].chars();
        chars.next(); // Skip current
        chars.next()
    }

    /// Advance to next character.
    ///
    /// ## Returns
    ///
    /// Character that was consumed, or None if at EOF
    ///
    /// ## Example
    ///
    /// ```rust
    /// let mut cursor = Cursor::new("ab");
    /// assert_eq!(cursor.advance(), Some('a'));
    /// assert_eq!(cursor.advance(), Some('b'));
    /// assert_eq!(cursor.advance(), None);
    /// ```
    pub fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;

        // Update position
        self.byte += c.len_utf8();

        if c == '\n' {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }

        // Advance internal iterator (keep in sync)
        self.chars.next();

        Some(c)
    }

    /// Advance while predicate is true.
    ///
    /// ## Parameters
    ///
    /// - `predicate`: Function that returns true to continue advancing
    ///
    /// ## Example
    ///
    /// ```rust
    /// let mut cursor = Cursor::new("abc123");
    /// cursor.advance_while(|c| c.is_alphabetic());
    /// assert_eq!(cursor.peek(), Some('1'));
    /// ```
    pub fn advance_while(&mut self, predicate: impl Fn(char) -> bool) {
        while let Some(c) = self.peek() {
            if !predicate(c) {
                break;
            }
            self.advance();
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_new() {
        let cursor = Cursor::new("hello");
        assert_eq!(cursor.position().byte, 0);
        assert!(!cursor.is_eof());
    }

    #[test]
    fn test_cursor_empty() {
        let cursor = Cursor::new("");
        assert!(cursor.is_eof());
        assert_eq!(cursor.peek(), None);
    }

    #[test]
    fn test_cursor_peek() {
        let cursor = Cursor::new("abc");
        assert_eq!(cursor.peek(), Some('a'));
        assert_eq!(cursor.peek(), Some('a')); // Should not advance
    }

    #[test]
    fn test_cursor_peek_next() {
        let cursor = Cursor::new("abc");
        assert_eq!(cursor.peek(), Some('a'));
        assert_eq!(cursor.peek_next(), Some('b'));
    }

    #[test]
    fn test_cursor_advance() {
        let mut cursor = Cursor::new("ab");
        assert_eq!(cursor.advance(), Some('a'));
        assert_eq!(cursor.position().byte, 1);
        assert_eq!(cursor.advance(), Some('b'));
        assert_eq!(cursor.position().byte, 2);
        assert_eq!(cursor.advance(), None);
        assert!(cursor.is_eof());
    }

    #[test]
    fn test_cursor_newline() {
        let mut cursor = Cursor::new("a\nb");
        cursor.advance(); // 'a'
        assert_eq!(cursor.position().line, 0);
        cursor.advance(); // '\n'
        assert_eq!(cursor.position().line, 1);
        assert_eq!(cursor.position().column, 0);
    }

    #[test]
    fn test_cursor_advance_while() {
        let mut cursor = Cursor::new("abc123");
        cursor.advance_while(|c| c.is_alphabetic());
        assert_eq!(cursor.peek(), Some('1'));
        assert_eq!(cursor.position().byte, 3);
    }

    #[test]
    fn test_cursor_utf8() {
        let mut cursor = Cursor::new("é");
        assert_eq!(cursor.advance(), Some('é'));
        assert_eq!(cursor.position().byte, 2); // é is 2 bytes in UTF-8
    }
}

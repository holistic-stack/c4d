//! # Tokens
//!
//! Token types for the OpenSCAD lexer.
//!
//! ## Example
//!
//! ```rust
//! use openscad_parser::lexer::{Token, TokenKind};
//!
//! let token = Token::new(TokenKind::Number, Span::from_bytes(0, 2), "10".to_string());
//! assert_eq!(token.kind, TokenKind::Number);
//! ```

use crate::span::{Span, Spanned};

// =============================================================================
// TOKEN
// =============================================================================

/// A token produced by the lexer.
///
/// ## Example
///
/// ```rust
/// let token = Token::new(TokenKind::Identifier, Span::from_bytes(0, 4), "cube".to_string());
/// assert_eq!(token.text, "cube");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token type.
    pub kind: TokenKind,
    /// Source span.
    pub span: Span,
    /// Token text.
    pub text: String,
}

impl Token {
    /// Create a new token.
    ///
    /// ## Parameters
    ///
    /// - `kind`: Token type
    /// - `span`: Source location
    /// - `text`: Token text
    pub fn new(kind: TokenKind, span: Span, text: String) -> Self {
        Self { kind, span, text }
    }

    /// Check if token is EOF.
    pub fn is_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }

    /// Check if token is an error.
    pub fn is_error(&self) -> bool {
        self.kind == TokenKind::Error
    }
}

impl Spanned for Token {
    fn span(&self) -> Span {
        self.span
    }
}

// =============================================================================
// TOKEN KIND
// =============================================================================

/// Types of tokens.
///
/// Based on OpenSCAD grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Literals
    /// Number literal like `10` or `3.14`
    Number,
    /// String literal like `"hello"`
    String,
    /// Boolean true
    True,
    /// Boolean false
    False,
    /// Undef value
    Undef,

    // Identifiers
    /// Identifier like `cube` or `myVar`
    Identifier,
    /// Special variable like `$fn`
    SpecialVariable,

    // Keywords
    /// `module` keyword
    Module,
    /// `function` keyword
    Function,
    /// `if` keyword
    If,
    /// `else` keyword
    Else,
    /// `for` keyword
    For,
    /// `let` keyword
    Let,
    /// `each` keyword
    Each,
    /// `include` keyword
    Include,
    /// `use` keyword
    Use,

    // Operators
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `^`
    Caret,
    /// `!`
    Bang,
    /// `=`
    Eq,
    /// `==`
    EqEq,
    /// `!=`
    BangEq,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `<=`
    LtEq,
    /// `>=`
    GtEq,
    /// `&&`
    AmpAmp,
    /// `||`
    PipePipe,
    /// `?`
    Question,
    /// `:`
    Colon,

    // Delimiters
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `.`
    Dot,

    // Modifiers
    /// `#`
    Hash,

    // Meta
    /// End of file
    Eof,
    /// Error token
    Error,
}

impl TokenKind {
    /// Check if this is a literal token.
    pub const fn is_literal(&self) -> bool {
        matches!(self, Self::Number | Self::String | Self::True | Self::False | Self::Undef)
    }

    /// Check if this is a keyword token.
    pub const fn is_keyword(&self) -> bool {
        matches!(
            self,
            Self::Module
                | Self::Function
                | Self::If
                | Self::Else
                | Self::For
                | Self::Let
                | Self::Each
                | Self::Include
                | Self::Use
                | Self::True
                | Self::False
                | Self::Undef
        )
    }

    /// Check if this is an operator token.
    pub const fn is_operator(&self) -> bool {
        matches!(
            self,
            Self::Plus
                | Self::Minus
                | Self::Star
                | Self::Slash
                | Self::Percent
                | Self::Caret
                | Self::Bang
                | Self::Eq
                | Self::EqEq
                | Self::BangEq
                | Self::Lt
                | Self::Gt
                | Self::LtEq
                | Self::GtEq
                | Self::AmpAmp
                | Self::PipePipe
                | Self::Question
                | Self::Colon
        )
    }

    /// Get display string for error messages.
    pub const fn display(&self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::String => "string",
            Self::True => "true",
            Self::False => "false",
            Self::Undef => "undef",
            Self::Identifier => "identifier",
            Self::SpecialVariable => "special variable",
            Self::Module => "module",
            Self::Function => "function",
            Self::If => "if",
            Self::Else => "else",
            Self::For => "for",
            Self::Let => "let",
            Self::Each => "each",
            Self::Include => "include",
            Self::Use => "use",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::Caret => "^",
            Self::Bang => "!",
            Self::Eq => "=",
            Self::EqEq => "==",
            Self::BangEq => "!=",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::LtEq => "<=",
            Self::GtEq => ">=",
            Self::AmpAmp => "&&",
            Self::PipePipe => "||",
            Self::Question => "?",
            Self::Colon => ":",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LBracket => "[",
            Self::RBracket => "]",
            Self::LBrace => "{",
            Self::RBrace => "}",
            Self::Semicolon => ";",
            Self::Comma => ",",
            Self::Dot => ".",
            Self::Hash => "#",
            Self::Eof => "end of file",
            Self::Error => "error",
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
    fn test_token_is_literal() {
        assert!(TokenKind::Number.is_literal());
        assert!(TokenKind::String.is_literal());
        assert!(!TokenKind::Identifier.is_literal());
    }

    #[test]
    fn test_token_is_keyword() {
        assert!(TokenKind::Module.is_keyword());
        assert!(TokenKind::True.is_keyword());
        assert!(!TokenKind::Identifier.is_keyword());
    }

    #[test]
    fn test_token_display() {
        assert_eq!(TokenKind::LParen.display(), "(");
        assert_eq!(TokenKind::Identifier.display(), "identifier");
    }
}

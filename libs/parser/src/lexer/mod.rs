//! # OpenSCAD Lexer
//!
//! Tokenizes OpenSCAD source code into tokens.
//! Inspired by tree-sitter's lexer design.
//!
//! ## Example
//!
//! ```rust
//! use openscad_parser::lexer::Lexer;
//!
//! let tokens = Lexer::new("cube(10);").tokenize();
//! assert_eq!(tokens[0].kind, TokenKind::Identifier);
//! ```

mod token;
mod cursor;

pub use token::{Token, TokenKind};
pub use cursor::Cursor;

use crate::span::{Position, Span};

// =============================================================================
// LEXER
// =============================================================================

/// OpenSCAD lexer.
///
/// Converts source text into a stream of tokens.
///
/// ## Example
///
/// ```rust
/// let mut lexer = Lexer::new("cube(10);");
/// let tokens = lexer.tokenize();
/// ```
pub struct Lexer<'a> {
    /// Source text being lexed.
    source: &'a str,
    /// Character cursor.
    cursor: Cursor<'a>,
    /// Collected tokens.
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for source text.
    ///
    /// ## Parameters
    ///
    /// - `source`: OpenSCAD source code
    ///
    /// ## Example
    ///
    /// ```rust
    /// let lexer = Lexer::new("cube(10);");
    /// ```
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: Cursor::new(source),
            tokens: Vec::new(),
        }
    }

    /// Tokenize the entire source.
    ///
    /// ## Returns
    ///
    /// Vector of tokens including EOF token.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let tokens = Lexer::new("cube(10);").tokenize();
    /// assert!(tokens.last().map(|t| t.kind == TokenKind::Eof).unwrap_or(false));
    /// ```
    pub fn tokenize(mut self) -> Vec<Token> {
        while !self.cursor.is_eof() {
            self.skip_whitespace_and_comments();
            if self.cursor.is_eof() {
                break;
            }
            self.scan_token();
        }

        // Add EOF token
        let eof_pos = self.cursor.position();
        self.tokens.push(Token::new(
            TokenKind::Eof,
            Span::new(eof_pos, eof_pos),
            String::new(),
        ));

        self.tokens
    }

    /// Skip whitespace and comments.
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.cursor.peek().map_or(false, |c| c.is_whitespace()) {
                self.cursor.advance();
            }

            // Skip line comments
            if self.cursor.peek() == Some('/') && self.cursor.peek_next() == Some('/') {
                self.cursor.advance(); // /
                self.cursor.advance(); // /
                while self.cursor.peek().map_or(false, |c| c != '\n') {
                    self.cursor.advance();
                }
                continue;
            }

            // Skip block comments
            if self.cursor.peek() == Some('/') && self.cursor.peek_next() == Some('*') {
                self.cursor.advance(); // /
                self.cursor.advance(); // *
                while !self.cursor.is_eof() {
                    if self.cursor.peek() == Some('*') && self.cursor.peek_next() == Some('/') {
                        self.cursor.advance(); // *
                        self.cursor.advance(); // /
                        break;
                    }
                    self.cursor.advance();
                }
                continue;
            }

            break;
        }
    }

    /// Scan a single token.
    fn scan_token(&mut self) {
        let start = self.cursor.position();
        let c = match self.cursor.advance() {
            Some(c) => c,
            None => return,
        };

        let kind = match c {
            // Single-character tokens
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '^' => TokenKind::Caret,
            '#' => TokenKind::Hash,
            '?' => TokenKind::Question,
            ':' => TokenKind::Colon,

            // Two-character tokens
            '=' => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    TokenKind::BangEq
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.cursor.peek() == Some('=') {
                    self.cursor.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            '&' => {
                if self.cursor.peek() == Some('&') {
                    self.cursor.advance();
                    TokenKind::AmpAmp
                } else {
                    TokenKind::Error
                }
            }
            '|' => {
                if self.cursor.peek() == Some('|') {
                    self.cursor.advance();
                    TokenKind::PipePipe
                } else {
                    TokenKind::Error
                }
            }

            // String literal
            '"' => return self.scan_string(start),

            // Number literal
            '0'..='9' => return self.scan_number(start, c),

            // Identifier or keyword
            'a'..='z' | 'A'..='Z' | '_' => return self.scan_identifier(start, c),

            // Special variable ($fn, $fa, etc.)
            '$' => return self.scan_special_variable(start),

            _ => TokenKind::Error,
        };

        let end = self.cursor.position();
        let text = &self.source[start.byte..end.byte];
        self.tokens.push(Token::new(kind, Span::new(start, end), text.to_string()));
    }

    /// Scan a string literal.
    fn scan_string(&mut self, start: Position) {
        while let Some(c) = self.cursor.peek() {
            if c == '"' {
                self.cursor.advance(); // Closing quote
                break;
            }
            if c == '\\' {
                self.cursor.advance(); // Backslash
                self.cursor.advance(); // Escaped char
            } else {
                self.cursor.advance();
            }
        }

        let end = self.cursor.position();
        let text = &self.source[start.byte..end.byte];
        self.tokens.push(Token::new(TokenKind::String, Span::new(start, end), text.to_string()));
    }

    /// Scan a number literal.
    fn scan_number(&mut self, start: Position, first_char: char) {
        let mut has_dot = false;
        let mut has_exponent = false;

        // Handle leading minus in number (already consumed first digit)
        while let Some(c) = self.cursor.peek() {
            match c {
                '0'..='9' => {
                    self.cursor.advance();
                }
                '.' if !has_dot && !has_exponent => {
                    // Check it's not range operator (..)
                    if self.cursor.peek_next() != Some('.') {
                        has_dot = true;
                        self.cursor.advance();
                    } else {
                        break;
                    }
                }
                'e' | 'E' if !has_exponent => {
                    has_exponent = true;
                    self.cursor.advance();
                    // Handle optional sign after exponent
                    if matches!(self.cursor.peek(), Some('+') | Some('-')) {
                        self.cursor.advance();
                    }
                }
                _ => break,
            }
        }

        let end = self.cursor.position();
        let text = &self.source[start.byte..end.byte];
        self.tokens.push(Token::new(TokenKind::Number, Span::new(start, end), text.to_string()));
    }

    /// Scan an identifier or keyword.
    fn scan_identifier(&mut self, start: Position, _first_char: char) {
        while let Some(c) = self.cursor.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.cursor.advance();
            } else {
                break;
            }
        }

        let end = self.cursor.position();
        let text = &self.source[start.byte..end.byte];

        // Check for keywords
        let kind = match text {
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "undef" => TokenKind::Undef,
            "module" => TokenKind::Module,
            "function" => TokenKind::Function,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "let" => TokenKind::Let,
            "each" => TokenKind::Each,
            "include" => TokenKind::Include,
            "use" => TokenKind::Use,
            _ => TokenKind::Identifier,
        };

        self.tokens.push(Token::new(kind, Span::new(start, end), text.to_string()));
    }

    /// Scan a special variable ($fn, $fa, etc.).
    fn scan_special_variable(&mut self, start: Position) {
        while let Some(c) = self.cursor.peek() {
            if c.is_alphanumeric() || c == '_' {
                self.cursor.advance();
            } else {
                break;
            }
        }

        let end = self.cursor.position();
        let text = &self.source[start.byte..end.byte];
        self.tokens.push(Token::new(TokenKind::SpecialVariable, Span::new(start, end), text.to_string()));
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_cube() {
        let tokens = Lexer::new("cube(10);").tokenize();
        
        // cube, (, 10, ), ;, EOF = 6 tokens
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].text, "cube");
        assert_eq!(tokens[1].kind, TokenKind::LParen);
        assert_eq!(tokens[2].kind, TokenKind::Number);
        assert_eq!(tokens[2].text, "10");
        assert_eq!(tokens[3].kind, TokenKind::RParen);
        assert_eq!(tokens[4].kind, TokenKind::Semicolon);
        assert_eq!(tokens[5].kind, TokenKind::Eof);
    }

    #[test]
    fn test_tokenize_with_comments() {
        let tokens = Lexer::new("// comment\ncube(10);").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].text, "cube");
    }

    #[test]
    fn test_tokenize_keywords() {
        let tokens = Lexer::new("true false undef").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::True);
        assert_eq!(tokens[1].kind, TokenKind::False);
        assert_eq!(tokens[2].kind, TokenKind::Undef);
    }

    #[test]
    fn test_tokenize_special_variable() {
        let tokens = Lexer::new("$fn").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::SpecialVariable);
        assert_eq!(tokens[0].text, "$fn");
    }

    #[test]
    fn test_tokenize_operators() {
        let tokens = Lexer::new("== != <= >= && ||").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::EqEq);
        assert_eq!(tokens[1].kind, TokenKind::BangEq);
        assert_eq!(tokens[2].kind, TokenKind::LtEq);
        assert_eq!(tokens[3].kind, TokenKind::GtEq);
        assert_eq!(tokens[4].kind, TokenKind::AmpAmp);
        assert_eq!(tokens[5].kind, TokenKind::PipePipe);
    }

    #[test]
    fn test_tokenize_float() {
        let tokens = Lexer::new("3.14").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::Number);
        assert_eq!(tokens[0].text, "3.14");
    }

    #[test]
    fn test_tokenize_named_argument() {
        let tokens = Lexer::new("center=true").tokenize();
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[1].kind, TokenKind::Eq);
        assert_eq!(tokens[2].kind, TokenKind::True);
    }
}

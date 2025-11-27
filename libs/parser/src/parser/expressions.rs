//! # Expression Parsing
//!
//! Parses OpenSCAD expressions using precedence climbing.
//!
//! ## Operator Precedence (from grammar.js)
//!
//! | Precedence | Operators | Associativity |
//! |------------|-----------|---------------|
//! | 1 | ?: (ternary) | Right |
//! | 2 | \|\| | Left |
//! | 3 | && | Left |
//! | 4 | == != | Left |
//! | 5 | < > <= >= | Left |
//! | 6 | + - | Left |
//! | 7 | * / % | Left |
//! | 8 | ^ | Left |
//! | 9 | ! (unary) | Right |
//! | 10 | () [] . | Left |

use super::Parser;
use crate::cst::{CstNode, NodeKind};
use crate::error::ParseError;
use crate::lexer::TokenKind;

// =============================================================================
// PRECEDENCE
// =============================================================================

/// Operator precedence levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None = 0,
    Ternary = 1,    // ?:
    Or = 2,         // ||
    And = 3,        // &&
    Equality = 4,   // == !=
    Comparison = 5, // < > <= >=
    Term = 6,       // + -
    Factor = 7,     // * / %
    Power = 8,      // ^
    Unary = 9,      // ! -
    Call = 10,      // () [] .
}

impl Precedence {
    /// Get precedence for binary operator.
    fn of_binary(kind: TokenKind) -> Option<Self> {
        match kind {
            TokenKind::Question => Some(Self::Ternary),
            TokenKind::PipePipe => Some(Self::Or),
            TokenKind::AmpAmp => Some(Self::And),
            TokenKind::EqEq | TokenKind::BangEq => Some(Self::Equality),
            TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => Some(Self::Comparison),
            TokenKind::Plus | TokenKind::Minus => Some(Self::Term),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some(Self::Factor),
            TokenKind::Caret => Some(Self::Power),
            _ => None,
        }
    }

    /// Get next higher precedence level.
    fn next(&self) -> Self {
        match self {
            Self::None => Self::Ternary,
            Self::Ternary => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Equality,
            Self::Equality => Self::Comparison,
            Self::Comparison => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => Self::Power,
            Self::Power => Self::Unary,
            Self::Unary => Self::Call,
            Self::Call => Self::Call,
        }
    }
}

// =============================================================================
// EXPRESSION PARSING
// =============================================================================

impl<'a> Parser<'a> {
    /// Parse an expression.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let expr = parser.parse_expression()?;
    /// ```
    pub(super) fn parse_expression(&mut self) -> Result<CstNode, ParseError> {
        self.parse_precedence(Precedence::Ternary)
    }

    /// Parse expression with minimum precedence.
    ///
    /// Uses precedence climbing algorithm.
    fn parse_precedence(&mut self, min_prec: Precedence) -> Result<CstNode, ParseError> {
        // Parse left-hand side (prefix expression)
        let mut left = self.parse_unary()?;

        // Parse binary operators
        while let Some(prec) = Precedence::of_binary(self.peek_kind()) {
            if prec < min_prec {
                break;
            }

            // Handle ternary operator specially
            if self.peek_kind() == TokenKind::Question {
                left = self.parse_ternary(left)?;
                continue;
            }

            let start = left.span.start;
            let op = self.advance().clone();
            
            // Right associativity for ^ operator
            let next_prec = if op.kind == TokenKind::Caret {
                prec
            } else {
                prec.next()
            };
            
            let right = self.parse_precedence(next_prec)?;

            left = CstNode::with_children(
                NodeKind::BinaryExpression,
                self.span_from(start),
                vec![
                    left,
                    CstNode::with_text(NodeKind::Identifier, op.span, op.text),
                    right,
                ],
            );
        }

        Ok(left)
    }

    /// Parse ternary expression.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// ternary = expression "?" expression ":" expression
    /// ```
    fn parse_ternary(&mut self, condition: CstNode) -> Result<CstNode, ParseError> {
        let start = condition.span.start;
        
        self.expect(TokenKind::Question)?;
        let then_expr = self.parse_expression()?;
        self.expect(TokenKind::Colon)?;
        let else_expr = self.parse_expression()?;

        Ok(CstNode::with_children(
            NodeKind::TernaryExpression,
            self.span_from(start),
            vec![condition, then_expr, else_expr],
        ))
    }

    /// Parse unary expression.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// unary = ("!" | "-" | "+") unary | primary
    /// ```
    fn parse_unary(&mut self) -> Result<CstNode, ParseError> {
        if matches!(self.peek_kind(), TokenKind::Bang | TokenKind::Minus | TokenKind::Plus) {
            let start = self.current_position();
            let op = self.advance().clone();
            let operand = self.parse_unary()?;

            return Ok(CstNode::with_children(
                NodeKind::UnaryExpression,
                self.span_from(start),
                vec![
                    CstNode::with_text(NodeKind::Identifier, op.span, op.text),
                    operand,
                ],
            ));
        }

        self.parse_postfix()
    }

    /// Parse postfix expressions (call, index, dot).
    ///
    /// ## Grammar
    ///
    /// ```text
    /// postfix = primary ("(" args ")" | "[" expr "]" | "." identifier)*
    /// ```
    fn parse_postfix(&mut self) -> Result<CstNode, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek_kind() {
                // Function call
                TokenKind::LParen => {
                    let start = expr.span.start;
                    self.advance();
                    let args = self.parse_arguments()?;
                    self.expect(TokenKind::RParen)?;

                    expr = CstNode::with_children(
                        NodeKind::FunctionCall,
                        self.span_from(start),
                        vec![expr, args],
                    );
                }
                // Index access
                TokenKind::LBracket => {
                    let start = expr.span.start;
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(TokenKind::RBracket)?;

                    expr = CstNode::with_children(
                        NodeKind::IndexExpression,
                        self.span_from(start),
                        vec![expr, index],
                    );
                }
                // Dot access
                TokenKind::Dot => {
                    let start = expr.span.start;
                    self.advance();
                    let name = self.expect(TokenKind::Identifier)?.clone();

                    expr = CstNode::with_children(
                        NodeKind::DotExpression,
                        self.span_from(start),
                        vec![
                            expr,
                            CstNode::with_text(NodeKind::Identifier, name.span, name.text),
                        ],
                    );
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse primary expression.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// primary = number | string | boolean | undef | identifier
    ///         | special_variable | list | range | "(" expression ")"
    /// ```
    fn parse_primary(&mut self) -> Result<CstNode, ParseError> {
        let token = self.peek().clone();
        let start = self.current_position();

        match token.kind {
            // Number
            TokenKind::Number => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::Number, self.span_from(start), token.text))
            }

            // String
            TokenKind::String => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::String, self.span_from(start), token.text))
            }

            // Boolean
            TokenKind::True => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::Boolean, self.span_from(start), "true"))
            }
            TokenKind::False => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::Boolean, self.span_from(start), "false"))
            }

            // Undef
            TokenKind::Undef => {
                self.advance();
                Ok(CstNode::new(NodeKind::Undef, self.span_from(start)))
            }

            // Identifier
            TokenKind::Identifier => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::Identifier, self.span_from(start), token.text))
            }

            // Special variable ($fn, $fa, etc.)
            TokenKind::SpecialVariable => {
                self.advance();
                Ok(CstNode::with_text(NodeKind::SpecialVariable, self.span_from(start), token.text))
            }

            // List or range: [...]
            TokenKind::LBracket => {
                self.parse_list_or_range()
            }

            // Parenthesized expression
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }

            _ => Err(ParseError::unexpected_token(
                &token.text,
                "expression",
            ).with_span(token.span)),
        }
    }

    /// Parse list or range.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// list = "[" (expression ("," expression)*)? "]"
    /// range = "[" expression ":" expression (":" expression)? "]"
    /// ```
    fn parse_list_or_range(&mut self) -> Result<CstNode, ParseError> {
        let start = self.current_position();
        self.expect(TokenKind::LBracket)?;

        // Empty list
        if self.check(TokenKind::RBracket) {
            self.advance();
            return Ok(CstNode::with_children(NodeKind::List, self.span_from(start), vec![]));
        }

        // First element
        let first = self.parse_expression()?;

        // Check for range syntax
        if self.check(TokenKind::Colon) {
            return self.parse_range(start, first);
        }

        // List
        let mut elements = vec![first];
        while self.match_token(TokenKind::Comma) {
            if self.check(TokenKind::RBracket) {
                break; // Trailing comma
            }
            elements.push(self.parse_expression()?);
        }

        self.expect(TokenKind::RBracket)?;
        Ok(CstNode::with_children(NodeKind::List, self.span_from(start), elements))
    }

    /// Parse range.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// range = "[" start ":" end "]"
    ///       | "[" start ":" step ":" end "]"
    /// ```
    fn parse_range(&mut self, start: crate::span::Position, first: CstNode) -> Result<CstNode, ParseError> {
        self.expect(TokenKind::Colon)?;
        let second = self.parse_expression()?;

        let children = if self.check(TokenKind::Colon) {
            self.advance();
            let third = self.parse_expression()?;
            vec![first, second, third] // start : step : end
        } else {
            vec![first, second] // start : end
        };

        self.expect(TokenKind::RBracket)?;
        Ok(CstNode::with_children(NodeKind::Range, self.span_from(start), children))
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use super::*;

    fn parse_expr(source: &str) -> CstNode {
        // Wrap in cube() to make it a valid statement
        let full = format!("x = {};", source);
        let tokens = Lexer::new(&full).tokenize();
        let mut parser = Parser::new(&full, tokens);
        let cst = parser.parse();
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        
        // Get the expression from assignment
        cst.root.children[0].children[1].clone()
    }

    #[test]
    fn test_parse_number() {
        let expr = parse_expr("42");
        assert_eq!(expr.kind, NodeKind::Number);
        assert_eq!(expr.text_or_empty(), "42");
    }

    #[test]
    fn test_parse_float() {
        let expr = parse_expr("3.14");
        assert_eq!(expr.kind, NodeKind::Number);
        assert_eq!(expr.text_or_empty(), "3.14");
    }

    #[test]
    fn test_parse_boolean() {
        let expr = parse_expr("true");
        assert_eq!(expr.kind, NodeKind::Boolean);
    }

    #[test]
    fn test_parse_list() {
        let expr = parse_expr("[1, 2, 3]");
        assert_eq!(expr.kind, NodeKind::List);
        assert_eq!(expr.children.len(), 3);
    }

    #[test]
    fn test_parse_range() {
        let expr = parse_expr("[0:10]");
        assert_eq!(expr.kind, NodeKind::Range);
        assert_eq!(expr.children.len(), 2);
    }

    #[test]
    fn test_parse_range_with_step() {
        let expr = parse_expr("[0:2:10]");
        assert_eq!(expr.kind, NodeKind::Range);
        assert_eq!(expr.children.len(), 3);
    }

    #[test]
    fn test_parse_binary_expression() {
        let expr = parse_expr("1 + 2");
        assert_eq!(expr.kind, NodeKind::BinaryExpression);
        assert_eq!(expr.children.len(), 3);
    }

    #[test]
    fn test_parse_precedence() {
        // 1 + 2 * 3 should be 1 + (2 * 3)
        let expr = parse_expr("1 + 2 * 3");
        assert_eq!(expr.kind, NodeKind::BinaryExpression);
        
        // Left should be 1
        assert_eq!(expr.children[0].kind, NodeKind::Number);
        // Operator should be +
        assert_eq!(expr.children[1].text_or_empty(), "+");
        // Right should be 2 * 3
        assert_eq!(expr.children[2].kind, NodeKind::BinaryExpression);
    }

    #[test]
    fn test_parse_unary() {
        let expr = parse_expr("-5");
        assert_eq!(expr.kind, NodeKind::UnaryExpression);
    }

    #[test]
    fn test_parse_ternary() {
        let expr = parse_expr("x > 0 ? 1 : 0");
        assert_eq!(expr.kind, NodeKind::TernaryExpression);
        assert_eq!(expr.children.len(), 3);
    }

    #[test]
    fn test_parse_function_call() {
        let expr = parse_expr("sin(x)");
        assert_eq!(expr.kind, NodeKind::FunctionCall);
    }

    #[test]
    fn test_parse_index_access() {
        let expr = parse_expr("arr[0]");
        assert_eq!(expr.kind, NodeKind::IndexExpression);
    }
}

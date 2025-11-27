//! # Operator Parsing
//!
//! Parses binary and unary operators using precedence climbing.
//!
//! ## Operator Precedence (from OpenSCAD grammar)
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
//! | 8 | ^ | Right |
//! | 9 | ! - + (unary) | Right |
//!
//! ## Example
//!
//! ```rust,ignore
//! let expr = parser.parse_precedence(Precedence::Ternary)?;
//! ```

use super::Parser;
use crate::cst::{CstNode, NodeKind};
use crate::error::ParseError;
use crate::lexer::TokenKind;

// =============================================================================
// PRECEDENCE
// =============================================================================

/// Operator precedence levels.
///
/// Higher values bind tighter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Precedence {
    /// No precedence (sentinel value)
    None = 0,
    /// Ternary: `?:`
    Ternary = 1,
    /// Logical or: `||`
    Or = 2,
    /// Logical and: `&&`
    And = 3,
    /// Equality: `== !=`
    Equality = 4,
    /// Comparison: `< > <= >=`
    Comparison = 5,
    /// Addition/subtraction: `+ -`
    Term = 6,
    /// Multiplication/division: `* / %`
    Factor = 7,
    /// Power: `^`
    Power = 8,
    /// Unary: `! - +`
    Unary = 9,
    /// Call/access: `() [] .`
    Call = 10,
}

impl Precedence {
    /// Get precedence for binary operator.
    ///
    /// ## Parameters
    ///
    /// - `kind`: Token kind of the operator
    ///
    /// ## Returns
    ///
    /// Precedence level if token is a binary operator, None otherwise
    pub(super) fn of_binary(kind: TokenKind) -> Option<Self> {
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
    ///
    /// Used for left-associative operators.
    pub(super) fn next(&self) -> Self {
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
// OPERATOR PARSING
// =============================================================================

impl<'a> Parser<'a> {
    /// Parse expression with minimum precedence.
    ///
    /// Uses precedence climbing algorithm for efficient parsing.
    ///
    /// ## Parameters
    ///
    /// - `min_prec`: Minimum precedence level to parse
    ///
    /// ## Returns
    ///
    /// Parsed expression node
    pub(super) fn parse_precedence(&mut self, min_prec: Precedence) -> Result<CstNode, ParseError> {
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

            left = self.parse_binary_op(left, prec)?;
        }

        Ok(left)
    }

    /// Parse binary operator.
    fn parse_binary_op(&mut self, left: CstNode, prec: Precedence) -> Result<CstNode, ParseError> {
        let start = left.span.start;
        let op = self.advance().clone();
        
        // Right associativity for ^ operator
        let next_prec = if op.kind == TokenKind::Caret {
            prec
        } else {
            prec.next()
        };
        
        let right = self.parse_precedence(next_prec)?;

        Ok(CstNode::with_children(
            NodeKind::BinaryExpression,
            self.span_from(start),
            vec![
                left,
                CstNode::with_text(NodeKind::Identifier, op.span, op.text),
                right,
            ],
        ))
    }

    /// Parse ternary expression.
    ///
    /// ## Grammar
    ///
    /// ```text
    /// ternary = expression "?" expression ":" expression
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// x > 0 ? 1 : 0
    /// ```
    pub(super) fn parse_ternary(&mut self, condition: CstNode) -> Result<CstNode, ParseError> {
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
    /// unary = ("!" | "-" | "+") unary | postfix
    /// ```
    ///
    /// ## Example
    ///
    /// ```text
    /// -5
    /// !true
    /// +x
    /// ```
    pub(super) fn parse_unary(&mut self) -> Result<CstNode, ParseError> {
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
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::cst::NodeKind;

    fn parse_expr(source: &str) -> crate::cst::CstNode {
        let full = format!("x = {};", source);
        let tokens = Lexer::new(&full).tokenize();
        let mut parser = Parser::new(&full, tokens);
        let cst = parser.parse();
        assert!(cst.errors.is_empty(), "Errors: {:?}", cst.errors);
        cst.root.children[0].children[1].clone()
    }

    #[test]
    fn test_parse_binary_add() {
        let expr = parse_expr("1 + 2");
        assert_eq!(expr.kind, NodeKind::BinaryExpression);
        assert_eq!(expr.children[1].text_or_empty(), "+");
    }

    #[test]
    fn test_parse_binary_precedence() {
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
    fn test_parse_unary_neg() {
        let expr = parse_expr("-5");
        assert_eq!(expr.kind, NodeKind::UnaryExpression);
        assert_eq!(expr.children[0].text_or_empty(), "-");
    }

    #[test]
    fn test_parse_unary_not() {
        let expr = parse_expr("!true");
        assert_eq!(expr.kind, NodeKind::UnaryExpression);
        assert_eq!(expr.children[0].text_or_empty(), "!");
    }

    #[test]
    fn test_parse_ternary() {
        let expr = parse_expr("x > 0 ? 1 : 0");
        assert_eq!(expr.kind, NodeKind::TernaryExpression);
        assert_eq!(expr.children.len(), 3);
    }

    #[test]
    fn test_parse_power_right_assoc() {
        // 2 ^ 3 ^ 4 should be 2 ^ (3 ^ 4)
        let expr = parse_expr("2 ^ 3 ^ 4");
        assert_eq!(expr.kind, NodeKind::BinaryExpression);
        
        // Left should be 2
        assert_eq!(expr.children[0].kind, NodeKind::Number);
        // Right should be 3 ^ 4
        assert_eq!(expr.children[2].kind, NodeKind::BinaryExpression);
    }

    #[test]
    fn test_parse_logical_operators() {
        let expr = parse_expr("true && false || true");
        assert_eq!(expr.kind, NodeKind::BinaryExpression);
        // Should be (true && false) || true
        assert_eq!(expr.children[1].text_or_empty(), "||");
    }
}

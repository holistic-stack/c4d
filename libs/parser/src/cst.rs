//! # Concrete Syntax Tree (CST)
//!
//! CST types representing parsed OpenSCAD source code.
//! Preserves all syntactic details including whitespace positions.
//!
//! ## Example
//!
//! ```rust
//! use openscad_parser::cst::{Cst, CstNode, NodeKind};
//!
//! let cst = openscad_parser::parse("cube(10);");
//! assert_eq!(cst.root.kind, NodeKind::SourceFile);
//! ```

use crate::error::ParseError;
use crate::span::{Span, Spanned};
use serde::{Deserialize, Serialize};

// =============================================================================
// CST
// =============================================================================

/// Concrete Syntax Tree result.
///
/// Contains the root node and any parse errors.
///
/// ## Example
///
/// ```rust
/// let cst = openscad_parser::parse("cube(10);");
/// if cst.errors.is_empty() {
///     println!("Parsed successfully!");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Cst {
    /// Root node of the syntax tree.
    pub root: CstNode,
    /// Parse errors encountered.
    pub errors: Vec<ParseError>,
}

impl Cst {
    /// Create a new CST.
    pub fn new(root: CstNode, errors: Vec<ParseError>) -> Self {
        Self { root, errors }
    }

    /// Check if parsing was successful (no errors).
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

// =============================================================================
// CST NODE
// =============================================================================

/// A node in the Concrete Syntax Tree.
///
/// ## Example
///
/// ```rust
/// let node = CstNode::new(NodeKind::Number, Span::from_bytes(0, 2));
/// assert_eq!(node.kind, NodeKind::Number);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CstNode {
    /// Node type.
    pub kind: NodeKind,
    /// Source span.
    pub span: Span,
    /// Child nodes.
    pub children: Vec<CstNode>,
    /// Text content (for terminals like identifiers, numbers).
    pub text: Option<String>,
}

impl CstNode {
    /// Create a new CST node.
    ///
    /// ## Parameters
    ///
    /// - `kind`: Node type
    /// - `span`: Source location
    pub fn new(kind: NodeKind, span: Span) -> Self {
        Self {
            kind,
            span,
            children: Vec::new(),
            text: None,
        }
    }

    /// Create node with text content.
    ///
    /// ## Parameters
    ///
    /// - `kind`: Node type
    /// - `span`: Source location
    /// - `text`: Text content
    pub fn with_text(kind: NodeKind, span: Span, text: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            children: Vec::new(),
            text: Some(text.into()),
        }
    }

    /// Create node with children.
    ///
    /// ## Parameters
    ///
    /// - `kind`: Node type
    /// - `span`: Source location
    /// - `children`: Child nodes
    pub fn with_children(kind: NodeKind, span: Span, children: Vec<CstNode>) -> Self {
        Self {
            kind,
            span,
            children,
            text: None,
        }
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: CstNode) {
        self.children.push(child);
    }

    /// Get text content, or empty string if none.
    pub fn text_or_empty(&self) -> &str {
        self.text.as_deref().unwrap_or("")
    }

    /// Find first child with given kind.
    pub fn find_child(&self, kind: NodeKind) -> Option<&CstNode> {
        self.children.iter().find(|c| c.kind == kind)
    }

    /// Find all children with given kind.
    pub fn find_children(&self, kind: NodeKind) -> Vec<&CstNode> {
        self.children.iter().filter(|c| c.kind == kind).collect()
    }
}

impl Spanned for CstNode {
    fn span(&self) -> Span {
        self.span
    }
}

// =============================================================================
// NODE KIND
// =============================================================================

/// Types of CST nodes.
///
/// Based on OpenSCAD grammar (grammar.js).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    // Top-level
    /// Root node containing all statements.
    SourceFile,

    // Statements
    /// Module call like `cube(10);`
    ModuleCall,
    /// Variable assignment like `x = 10;`
    Assignment,
    /// Module definition like `module foo() { ... }`
    ModuleDeclaration,
    /// Function definition like `function foo() = ...;`
    FunctionDeclaration,
    /// For loop like `for (i = [0:10]) { ... }`
    ForBlock,
    /// For loop assignments like `i = [0:10], j = [0:5]`
    ForAssignments,
    /// Single for assignment like `i = [0:10]`
    ForAssignment,
    /// If statement like `if (x > 0) { ... }`
    IfBlock,
    /// Let block like `let (x = 1) { ... }`
    LetBlock,
    /// Include statement
    IncludeStatement,
    /// Use statement
    UseStatement,

    // Expressions
    /// Binary operation like `a + b`
    BinaryExpression,
    /// Unary operation like `-x` or `!x`
    UnaryExpression,
    /// Ternary operation like `a ? b : c`
    TernaryExpression,
    /// Function call like `sin(x)`
    FunctionCall,
    /// Index access like `arr[0]`
    IndexExpression,
    /// Dot access like `obj.x`
    DotExpression,
    /// List comprehension like `[for (i = [0:10]) i]`
    ListComprehension,
    /// Range like `[0:10]` or `[0:1:10]`
    Range,
    /// List literal like `[1, 2, 3]`
    List,

    // Terminals
    /// Identifier like `cube` or `myVar`
    Identifier,
    /// Special variable like `$fn` or `$fa`
    SpecialVariable,
    /// Number literal like `10` or `3.14`
    Number,
    /// String literal like `"hello"`
    String,
    /// Boolean literal `true` or `false`
    Boolean,
    /// Undef literal
    Undef,

    // Arguments
    /// Arguments list `(10, center=true)`
    Arguments,
    /// Single argument
    Argument,
    /// Named argument `center=true`
    NamedArgument,

    // Parameters
    /// Parameters list `(x, y=0)`
    Parameters,
    /// Single parameter
    Parameter,

    // Other
    /// Modifier like `*`, `!`, `#`, `%`
    Modifier,
    /// Block of statements `{ ... }`
    Block,
    /// Semicolon
    Semicolon,
    /// Comment
    Comment,
    /// Error node
    Error,
}

impl NodeKind {
    /// Check if this is an expression node.
    pub const fn is_expression(&self) -> bool {
        matches!(
            self,
            Self::BinaryExpression
                | Self::UnaryExpression
                | Self::TernaryExpression
                | Self::FunctionCall
                | Self::IndexExpression
                | Self::DotExpression
                | Self::ListComprehension
                | Self::Range
                | Self::List
                | Self::Identifier
                | Self::SpecialVariable
                | Self::Number
                | Self::String
                | Self::Boolean
                | Self::Undef
        )
    }

    /// Check if this is a statement node.
    pub const fn is_statement(&self) -> bool {
        matches!(
            self,
            Self::ModuleCall
                | Self::Assignment
                | Self::ModuleDeclaration
                | Self::FunctionDeclaration
                | Self::ForBlock
                | Self::IfBlock
                | Self::LetBlock
                | Self::IncludeStatement
                | Self::UseStatement
                | Self::Block
        )
    }

    /// Check if this is a literal node.
    pub const fn is_literal(&self) -> bool {
        matches!(
            self,
            Self::Number | Self::String | Self::Boolean | Self::Undef
        )
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cst_node_new() {
        let node = CstNode::new(NodeKind::Number, Span::from_bytes(0, 2));
        assert_eq!(node.kind, NodeKind::Number);
        assert!(node.children.is_empty());
        assert!(node.text.is_none());
    }

    #[test]
    fn test_cst_node_with_text() {
        let node = CstNode::with_text(NodeKind::Identifier, Span::from_bytes(0, 4), "cube");
        assert_eq!(node.kind, NodeKind::Identifier);
        assert_eq!(node.text_or_empty(), "cube");
    }

    #[test]
    fn test_cst_node_find_child() {
        let mut parent = CstNode::new(NodeKind::ModuleCall, Span::zero());
        parent.add_child(CstNode::with_text(NodeKind::Identifier, Span::zero(), "cube"));
        parent.add_child(CstNode::new(NodeKind::Arguments, Span::zero()));

        assert!(parent.find_child(NodeKind::Identifier).is_some());
        assert!(parent.find_child(NodeKind::Number).is_none());
    }

    #[test]
    fn test_node_kind_is_expression() {
        assert!(NodeKind::Number.is_expression());
        assert!(NodeKind::BinaryExpression.is_expression());
        assert!(!NodeKind::ModuleCall.is_expression());
    }

    #[test]
    fn test_node_kind_is_statement() {
        assert!(NodeKind::ModuleCall.is_statement());
        assert!(NodeKind::Assignment.is_statement());
        assert!(!NodeKind::Number.is_statement());
    }
}

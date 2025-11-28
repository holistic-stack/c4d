//! # AST Types
//!
//! Abstract Syntax Tree node types for OpenSCAD.
//!
//! ## Example
//!
//! ```rust
//! use openscad_ast::ast::{Ast, Statement, Expression};
//!
//! let ast = Ast { statements: vec![] };
//! ```

use openscad_parser::Span;
use serde::{Deserialize, Serialize};

// =============================================================================
// AST
// =============================================================================

/// Abstract Syntax Tree for OpenSCAD source.
///
/// ## Example
///
/// ```rust
/// use openscad_ast::Ast;
/// let ast = Ast { statements: vec![] };
/// assert!(ast.statements.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ast {
    /// Top-level statements.
    pub statements: Vec<Statement>,
}

impl Ast {
    /// Create a new empty AST.
    pub fn new() -> Self {
        Self { statements: Vec::new() }
    }

    /// Create AST with statements.
    pub fn with_statements(statements: Vec<Statement>) -> Self {
        Self { statements }
    }
}

impl Default for Ast {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// STATEMENT
// =============================================================================

/// A statement in OpenSCAD.
///
/// ## Variants
///
/// - `ModuleCall` - Call to a module like `cube(10);`
/// - `Assignment` - Variable assignment like `x = 10;`
/// - `ModuleDeclaration` - Module definition
/// - `FunctionDeclaration` - Function definition
/// - `ForLoop` - For loop
/// - `IfElse` - If/else statement
/// - `Block` - Block of statements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    /// Module call like `cube(10);` or `translate([1,2,3]) cube(5);`
    ModuleCall {
        /// Module name (e.g., "cube", "translate").
        name: String,
        /// Arguments passed to module.
        args: Vec<Argument>,
        /// Child statements (for transforms).
        children: Vec<Statement>,
        /// Source span.
        span: Span,
    },

    /// Variable assignment like `x = 10;`
    Assignment {
        /// Variable name.
        name: String,
        /// Assigned value.
        value: Expression,
        /// Source span.
        span: Span,
    },

    /// Module declaration like `module foo() { ... }`
    ModuleDeclaration {
        /// Module name.
        name: String,
        /// Parameters.
        params: Vec<Parameter>,
        /// Body statements.
        body: Vec<Statement>,
        /// Source span.
        span: Span,
    },

    /// Function declaration like `function foo(x) = x * 2;`
    FunctionDeclaration {
        /// Function name.
        name: String,
        /// Parameters.
        params: Vec<Parameter>,
        /// Body expression.
        body: Expression,
        /// Source span.
        span: Span,
    },

    /// For loop like `for (i = [0:10]) { ... }`
    ForLoop {
        /// Loop variable assignments.
        assignments: Vec<(String, Expression)>,
        /// Body statements.
        body: Vec<Statement>,
        /// Source span.
        span: Span,
    },

    /// If/else statement.
    IfElse {
        /// Condition expression.
        condition: Expression,
        /// Then branch.
        then_body: Vec<Statement>,
        /// Optional else branch.
        else_body: Option<Vec<Statement>>,
        /// Source span.
        span: Span,
    },

    /// Block of statements.
    Block {
        /// Statements in block.
        statements: Vec<Statement>,
        /// Source span.
        span: Span,
    },
}

// =============================================================================
// EXPRESSION
// =============================================================================

/// An expression in OpenSCAD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    /// Number literal like `10` or `3.14`.
    Number(f64),

    /// String literal like `"hello"`.
    String(String),

    /// Boolean literal `true` or `false`.
    Boolean(bool),

    /// Undef value.
    Undef,

    /// Identifier reference like `x` or `myVar`.
    Identifier(String),

    /// Special variable like `$fn` or `$fa`.
    SpecialVariable(String),

    /// List literal like `[1, 2, 3]`.
    List(Vec<Expression>),

    /// Range like `[0:10]` or `[0:1:10]`.
    Range {
        /// Start value.
        start: Box<Expression>,
        /// End value.
        end: Box<Expression>,
        /// Optional step value.
        step: Option<Box<Expression>>,
    },

    /// Binary operation like `a + b`.
    BinaryOp {
        /// Operator.
        op: BinaryOp,
        /// Left operand.
        left: Box<Expression>,
        /// Right operand.
        right: Box<Expression>,
    },

    /// Unary operation like `-x` or `!x`.
    UnaryOp {
        /// Operator.
        op: UnaryOp,
        /// Operand.
        operand: Box<Expression>,
    },

    /// Ternary operation like `a ? b : c`.
    Ternary {
        /// Condition.
        condition: Box<Expression>,
        /// Then value.
        then_expr: Box<Expression>,
        /// Else value.
        else_expr: Box<Expression>,
    },

    /// Function call like `sin(x)`.
    FunctionCall {
        /// Function name.
        name: String,
        /// Arguments.
        args: Vec<Argument>,
    },

    /// Index access like `arr[0]`.
    Index {
        /// Object being indexed.
        object: Box<Expression>,
        /// Index expression.
        index: Box<Expression>,
    },

    /// Dot access like `point.x`.
    Member {
        /// Object.
        object: Box<Expression>,
        /// Member name.
        member: String,
    },
}

// =============================================================================
// ARGUMENT
// =============================================================================

/// An argument to a module or function call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Argument {
    /// Positional argument.
    Positional(Expression),

    /// Named argument like `center=true`.
    Named {
        /// Parameter name.
        name: String,
        /// Argument value.
        value: Expression,
    },
}

// =============================================================================
// PARAMETER
// =============================================================================

/// A parameter in a module or function declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Parameter name.
    pub name: String,
    /// Default value (optional).
    pub default: Option<Expression>,
}

// =============================================================================
// OPERATORS
// =============================================================================

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    /// Addition `+`
    Add,
    /// Subtraction `-`
    Sub,
    /// Multiplication `*`
    Mul,
    /// Division `/`
    Div,
    /// Modulo `%`
    Mod,
    /// Power `^`
    Pow,
    /// Less than `<`
    Lt,
    /// Greater than `>`
    Gt,
    /// Less than or equal `<=`
    Le,
    /// Greater than or equal `>=`
    Ge,
    /// Equal `==`
    Eq,
    /// Not equal `!=`
    Ne,
    /// Logical and `&&`
    And,
    /// Logical or `||`
    Or,
}

impl BinaryOp {
    /// Parse operator from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "+" => Some(Self::Add),
            "-" => Some(Self::Sub),
            "*" => Some(Self::Mul),
            "/" => Some(Self::Div),
            "%" => Some(Self::Mod),
            "^" => Some(Self::Pow),
            "<" => Some(Self::Lt),
            ">" => Some(Self::Gt),
            "<=" => Some(Self::Le),
            ">=" => Some(Self::Ge),
            "==" => Some(Self::Eq),
            "!=" => Some(Self::Ne),
            "&&" => Some(Self::And),
            "||" => Some(Self::Or),
            _ => None,
        }
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// Negation `-`
    Neg,
    /// Logical not `!`
    Not,
    /// Positive `+` (no-op)
    Pos,
}

impl UnaryOp {
    /// Parse operator from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "-" => Some(Self::Neg),
            "!" => Some(Self::Not),
            "+" => Some(Self::Pos),
            _ => None,
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
    fn test_ast_new() {
        let ast = Ast::new();
        assert!(ast.statements.is_empty());
    }

    #[test]
    fn test_binary_op_from_str() {
        assert_eq!(BinaryOp::from_str("+"), Some(BinaryOp::Add));
        assert_eq!(BinaryOp::from_str("=="), Some(BinaryOp::Eq));
        assert_eq!(BinaryOp::from_str("invalid"), None);
    }

    #[test]
    fn test_unary_op_from_str() {
        assert_eq!(UnaryOp::from_str("-"), Some(UnaryOp::Neg));
        assert_eq!(UnaryOp::from_str("!"), Some(UnaryOp::Not));
    }
}

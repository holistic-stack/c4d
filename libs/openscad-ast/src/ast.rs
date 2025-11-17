use crate::span::Span;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ast {
    pub items: Vec<Item>,
    pub span: Span,
    pub context: Context,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Context {
    pub scopes: Vec<Scope>,
    pub specials: SpecialRegistry,
    pub modules: Vec<ModuleSig>,
    pub functions: Vec<FunctionSig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scope {
    pub vars: Vec<VarDecl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleSig {
    pub name: String,
    pub params: Vec<Param>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionSig {
    pub name: String,
    pub params: Vec<Param>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpecialRegistry {
    pub fn_: Option<Expr>,
    pub fa: Option<Expr>,
    pub fs: Option<Expr>,
    pub t: Option<Expr>,
    pub preview: Option<Expr>,
    pub children: Option<Expr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Item {
    ModuleDef(ModuleDef),
    FunctionDef(FunctionDef),
    VarDecl(VarDecl),
    Stmt(Stmt),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleDef {
    pub name: Ident,
    pub params: Vec<Param>,
    pub body: Stmt,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionDef {
    pub name: Ident,
    pub params: Vec<Param>,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VarDecl {
    pub name: Name,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Stmt {
    UnionBlock(Block),
    TransformChain(TransformChain),
    ForBlock(ForBlock),
    IntersectionForBlock(ForBlock),
    IfBlock(IfBlock),
    LetBlock(BodiedBlock),
    AssignBlock(BodiedBlock),
    Include(Include),
    Use(Include),
    AssertStmt(AssertStmt),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub items: Vec<Item>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransformChain {
    pub call: ModuleCall,
    pub tail: Box<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleCall {
    pub name: Ident,
    pub args: Vec<Arg>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Include {
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssertStmt {
    pub args: Vec<Arg>,
    pub stmt: Box<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BodiedBlock {
    pub assigns: Vec<Assignment>,
    pub body: Box<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ForBlock {
    pub binds: ForBinds,
    pub body: Box<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ForBinds {
    Assigns(Vec<Assignment>),
    CondUpdate(Box<CondUpdate>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CondUpdate {
    pub init: Vec<Assignment>,
    pub cond: Box<Expr>,
    pub update: Vec<Assignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IfBlock {
    pub condition: Expr,
    pub consequence: Box<Stmt>,
    pub alternative: Option<Box<Stmt>>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Arg {
    Positional(Expr),
    Named { name: Name, value: Expr },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Param {
    pub name: Name,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Assignment {
    pub name: Name,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Ident(Ident),
    SpecialIdent(SpecialIdent),
    Call { callee: Box<Expr>, args: Vec<Arg>, span: Span },
    Index { value: Box<Expr>, index: Box<Expr>, span: Span },
    DotIndex { value: Box<Expr>, field: Ident, span: Span },
    Unary { op: UnaryOp, expr: Box<Expr>, span: Span },
    Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr>, span: Span },
    Ternary { cond: Box<Expr>, cons: Box<Expr>, alt: Box<Expr>, span: Span },
    Paren { inner: Box<Expr>, span: Span },
    LetExpr { assigns: Vec<Assignment>, body: Box<Expr>, span: Span },
    AssertExpr { args: Vec<Arg>, tail: Option<Box<Expr>>, span: Span },
    EchoExpr { args: Vec<Arg>, tail: Option<Box<Expr>>, span: Span },
    List { cells: Vec<ListCell>, span: Span },
    Range { start: Box<Expr>, inc: Option<Box<Expr>>, end: Box<Expr>, span: Span },
    FunctionLit { params: Vec<Param>, body: Box<Expr>, span: Span },
    Each { value: Box<Expr>, span: Span },
    ListComp { kind: Box<ListCompKind>, span: Span },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ListCell {
    Expr(Expr),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ListCompKind {
    For { binds: Box<ForBinds>, cell: Box<Expr> },
    If { condition: Box<Expr>, consequence: Box<Expr>, alternative: Option<Box<Expr>> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Literal {
    String(String, Span),
    Integer(i64, Span),
    Float(f64, Span),
    Boolean(bool, Span),
    Undef(Span),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpecialIdent {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Name {
    Ident(Ident),
    Special(SpecialIdent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BinaryOp {
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnaryOp {
    Not,
    Pos,
    Neg,
}
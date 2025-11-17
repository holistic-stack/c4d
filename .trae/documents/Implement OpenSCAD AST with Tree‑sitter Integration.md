## Objectives

- Export `libs/openscad-ast` as a Cargo package that parses OpenSCAD into a comprehensive, strongly typed AST.
- Use `libs/openscad-parser/bindings/rust` Tree‑sitter language binding as a dependency to produce a CST, then convert to AST.
- Preserve full source information, maintain scope/context, and implement robust error handling and unit tests.
- Align Rust AST closely with the reference TypeScript AST types in `c:\Users\luciano\git\rust-openscad\.trae\documents\ast.ts`.

## Dependencies and Packaging

- `openscad-parser` Rust binding (at `libs/openscad-parser/bindings/rust`) exposes `fn language() -> tree_sitter::Language` and includes `node-types.json`.
- `openscad-ast/Cargo.toml` dependencies:
  - `tree-sitter = "0.22"`
  - `serde = { version = "1", features = ["derive"] }`
  - `thiserror = "1"`
  - `openscad-parser = { path = "../openscad-parser/bindings/rust" }`
- Export `openscad-ast` as a Cargo package with `lib` target and optional features: `strict` (semantic validation), `serde` (default), `pretty` (round‑trip printing subset).

## AST Design (Rust)

- Base types
  - `Span { start_byte, end_byte, start: Position { row, col }, end: Position { row, col }, text: Option<String> }`
  - `trait AstNode { fn span(&self) -> &Span; fn kind(&self) -> &'static str; }`
  - `enum Ast` root: `Ast { items: Vec<Item>, span: Span, context: Context }`

- Context/state
  - `Context { scopes: Vec<Scope>, specials: SpecialRegistry, modules: ModuleRegistry, functions: FunctionRegistry }`
  - `Scope { vars: HashMap<String, VarDecl> }`
  - `SpecialRegistry { fn_: Option<Expr>, fa: Option<Expr>, fs: Option<Expr>, t: Option<Expr>, preview: Option<Expr>, children: Option<Expr> }` (track, do not evaluate)
  - `ModuleRegistry/FunctionRegistry`: name → signature + span

- Items and declarations
  - `enum Item { ModuleDef(ModuleDef), FunctionDef(FunctionDef), VarDecl(VarDecl), Stmt(Stmt) }`
  - `ModuleDef { name: Ident, params: Vec<Param>, body: Stmt, span }`
  - `FunctionDef { name: Ident, params: Vec<Param>, body: Expr, span }`
  - `Param { name: Name, default: Option<Expr>, span }` where `Name = Ident | SpecialIdent`
  - `VarDecl { name: Name, value: Expr, span }`

- Statements
  - `enum Stmt { UnionBlock(Block), TransformChain(TransformChain), ForBlock(ForBlock), IntersectionForBlock(ForBlock), IfBlock(IfBlock), LetBlock(BodiedBlock), AssignBlock(BodiedBlock), Include(Include), Use(Use), AssertStmt(AssertStmt), Empty }`
  - `Block { items: Vec<Item>, span }`
  - `TransformChain { modifiers: Vec<Modifier>, call: ModuleCall, tail: Box<Stmt>, span }`
  - `ModuleCall { name: Ident, args: Vec<Arg>, span }`
  - `Include/Use { path: String, span }`

- Expressions
  - `enum Expr { Literal(Literal), Ident(Ident), SpecialIdent(SpecialIdent), Call { callee: Box<Expr>, args: Vec<Arg>, span }, Index { value: Box<Expr>, index: Box<Expr>, span }, DotIndex { value: Box<Expr>, field: Ident, span }, Unary { op: UnaryOp, expr: Box<Expr>, span }, Binary { op: BinaryOp, left: Box<Expr>, right: Box<Expr>, span }, Ternary { cond: Box<Expr>, cons: Box<Expr>, alt: Box<Expr>, span }, Paren(Box<Expr>, span), LetExpr { assigns: Vec<Assignment>, body: Box<Expr>, span }, AssertExpr { args: Vec<Arg>, tail: Option<Box<Expr>>, span }, EchoExpr { args: Vec<Arg>, tail: Option<Box<Expr>>, span }, List(Vec<ListCell>, span), Range { start: Box<Expr>, inc: Option<Box<Expr>>, end: Box<Expr>, span }, FunctionLit { params: Vec<Param>, body: Box<Expr>, span } }`
  - `ListCell = Expr | Each | ListComprehension`
  - `Each { value: ExprOrComp, span }` where `ExprOrComp = Expr | ListComprehension`
  - `ListComprehension { ForClause | IfClause }` modeled with dedicated structs

- Literals and args
  - `enum Literal { String(String), Integer(i64), Float(f64), Boolean(bool), Undef }`
  - `enum Arg { Positional(Expr), Named { name: Name, value: Expr } }`
  - `Assignment { name: Name, value: Expr, span }`

- Operators
  - `BinaryOp = || && == != < > <= >= + - * / % ^`
  - `UnaryOp = ! + -`

- Names
  - `Ident { name: String, span }`
  - `SpecialIdent { name: String, span }` (e.g., `$fn`, `$fa`, `$fs`, `$t`, `$preview`, `$children`)

- Boolean operations representation
  - Keep as `TransformChain` with `ModuleCall` (`union`, `difference`, `intersection`, `hull`, `minkowski`) followed by a `Block` tail.

## Parser Integration & Visitor

- Use `tree_sitter::Parser` with `openscad_parser::language()` to parse source into a CST.
- Implement `CursorWalker` over `tree_sitter::TreeCursor` to map CST nodes to AST (`read_item`, `read_stmt`, `read_expr`, `read_args`, `read_block`, etc.).
- Build `Span` from `Node` (`start_byte`, `end_byte`, `point` positions) and include `text` slices from the original source for debugging.
- Implement a classic Visitor pattern:
  - `trait Visitor { fn visit_item(&mut self, item: &Item); fn visit_stmt(&mut self, stmt: &Stmt); fn visit_expr(&mut self, expr: &Expr); /* ... */ }`
  - Provide a default `walk_*` for traversals; downstream stages can implement analyses and transforms.

## Error Handling

- `enum Error { Syntax { message, span }, Semantic { kind: SemanticKind, span, details: Option<String> }, Unsupported { node_type: String, span } }`
- Detect Tree‑sitter `ERROR` nodes and malformed constructs (unterminated lists/ranges, invalid dot index field, empty parens in `for/intersection_for`, malformed assign/let blocks).
- Semantic validation (configurable strict mode): validate ranges, index types, parameter shapes (e.g., `multmatrix` 4×4), and track special variables written in suspicious contexts.
- Return `Result<Ast, Vec<Error>>` or `Result<Ast, Error>` with an internal diagnostics buffer; expose diagnostics accessor.

## Code Preservation

- Never evaluate expressions; store identifiers and references as‐is.
- Maintain original scope nesting; push/pop scopes when entering `let`/blocks/defs.
- Track special variables without evaluation; preserve their assignment expressions.
- Leave function calls and module instantiations unresolved; only record their names, args, and spans.
- Populate `Span.text` with exact source slices for nodes when feasible.

## Output Structure

- `Ast` root returns:
  - `items: Vec<Item>` (all top‑level statements and declarations)
  - `context: Context` (scopes, specials, modules/functions)
  - `span` and `source_map` (derivable from `Span`s)
- Derive `serde` on AST types for storage, codegen, and static analysis.

## Tests

- Co‑located tests under `libs/openscad-ast/src/*_test.rs`:
  - Basic constructs: primitives, variables, includes, literals, transforms.
  - Special variables: `$fn`, `$fa`, `$fs`, `$t`, `$preview`, `$children` in declarations and expressions.
  - Complex nesting: deep transform chains, nested modules, comprehensions, projections.
  - Edge/invalid: syntax errors and semantic errors; verify `Error` spans.
  - Round‑trip: implement a minimal pretty printer for a supported subset; assert parse→print→parse structural equivalence.

## Documentation (README.md)

- Document AST taxonomy; map core OpenSCAD constructs to Rust types.
- Explain spans, source preservation, context/state.
- Describe Visitor usage and extension points.
- Provide examples with input source → AST JSON snapshots.

## Milestones

1. Parser binding consumption: add `openscad-parser/bindings/rust` dependency; sanity parse.
2. AST core types and spans; serde derivations.
3. CST→AST mapper for expressions and calls.
4. Statements and blocks mapping; boolean ops via `TransformChain`.
5. Declarations (modules/functions/vars) with parameters and defaults.
6. Context/state tracking; special variables registry.
7. Error handling and semantic checks.
8. Unit tests (basic → advanced → edge cases); round‑trip printer subset.
9. README.md documentation and examples.

## Acceptance Criteria

- `parse(&str) -> Result<Ast, Error>` builds full AST with spans and context without evaluation.
- Tests cover the requested features and edge cases; round‑trip succeeds on the supported subset.
- Crate is exported and consumable by downstream processing stages.
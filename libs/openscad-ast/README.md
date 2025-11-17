# OpenSCAD AST (Rust)

## Overview

- Parses OpenSCAD source into a Concrete Syntax Tree (CST) using `tree-sitter-openscad`, then builds a strongly typed Abstract Syntax Tree (AST) with full source mapping and context.
- Preserves exact source text and structure, does not evaluate or resolve calls; suitable for static analysis, code generation, and downstream tooling.

## Installation

- Add as a workspace member and depend on it:
  - `libs/openscad-ast/Cargo.toml` depends on `tree-sitter-openscad` via path `../openscad-parser/bindings/rust` and on `tree-sitter` `0.25.10`.
- Binding crate compiles the generated `parser.c` automatically (`libs/openscad-parser/bindings/rust/build.rs`).

## Quick Start

- Parse to AST:
  - `use openscad_ast::parse;`
  - `let ast = parse("translate([1,2,3]) cube(1);")?;`
- Strict parsing with semantic validation:
  - `use openscad_ast::parse_strict;`
  - `let ast = parse_strict("multmatrix([[1,0],[0,1]]) cube(1);")?;` returns `Semantic` error.
- Traverse with Visitor:
  - Implement `openscad_ast::Visitor` and call `visit_item`, `visit_stmt`, `visit_expr` to walk the tree.
- Pretty print and round-trip:
  - `use openscad_ast::print;`
  - `let text = print(&ast); let ast2 = parse(&text)?;`

## API

- `parse(&str) -> Result<Ast, ParseError>`: CST → AST with spans and context.
- `parse_strict(&str) -> Result<Ast, ParseError>`: adds semantic checks (e.g., empty `let`/`for`, invalid `multmatrix`).
- `print(&Ast) -> String`: minimal pretty-printer for common constructs.
- `Visitor` trait: default traversal over items, statements and expressions.

## AST Structure

- Root: `Ast { items: Vec<Item>, span: Span, context: Context }`.
- Context: scopes, special variables registry, module/function registries.
- Items: `ModuleDef`, `FunctionDef`, `VarDecl`, `Stmt`.
- Statements: `UnionBlock`, `TransformChain`, `ForBlock`, `IntersectionForBlock`, `IfBlock`, `LetBlock`, `AssignBlock`, `Include`, `Use`, `AssertStmt`, `Empty`.
- Expressions: `Literal`, `Ident`, `SpecialIdent`, `Call`, `Index`, `DotIndex`, `Unary`, `Binary`, `Ternary`, `Paren`, `LetExpr`, `AssertExpr`, `EchoExpr`, `List`, `Range`, `FunctionLit`, `Each`, `ListComp`.
- Calls/Args: `ModuleCall { name, args, span }`, `Arg = Positional | Named`.
- Declarations: `Param`, `Assignment`, `VarDecl`.
- Names: `Ident`, `SpecialIdent`, `Name`.
- Operators: `BinaryOp`, `UnaryOp`.
- Span: byte offsets, positions, and optional `text` slice.

## Mapping Notes

- Boolean operations (`union`, `difference`, `intersection`, `hull`, `minkowski`) map to `Stmt::TransformChain` with a `UnionBlock` tail for bodies.
- Projections and transforms (e.g., `projection`, `translate`, `rotate`, `scale`, `resize`, `mirror`, `multmatrix`, `color`) are represented as `ModuleCall` names and argument lists.
- List comprehensions and `each` are captured under `Expr::ListComp` and `Expr::Each` with binds/clauses.
- Special variables (`$fn`, `$fa`, `$fs`, `$t`, `$preview`, `$children`) are recorded in `context.specials` upon assignment.

## Error Model

- `ParseError::Syntax { message, span }`: CST includes `ERROR` nodes or malformed syntax.
- `ParseError::Semantic { kind, span, details }`: invalid constructs in strict mode (e.g., empty `let`/`for`, invalid `multmatrix`).
- `ParseError::Unsupported { node_type, span }`: CST node not mapped yet.

## Examples

- Input: `cube(10);`
- AST (conceptual): a `Stmt::TransformChain` with `ModuleCall { name: "cube", args: [Positional(Integer(10))] }` and `tail = Empty`.
- Input: `union() { cube(10); sphere(5); }`
- AST (conceptual): `TransformChain(name = "union")` with tail `UnionBlock(items = [...])` containing two nested `TransformChain`s.

## Tests

- Location: `libs/openscad-ast/tests/`.
- Coverage:
  - Basic constructs and specials (`basic.rs`).
  - Context registry population (`context.rs`).
  - Edge cases and strict mode semantics (`edge_cases.rs`, `semantics.rs`).
  - Round-trip parsing and printing (`roundtrip.rs`).

## Code Pointers

- AST types: `libs/openscad-ast/src/ast.rs`.
- Parser mapper: `libs/openscad-ast/src/parser.rs`.
- Visitor: `libs/openscad-ast/src/visitor.rs`.
- Printer: `libs/openscad-ast/src/printer.rs`.
- Validation: `libs/openscad-ast/src/validate.rs`.
- Binding: `libs/openscad-parser/bindings/rust/lib.rs`.

## Development

- Run tests: `cargo test -p openscad-ast`.
- Suggested workflow:
  - Add new coverage to corpus in `libs/openscad-parser/test/corpus/**`.
  - Map new CST nodes in `parser.rs` and extend AST/Printer/Validation as needed.
  - Add unit tests under `libs/openscad-ast/tests/` mirroring new constructs.
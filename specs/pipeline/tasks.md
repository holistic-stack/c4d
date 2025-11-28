# Rust OpenSCAD Pipeline – Task Breakdown

_Last updated: 2025-11-27 — **Pipeline + Visitor Pattern!** Strict layer dependencies with visitor pattern for tree traversal._

> This file is the **actionable backlog** for the Rust OpenSCAD pipeline.  
> It is structured into small, test-driven tasks and subtasks.  
> See `plan-detailed.md` for goals, architecture, and coding standards.

---

## Pipeline Overview

### Simple Flow (for `cube(10);`)

```
playground ─► wasm ─► openscad-mesh ─► openscad-eval ─► openscad-ast ─► openscad-parser
                                                                              │
                                                                              ▼
                                                                         parse("cube(10);")
                                                                              │
                                                                              ▼
                                                                         Cst (tokens + tree)
                                                                              │
                                                              ◄───────────────┘
                                                              ▼
                                                         Ast (typed statements)
                                                              │
                                              ◄───────────────┘
                                              ▼
                                         EvaluatedAst (resolved geometry)
                                              │
                              ◄───────────────┘
                              ▼
                         Mesh (vertices, indices, normals)
                              │
              ◄───────────────┘
              ▼
         Float32Array / Uint32Array
              │
◄─────────────┘
▼
Three.js BufferGeometry → WebGL Render
```

### Crate Public Interfaces

| Crate | Public Function | Calls | Returns |
|-------|-----------------|-------|---------|
| `openscad-parser` | `parse(source: &str)` | (lexer/parser) | `Cst` |
| `openscad-ast` | `parse(source: &str)` | `openscad_parser::parse()` | `Ast` |
| `openscad-eval` | `evaluate(source: &str)` | `openscad_ast::parse()` | `EvaluatedAst` |
| `openscad-mesh` | `render(source: &str)` | `openscad_eval::evaluate()` | `Mesh` |
| `wasm` | `render(source: &str)` | `openscad_mesh::render()` | `MeshResult` |

### Dependency Chain (Strict)

```
openscad-parser  (no dependencies)
       ▲
openscad-ast     (depends on: openscad-parser)
       ▲
openscad-eval    (depends on: openscad-ast)
       ▲
openscad-mesh    (depends on: openscad-eval)
       ▲
wasm             (depends on: openscad-mesh)
       ▲
playground       (uses: wasm via JS)
```

**Rule: Each layer only calls the layer directly below it. No skipping.**

### Data Structures Summary

| Crate | Output Type | Description |
|-------|-------------|-------------|
| `openscad-parser` | `Cst` | Concrete Syntax Tree (tokens + tree structure + spans) |
| `openscad-ast` | `Ast` | Abstract Syntax Tree (typed statements/expressions) |
| `openscad-eval` | `EvaluatedAst` | Resolved geometry tree (all values computed) |
| `openscad-mesh` | `Mesh` | Triangle mesh (vertices, indices, normals) |
| `wasm` | `MeshResult` | WASM-safe typed arrays (Float32Array, Uint32Array) |

### Visitor Pattern (SRP - One File Per Concern)

Each crate uses visitors for tree traversal. **Complex visitors are broken into subdirectories.**

```
openscad-ast/src/visitor/
├── mod.rs                → CstVisitor trait + public API
├── ast_printer.rs        → AstPrinterVisitor: Ast → String (debug)
└── cst_to_ast/           → CstToAstVisitor (SRP breakdown)
    ├── mod.rs            → Struct + dispatch logic
    ├── statements.rs     → ModuleCall, Assignment, ForLoop, IfElse
    ├── expressions.rs    → Binary, Unary, Literal, List, Range
    └── declarations.rs   → ModuleDefinition, FunctionDefinition

openscad-eval/src/visitor/
├── mod.rs                → AstVisitor trait + public API
├── scope_builder.rs      → ScopeBuilderVisitor: Collect declarations
├── dependency.rs         → DependencyVisitor: Build dependency graph
└── evaluator/            → EvaluatorVisitor (SRP breakdown)
    ├── mod.rs            → Struct + dispatch logic
    ├── context.rs        → EvaluationContext, scopes, variables
    ├── expressions.rs    → Expression evaluation (binary, unary, etc.)
    ├── statements.rs     → Statement evaluation (for, if, let, etc.)
    ├── builtins.rs       → Built-in functions (sin, cos, len, str, etc.)
    └── primitives.rs     → Primitive modules (cube, sphere, cylinder)

openscad-mesh/src/visitor/
├── mod.rs                → GeometryVisitor trait + public API
└── mesh_builder/         → MeshBuilderVisitor (SRP breakdown)
    ├── mod.rs            → Struct + dispatch logic
    ├── primitives.rs     → Cube, Sphere, Cylinder, Polyhedron meshes
    ├── transforms.rs     → Translate, Rotate, Scale, Mirror, Multmatrix
    ├── booleans.rs       → Union, Difference, Intersection (CSG)
    └── extrusions.rs     → LinearExtrude, RotateExtrude
```

**SRP Rule**: Each file handles ONE type of node or ONE category of operations.

---

## 🎯 Priority 1: Pure Rust Parser (libs/openscad-parser)

### Goal

Replace tree-sitter C dependencies with a pure Rust parser, enabling **single WASM output**.

### Design (tree-sitter-inspired)

Based on tree-sitter's `lib/src/` implementation, adapted for pure Rust:

| tree-sitter Component | Pure Rust Equivalent | Purpose |
|-----------------------|---------------------|---------|
| `lexer.c` / `lexer.h` | `lexer.rs` | Character-by-character tokenization |
| `parser.c` / `parser.h` | `parser.rs` | Recursive descent (simpler than GLR) |
| `subtree.c` / `subtree.h` | `cst.rs` | CST nodes with spans |
| `stack.c` / `stack.h` | Not needed | OpenSCAD is LL(k), no ambiguity |
| `grammar.js` | `grammar.rs` | Grammar rules as Rust functions |

### Key Simplifications

1. **Recursive Descent vs GLR** - OpenSCAD is LL(k) compatible, no ambiguity
2. **No External Scanner** - No heredocs or indentation-sensitive syntax
3. **No Incremental Parsing** - Full reparse on change (fast enough for OpenSCAD)
4. **Direct AST** - Can emit AST directly instead of CST → AST conversion

### tree-sitter Source Analysis (`tree-sitter/lib/src/`)

| File | Lines | Purpose | Pure Rust Adaptation |
|------|-------|---------|---------------------|
| `lexer.c` | 484 | Character cursor, lookahead, UTF-8 decode | `Lexer` struct with `Peekable<CharIndices>` |
| `parser.c` | 2263 | GLR parser, shift/reduce actions | Recursive descent (simpler) |
| `subtree.c` | 1100 | Tree nodes, inline/heap allocation | `enum Node` with `Box<>` children |
| `stack.c` | 800 | Parse stack for GLR ambiguity | Not needed (LL(k) grammar) |
| `parser.h` | 287 | Parse actions, lex modes, symbol metadata | `TokenKind` enum, `Span` struct |

**Key tree-sitter concepts to adopt:**
- `TSLexer.lookahead` → `Lexer.peek()`
- `ts_lexer_advance()` → `Lexer.advance()`
- `ts_lexer_mark_end()` → `Lexer.mark_end()`
- `TSPoint` (row, column) → `Position { line, column, byte }`
- `Subtree` (with span) → `Node { kind, span, children }`

### Task Breakdown

#### Phase 1: Lexer (`libs/openscad-parser/src/lexer/`)

| Task | File | Description | Status |
|------|------|-------------|--------|
| 1.1 | `token.rs` | Token enum (all token types from grammar.js) | ⏳ |
| 1.2 | `span.rs` | Source span (byte offset, line, column) | ⏳ |
| 1.3 | `cursor.rs` | Peekable character cursor with position tracking | ⏳ |
| 1.4 | `lexer.rs` | Main lexer: `fn lex(source: &str) -> Vec<Token>` | ⏳ |
| 1.5 | `tests.rs` | Lexer unit tests | ⏳ |

**Token Types (from grammar.js):**
```rust
pub enum TokenKind {
    // Literals
    Integer, Float, String, Boolean, Undef,
    // Identifiers
    Identifier, SpecialVariable,  // $fn, $fa, etc.
    // Keywords
    Module, Function, If, Else, For, Let, Each,
    Include, Use, True, False, Undef,
    // Operators
    Plus, Minus, Star, Slash, Percent, Caret,
    Lt, Gt, Le, Ge, Eq, Ne, And, Or, Not,
    Question, Colon, Semicolon, Comma, Dot,
    // Delimiters
    LParen, RParen, LBracket, RBracket, LBrace, RBrace,
    // Special
    IncludePath,  // <path/to/file.scad>
    Modifier,     // *, !, #, %
    // Meta
    Comment, Whitespace, Eof, Error,
}
```

#### Phase 2: Parser (`libs/openscad-parser/src/parser/`)

| Task | File | Description | Status |
|------|------|-------------|--------|
| 2.1 | `ast.rs` | AST node types (Statement, Expression, etc.) | ⏳ |
| 2.2 | `parser.rs` | Recursive descent parser | ⏳ |
| 2.3 | `expr.rs` | Expression parsing with precedence climbing | ⏳ |
| 2.4 | `stmt.rs` | Statement parsing | ⏳ |
| 2.5 | `error.rs` | Parse errors with spans | ⏳ |
| 2.6 | `tests.rs` | Parser unit tests | ⏳ |

**Grammar Rules (from grammar.js line 124-464):**

```rust
// Top-level
fn parse_source_file(&mut self) -> Vec<Item>;
fn parse_item(&mut self) -> Item;

// Declarations
fn parse_module_item(&mut self) -> ModuleItem;
fn parse_function_item(&mut self) -> FunctionItem;
fn parse_var_declaration(&mut self) -> VarDeclaration;

// Statements
fn parse_statement(&mut self) -> Statement;
fn parse_for_block(&mut self) -> ForBlock;
fn parse_if_block(&mut self) -> IfBlock;
fn parse_let_block(&mut self) -> LetBlock;
fn parse_transform_chain(&mut self) -> TransformChain;
fn parse_module_call(&mut self) -> ModuleCall;

// Expressions (precedence climbing)
fn parse_expression(&mut self) -> Expression;
fn parse_binary_expr(&mut self, min_prec: u8) -> Expression;
fn parse_unary_expr(&mut self) -> Expression;
fn parse_primary_expr(&mut self) -> Expression;
fn parse_literal(&mut self) -> Literal;
fn parse_list(&mut self) -> List;
fn parse_range(&mut self) -> Range;
```

**Operator Precedence (from grammar.js line 358-372):**

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 | `? :` (ternary) | Right |
| 2 | `\|\|` | Left |
| 3 | `&&` | Left |
| 4 | `==` `!=` | Left |
| 5 | `<` `>` `<=` `>=` | Left |
| 6 | `+` `-` | Left |
| 7 | `*` `/` `%` | Left |
| 8 | `^` | Left |
| 9 | `!` (unary) | Right |
| 10 | `()` `[]` `.` (call/index) | Left |

#### Phase 3: Integration

| Task | Description | Status |
|------|-------------|--------|
| 3.1 | Public API: `parse(source: &str) -> ParseResult` | ⏳ |
| 3.2 | Update `libs/wasm` to use pure Rust parser | ⏳ |
| 3.3 | Remove web-tree-sitter from playground | ⏳ |
| 3.4 | Single WASM build verification | ⏳ |

---

## ✅ Full Pipeline Complete (2025-11-27)

### Current Architecture (Pure Rust)

```text
OpenSCAD Source ("cube(10);")
      ↓
[Rust WASM] render(source) - Full pipeline in pure Rust
  ├─ openscad-parser: Lexer + Parser → CST
  ├─ openscad-ast: CST → AST transformation
  ├─ openscad-eval: AST → GeometryNode evaluation
  └─ openscad-mesh: GeometryNode → Mesh generation
      ↓ (Mesh Data: vertices, indices, normals)
[JavaScript] Three.js WebGL
```

### Build & Run

```bash
# Build WASM (from playground directory)
cd apps/playground
pnpm run build:wasm

# Start playground
pnpm dev
# Opens http://localhost:5173/
```

### Verified Features

| Feature | Status | Notes |
|---------|--------|-------|
| `cube(size)` | ✅ | 24 vertices, 12 triangles |
| `sphere(r)` | ✅ | 496 vertices, 840 triangles |
| `cylinder(h, r)` | ✅ | With r1/r2 for cones |
| `translate([x,y,z])` | ✅ | Working |
| `rotate([x,y,z])` | ✅ | Working |
| `scale([x,y,z])` | ✅ | Working |
| `union() { ... }` | ✅ | BSP-based |
| `difference() { ... }` | ✅ | BSP-based |
| `intersection() { ... }` | ✅ | BSP-based |
| `$fn/$fa/$fs` | ✅ | Resolution parameters |
| `x = 10;` | ✅ | Variable assignment |
| Lexical scoping | ✅ | Block-level scoping |
| `for (i=[0:10])` | ✅ | For loop iteration |
| `if/else` | ✅ | Conditional geometry |
| Pure Rust parser | ✅ | No tree-sitter dependency |
| Single WASM output | ✅ | ~160KB optimized |
| SRP refactoring | ✅ | Parser (9 modules) + AST (8 modules) + Evaluator (6 modules) |
| `mirror([x,y,z])` | ✅ | Reflection transform |
| `color([r,g,b,a])` | ✅ | Color modifier |
| `function name(params) = expr` | ✅ | User-defined functions |
| `module name(params) { ... }` | ✅ | User-defined modules |
| `children()` | ✅ | Module children access |
| `hull()` | ✅ | QuickHull (correct shape, may differ in triangle count) |
| `minkowski()` | ✅ | Minkowski sum via vertex sums + hull |
| `polyhedron()` | ✅ | Custom mesh primitive |
| `circle()` | ✅ | 2D circle primitive |
| `square()` | ✅ | 2D rectangle primitive |
| `polygon()` | ✅ | 2D polygon primitive (fan triangulation) |
| `linear_extrude()` | ✅ | 2D to 3D with height, twist, scale |
| `rotate_extrude()` | ✅ | 2D to 3D rotation around Z |
| **SRP Mesh Builder** | ✅ | Split into 6 modules (<400 lines each) |
| `offset()` | ✅ | 2D polygon expand/shrink |
| `projection()` | ✅ | 3D to 2D projection |
| **363 tests passing** | ✅ | Full workspace |

---

## 🔮 Next Steps

| Priority | Task | Description |
|----------|------|-------------|
| ~~1~~ | ~~**Mirror**~~ | ✅ mirror([x,y,z]) transform - DONE |
| ~~2~~ | ~~**Color**~~ | ✅ color([r,g,b,a]) modifier - DONE |
| ~~3~~ | ~~**User-defined Functions**~~ | ✅ function name(params) = expr; - DONE |
| ~~4~~ | ~~**User-defined Modules**~~ | ✅ module name(params) { ... } - DONE |
| ~~5~~ | ~~**Hull/Minkowski**~~ | ✅ QuickHull + Minkowski sum - DONE |
| ~~6~~ | ~~**Polyhedron**~~ | ✅ Custom mesh support - DONE |
| ~~7~~ | ~~**2D primitives**~~ | ✅ circle, square, polygon - DONE |
| ~~8~~ | ~~**Extrusions**~~ | ✅ linear_extrude, rotate_extrude - DONE |
| ~~9~~ | ~~**SRP Refactor**~~ | ✅ Mesh builder split to 6 modules - DONE |
| ~~10~~ | ~~**offset()**~~ | ✅ 2D offset/inset operation - DONE |
| ~~11~~ | ~~**projection()**~~ | ✅ 3D to 2D projection - DONE |
| 1 | **import()** | STL/SVG file import |
| 2 | **text()** | 2D text shapes |

---

## Feature Roadmap

### Phase 1: Core Pipeline ✅ COMPLETE

| Feature | Status | Notes |
|---------|--------|-------|
| Pure Rust parser | ✅ | Lexer + recursive descent |
| CST → AST transformation | ✅ | Visitor pattern |
| AST evaluation | ✅ | GeometryNode IR |
| Mesh generation | ✅ | Primitives + transforms |
| WASM integration | ✅ | render(source) API |
| Three.js rendering | ✅ | Z-up, orbit controls |

### Phase 2: Primitives & Transforms ✅ COMPLETE

| Feature | Status | Notes |
|---------|--------|-------|
| cube(size, center) | ✅ | Working |
| sphere(r\|d, $fn) | ✅ | 100% OpenSCAD compatible tessellation |
| cylinder(h, r1, r2, $fn) | ✅ | With cone support |
| translate | ✅ | Working |
| rotate | ✅ | Working |
| scale | ✅ | Working |
| mirror | ✅ | Reflection transform |
| color modifier | ✅ | RGBA color support |

### Phase 3: Boolean Operations ✅ COMPLETE

| Feature | Status | Notes |
|---------|--------|-------|
| union() | ✅ | BSP-based |
| difference() | ✅ | BSP-based |
| intersection() | ✅ | BSP-based |
| hull() | ✅ | QuickHull algorithm |
| minkowski() | ✅ | Vertex sum + hull |

### Phase 4: Variables & Functions ✅ COMPLETE

| Feature | Status | Notes |
|---------|--------|-------|
| Variable assignment | ✅ | x = 10; |
| $fn/$fa/$fs params | ✅ | Resolution calculation |
| Lexical scoping | ✅ | Block-level scoping |
| For loops | ✅ | for(i=[0:10]) |
| If/else | ✅ | Conditional geometry |
| User functions | ✅ | function name(params) = expr; |
| User modules | ✅ | module name(params) { ... } |

### Phase 5: Advanced Features

| Feature | Status | Notes |
|---------|--------|-------|
| linear_extrude | ⏳ | twist, scale, slices |
| rotate_extrude | ⏳ | angle, segments |
| polyhedron | ⏳ | Custom mesh |
| 2D primitives | ⏳ | circle, square, polygon |

---

## Design Principles

### Browser Safety

- Pure Rust parser (no C dependencies)
- NO WASI or file system access
- Single WASM output file
- Zero external runtime dependencies

### Algorithm Selection

- tree-sitter-inspired lexer/parser architecture
- Recursive descent parsing (LL(k) grammar)
- Manifold-style algorithms for CSG (intersection-based)
- OpenSCAD-compatible API and output

### Code Standards

- TDD with small, focused tests
- SRP: Each module has one responsibility
- DRY: No code duplication
- KISS: Simple solutions first
- Files under 500 lines
- Comprehensive documentation

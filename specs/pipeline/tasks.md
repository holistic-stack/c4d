# Rust OpenSCAD Pipeline – Task Breakdown

_Last updated: 2025-11-28 — **Manifold-RS Migration Complete!** Deleted `libs/openscad-mesh`, now using `libs/manifold-rs` exclusively. Full Rust port of Manifold-3D algorithms. QuickHull convex hull, Minkowski sum, exact CSG booleans. OpenSCAD $fn/$fa/$fs compatibility. 102+ unit tests passing. Browser-safe WASM._

> This file is the **actionable backlog** for the Rust OpenSCAD pipeline.  
> It is structured into small, test-driven tasks and subtasks.  
> See `plan-detailed.md` for goals, architecture, and coding standards.

---

## Pipeline Overview

### Simple Flow (for `cube(10);`)

```
playground ─► wasm ─► manifold-rs ─► openscad-eval ─► openscad-ast ─► openscad-parser
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
| `manifold-rs` | `render(source: &str)` | `openscad_eval::evaluate()` | `Mesh` |
| `wasm` | `render(source: &str)` | `manifold_rs::render()` | `MeshResult` |

### Dependency Chain (Strict)

```
openscad-parser  (no dependencies)
       ▲
openscad-ast     (depends on: openscad-parser)
       ▲
openscad-eval    (depends on: openscad-ast)
       ▲
manifold-rs      (depends on: openscad-eval)
       ▲
wasm             (depends on: manifold-rs)
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
| `manifold-rs` | `Mesh` | Triangle mesh via Manifold (vertices, indices, normals) |
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

manifold-rs/src/
├── lib.rs                → Public API: render(source) -> Mesh
├── manifold/             → 3D Solid Operations (Manifold-3D port)
│   ├── mod.rs            → Manifold struct + methods
│   ├── boolean3.rs       → Union, Difference, Intersection (exact)
│   ├── constructors.rs   → Cube, Sphere, Cylinder, Tetrahedron
│   ├── csg_tree.rs       → CSG tree evaluation and optimization
│   └── impl.rs           → Core Manifold implementation
├── cross_section/        → 2D Polygon Operations (CrossSection)
│   ├── mod.rs            → CrossSection struct + methods
│   ├── offset.rs         → Polygon offset/inset
│   └── boolean.rs        → 2D union/diff/intersection
├── mesh/                 → Output Mesh Format
│   ├── mod.rs            → Mesh struct (vertices, indices, normals)
│   └── halfedge.rs       → HalfEdge mesh representation
├── openscad/             → OpenSCAD Compatibility Wrapper
│   ├── mod.rs            → OpenSCAD API compatibility layer
│   ├── segments.rs       → $fn/$fa/$fs → circularSegments converter
│   └── from_ir.rs        → GeometryNode → Manifold conversion
└── gpu/                  → WebGPU Acceleration (Optional)
    ├── mod.rs            → GPU context and mode selection
    ├── sdf.rs            → SDF-based CSG compute shaders
    └── marching_cubes.rs → Mesh extraction from SDF
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

## ✅ Full Pipeline Complete (2025-11-28)

### Current Architecture (Pure Rust + Manifold-RS)

```text
OpenSCAD Source ("cube(10);")
      ↓
[Rust WASM] render(source) - Full pipeline in pure Rust
  ├─ openscad-parser: Lexer + Parser → CST
  ├─ openscad-ast: CST → AST transformation
  ├─ openscad-eval: AST → GeometryNode evaluation
  └─ manifold-rs: GeometryNode → Manifold → Mesh
      │ ├─ OpenSCAD wrapper: $fn/$fa/$fs → circularSegments
      │ ├─ Manifold: 3D solid operations (exact CSG)
      │ └─ CrossSection: 2D polygon operations
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
| `union() { ... }` | ✅ | Manifold exact |
| `difference() { ... }` | ✅ | Manifold exact |
| `intersection() { ... }` | ✅ | Manifold exact |
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
| `hull()` | ✅ | QuickHull algorithm - correct convex hull with horizon edge finding |
| `minkowski()` | ✅ | Minkowski sum via pairwise vertex sums + QuickHull |
| `polyhedron()` | ✅ | Custom mesh primitive |
| `circle()` | ✅ | 2D circle primitive |
| `square()` | ✅ | 2D rectangle primitive |
| `polygon()` | ✅ | 2D polygon primitive (fan triangulation) |
| `linear_extrude()` | ✅ | 2D to 3D with height, twist, scale |
| `rotate_extrude()` | ✅ | 2D to 3D rotation around Z |
| **Manifold-RS** | ✅ | Full Manifold-3D port with OpenSCAD wrapper |
| `offset()` | ✅ | 2D polygon expand/shrink |
| `projection()` | ✅ | 3D to 2D projection |
| `rotate_extrude()` | ✅ | Fixed missing faces and incorrect normals |
| **HalfEdgeMesh** | ✅ | Manifold-style compact half-edge mesh structure |
| **470+ tests passing** | ✅ | Full workspace (57 manifold-rs + parser + ast + eval + wasm) |

---

## 🚀 Phase 10: Manifold-RS (Full Manifold-3D Port)

### Overview

Replace `libs/openscad-mesh` with `libs/manifold-rs` - a complete Rust port of Manifold-3D with:
- **Manifold**: 3D solid operations (exact CSG)
- **CrossSection**: 2D polygon operations
- **OpenSCAD Wrapper**: $fn/$fa/$fs → circularSegments compatibility
- **CPU + WebGPU modes**: Toggle between exact CPU and fast GPU processing

### Source Reference

Port from `manifold/src/` (C++) to `libs/manifold-rs/src/` (Rust):

| C++ Source | Rust Target | Description |
|------------|-------------|-------------|
| `manifold.cpp` | `manifold/mod.rs` | Main Manifold struct |
| `constructors.cpp` | `manifold/constructors.rs` | Cube, Sphere, Cylinder, etc. |
| `boolean3.cpp` | `manifold/boolean3.rs` | Union, Difference, Intersection |
| `csg_tree.cpp` | `manifold/csg_tree.rs` | CSG tree optimization |
| `impl.cpp` | `manifold/impl.rs` | Core implementation |
| `cross_section/*.cpp` | `cross_section/` | 2D operations |
| `polygon.cpp` | `cross_section/polygon.rs` | Polygon triangulation |
| `quickhull.cpp` | `manifold/quickhull.rs` | Convex hull |
| `sdf.cpp` | `gpu/sdf.rs` | SDF operations (for GPU) |

### Task Breakdown

#### Phase 10.1: Core Data Structures

| ID | Task | Details | Status |
|----|------|---------|--------|
| 1 | **Mesh struct** | Port `Mesh` with vertices, indices, normals, properties | ⏳ |
| 2 | **HalfEdgeMesh** | Port compact half-edge mesh (impl.h) | ✅ EXISTS |
| 3 | **BoundingBox** | Port AABB with Morton codes | ⏳ |
| 4 | **Properties** | Generic vertex property channels | ⏳ |

#### Phase 10.2: Manifold Constructors (OpenSCAD Compatible)

| ID | Task | Details | Status |
|----|------|---------|--------|
| 1 | **cube()** | `Manifold::cube(size, center)` | ⏳ |
| 2 | **sphere()** | `Manifold::sphere(radius, circularSegments)` - geodesic | ⏳ |
| 3 | **cylinder()** | `Manifold::cylinder(h, r1, r2, segments, center)` | ⏳ |
| 4 | **tetrahedron()** | `Manifold::tetrahedron()` | ⏳ |
| 5 | **of_mesh()** | `Manifold::of_mesh(&Mesh)` | ⏳ |

#### Phase 10.3: Boolean Operations (Exact)

| ID | Task | Details | Status |
|----|------|---------|--------|
| 1 | **union()** | `manifold.union(&other)` | ⏳ |
| 2 | **subtract()** | `manifold.subtract(&other)` (difference) | ⏳ |
| 3 | **intersect()** | `manifold.intersect(&other)` | ⏳ |
| 4 | **hull()** | `manifold.hull()` | ⏳ |
| 5 | **Collider** | BVH spatial index for edge-face tests | ✅ EXISTS |
| 6 | **Kernel12** | Edge-face intersection | ✅ EXISTS |
| 7 | **Boolean3** | Winding-number classification | ✅ EXISTS |

#### Phase 10.4: Transforms

| ID | Task | Details | Status |
|----|------|---------|--------|
| 1 | **translate()** | `manifold.translate([x, y, z])` | ⏳ |
| 2 | **rotate()** | `manifold.rotate([x, y, z])` (degrees) | ⏳ |
| 3 | **scale()** | `manifold.scale([x, y, z])` | ⏳ |
| 4 | **mirror()** | `manifold.mirror([nx, ny, nz])` | ⏳ |
| 5 | **transform()** | `manifold.transform(mat4x4)` | ⏳ |

#### Phase 10.5: CrossSection (2D Operations)

| ID | Task | Details | Status |
|----|------|---------|--------|
| 1 | **circle()** | `CrossSection::circle(r, segments)` | ⏳ |
| 2 | **square()** | `CrossSection::square(size, center)` | ⏳ |
| 3 | **of_polygons()** | `CrossSection::of_polygons(...)` | ⏳ |
| 4 | **union/subtract/intersect** | 2D boolean ops | ⏳ |
| 5 | **offset()** | Polygon offset with JoinType | ⏳ |
| 6 | **hull()** | 2D convex hull | ⏳ |

#### Phase 10.6: Extrusions

| ID | Task | Details | Status |
|----|------|---------|--------|
| 1 | **extrude()** | `cross_section.extrude(height, nDivisions, twist, scale)` | ⏳ |
| 2 | **revolve()** | `cross_section.revolve(segments, degrees)` | ⏳ |

#### Phase 10.7: OpenSCAD Compatibility Wrapper

| ID | Task | Details | Status |
|----|------|---------|--------|
| 1 | **SegmentParams** | $fn/$fa/$fs → circularSegments conversion | ⏳ |
| 2 | **from_ir.rs** | GeometryNode → Manifold conversion | ⏳ |
| 3 | **render()** | Public API: `render(source) -> Mesh` | ⏳ |

**Segment Calculation Formula (OpenSCAD exact):**
```rust
/// max($fn, ceil(360/$fa), ceil(2*PI*r/$fs))
fn calculate_segments(fn_: Option<u32>, fa: f64, fs: f64, radius: f64) -> u32 {
    if let Some(fn_) = fn_ { if fn_ > 0 { return fn_; } }
    let from_fa = (360.0 / fa).ceil() as u32;
    let from_fs = (2.0 * PI * radius / fs).ceil() as u32;
    from_fa.max(from_fs).max(3)
}
```

#### Phase 10.8: WebGPU Mode (Optional)

| ID | Task | Details | Status |
|----|------|---------|--------|
| 1 | **wgpu Setup** | Initialize WebGPU in WASM | ⏳ |
| 2 | **SDF Compute** | WGSL shaders for CSG | ⏳ |
| 3 | **Marching Cubes** | Extract mesh from SDF | ⏳ |
| 4 | **CsgMode Toggle** | CPU/GPU mode selection | ⏳ |

### Migration Steps

1. **Create `libs/manifold-rs/`** with Cargo.toml
2. **Port core structures** (Mesh, HalfEdgeMesh, BoundingBox)
3. **Port constructors** (cube, sphere, cylinder)
4. **Port boolean3** (union, subtract, intersect)
5. **Port transforms** (translate, rotate, scale, mirror)
6. **Create OpenSCAD wrapper** (segments.rs, from_ir.rs)
7. **Update `libs/wasm`** to use manifold-rs
8. **Delete `libs/openscad-mesh`**
9. **Run all tests** and verify playground

### API Comparison

| OpenSCAD | Manifold-RS |
|----------|-------------|
| `cube(10)` | `Manifold::cube([10.0, 10.0, 10.0], false)` |
| `sphere(5, $fn=32)` | `Manifold::sphere(5.0, 32)` |
| `cylinder(h=10, r=5)` | `Manifold::cylinder(10.0, 5.0, 5.0, segments, false)` |
| `union() { a; b; }` | `a.union(&b)` |
| `difference() { a; b; }` | `a.subtract(&b)` |
| `intersection() { a; b; }` | `a.intersect(&b)` |
| `linear_extrude(h) circle(r)` | `CrossSection::circle(r, seg).extrude(h, 0, 0.0, [1.0, 1.0])` |
| `rotate_extrude() square(s)` | `CrossSection::square(s, false).revolve(seg, 360.0)` |

---

## 🔮 Next Steps

### ✅ Completed
| Feature | Description |
|---------|-------------|
| **Mirror** | mirror([x,y,z]) transform |
| **Color** | color([r,g,b,a]) modifier |
| **User-defined Functions** | function name(params) = expr; |
| **User-defined Modules** | module name(params) { ... } with children() |
| **Hull** | QuickHull convex hull algorithm |
| **Minkowski** | Minkowski sum via vertex sums + hull |
| **Polyhedron** | Custom mesh primitive |
| **2D Primitives** | circle, square, polygon |
| **Extrusions** | linear_extrude (twist, scale), rotate_extrude |
| **offset()** | 2D polygon expand/shrink |
| **projection()** | 3D to 2D projection |
| **openscad-mesh Deleted** | Migrated to manifold-rs exclusively |

### 🚀 Next Priority
| Priority | Task | Description | Browser-Safe Crate |
|----------|------|-------------|-------------------|
| 1 | **import("file.stl")** | STL file import for 3D meshes | `nom_stl` (pure Rust, nom-based) |
| 2 | **import("file.svg")** | SVG file import for 2D shapes | `usvg` (pure Rust, WASM-safe) |
| 3 | **text()** | 2D text shapes from fonts | `fontdue` (pure Rust font rasterizer) |
| 4 | **WebGPU Mode** | GPU-accelerated CSG via wgpu | `wgpu` (WebGPU in Rust) |
| 5 | **resize()** | Auto-size geometry | Built-in (bounding box) |
| 6 | **surface()** | Height map import | Custom (image parsing) |

---

### Phase 8: Boolean Engine Improvements

**✅ FIXED**: The `intersection()`, `difference()`, and `union()` operations now work robustly using a hybrid approach:
1. **Intersection**: `(A inside B) U (B inside A)`
2. **Difference**: `(A outside B) U (B inside A reversed)`
3. **Union**: `(A outside B) U (B outside A)`

All use robust point-in-mesh voting for leaf classification, handling boundary cases correctly.

| ID | Task | Details | Status |
|----|------|---------|--------|
| 1 | **Intersection Fix** | Implement robust point-in-mesh and hybrid BSP logic | ✅ DONE |
| 2 | **Difference Fix** | Implement `A-B` logic with normal flipping | ✅ DONE |
| 3 | **Union Fix** | Implement `A U B` logic for robust merging | ✅ VERIFIED |
| 4 | **BSP Optimization** | Vertex welding + polygon merging. BSP: ~620 vertices/~1450 tris (OpenSCAD: 506/1008) | ✅ DONE |
| 5 | **Sphere Compat** | Match OpenSCAD's Lat-Lon triangulation (split diagonal caps) | ✅ VERIFIED |
| 6 | **Manifold Algorithm** | Replace BSP with edge-intersection algorithm for 100% OpenSCAD mesh parity | ⏳ FUTURE |

#### BSP vs Manifold Analysis

**Current BSP Implementation** produces ~44% more triangles than OpenSCAD/Manifold because:
- BSP splits triangles along arbitrary planes (every triangle plane from the other mesh)
- Creates excessive fragmentation that polygon merging can only partially recover

**Manifold's Algorithm** (from `boolean3.cpp`) uses:
- Edge-edge intersection computation with exact predicates
- Only splits triangles along actual intersection curves
- Minimal mesh growth (~10% overhead)

**To achieve 100% OpenSCAD parity**, need to implement Manifold's approach:
1. Halfedge mesh representation
2. Edge-edge intersection with robust predicates
3. Sweep-line algorithm for finding intersections
4. Triangle re-meshing along intersection curves only

### Phase 9: WebGPU CSG (GPU - Parallel Acceleration)

| ID | Task | Details | Status |
|----|------|---------|--------|
| 1 | **wgpu Setup** | Initialize WebGPU device/adapter in WASM | ⏳ |
| 2 | **Mesh Buffers** | GPU storage buffers for vertices/indices | ⏳ |
| 3 | **SDF Compute** | WGSL compute shader for SDF operations | ⏳ |
| 4 | **Voxelization** | Mesh to voxel grid conversion shader | ⏳ |
| 5 | **CSG Ops** | Union/Diff/Intersect via min/max SDF | ⏳ |
| 6 | **Mesh Extract** | Marching cubes to extract result mesh | ⏳ |
| 7 | **Toggle UI** | CPU/GPU CSG mode selector in playground | ⏳ |

**Algorithm**: SDF-based CSG on GPU
- Union: `min(sdf_a, sdf_b)`
- Intersection: `max(sdf_a, sdf_b)`
- Difference: `max(sdf_a, -sdf_b)`

**Key crates**: `wgpu`, `bytemuck`, `web-sys` (WebGPU bindings)

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
| union() | ✅ | Manifold exact |
| difference() | ✅ | Manifold exact |
| intersection() | ✅ | Manifold exact |
| hull() | ✅ | Manifold hull |
| minkowski() | ✅ | Manifold minkowski |

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
- Manifold-3D algorithms for CSG (full Rust port)
- OpenSCAD-compatible API via wrapper ($fn/$fa/$fs → circularSegments)
- CPU + WebGPU modes

### Code Standards

- TDD with small, focused tests
- SRP: Each module has one responsibility
- DRY: No code duplication
- KISS: Simple solutions first
- Files under 500 lines
- Comprehensive documentation

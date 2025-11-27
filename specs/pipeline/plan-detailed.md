# Rust OpenSCAD Pipeline – Overview Plan

_Last updated: 2025-11-27 — **Pipeline Architecture + Visitor Pattern!** Clear layer-by-layer pipeline with strict dependencies. Each crate uses visitors for tree traversal (dedicated files per visitor)._

> This document is the high-level source of truth for the Rust OpenSCAD pipeline. It describes **goals**, **architecture**, and **standards**. See `tasks.md` in the same folder for the detailed, phase-by-phase backlog.

---

## 1. Project Goal

Create a complete, robust, and performant **OpenSCAD-to-3D-Mesh pipeline** in Rust, targeting WebAssembly for a browser-based playground.

The system must:

- **Support real-time compilation** for interactive editing.
- **Run fully in the browser** via WASM (browser-safe Rust crates and code only).
- **Avoid unnecessary copies** between WASM and JS (zero-copy mesh transfer).
- **Provide precise source mapping** from errors and geometry back to OpenSCAD source.
- **100% OpenSCAD API Compatibility**: Public API mirrors OpenSCAD expectations (parameters, output shapes) using best-in-class 3D/2D algorithms for mesh generation and operations.
- **Best Algorithms for Mesh Operations**: Use proven, browser-safe algorithms (BSP trees for CSG, ear clipping for triangulation, etc.) that deliver correct results with OpenSCAD-compatible output.

### 1.0.1 Pipeline Architecture

The pipeline flows through 5 Rust crates. **Each layer only calls the layer directly below it.**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        RUST OPENSCAD PIPELINE                               │
│                                                                             │
│   Example: cube(10);                                                        │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  apps/playground (TypeScript + Three.js)                            │   │
│   │  ├─ User types: cube(10);                                           │   │
│   │  ├─ Calls: wasm.render("cube(10);")                                 │   │
│   │  ├─ Receives: mesh data (vertices, indices, normals)                │   │
│   │  └─ Renders: Three.js BufferGeometry                                │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│          │ render("cube(10);")                                              │
│          ▼                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  libs/wasm (Thin WASM API layer)                                    │   │
│   │  ├─ Public: render(source: &str) -> MeshResult                      │   │
│   │  ├─ Calls: openscad_mesh::render(source)                            │   │
│   │  └─ Returns: Zero-copy Float32Array to JS                           │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│          │ openscad_mesh::render("cube(10);")                               │
│          ▼                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  libs/openscad-mesh (Mesh Generation)                               │   │
│   │  ├─ Public: render(source: &str) -> Mesh                            │   │
│   │  ├─ Calls: openscad_eval::evaluate(source)                          │   │
│   │  ├─ Receives: EvaluatedAst (flattened geometry tree)                │   │
│   │  ├─ Generates: Primitives, Transforms, Booleans, Extrusions         │   │
│   │  └─ Returns: Mesh { vertices, indices, normals, colors }            │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│          │ openscad_eval::evaluate("cube(10);")                             │
│          ▼                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  libs/openscad-eval (AST Evaluation)                                │   │
│   │  ├─ Public: evaluate(source: &str) -> EvaluatedAst                  │   │
│   │  ├─ Calls: openscad_ast::parse(source)                              │   │
│   │  ├─ Receives: Ast (unevaluated syntax tree)                         │   │
│   │  ├─ Evaluates: Variables, functions, loops, conditionals            │   │
│   │  ├─ Resolves: $fn/$fa/$fs, module calls, expressions                │   │
│   │  └─ Returns: EvaluatedAst (flattened, resolved geometry tree)       │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│          │ openscad_ast::parse("cube(10);")                                 │
│          ▼                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  libs/openscad-ast (AST Generation)                                 │   │
│   │  ├─ Public: parse(source: &str) -> Ast                              │   │
│   │  ├─ Calls: openscad_parser::parse(source)                           │   │
│   │  ├─ Receives: Cst (concrete syntax tree with spans)                 │   │
│   │  ├─ Transforms: CST nodes → typed AST nodes                         │   │
│   │  └─ Returns: Ast { statements: Vec<Statement> }                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│          │ openscad_parser::parse("cube(10);")                              │
│          ▼                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  libs/openscad-parser (Pure Rust Parser)                            │   │
│   │  ├─ Public: parse(source: &str) -> Cst                              │   │
│   │  ├─ Lexer: Tokenizes source into tokens with spans                  │   │
│   │  ├─ Parser: Recursive descent, produces CST                         │   │
│   │  └─ Returns: Cst { root: Node, errors: Vec<ParseError> }            │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.0.2 Crate Dependencies (Strict Layering)

```
playground ──uses──> wasm ──uses──> openscad-mesh ──uses──> openscad-eval ──uses──> openscad-ast ──uses──> openscad-parser
```

**Each crate only depends on the crate directly below it. No skipping layers.**

| Crate | Depends On | Public Interface | Input | Output |
|-------|------------|------------------|-------|--------|
| `openscad-parser` | (none) | `parse(source) -> Cst` | Source text | CST with spans |
| `openscad-ast` | `openscad-parser` | `parse(source) -> Ast` | Source text | Unevaluated AST |
| `openscad-eval` | `openscad-ast` | `evaluate(source) -> EvaluatedAst` | Source text | Evaluated/flattened AST |
| `openscad-mesh` | `openscad-eval` | `render(source) -> Mesh` | Source text | Mesh (verts/indices) |
| `wasm` | `openscad-mesh` | `render(source) -> MeshResult` | Source text | WASM-safe mesh arrays |

### 1.0.2.1 Visitor Pattern

The pipeline uses the **Visitor Pattern** for tree traversal. Each visitor is in a dedicated file.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    VISITOR PATTERN USAGE (SRP - One file per concern)       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   openscad-ast/src/visitor/                                                 │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  mod.rs              - CstVisitor trait + public API                │   │
│   │  ast_printer.rs      - AstPrinterVisitor: Ast → String (debug)      │   │
│   │                                                                     │   │
│   │  cst_to_ast/         - CstToAstVisitor (broken by SRP)              │   │
│   │  ├─ mod.rs           - CstToAstVisitor struct + dispatch            │   │
│   │  ├─ statements.rs    - ModuleCall, Assignment, ForLoop, IfElse      │   │
│   │  ├─ expressions.rs   - Binary, Unary, Literal, List, Range          │   │
│   │  └─ declarations.rs  - ModuleDefinition, FunctionDefinition         │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   openscad-eval/src/visitor/                                                │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  mod.rs              - AstVisitor trait + public API                │   │
│   │  scope_builder.rs    - ScopeBuilderVisitor: Collect declarations    │   │
│   │  dependency.rs       - DependencyVisitor: Build dependency graph    │   │
│   │                                                                     │   │
│   │  evaluator/          - EvaluatorVisitor (broken by SRP)             │   │
│   │  ├─ mod.rs           - EvaluatorVisitor struct + dispatch           │   │
│   │  ├─ context.rs       - EvaluationContext, scopes, variables         │   │
│   │  ├─ expressions.rs   - Expression evaluation (binary, unary, etc.)  │   │
│   │  ├─ statements.rs    - Statement evaluation (for, if, let, etc.)    │   │
│   │  ├─ builtins.rs      - Built-in functions (sin, cos, len, str, etc.)│   │
│   │  └─ primitives.rs    - Primitive modules (cube, sphere, cylinder)   │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   openscad-mesh/src/visitor/                                                │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  mod.rs              - GeometryVisitor trait + public API           │   │
│   │                                                                     │   │
│   │  mesh_builder/       - MeshBuilderVisitor (broken by SRP)           │   │
│   │  ├─ mod.rs           - MeshBuilderVisitor struct + dispatch         │   │
│   │  ├─ primitives.rs    - Cube, Sphere, Cylinder, Polyhedron meshes    │   │
│   │  ├─ transforms.rs    - Translate, Rotate, Scale, Mirror, Multmatrix │   │
│   │  ├─ booleans.rs      - Union, Difference, Intersection (CSG)        │   │
│   │  └─ extrusions.rs    - LinearExtrude, RotateExtrude                 │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Visitor Trait Definitions

**openscad-ast: `CstVisitor`** (traverses CST nodes)

```rust
// libs/openscad-ast/src/visitor/mod.rs
pub trait CstVisitor {
    type Output;
    
    fn visit_node(&mut self, node: &CstNode) -> Self::Output;
    fn visit_source_file(&mut self, node: &CstNode) -> Self::Output;
    fn visit_module_call(&mut self, node: &CstNode) -> Self::Output;
    fn visit_module_declaration(&mut self, node: &CstNode) -> Self::Output;
    fn visit_function_declaration(&mut self, node: &CstNode) -> Self::Output;
    fn visit_assignment(&mut self, node: &CstNode) -> Self::Output;
    fn visit_expression(&mut self, node: &CstNode) -> Self::Output;
    fn visit_for_block(&mut self, node: &CstNode) -> Self::Output;
    fn visit_if_block(&mut self, node: &CstNode) -> Self::Output;
    // ... other CST node types
}
```

**openscad-eval: `AstVisitor`** (traverses AST nodes)

```rust
// libs/openscad-eval/src/visitor/mod.rs
pub trait AstVisitor {
    type Output;
    
    fn visit_statement(&mut self, stmt: &Statement) -> Self::Output;
    fn visit_module_call(&mut self, call: &ModuleCall) -> Self::Output;
    fn visit_module_def(&mut self, def: &ModuleDefinition) -> Self::Output;
    fn visit_function_def(&mut self, def: &FunctionDefinition) -> Self::Output;
    fn visit_assignment(&mut self, assign: &Assignment) -> Self::Output;
    fn visit_expression(&mut self, expr: &Expression) -> Self::Output;
    fn visit_for_loop(&mut self, loop_: &ForLoop) -> Self::Output;
    fn visit_if_else(&mut self, if_else: &IfElse) -> Self::Output;
    // ... other AST node types
}
```

**openscad-mesh: `GeometryVisitor`** (traverses geometry nodes)

```rust
// libs/openscad-mesh/src/visitor/mod.rs
pub trait GeometryVisitor {
    type Output;
    
    fn visit_geometry(&mut self, node: &GeometryNode) -> Self::Output;
    fn visit_cube(&mut self, size: [f64; 3], center: bool) -> Self::Output;
    fn visit_sphere(&mut self, radius: f64, fn_: u32) -> Self::Output;
    fn visit_cylinder(&mut self, h: f64, r1: f64, r2: f64, center: bool, fn_: u32) -> Self::Output;
    fn visit_transform(&mut self, matrix: [[f64; 4]; 4], child: &GeometryNode) -> Self::Output;
    fn visit_union(&mut self, children: &[GeometryNode]) -> Self::Output;
    fn visit_difference(&mut self, children: &[GeometryNode]) -> Self::Output;
    fn visit_intersection(&mut self, children: &[GeometryNode]) -> Self::Output;
    // ... other geometry types
}
```

### 1.0.3 Data Flow Example: `cube(10);`

```
1. playground: User types "cube(10);"
   │
   ▼ wasm.render("cube(10);")
   
2. wasm: Thin layer, delegates to openscad-mesh
   │
   ▼ openscad_mesh::render("cube(10);")
   
3. openscad-mesh: Needs evaluated AST to generate geometry
   │
   ▼ openscad_eval::evaluate("cube(10);")
   
4. openscad-eval: Needs AST to evaluate
   │
   ▼ openscad_ast::parse("cube(10);")
   
5. openscad-ast: Needs CST to build AST
   │
   ▼ openscad_parser::parse("cube(10);")
   
6. openscad-parser: Lexes and parses source
   │
   └─► Returns: Cst {
         root: Node {
           type: "source_file",
           children: [Node { type: "module_call", name: "cube", args: [...] }]
         }
       }
   
7. openscad-ast: Transforms CST to AST
   │
   └─► Returns: Ast {
         statements: [
           Statement::ModuleCall {
             name: "cube",
             args: [Argument::Positional(Expression::Number(10.0))]
           }
         ]
       }
   
8. openscad-eval: Evaluates AST, resolves primitives
   │
   └─► Returns: EvaluatedAst {
         geometry: GeometryNode::Cube { size: [10.0, 10.0, 10.0], center: false }
       }
   
9. openscad-mesh: Generates mesh from evaluated geometry
   │
   └─► Returns: Mesh {
         vertices: [0,0,0, 10,0,0, 10,10,0, ...],  // 8 vertices × 3
         indices: [0,1,2, 0,2,3, ...],              // 12 triangles × 3
         normals: [0,0,-1, 0,0,-1, ...],            // per-vertex normals
       }
   
10. wasm: Wraps mesh as typed arrays
    │
    └─► Returns: MeshResult { vertices: Float32Array, indices: Uint32Array, ... }
   
11. playground: Creates Three.js geometry and renders
    │
    └─► new THREE.BufferGeometry() with vertices/indices/normals
```

### 1.0.4 Core Data Structures

Each layer produces a specific data structure that the layer above consumes:

#### openscad-parser → Cst (Concrete Syntax Tree)

```rust
/// Concrete Syntax Tree - preserves all source details
pub struct Cst {
    pub root: CstNode,
    pub errors: Vec<ParseError>,
}

pub struct CstNode {
    pub node_type: String,      // "module_call", "binary_expression", etc.
    pub text: Option<String>,   // Source text for terminals
    pub span: Span,             // Source location
    pub children: Vec<CstNode>, // Child nodes
}
```

#### openscad-ast → Ast (Abstract Syntax Tree)

```rust
/// Abstract Syntax Tree - typed, no syntax noise
pub struct Ast {
    pub statements: Vec<Statement>,
}

pub enum Statement {
    ModuleCall { name: String, args: Vec<Argument>, body: Option<Vec<Statement>>, span: Span },
    Assignment { name: String, value: Expression, span: Span },
    ModuleDefinition { name: String, params: Vec<Parameter>, body: Vec<Statement>, span: Span },
    FunctionDefinition { name: String, params: Vec<Parameter>, body: Expression, span: Span },
    ForLoop { assignments: Vec<Assignment>, body: Vec<Statement>, span: Span },
    IfElse { condition: Expression, then_body: Vec<Statement>, else_body: Option<Vec<Statement>>, span: Span },
    // ...
}

pub enum Expression {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    BinaryOp { op: BinaryOperator, lhs: Box<Expression>, rhs: Box<Expression> },
    FunctionCall { name: String, args: Vec<Argument> },
    List(Vec<Expression>),
    Range { start: Box<Expression>, end: Box<Expression>, step: Option<Box<Expression>> },
    // ...
}
```

#### openscad-eval → EvaluatedAst (Resolved Geometry Tree)

```rust
/// Evaluated AST - all expressions resolved, geometry tree flattened
pub struct EvaluatedAst {
    pub geometry: GeometryNode,
    pub warnings: Vec<Warning>,
}

pub enum GeometryNode {
    // Primitives (fully resolved parameters)
    Cube { size: [f64; 3], center: bool },
    Sphere { radius: f64, fn_: u32, fa: f64, fs: f64 },
    Cylinder { height: f64, radius1: f64, radius2: f64, center: bool, fn_: u32 },
    Polyhedron { points: Vec<[f64; 3]>, faces: Vec<Vec<usize>> },
    
    // Transforms (matrices resolved)
    Transform { matrix: [[f64; 4]; 4], child: Box<GeometryNode> },
    Color { rgba: [f64; 4], child: Box<GeometryNode> },
    
    // Boolean operations
    Union { children: Vec<GeometryNode> },
    Difference { children: Vec<GeometryNode> },
    Intersection { children: Vec<GeometryNode> },
    
    // Extrusions
    LinearExtrude { height: f64, twist: f64, scale: [f64; 2], slices: u32, child: Box<GeometryNode> },
    RotateExtrude { angle: f64, fn_: u32, child: Box<GeometryNode> },
    
    // 2D shapes (for extrusion)
    Circle { radius: f64, fn_: u32 },
    Square { size: [f64; 2], center: bool },
    Polygon { points: Vec<[f64; 2]>, paths: Option<Vec<Vec<usize>>> },
    
    // Empty (for conditionals that evaluate to nothing)
    Empty,
}
```

#### openscad-mesh → Mesh (Triangle Mesh)

```rust
/// Final triangle mesh ready for rendering
pub struct Mesh {
    pub vertices: Vec<f32>,   // [x, y, z, x, y, z, ...] - flat array
    pub indices: Vec<u32>,    // [i0, i1, i2, ...] - triangle indices
    pub normals: Vec<f32>,    // [nx, ny, nz, ...] - per-vertex normals
    pub colors: Option<Vec<f32>>, // [r, g, b, a, ...] - per-vertex colors
}
```

#### wasm → MeshResult (WASM-safe)

```rust
/// WASM export - typed arrays for JavaScript
#[wasm_bindgen]
pub struct MeshResult {
    vertices: Vec<f32>,
    indices: Vec<u32>,
    normals: Vec<f32>,
}

#[wasm_bindgen]
impl MeshResult {
    pub fn vertices(&self) -> Float32Array;  // Zero-copy view
    pub fn indices(&self) -> Uint32Array;    // Zero-copy view
    pub fn normals(&self) -> Float32Array;   // Zero-copy view
}
```

---

### 1.0.5 Pure Rust Parser Design

The parser is inspired by tree-sitter's architecture but simplified for OpenSCAD:

#### Why Pure Rust?

1. **Single WASM Output** - No separate grammar WASM file needed
2. **No C Dependencies** - Fully browser-safe, no Emscripten required
3. **Simpler Build** - Standard `cargo build --target wasm32-unknown-unknown`
4. **Full Control** - Direct AST emission, custom error recovery

#### tree-sitter vs Pure Rust Comparison

| Aspect | tree-sitter | Pure Rust |
|--------|-------------|-----------|
| Algorithm | GLR (handles ambiguity) | Recursive Descent (LL(k)) |
| Grammar | JavaScript DSL → C tables | Rust functions |
| Lexer | State machine from grammar | Hand-written cursor-based |
| Incremental | Yes (reuse unchanged subtrees) | No (full reparse) |
| Error Recovery | Automatic via error cost | Manual via synchronization |
| Output | CST → AST conversion | Direct AST |
| WASM Size | ~43KB grammar + runtime | ~0KB (integrated) |

#### Lexer Design (from tree-sitter `lexer.c`)

```rust
/// Lexer state machine inspired by tree-sitter
pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    current_pos: Position,
    token_start: Position,
}

impl<'a> Lexer<'a> {
    /// Advance one character (like ts_lexer_advance)
    fn advance(&mut self) -> Option<char>;
    
    /// Peek without consuming (like lexer->lookahead)
    fn peek(&self) -> Option<char>;
    
    /// Mark current position as token end (like ts_lexer_mark_end)
    fn mark_end(&mut self);
    
    /// Get next token
    pub fn next_token(&mut self) -> Token;
}
```

#### Parser Design (from tree-sitter `parser.c`)

```rust
/// Recursive descent parser (simpler than tree-sitter's GLR)
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    /// Parse entire source file
    pub fn parse(source: &str) -> ParseResult {
        let mut parser = Parser::new(source);
        let items = parser.parse_source_file();
        ParseResult { items, errors: parser.errors }
    }
    
    /// Consume token if it matches expected kind
    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError>;
    
    /// Try to parse, backtrack on failure
    fn try_parse<T>(&mut self, f: impl FnOnce(&mut Self) -> Option<T>) -> Option<T>;
    
    /// Error recovery: skip to synchronization point
    fn synchronize(&mut self);
}
```

#### Span Tracking (from tree-sitter `subtree.h`)

```rust
/// Source location (like tree-sitter's TSPoint + byte offset)
#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub byte: usize,    // Byte offset in source
    pub line: usize,    // 0-indexed line number
    pub column: usize,  // 0-indexed column (bytes, not chars)
}

/// Source span (like tree-sitter's start_byte/end_byte + start_point/end_point)
#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

/// AST node with span
pub trait Spanned {
    fn span(&self) -> Span;
}
```

#### Grammar Rules (from `grammar.js`)

The existing `libs/openscad-parser/grammar.js` defines the complete grammar.
Key patterns to implement:

```rust
// From grammar.js line 124-157
fn parse_source_file(&mut self) -> Vec<Item> {
    // source_file: $ => repeat(choice($.use_statement, $._item))
    let mut items = Vec::new();
    while !self.at_end() {
        if self.check(TokenKind::Use) {
            items.push(self.parse_use_statement());
        } else {
            items.push(self.parse_item());
        }
    }
    items
}

// From grammar.js line 358-372 (operator precedence)
fn parse_binary_expr(&mut self, min_prec: u8) -> Expression {
    let mut lhs = self.parse_unary_expr();
    while let Some((op, prec, assoc)) = self.current_binary_op() {
        if prec < min_prec { break; }
        self.advance();
        let rhs = self.parse_binary_expr(
            if assoc == Assoc::Left { prec + 1 } else { prec }
        );
        lhs = Expression::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) };
    }
    lhs
}
```

#### Error Recovery Strategy

Unlike tree-sitter's automatic error recovery, we use manual synchronization:

```rust
fn synchronize(&mut self) {
    // Skip tokens until we find a statement boundary
    while !self.at_end() {
        match self.current.kind {
            // Statement terminators
            TokenKind::Semicolon => {
                self.advance();
                return;
            }
            // Statement starters
            TokenKind::Module | TokenKind::Function | 
            TokenKind::For | TokenKind::If | TokenKind::Let => return,
            // Block boundaries
            TokenKind::RBrace => return,
            _ => self.advance(),
        }
    }
}
```

#### Module Boundaries (Strict)

```
openscad-parser → openscad-ast → openscad-eval → openscad-mesh → wasm
      ↓                ↓               ↓              ↓           ↓
     CST             AST           IR/Context       Mesh        JS API
```

- **No skipping layers**: Each crate only depends on the previous one
- **No reverse dependencies**: Lower crates never import higher ones
- **wasm orchestrates only**: It calls public APIs, never implements geometry

### 1.0.2 CPU vs GPU Processing Model

The operations in this pipeline are **CPU-bound geometry processing tasks**, not native GPU-bound rendering tasks:

| Layer | Role | Technology |
|-------|------|------------|
| **Rust (via WASM)** | Heavy geometry processing: calculating new mesh vertices, normals, indices for operations like `union()`, `linear_extrude()`, `hull()`, etc. | `openscad-mesh` crate with browser-safe algorithms |
| **WebGL** | Efficient rendering: displaying the generated mesh data in the browser's `<canvas>` element | Three.js + `BufferGeometry` |

**Workflow:**
1. **Parse**: `openscad-parser` produces CST from source via tree-sitter
2. **Convert**: `openscad-ast` transforms CST to typed AST with spans
3. **Evaluate**: `openscad-eval` resolves variables, modules, loops → Geometry IR
4. **Generate**: `openscad-mesh` converts IR to mesh (vertices + indices + normals)
5. **Transfer**: `wasm` passes mesh data to JavaScript via zero-copy typed arrays
6. **Render**: Three.js displays mesh via WebGL

**Rendering Details:**
- **Mesh Material**: `MeshStandardMaterial` with metalness/roughness for realistic lighting
- **Edge Highlighting**: `WireframeGeometry` overlay showing ALL triangle edges (not just sharp edges)
  - Uses `LineSegments` with semi-transparent black lines
  - Matches OpenSCAD's behavior where smooth surfaces show tessellation wireframe
  - `EdgesGeometry` was rejected because it only shows edges where face normals differ by threshold angle
- **Coordinate System**: Z-up axis to match OpenSCAD/CAD conventions

This approach gives full power of Rust's computational capabilities within the browser, perfect for interactive 3D modeling.

All development must be broken down into **small, test-driven steps** that a developer can execute without needing external resources.

### 1.0.2 Preview vs Render Modes

OpenSCAD distinguishes two modes for displaying geometry. Our pipeline must support both:

| Mode | Purpose | Speed | Geometry Accuracy |
|------|---------|-------|-------------------|
| **Preview (F5)** | Interactive editing feedback | Fast (~ms) | Approximate (CSG via GPU) |
| **Render (F6)** | Final mesh for export | Slower (~s) | Exact (full boolean evaluation) |

#### Preview Mode

Preview builds a **CSG tree** from the AST but does **not** fully evaluate boolean operations on the CPU. Instead:

1. **CSG Tree Evaluation** – Walk the AST, wrap each primitive in a `CSGLeaf` (PolySet + transform + color), and combine leaves with `CSGOperation` nodes (union/difference/intersection).
2. **Tree Normalization** – Rewrite the tree into a form suitable for GPU rendering (Goldfeather algorithm).
3. **GPU Boolean Rendering** – Use OpenCSG (or fallback "thrown together") to render the CSG tree directly via multi-pass stencil/depth tricks.

**Key characteristics:**
- Primitives are tessellated individually (cube, sphere, cylinder).
- Booleans are **not** computed on the CPU; the GPU fakes them visually.
- Highlight (`#`) and background (`%`) modifiers are rendered separately.
- 2D polygons are extruded to thin 3D slabs for display.

**Browser adaptation:** We can implement GPU-accelerated CSG preview in WebGL using stencil buffer techniques from Rust/WASM.

##### WebGL CSG Implementation (SCS Algorithm)

The **SCS (Sequenced Convex Subtractions)** algorithm is well-suited for WebGL because it uses fewer rendering passes than Goldfeather. It operates on convex primitives (non-convex must be decomposed).

**Algorithm outline** (per CSG product):

1. **Render convex intersections** (3 passes):
   - Pass 1: Render furthest front faces into Z buffer (other buffers disabled)
   - Pass 2: Count hidden backfaces (increment stencil on depth fail)
   - Pass 3: Reset Z buffer where `stencil != N` (N = number of intersecting objects)

2. **Render convex differences** (per subtraction):
   - Pass 1: Mark front fragments passing Z test (stencil only)
   - Pass 2: Render back faces with inverted Z test, masked by stencil

3. **Clip transparent areas**:
   - Mark visible back fragments of intersections (stencil)
   - Reset Z buffer where stencil is set

4. **Render with correct material**:
   - Render front faces of intersections + back faces of subtractions with `depth = EQUAL`

5. **Merge products** into accumulation buffer (use alpha channel as synthetic Z)

**Special case optimizations**:
- Single object with no differences → render directly to framebuffer
- Only intersections → skip subtraction and clipping passes
- Non-overlapping objects → batch into same stencil pass

##### Rust/WASM WebGL Stack

| Crate | Purpose |
|-------|---------|
| `web-sys` | WebGL2 bindings (stencil, depth, shaders) |
| `wasm-bindgen` | JS ↔ Rust interop |
| `js-sys` | Float32Array for vertex buffers |
| `glam` | Math (same as mesh crate) |

**Required WebGL features** (via `web-sys` Cargo features):
```toml
[dependencies.web-sys]
features = [
  "WebGl2RenderingContext",
  "WebGlProgram", "WebGlShader", "WebGlBuffer",
  "WebGlFramebuffer", "WebGlRenderbuffer", "WebGlTexture",
  "WebGlUniformLocation", "WebGlVertexArrayObject",
]
```

**Key WebGL2 APIs for CSG**:
- `stencilFunc()`, `stencilOp()`, `stencilMask()` – stencil test configuration
- `depthFunc()`, `depthMask()` – depth test configuration
- `colorMask()` – selective buffer writes
- `createFramebuffer()` – off-screen render targets for accumulation

##### Implementation Phases

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Basic WebGL2 context from Rust | ⏳ Pending |
| 2 | Shader compilation (GLSL ES 3.0) | ⏳ Pending |
| 3 | VBO/VAO mesh upload | ⏳ Pending |
| 4 | Single-object rendering | ⏳ Pending |
| 5 | Stencil-based intersection | ⏳ Pending |
| 6 | Stencil-based difference | ⏳ Pending |
| 7 | Product accumulation | ⏳ Pending |
| 8 | Integration with CSG tree | ⏳ Pending |

##### Rust/WASM Implementation Code

**Cargo.toml dependencies:**
```toml
[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
glam = "0.24"

[dependencies.web-sys]
version = "0.3"
features = [
  "Window", "Document", "HtmlCanvasElement",
  "WebGl2RenderingContext", "WebGlProgram", "WebGlShader",
  "WebGlBuffer", "WebGlFramebuffer", "WebGlRenderbuffer",
  "WebGlTexture", "WebGlUniformLocation", "WebGlVertexArrayObject",
]
```

**WebGL2 Context Setup (Rust):**
```rust
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext as GL, HtmlCanvasElement};

pub struct CsgRenderer {
    gl: GL,
    program: web_sys::WebGlProgram,
}

impl CsgRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // Request stencil buffer in context options
        let context_options = js_sys::Object::new();
        js_sys::Reflect::set(&context_options, &"stencil".into(), &true.into())?;
        
        let gl = canvas
            .get_context_with_context_options("webgl2", &context_options)?
            .ok_or("WebGL2 not supported")?
            .dyn_into::<GL>()?;
        
        let program = Self::create_shader_program(&gl)?;
        
        Ok(Self { gl, program })
    }
}
```

**Stencil-Based CSG Operations (Rust):**
```rust
impl CsgRenderer {
    /// Render CSG intersection: A ∩ B
    /// Uses face-counting: pixel is "inside" if front_faces - back_faces > 0
    pub fn render_intersection(&self, mesh_a: &Mesh, mesh_b: &Mesh) {
        let gl = &self.gl;
        
        // Clear buffers
        gl.clear_stencil(0);
        gl.clear_depth(1.0);
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT | GL::STENCIL_BUFFER_BIT);
        
        gl.enable(GL::STENCIL_TEST);
        gl.enable(GL::DEPTH_TEST);
        gl.enable(GL::CULL_FACE);
        gl.color_mask(false, false, false, false); // Stencil pass only
        
        // === Pass 1: Count faces of object A ===
        // Front faces: increment stencil
        gl.cull_face(GL::BACK);
        gl.stencil_func(GL::ALWAYS, 0, 0xFF);
        gl.stencil_op(GL::KEEP, GL::KEEP, GL::INCR_WRAP);
        self.draw_mesh(mesh_a);
        
        // Back faces: decrement stencil
        gl.cull_face(GL::FRONT);
        gl.stencil_op(GL::KEEP, GL::KEEP, GL::DECR_WRAP);
        self.draw_mesh(mesh_a);
        
        // === Pass 2: Count faces of object B (use different stencil bits) ===
        gl.stencil_func(GL::ALWAYS, 0, 0xFF);
        gl.cull_face(GL::BACK);
        gl.stencil_op(GL::KEEP, GL::KEEP, GL::INCR_WRAP);
        self.draw_mesh(mesh_b);
        
        gl.cull_face(GL::FRONT);
        gl.stencil_op(GL::KEEP, GL::KEEP, GL::DECR_WRAP);
        self.draw_mesh(mesh_b);
        
        // === Pass 3: Render where stencil indicates inside both ===
        gl.color_mask(true, true, true, true);
        gl.stencil_func(GL::EQUAL, 2, 0xFF); // Inside both = stencil value 2
        gl.stencil_op(GL::KEEP, GL::KEEP, GL::KEEP);
        gl.cull_face(GL::BACK);
        
        self.draw_mesh(mesh_a);
        self.draw_mesh(mesh_b);
        
        gl.disable(GL::STENCIL_TEST);
    }
    
    /// Render CSG difference: A - B
    /// Shows A where B is not present
    pub fn render_difference(&self, mesh_a: &Mesh, mesh_b: &Mesh) {
        let gl = &self.gl;
        
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT | GL::STENCIL_BUFFER_BIT);
        gl.enable(GL::STENCIL_TEST);
        gl.enable(GL::DEPTH_TEST);
        gl.enable(GL::CULL_FACE);
        
        // === Pass 1: Render A's front faces, mark in stencil ===
        gl.color_mask(false, false, false, false);
        gl.stencil_func(GL::ALWAYS, 1, 0xFF);
        gl.stencil_op(GL::KEEP, GL::KEEP, GL::REPLACE);
        gl.cull_face(GL::BACK);
        self.draw_mesh(mesh_a);
        
        // === Pass 2: Where B's volume overlaps A, clear stencil ===
        gl.depth_mask(false);
        gl.stencil_func(GL::EQUAL, 1, 0xFF); // Only where A exists
        
        // B front faces: decrement (entering B)
        gl.cull_face(GL::BACK);
        gl.stencil_op(GL::KEEP, GL::KEEP, GL::DECR_WRAP);
        self.draw_mesh(mesh_b);
        
        // B back faces: increment (exiting B)
        gl.cull_face(GL::FRONT);
        gl.stencil_op(GL::KEEP, GL::KEEP, GL::INCR_WRAP);
        self.draw_mesh(mesh_b);
        
        // === Pass 3: Render A where stencil still = 1 (not inside B) ===
        gl.depth_mask(true);
        gl.color_mask(true, true, true, true);
        gl.stencil_func(GL::EQUAL, 1, 0xFF);
        gl.stencil_op(GL::KEEP, GL::KEEP, GL::KEEP);
        gl.cull_face(GL::BACK);
        self.draw_mesh(mesh_a);
        
        // === Pass 4: Render B's back faces where inside A (carved surface) ===
        gl.stencil_func(GL::NOTEQUAL, 1, 0xFF);
        gl.cull_face(GL::FRONT); // Back faces of B
        self.draw_mesh(mesh_b);
        
        gl.disable(GL::STENCIL_TEST);
    }
    
    /// Render CSG union: A ∪ B
    /// Simple: render both with depth test
    pub fn render_union(&self, mesh_a: &Mesh, mesh_b: &Mesh) {
        let gl = &self.gl;
        
        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
        gl.enable(GL::DEPTH_TEST);
        gl.enable(GL::CULL_FACE);
        gl.cull_face(GL::BACK);
        
        self.draw_mesh(mesh_a);
        self.draw_mesh(mesh_b);
    }
}
```

**GLSL Shaders (ES 3.0):**
```glsl
// Vertex Shader
#version 300 es
in vec3 a_position;
in vec3 a_normal;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

out vec3 v_normal;
out vec3 v_position;

void main() {
    vec4 worldPos = u_model * vec4(a_position, 1.0);
    v_position = worldPos.xyz;
    v_normal = normalize(mat3(u_model) * a_normal);
    gl_Position = u_projection * u_view * worldPos;
}

// Fragment Shader
#version 300 es
precision highp float;

in vec3 v_normal;
in vec3 v_position;

uniform vec3 u_light_dir;
uniform vec4 u_color;

out vec4 fragColor;

void main() {
    vec3 N = normalize(v_normal);
    vec3 L = normalize(u_light_dir);
    
    float ambient = 0.3;
    float diffuse = max(dot(N, L), 0.0) * 0.7;
    
    fragColor = vec4(u_color.rgb * (ambient + diffuse), u_color.a);
}
```

**CSG Tree Normalization:**
```rust
/// Normalize CSG tree to sum-of-products form for GPU rendering
pub fn normalize_csg_tree(node: &CsgNode) -> Vec<CsgProduct> {
    // Apply production rules:
    // 1. A ∪ (B ∪ C) → (A ∪ B) ∪ C  (flatten unions)
    // 2. A ∩ (B ∪ C) → (A ∩ B) ∪ (A ∩ C)  (distribute)
    // 3. A - (B ∪ C) → (A - B) - C
    // 4. (A ∪ B) ∩ C → (A ∩ C) ∪ (B ∩ C)
    // 5. (A ∪ B) - C → (A - C) ∪ (B - C)
    
    let mut products = Vec::new();
    collect_products(node, &mut products);
    products
}

pub struct CsgProduct {
    pub intersections: Vec<MeshRef>,  // Positive terms (∩)
    pub subtractions: Vec<MeshRef>,   // Negative terms (-)
}
```

**References**:
- [OpenSCAD WebGL CSG Wiki](https://github.com/openscad/openscad/wiki/WebGL-CSG-Implementation)
- [OpenCSG library](https://opencsg.org/) (Goldfeather/SCS algorithms)
- [wasm-bindgen WebGL guide](https://rustwasm.github.io/wasm-bindgen/examples/webgl.html)
- [SIGGRAPH CSG Stencil Tutorial](https://www.opengl.org/archives/resources/code/samples/sig99/advanced99/notes/node22.html)

#### Render Mode

Render performs **full geometric boolean evaluation** on the CPU:

1. **Geometry Evaluation** – Recursively evaluate each node, producing concrete `PolySet` meshes.
2. **Boolean Operations** – Apply union/difference/intersection using BSP trees (or Manifold algorithm) to produce a single watertight mesh.
3. **Mesh Output** – The final mesh is suitable for STL/3MF export and 3D printing.

**Key characteristics:**
- Expensive but geometrically exact.
- Required for export (`export_stl`, `export_3mf`).
- Uses `$fn/$fa/$fs` to control tessellation resolution.
- Result is cached in the geometry cache for repeated renders.

#### Current Implementation Status

| Feature | Preview | Render |
|---------|---------|--------|
| CSG tree building | ✅ | ✅ |
| Full boolean eval (BSP/Manifold) | ✅ (fallback) | ✅ |
| GPU-accelerated preview (WebGL SCS) | ⏳ Planned | N/A |
| Mesh export | N/A | ✅ |
| Highlight/background modifiers | ⚠️ Partial | ✅ |

#### Architecture Comparison

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        PREVIEW MODE (GPU)                                │
├─────────────────────────────────────────────────────────────────────────┤
│  OpenSCAD Source                                                         │
│       ↓                                                                  │
│  openscad-parser (tree-sitter) → CST                                    │
│       ↓                                                                  │
│  openscad-ast → AST                                                     │
│       ↓                                                                  │
│  openscad-eval → CSG Tree (primitives + operations, not evaluated)      │
│       ↓                                                                  │
│  Normalize CSG Tree (sum-of-products form)                              │
│       ↓                                                                  │
│  WebGL Multi-Pass Rendering (SCS Algorithm via wasm)                    │
│   ├─ Pass 1-3: Stencil-based intersections                              │
│   ├─ Pass 4-5: Stencil-based differences                                │
│   └─ Pass 6: Material rendering with depth=EQUAL                        │
│       ↓                                                                  │
│  Display (fast, approximate)                                            │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                        RENDER MODE (CPU)                                 │
├─────────────────────────────────────────────────────────────────────────┤
│  OpenSCAD Source                                                         │
│       ↓                                                                  │
│  openscad-parser (tree-sitter) → CST                                    │
│       ↓                                                                  │
│  openscad-ast → AST                                                     │
│       ↓                                                                  │
│  openscad-eval → Geometry IR (fully evaluated)                          │
│       ↓                                                                  │
│  openscad-mesh → Primitives to meshes                                   │
│       ↓                                                                  │
│  openscad-mesh → Boolean Operations (BSP/Manifold on CPU)               │
│       ↓                                                                  │
│  wasm → Final Mesh (vertices + indices + normals)                       │
│       ↓                                                                  │
│  Display / Export STL/3MF (exact, watertight)                           │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.1 Target Validation Test Case

The following OpenSCAD program must render correctly in the pipeline as the primary acceptance test:

```openscad
translate([-24,0,0]) {
    union() {
        cube(15, center=true);
        sphere(10);
    }
}

intersection() {
    cube(15, center=true);
    sphere(10);
}

translate([24,0,0]) {
    difference() {
        cube(15, center=true);
        sphere(10);
    }
}
```

This test validates:
- **3D Primitives**: `cube(size, center)`, `sphere(radius)`
- **Boolean Operations**: `union()`, `intersection()`, `difference()`
- **Transformations**: `translate([x,y,z])`
- **Block Scoping**: Nested children within transform/boolean blocks

---

## 2. Core Philosophy (Strict Adherence Required)

- **Vertical Slices**  
  Implement one feature at a time through the *entire* pipeline:
  
  ```
  Playground UI → Web Worker → libs/wasm
       ↓
  libs/openscad-parser (tree-sitter) → CST
       ↓
  libs/openscad-ast → AST
       ↓
  libs/openscad-eval → Geometry IR
       ↓
  libs/openscad-mesh → Mesh
       ↓
  libs/wasm → Float32Array
       ↓
  Three.js → WebGL Canvas
  ```

- **SRP & Structure**  
  Every *single-responsibility unit* (feature/struct/module) must live in its own folder with:
  
  - `mod.rs` – implementation
  - `tests.rs` – unit tests (TDD)
  
  Example:  
  `libs/openscad-mesh/src/primitives/cube/{mod.rs, tests.rs}`.

- **TDD (Test-Driven Development)**  
  - Write tests **before** implementation.  
  - Workflow: **Red → Green → Refactor**.  

# Rust OpenSCAD Pipeline – Overview Plan

_Last updated: 2025-11-28 — **Manifold-RS Migration Complete!** Deleted `libs/openscad-mesh`. Now using `libs/manifold-rs` exclusively - full Rust port of Manifold-3D algorithms. Browser-safe WASM, exact CSG booleans, QuickHull convex hull, Minkowski sum. OpenSCAD $fn/$fa/$fs compatibility wrapper._

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
- **Best Algorithms for Mesh Operations**: Use Manifold-3D algorithms (ported to pure Rust) for robust CSG, exact boolean operations, and high-performance mesh generation.

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
│          │ manifold_rs::render("cube(10);")                               │
│          ▼                                                                  │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  libs/manifold-rs (Manifold-3D Rust Port)                           │   │
│   │  ├─ Public: render(source: &str) -> Mesh                            │   │
│   │  ├─ Calls: openscad_eval::evaluate(source)                          │   │
│   │  ├─ Receives: EvaluatedAst (flattened geometry tree)                │   │
│   │  ├─ Core: Manifold (3D), CrossSection (2D), Mesh (output)           │   │
│   │  ├─ Wrapper: OpenSCAD $fn/$fa/$fs → circularSegments                │   │
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
playground ──uses──> wasm ──uses──> manifold-rs ──uses──> openscad-eval ──uses──> openscad-ast ──uses──> openscad-parser
```

**Each crate only depends on the crate directly below it. No skipping layers.**

| Crate | Depends On | Public Interface | Input | Output |
|-------|------------|------------------|-------|--------|
| `openscad-parser` | (none) | `parse(source) -> Cst` | Source text | CST with spans |
| `openscad-ast` | `openscad-parser` | `parse(source) -> Ast` | Source text | Unevaluated AST |
| `openscad-eval` | `openscad-ast` | `evaluate(source) -> EvaluatedAst` | Source text | Evaluated/flattened AST |
| `manifold-rs` | `openscad-eval` | `render(source) -> Mesh` | Source text | Mesh (verts/indices) |
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
│   manifold-rs/src/                                                          │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │  lib.rs              - Public API: render(source) -> Mesh           │   │
│   │                                                                     │   │
│   │  manifold/           - 3D Solid Operations (Manifold-3D port)       │   │
│   │  ├─ mod.rs           - Manifold struct + methods                    │   │
│   │  ├─ boolean3.rs      - Union, Difference, Intersection (exact)      │   │
│   │  ├─ constructors.rs  - Cube, Sphere, Cylinder, Tetrahedron          │   │
│   │  ├─ csg_tree.rs      - CSG tree evaluation and optimization         │   │
│   │  └─ impl.rs          - Core Manifold implementation                 │   │
│   │                                                                     │   │
│   │  cross_section/      - 2D Polygon Operations                        │   │
│   │  ├─ mod.rs           - CrossSection struct + methods                │   │
│   │  ├─ offset.rs        - Polygon offset/inset                         │   │
│   │  └─ boolean.rs       - 2D union/diff/intersection                   │   │
│   │                                                                     │   │
│   │  mesh/               - Output Mesh Format                           │   │
│   │  ├─ mod.rs           - Mesh struct (vertices, indices, normals)     │   │
│   │  └─ halfedge.rs      - HalfEdge mesh representation                 │   │
│   │                                                                     │   │
│   │  openscad/           - OpenSCAD Compatibility Wrapper               │   │
│   │  ├─ mod.rs           - OpenSCAD API compatibility layer             │   │
│   │  ├─ segments.rs      - $fn/$fa/$fs → circularSegments converter     │   │
│   │  └─ from_ir.rs       - GeometryNode → Manifold conversion           │   │
│   │                                                                     │   │
│   │  gpu/                - WebGPU Acceleration (Optional)               │   │
│   │  ├─ mod.rs           - GPU context and mode selection               │   │
│   │  ├─ sdf.rs           - SDF-based CSG compute shaders                │   │
│   │  └─ marching_cubes.rs - Mesh extraction from SDF                    │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### SRP Module Structures

**openscad-parser: SRP Module Structure** (parses source to CST)

```
libs/parser/src/parser/
├── mod.rs           # Public API (Parser struct, parse)
├── statements.rs    # Statement dispatch facade (184 lines)
├── module_call.rs   # Module call and arguments (228 lines)
├── control_flow.rs  # For, if/else, let, blocks (261 lines)
├── declarations.rs  # Module/function declarations (266 lines)
├── expressions.rs   # Expression dispatch facade (74 lines)
├── operators.rs     # Binary, unary, ternary (285 lines)
├── primaries.rs     # Literals and identifiers (170 lines)
├── postfix.rs       # Call, index, member access (187 lines)
└── collections.rs   # List and range parsing (178 lines)
```

```rust
// libs/parser/src/parser/mod.rs
pub struct Parser<'a> { ... }
pub fn parse(&mut self) -> Cst;
```

**openscad-ast: SRP Module Structure** (transforms CST to AST)

```
libs/openscad-ast/src/visitor/cst_to_ast/
├── mod.rs           # Public API (transform)
├── statements.rs    # Statement transformation facade (205 lines)
├── expressions.rs   # Expression transformation facade (246 lines)
├── arguments.rs     # Shared argument handling (182 lines)
├── literals.rs      # Number, string, boolean (199 lines)
├── operators.rs     # Binary, unary, ternary (238 lines)
├── control_flow.rs  # For loops, if/else, blocks (248 lines)
└── declarations.rs  # Module/function declarations (204 lines)
```

```rust
// libs/openscad-ast/src/visitor/cst_to_ast/mod.rs
pub fn transform(cst: &Cst) -> Result<Ast, AstError>;
```

**openscad-eval: SRP Module Structure** (evaluates AST to geometry)

```
libs/openscad-eval/src/visitor/
├── mod.rs           # Public API (evaluate_ast)
├── context.rs       # EvalContext, statement evaluation (325 lines)
├── expressions.rs   # Expression evaluation (422 lines)
├── primitives.rs    # 3D/2D primitive evaluators (335 lines)
├── boolean.rs       # Boolean operation evaluators (179 lines)
├── transforms.rs    # Transform evaluators (281 lines)
└── extrusions.rs    # Extrusion evaluators (196 lines)
```

```rust
// libs/openscad-eval/src/visitor/mod.rs
pub fn evaluate_ast(ast: &Ast) -> Result<EvaluatedAst, EvalError>;

// libs/openscad-eval/src/visitor/context.rs
pub struct EvalContext { warnings: Vec<String>, scope: Scope }
pub fn evaluate_statements(ctx: &mut EvalContext, stmts: &[Statement]) -> Result<GeometryNode, EvalError>;
```

**manifold-rs: OpenSCAD Compatibility Wrapper** (converts GeometryNode to Manifold)

```rust
// libs/manifold-rs/src/openscad/segments.rs
/// OpenSCAD segment calculation - 100% compatible with $fn/$fa/$fs
pub struct SegmentParams {
    pub fn_: Option<u32>,  // $fn - explicit segment count
    pub fa: f64,           // $fa - minimum angle per segment (default: 12°)
    pub fs: f64,           // $fs - minimum segment size (default: 2mm)
}

impl SegmentParams {
    /// Calculate circularSegments for Manifold from OpenSCAD params
    /// Formula: max($fn, ceil(360/$fa), ceil(2*PI*r/$fs))
    pub fn calculate_segments(&self, radius: f64) -> u32 {
        if let Some(fn_) = self.fn_ {
            if fn_ > 0 { return fn_; }
        }
        
        let from_fa = (360.0 / self.fa).ceil() as u32;
        let circumference = 2.0 * std::f64::consts::PI * radius;
        let from_fs = (circumference / self.fs).ceil() as u32;
        
        from_fa.max(from_fs).max(3)  // Minimum 3 segments
    }
}

// libs/manifold-rs/src/openscad/from_ir.rs
/// Convert GeometryNode (from openscad-eval) to Manifold
pub fn geometry_to_manifold(node: &GeometryNode, params: &SegmentParams) -> Manifold {
    match node {
        GeometryNode::Sphere { radius, fn_, fa, fs } => {
            let segments = SegmentParams { fn_: *fn_, fa: *fa, fs: *fs }
                .calculate_segments(*radius);
            Manifold::sphere(*radius, segments)
        }
        GeometryNode::Cylinder { height, radius1, radius2, center, fn_, .. } => {
            let segments = params.calculate_segments(radius1.max(*radius2));
            Manifold::cylinder(*height, *radius1, *radius2, segments, *center)
        }
        // ... other primitives
    }
}
```

### 1.0.3 Data Flow Example: `cube(10);`

```
1. playground: User types "cube(10);"
   │
   ▼ wasm.render("cube(10);")
   
2. wasm: Thin layer, delegates to manifold-rs
   │
   ▼ manifold_rs::render("cube(10);")
   
3. manifold-rs: Needs evaluated AST to generate geometry
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
   
9. manifold-rs: Generates mesh from evaluated geometry via Manifold
   │
   ├─ Creates: Manifold::cube([10.0, 10.0, 10.0], false)
   ├─ Extracts: manifold.get_mesh()
   └► Returns: Mesh {
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

#### manifold-rs → Manifold, CrossSection, Mesh

```rust
/// 3D Solid - watertight mesh with CSG operations (Manifold-3D port)
pub struct Manifold {
    impl_: ManifoldImpl,  // Internal half-edge mesh representation
}

impl Manifold {
    // Constructors (OpenSCAD compatible via wrapper)
    pub fn cube(size: [f64; 3], center: bool) -> Self;
    pub fn sphere(radius: f64, circular_segments: u32) -> Self;
    pub fn cylinder(height: f64, r_low: f64, r_high: f64, segments: u32, center: bool) -> Self;
    pub fn tetrahedron() -> Self;
    pub fn of_mesh(mesh: &Mesh) -> Self;
    
    // Boolean operations (exact, robust)
    pub fn union(&self, other: &Manifold) -> Self;
    pub fn subtract(&self, other: &Manifold) -> Self;  // difference
    pub fn intersect(&self, other: &Manifold) -> Self;
    pub fn hull(&self) -> Self;
    
    // Transforms
    pub fn translate(&self, offset: [f64; 3]) -> Self;
    pub fn rotate(&self, degrees: [f64; 3]) -> Self;
    pub fn scale(&self, factor: [f64; 3]) -> Self;
    pub fn mirror(&self, normal: [f64; 3]) -> Self;
    pub fn transform(&self, matrix: [[f64; 4]; 4]) -> Self;
    
    // Output
    pub fn get_mesh(&self) -> Mesh;
    pub fn num_vert(&self) -> usize;
    pub fn num_tri(&self) -> usize;
    pub fn bounding_box(&self) -> BoundingBox;
}

/// 2D Polygon - for extrusions and 2D operations (Manifold-3D CrossSection port)
pub struct CrossSection {
    polygons: Vec<Polygon2D>,
}

impl CrossSection {
    // Constructors
    pub fn circle(radius: f64, circular_segments: u32) -> Self;
    pub fn square(size: [f64; 2], center: bool) -> Self;
    pub fn of_polygons(polygons: Vec<Vec<[f64; 2]>>) -> Self;
    
    // Boolean operations (2D)
    pub fn union(&self, other: &CrossSection) -> Self;
    pub fn subtract(&self, other: &CrossSection) -> Self;
    pub fn intersect(&self, other: &CrossSection) -> Self;
    pub fn hull(&self) -> Self;
    
    // Operations
    pub fn offset(&self, delta: f64, join_type: JoinType, segments: u32) -> Self;
    
    // Extrusions (returns Manifold)
    pub fn extrude(&self, height: f64, n_divisions: u32, twist: f64, scale: [f64; 2]) -> Manifold;
    pub fn revolve(&self, circular_segments: u32, revolve_degrees: f64) -> Manifold;
}

/// Final triangle mesh ready for rendering
pub struct Mesh {
    pub vertices: Vec<f32>,       // [x, y, z, x, y, z, ...] - flat array
    pub indices: Vec<u32>,        // [i0, i1, i2, ...] - triangle indices
    pub normals: Vec<f32>,        // [nx, ny, nz, ...] - per-vertex normals
    pub colors: Option<Vec<f32>>, // [r, g, b, a, ...] - per-vertex colors
    pub properties: Option<Vec<f32>>, // Custom vertex properties
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
openscad-parser → openscad-ast → openscad-eval → manifold-rs → wasm
      ↓                ↓               ↓              ↓           ↓
     CST             AST           IR/Context     Manifold      JS API
```

- **No skipping layers**: Each crate only depends on the previous one
- **No reverse dependencies**: Lower crates never import higher ones
- **wasm orchestrates only**: It calls public APIs, never implements geometry

### 1.0.2 CPU vs GPU Processing Model

The operations in this pipeline are **CPU-bound geometry processing tasks**, not native GPU-bound rendering tasks:

| Layer | Role | Technology |
|-------|------|------------|
| **Rust (via WASM)** | Heavy geometry processing: calculating new mesh vertices, normals, indices for operations like `union()`, `linear_extrude()`, `hull()`, etc. | `manifold-rs` crate (Manifold-3D port) with CPU + WebGPU modes |
| **WebGL** | Efficient rendering: displaying the generated mesh data in the browser's `<canvas>` element | Three.js + `BufferGeometry` |

**Workflow:**
1. **Parse**: `openscad-parser` produces CST from source via tree-sitter
2. **Convert**: `openscad-ast` transforms CST to typed AST with spans
3. **Evaluate**: `openscad-eval` resolves variables, modules, loops → Geometry IR
4. **Generate**: `manifold-rs` converts IR to Manifold → Mesh (vertices + indices + normals)
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
2. **Boolean Operations** – Apply union/difference/intersection using Manifold algorithm (exact, robust) to produce a single watertight mesh.
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
│  manifold-rs → GeometryNode to Manifold primitives                      │
│       ↓                                                                  │
│  manifold-rs → Boolean Operations (Manifold exact on CPU/GPU)           │
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

## Phase 8: Manifold Boolean Engine (Performance Upgrade)

Replace BSP-based CSG with Manifold-style algorithms for robust, high-performance boolean operations on the CPU.

### 8.1 Data Structures
- **HalfEdgeMesh**: Doubly-linked half-edge structure for efficient topology traversal.
  - `HalfEdge { startVert, endVert, pairedHalfedge, face }`
  - `Vertex { position, halfedge }`
  - `Face { halfedge }`
- **Properties**: Generic property channels (normals, UVs, etc.) stored alongside vertices.

### 8.2 Spatial Indexing
- **BVH (Bounding Volume Hierarchy)**:
  - `Collider` struct for accelerating edge-face intersection tests.
  - Support for localized intersection checks to avoid O(N^2) comparisons.

### 8.3 Robust Geometry
- **Symbolic Perturbation**: Handle coplanar/coincident features by conceptually expanding/contracting meshes.
- **Exact Predicates**: Use `robust` crate for orientation and in-circle tests.
- **Intersection Kernel**:
  - `Kernel12`: Edge-Face intersection.
  - `Winding03`: Generalized winding number computation for classification.

### 8.4 Implementation Steps
1. Define `HalfEdgeMesh` struct and conversion from/to shared-vertex `Mesh`.
2. Implement `Collider` (BVH) for `HalfEdgeMesh`.
3. Implement `boolean3` algorithm (Union, Difference, Intersection) using `HalfEdgeMesh`.
4. Integrate into `manifold_rs::manifold::boolean3`.

---

## Phase 9: WebGPU CSG (GPU - Parallel Acceleration)

Alternative to CPU-based CSG using WebGPU compute shaders for massive parallelism.
Provides a **toggle option** between CPU Manifold and GPU SDF-based CSG.

### 9.1 Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CSG MODE TOGGLE                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────────┐         ┌─────────────────────┐                   │
│   │   CPU CSG (Phase 8)  │  ←→   │   GPU CSG (Phase 9)  │                   │
│   │   Manifold Algorithm │        │   SDF + Compute     │                   │
│   │   - Exact geometry   │        │   - Parallel ops    │                   │
│   │   - Export-quality   │        │   - Real-time       │                   │
│   └─────────────────────┘         └─────────────────────┘                   │
│                                                                             │
│   Toggle: render_options.csg_mode = CsgMode::Cpu | CsgMode::Gpu             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 9.2 SDF-Based CSG Algorithm

Signed Distance Fields (SDFs) naturally parallelize across voxels:

```wgsl
// WGSL Compute Shader for CSG Operations
@group(0) @binding(0) var<storage, read> sdf_a: array<f32>;
@group(0) @binding(1) var<storage, read> sdf_b: array<f32>;
@group(0) @binding(2) var<storage, read_write> sdf_result: array<f32>;
@group(0) @binding(3) var<uniform> params: CsgParams;

struct CsgParams {
    grid_size: vec3<u32>,
    operation: u32,  // 0=union, 1=intersection, 2=difference
}

@compute @workgroup_size(8, 8, 8)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x + id.y * params.grid_size.x + id.z * params.grid_size.x * params.grid_size.y;
    
    let a = sdf_a[idx];
    let b = sdf_b[idx];
    
    switch params.operation {
        case 0u: { sdf_result[idx] = min(a, b); }      // Union
        case 1u: { sdf_result[idx] = max(a, b); }      // Intersection
        case 2u: { sdf_result[idx] = max(a, -b); }     // Difference
        default: { sdf_result[idx] = a; }
    }
}
```

### 9.3 Implementation Steps

1. **wgpu Setup**
   - Initialize `wgpu::Device` and `wgpu::Queue` in WASM
   - Handle browser WebGPU feature detection
   - Fallback to CPU if WebGPU unavailable

2. **Mesh to SDF Conversion**
   - Voxelize mesh to 3D grid (compute shader)
   - Calculate signed distance at each voxel

3. **CSG Compute Pipeline**
   - Create compute shader modules for each operation
   - Bind SDF buffers as storage
   - Dispatch workgroups for parallel processing

4. **SDF to Mesh Extraction**
   - Marching Cubes algorithm (compute shader)
   - Generate triangle mesh from SDF zero-crossing

5. **Toggle Integration**
   - Add `CsgMode` enum to render options
   - UI toggle in playground

### 9.4 Key Crates

| Crate | Purpose |
|-------|---------|
| `wgpu` | Cross-platform WebGPU API |
| `bytemuck` | Safe buffer casting |
| `web-sys` | WebGPU browser bindings |
| `futures` | Async buffer mapping |

### 9.5 Rust/WASM WebGPU Setup

```rust
use wgpu::util::DeviceExt;

/// CSG computation mode
#[derive(Clone, Copy, Debug, Default)]
pub enum CsgMode {
    #[default]
    Cpu,  // Phase 8: Manifold algorithm
    Gpu,  // Phase 9: WebGPU compute shaders
}

/// WebGPU CSG compute context
pub struct GpuCsgContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    csg_pipeline: wgpu::ComputePipeline,
}

impl GpuCsgContext {
    /// Initialize WebGPU for CSG operations
    pub async fn new() -> Result<Self, CsgError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok_or(CsgError::NoGpuAdapter)?;
        
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await?;
        
        // Create CSG compute pipeline...
        Ok(Self { device, queue, csg_pipeline })
    }
}
```

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
  libs/manifold-rs → Manifold → Mesh
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
  `libs/manifold-rs/src/manifold/constructors/{mod.rs, tests.rs}`.

- **TDD (Test-Driven Development)**  
  - Write tests **before** implementation.  
  - Workflow: **Red → Green → Refactor**.  

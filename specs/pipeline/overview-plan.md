# Rust OpenSCAD Pipeline – Overview Plan

_Last updated: 2025-11-25 — incorporates 2025 Rust/WASM best practices._

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

### 1.0.1 CPU vs GPU Processing Model

The operations in this pipeline are **CPU-bound geometry processing tasks**, not native GPU-bound rendering tasks:

| Layer | Role | Technology |
|-------|------|------------|
| **Rust (via WASM)** | Heavy geometry processing: calculating new mesh vertices, normals, indices for operations like `union()`, `linear_extrude()`, `hull()`, etc. | `openscad-mesh` crate with browser-safe algorithms |
| **WebGL** | Efficient rendering: displaying the generated mesh data in the browser's `<canvas>` element | Three.js + `BufferGeometry` |

**Workflow:**
1. **Define Shapes**: Rust code in `openscad-mesh` defines 2D/3D primitives from the evaluated AST.
2. **Apply Operations**: Rust calls mesh operations (`union()`, `linear_extrude()`, `translate()`, etc.) via browser-safe algorithms.
3. **Generate Mesh**: The library computes resulting geometry and produces a mesh (vertices + indices + normals).
4. **Render with WebGL**: The mesh data is passed to Three.js via typed arrays for GPU rendering.

This approach gives full power of Rust's computational capabilities within the browser, perfect for interactive 3D modeling.

All development must be broken down into **small, test-driven steps** that a developer can execute without needing external resources.

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
  
  `Playground UI -> Worker -> WASM -> openscad-mesh -> openscad-eval -> openscad-ast -> openscad-parser -> back up -> Mesh -> UI`

- **SRP & Structure**  
  Every *single-responsibility unit* (feature/struct/module) must live in its own folder with:
  
  - `mod.rs` – implementation
  - `tests.rs` – unit tests (TDD)
  
  Example:  
  `libs/openscad-mesh/src/primitives/cube/{mod.rs, tests.rs}`.

- **TDD (Test-Driven Development)**  
  - Write tests **before** implementation.  
  - Workflow: **Red → Green → Refactor**.  
  - Prefer many small tests over large, fragile ones.

- **No Mocks (Except I/O)**  
  - Use real implementations in tests.  
  - Only mock external systems (e.g. filesystem, network).  
  - Avoid mocking internal logic.

- **Explicit Errors, No Fallbacks**  
  - Use `Result<T, Error>` everywhere for fallible operations.  
  - **Never** silently ignore failures or add hidden fallbacks.  
  - All failures must surface as explicit errors with diagnostics.

- **Centralized Configuration**  
  - All magic numbers and configuration must live in `config.rs` per crate.  
  - Complex or cross-crate configuration should be funneled through a small number of well-documented config modules.

- **Zero-Copy Data Transfer**  
  - Never serialize mesh data to JSON for rendering.  
  - Use `Float32Array` (and related typed arrays) over WASM memory.

- **File Size Limits**  
  - Max **500 lines per file**.  
  - If a file grows beyond this, split it into smaller SRP-driven modules.

- **Documentation & Examples**  
  - Every public function, struct, and module must have comments.  
  - Where practical, include small examples in comments or tests showing expected usage.

- **Readability Over Cleverness**  
  - Prefer code that is easy to read, debug, and extend over clever one-liners.  
  - Choose predictable control flow and clear naming.

- **DRY & KISS**  
  - Avoid duplication; refactor into shared utilities when patterns appear.  
  - Keep algorithms as simple as possible while remaining correct and performant.

- **Naming Matters**  
  - Names must be self-explanatory and describe intent.  
  - Avoid abbreviations unless they are absolutely standard (`pos`, `idx`, `fn`).

- **Continuous Cleanup**  
  - Remove legacy/unused code as you go.  
  - Keep specifications and documentation aligned with the implementation; delete outdated docs.

---

## 3. Architecture & Data Flow

### 3.0 Simplified Pipeline Overview

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SIMPLIFIED PIPELINE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  playground ──► wasm ──► openscad-mesh ──► openscad-eval ──► openscad-ast  │
│                                                       │              │      │
│                                                       │              ▼      │
│                                               openscad-parser ◄──────┘      │
│                                                       │                     │
│  playground ◄── wasm ◄── openscad-mesh ◄─────────────┘                     │
│      │                         │                                            │
│      ▼                         │                                            │
│  Three.js                   Mesh                                            │
│  BufferGeometry            Buffers                                          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Example: `cube(10);` flow:**
1. **Playground** → sends `cube(10);` to WASM via Web Worker
2. **WASM** → calls `openscad-mesh::compile_and_render("cube(10);")`
3. **openscad-mesh** → calls `openscad-eval::evaluate("cube(10);")`
4. **openscad-eval** → calls `openscad-ast::parse_to_ast("cube(10);")`
5. **openscad-ast** → calls `openscad-parser::parse("cube(10);")` → returns **CST**
6. **openscad-ast** → transforms CST to typed **AST** → returns to openscad-eval
7. **openscad-eval** → evaluates AST → returns **Evaluated AST (Geometry IR)** to openscad-mesh
8. **openscad-mesh** → transforms Evaluated AST to **Mesh** → returns to WASM
9. **WASM** → returns `MeshHandle` (vertex + index buffers) to Playground
10. **Playground** → builds `THREE.BufferGeometry` and renders

### 3.0.1 High-Level Pipeline Components

1. **Input**  
   OpenSCAD source text from the Playground editor.

2. **Parser – `libs/openscad-parser`**  
   - Tree-sitter grammar for OpenSCAD via Rust bindings.  
   - Produces a typed **CST** (Concrete Syntax Tree) with spans.
   - **Public API**: `parse(source: &str) -> Result<Tree, ParseError>`

3. **AST – `libs/openscad-ast`**  
   - Converts CST into a typed **AST** (Abstract Syntax Tree).  
   - Every node carries a `Span { start: usize, end: usize }` for source mapping.
   - **Public API**: `parse_to_ast(source: &str) -> Result<Vec<Statement>, AstError>`
   - Uses `openscad-parser` internally to obtain CST.

4. **Evaluator – `libs/openscad-eval`**  
   - Walks the AST, manages scopes, modules, functions, special vars (`$fn`, `$fa`, `$fs`).  
   - Produces **Evaluated AST / Geometry IR** (e.g. `GeometryNode::Cube { size, center, span }`).  
   - **Public API**: `evaluate(source: &str) -> Result<Vec<GeometryNode>, EvalError>`
   - Uses `openscad-ast` internally to obtain AST.
   - Includes a **memoization/caching layer**:
     - Key: `hash(AST node + scope variables)`.  
     - If a subtree and its dependencies are unchanged, reuse cached result.
   - Includes **recursion depth checks** and uses `stacker` to avoid WASM stack overflows for complex recursive scripts.

5. **Mesh Generator – `libs/openscad-mesh`**  
   - Consumes Evaluated AST (Geometry IR) from `openscad-eval` and outputs a mesh.
   - **Public API**: `compile_and_render(source: &str) -> Result<Mesh, MeshError>`
   - Uses `openscad-eval` internally to obtain Evaluated AST.
   - Implements **browser-safe algorithms** for:
     - **3D Primitives**: cube, sphere, cylinder, polyhedron
     - **2D Primitives**: circle, square, polygon
     - **Boolean Operations**: union, difference, intersection (using BSP trees)
     - **Transformations**: translate, rotate, scale, mirror, multmatrix, resize, color
     - **Extrusions**: linear_extrude, rotate_extrude
     - **Advanced**: hull, minkowski, offset
   - Internal math uses `f64`; export to GPU-friendly `f32` at the boundary.

6. **WASM – `libs/wasm`**  
   - Thin interface-only orchestration layer.  
   - **Public API**: `compile_and_render(source: &str) -> Result<MeshHandle, JsValue>`
   - Calls `openscad-mesh` to process code and return mesh buffers.  
   - No mesh logic, no parsing logic—pure delegation.
   - Serializes rich diagnostics from error chains.  
   - Initializes panic hooks in debug builds.

7. **Playground – `apps/playground`**  
   - Svelte + Three.js front-end.  
   - Uses a Web Worker to invoke the WASM pipeline.  
   - Receives typed arrays and constructs `THREE.BufferGeometry` without copying.

### 3.1 Precision & Robustness

- **Floating Point Precision**  
  - All geometry calculations use `f64`.  
  - Types: `glam::DVec3`, `DMat4`, `DQuat`.  
  - `Vec3` type alias in `libs/openscad-mesh` must point to `DVec3`.

- **Export Precision**  
  - `f32` is allowed **only** for GPU-bound data in mesh buffers.

- **2D Operations**  
  - When converting `f64` → `i64` for integer-based algorithms, use a **standardized scaling factor** (e.g. `1e6`) configured centrally in `config.rs`.  
  - Avoid ad-hoc scaling to prevent grid-snapping artifacts.

- **Robust Predicates**  
  - Use the `robust` crate for exact predicates (e.g. orientation, incircle).  
  - Do **not** rely on naive epsilon comparisons for validity checks.

- **Browser Safety**  
  - All Rust crates must compile to `wasm32-unknown-unknown` without native dependencies.  
  - Verify generated WASM is browser-safe (no filesystem, no threads unless explicitly enabled).

### 3.1.1 Rust Libraries & Browser-Safe Algorithms

All geometry processing is CPU-bound and runs in Rust via WASM. The following algorithms and libraries provide the required functionality:

| Operation Category | OpenSCAD Functions | Rust Algorithm / Library |
|-------------------|-------------------|-------------------------|
| **Linear Algebra** | Matrix transforms | `glam` (f64 `DMat4`, `DVec3`, `DQuat`) |
| **3D Primitives** | `cube`, `sphere`, `cylinder`, `polyhedron` | Custom mesh generation (vertex + index buffers) |
| **2D Primitives** | `circle`, `square`, `polygon` | Custom 2D geometry with `robust` predicates |
| **Triangulation** | Polygon → triangles | Ear clipping algorithm (custom implementation) |
| **Boolean Ops** | `union`, `difference`, `intersection` | BSP trees (csg.js algorithm port) |
| **Hull** | `hull()` | QuickHull algorithm (custom implementation) |
| **Minkowski** | `minkowski()` | Convex sum / vertex-face iteration |
| **Offset** | `offset(r\|delta)` | Clipper2-style polygon offset (custom) |
| **Extrusions** | `linear_extrude`, `rotate_extrude` | Custom mesh generation with twist/scale support |
| **Transformations** | `translate`, `rotate`, `scale`, `mirror`, `multmatrix`, `resize` | `glam` matrix operations |
| **Color** | `color()` | Metadata propagation (RGBA per-vertex) |
| **Robust Predicates** | Geometric tests | `robust` crate for exact orientation/incircle |

**Key Design Decisions:**
- All algorithms implemented in pure Rust (no C/C++ dependencies) for browser safety.
- `f64` precision internally; `f32` only for GPU export buffers.
- BSP trees chosen over CGAL/Manifold for browser compatibility.
- `glam` used for all linear algebra (f64 mode enabled).

### 3.2 Source Mapping & Diagnostics

- Spans (`Span { start, end }`) must be preserved from:  
  `Parser -> AST -> Evaluator -> Geometry IR -> Mesh/diagnostics`.

- A common `Diagnostic` struct is used across the pipeline:

  ```rust
  struct Diagnostic {
      severity: Severity,  // Error, Warning
      message: String,
      span: Span,
      hint: Option<String>,
  }
  ```

- The Playground uses `Diagnostic` data for:
  - Editor squiggles.  
  - Mapping 3D triangle selections back to source.
- **WASM diagnostics bridge**  
  - `libs/openscad-ast`, `libs/openscad-eval`, and `libs/openscad-mesh` emit Rust-native `Diagnostic` values.  
  - `libs/wasm::diagnostics` converts these into WASM-visible `Diagnostic` / `DiagnosticList` types that can be consumed from JavaScript.  
  - The main WASM entry point for geometry, `compile_and_render(source: &str) -> Result<MeshHandle, JsValue>`, returns `Ok(MeshHandle)` on success and throws an error object on failure with the shape `{ diagnostics: Diagnostic[] }`.

### 3.3 Crate Dependency Graph

The workspace must form a clear, acyclic dependency graph:

```text
apps/playground (SvelteKit + Three.js)
  └─> wasm
        └─> openscad-mesh      (consumes Evaluated AST, generates Mesh)
              └─> openscad-eval   (produces Evaluated AST / IR)
                    └─> openscad-ast    (builds AST from CST)
                          └─> openscad-parser   (produces CST from source)

editors (VSCode, Neovim, etc.)
  └─> openscad-lsp          (tower-lsp + Tree-sitter)
        └─> openscad-ast?   (optional, for semantic features)
              └─> openscad-parser
```

Key rules:

- `openscad-parser` takes OpenSCAD source and produces a Tree-sitter CST.
- `openscad-ast` depends on `openscad-parser` to build typed AST nodes.
- `openscad-eval` depends on `openscad-ast` and produces an **Evaluated AST** (Geometry IR). It does **not** depend on `openscad-mesh`.
- `openscad-mesh` depends on `openscad-eval` and consumes the Evaluated AST to generate the mesh.
- `wasm` orchestrates via `openscad-mesh` public APIs only (e.g. `compile_and_render`), and does not implement mesh logic or import parser/AST crates directly.

### 3.4 Library Responsibilities & Relationships

- **`libs/openscad-parser`**  
  - Boundary to Tree-sitter, implemented entirely in Rust.  
  - Uses the generated Tree-sitter parser and its Rust bindings to turn **OpenSCAD source text** into a **CST** (Concrete Syntax Tree).

- **`libs/openscad-ast`**  
  - Owns the **typed AST** data structures.  
  - Converts CST to AST, normalizing positional and named arguments (e.g. `cube(size=[10,10,10], center=true)` and `cube([10,10,10], center=false)`) into strongly-typed nodes validated by evaluator tests.
  - **Modular Architecture** (SRP-compliant):
    - `parser/mod.rs`: Public API and entry point (`parse_to_ast`)
    - `parser/statement.rs`: Top-level statement dispatcher
    - `parser/module_call.rs`: Primitive parsing (cube, sphere, cylinder)
    - `parser/transform_chain.rs`: Transform operations (translate, rotate, scale)
    - `parser/assignments.rs`: Variable declarations
    - `parser/arguments/`: Specialized argument parsers per primitive
      - `cube.rs`, `sphere.rs`, `cylinder.rs`: Primitive-specific argument parsing
      - `shared.rs`: Common utilities (`parse_f64`, `parse_u32`, `parse_bool`, `parse_vector`)
    - All files under 500 lines, comprehensive documentation, tests co-located with code

- **`libs/openscad-eval`**  
  - Interprets the AST and produces **Evaluated AST** (Geometry IR).  
  - Handles variables, modules, loops, functions.  
  - **Pure data transformation**: AST -> Evaluated AST. No geometry generation.  
  - Tracks `$fn`, `$fa`, `$fs` in `EvaluationContext` and uses a shared `resolution::compute_segments` helper so `sphere()` nodes inherit the exact OpenSCAD fragment rules (if `$fn>0` use it, else `ceil(min(360/$fa, 2πr/$fs))` with a lower bound of five fragments).  
  - **Task 5.1 plan:** translate/rotate/scale statements wrap child nodes inside `GeometryNode::Transform { matrix, child, span }`, composing glam matrices in OpenSCAD’s inside-out order. Evaluator unit tests will cover (1) translate-only bounding-box shifts, (2) rotate-then-translate ordering, and (3) scale anchored at the origin. Each test will include inline comments plus doc examples per project rules.  
  - **Transform data flow:**
    1. Parser emits `Statement::Translate/Rotate/Scale` nodes with spans anchored at the keyword location.
    2. Evaluator converts those nodes into 4×4 glam matrices using column-vector semantics (matching [OpenSCAD Transformations Manual](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Transformations)), multiplying in reverse order of appearance to respect inside-out evaluation.
    3. The resulting `GeometryNode::Transform` is appended to the IR, preserving the span so diagnostics bubble up to the original call site.
    4. Downstream crates never recompute transforms; they consume the already-baked matrix, keeping SRP boundaries intact.

- **`libs/openscad-mesh`**  
  - Mesh generator that consumes Evaluated AST from `openscad-eval`.  
  - **Public API**: `compile_and_render(source: &str) -> Result<Mesh, MeshError>`
  - Uses `openscad-eval` internally to parse and evaluate OpenSCAD source.
  - Implements **browser-safe algorithms** for all geometry operations:
    - **3D Primitives**: `cube`, `sphere`, `cylinder`, `polyhedron`
    - **2D Primitives**: `circle`, `square`, `polygon`
    - **Boolean Operations**: `union`, `difference`, `intersection` (BSP tree algorithm)
    - **Transformations**: `translate`, `rotate`, `scale`, `mirror`, `multmatrix`, `resize`, `color`
    - **Extrusions**: `linear_extrude`, `rotate_extrude`
    - **Advanced**: `hull`, `minkowski`, `offset`
  - **100% OpenSCAD API Compatibility**: Parameters and output shapes match OpenSCAD exactly.
  - **SRP structure**: Each primitive/operation in its own folder with `mod.rs` + `tests.rs`.
  - **Transform application**: `from_ir` invokes transform applicators that multiply vertex positions by the 4×4 matrix from the evaluator.

- **`libs/wasm`**  
  - Thin bridge between browser and Rust.  
  - Calls `openscad-mesh` to process code and return mesh buffers.  
  - No mesh logic, no parsing logic—pure delegation.
  - Exposes a small set of WASM entry points, notably `compile_and_render(source: &str)` and `compile_and_count_nodes(source: &str)`, where `compile_and_render` returns a `MeshHandle` that owns vertex (`Float32Array`) and index (`Uint32Array`) buffers plus basic counts.  
  - Produces the **only browser-facing WASM bundle**: the wasm-bindgen output under `libs/wasm/pkg` (for example `wasm.js` + `wasm_bg.wasm`).  
  - Is consumed by TypeScript via a small wrapper (for example `initWasm`, `compile(...)`) that forwards the resulting `MeshHandle` into the Three.js scene.

- **`libs/openscad-lsp`**  
  - Rust-native Language Server built with `tower-lsp`.  
  - Uses `libs/openscad-parser` (Tree-sitter grammar + Rust bindings) to maintain incremental parse trees for open documents.  
  - Provides editor features such as diagnostics, document symbols, and later go-to-definition/completion via high-level domain types.  
  - Runs outside WASM (as a native process) and never exposes raw `tree_sitter::Node` values to clients.

#### 3.4.1 Data Flow Diagram

```text
OpenSCAD source text
        │
        ▼
libs/openscad-parser   (CST)
        │
        ▼
libs/openscad-ast      (AST)
        │
        ▼
libs/openscad-eval     (Evaluated AST / IR)
        │
        ▼
libs/openscad-mesh     (Mesh)
        │
        ▼
libs/wasm              (MeshHandle)
        │
        ▼
apps/playground
```

### 3.5 Minimal `cube(10);` Pipeline (Simplified)

For the initial tracer-bullet implementation, all layers must participate in a minimal `cube(10);` flow with no shortcuts:

- **Playground**  
  - The user edits `cube(10);` in the OpenSCAD editor and triggers a compile/render action.
- **`libs/wasm`**  
  - Receives the source string and exposes `compile_and_render(source: &str)` (plus a helper `compile_and_count_nodes` for diagnostics/tests).  
  - Forwards the raw OpenSCAD source to `libs/openscad-mesh` as the orchestrator of the geometry pipeline, then wraps the resulting `MeshBuffers` in a WASM-friendly `MeshHandle` that owns vertex and index buffers alongside simple counts.
- **`libs/openscad-mesh`**  
  - Calls into `libs/openscad-eval` with the original OpenSCAD source string `cube(10);` to obtain an **evaluated/flattened AST**.  
  - Transforms this evaluated AST into a mesh and returns it to `libs/wasm`.
- **`libs/openscad-eval`**  
  - Invokes `libs/openscad-ast` with the original source so that the AST layer can obtain a typed AST from the CST.  
  - Decides whether evaluation is required for the current AST, evaluates and resolves it, and returns an evaluated/flattened AST to `libs/openscad-mesh`.
- **`libs/openscad-ast`**  
  - Calls `libs/openscad-parser` with `cube(10);` to obtain a Tree-sitter CST.  
  - Transforms the CST into a typed AST and returns it to `libs/openscad-eval`.
- **`libs/openscad-parser`**  
  - Parses the OpenSCAD source into a CST using the Tree-sitter grammar and returns it to `libs/openscad-ast`.
- **Back up to geometry and rendering**  
  - `libs/openscad-mesh` converts the evaluated AST for `cube(10);` into a mesh and returns this mesh to `libs/wasm`.  
  - `libs/wasm` returns the generated mesh to the Playground as a `MeshHandle` (node/vertex/triangle counts plus vertex and index buffers).  
  - The Playground's Web Worker forwards this handle to the Three.js `SceneManager`, which builds a `THREE.BufferGeometry` from the typed arrays and renders the result, without any hard-coded primitive geometry.

This minimal vertical slice is the reference "happy path" for all future primitives and features.

---

## 4. Implementation Phases (High-Level)

A vertical slice is always preferred over broad, unfinished scaffolding.

- **Phase 1 – Infrastructure & Tracer Bullet**  
  - Set up the Cargo workspace, core crates, `libs/openscad-lsp`, and basic Svelte+WASM Playground.  
  - Implement minimal “hello world” paths:  
    - Playground → Worker → WASM (for example, echo a string or return a trivial mesh).  
    - Editor → `openscad-lsp` (a tower-lsp server that can parse a file and publish basic syntax diagnostics).

- **Phase 2 – First Primitive (Cube)**  
  - Implement a fully working `cube()` primitive end-to-end.  
  - Include AST, IR, Evaluator, `openscad-mesh` cube, and zero-copy rendering.  
  - Introduce diagnostics and basic error handling.

- **Phase 3 – Filesystem & Parameters**  
  - Add named arguments and a `FileSystem` abstraction with a virtual in-memory implementation for WASM.  
  - Introduce an `EvaluationContext` that tracks resolution special variables (`$fn`, `$fa`, `$fs`) and other global parameters.  
  - Support `include` / `use` semantics.

- **Phase 4 – Sphere & Resolution Controls**  
  - Implement `sphere()` with OpenSCAD-compatible tessellation.  
  - Use the `EvaluationContext` resolution parameters when tessellating the sphere so `$fn/$fa/$fs` yield the same fragment counts as OpenSCAD.  
  - Keep regression tests comparing vertex/triangle counts and fragment clamping to guard against regressions.

- **Phase 4b – 2D Primitives**  
  - Implement OpenSCAD's 2D primitives: `square`, `circle`, and `polygon`. Each primitive must emit identical vertex ordering, winding, and diagnostics.  
  - **`square(size, center)`**: scalar/vector `size`, optional `center`, range checking.  
  - **`circle(r | d)`**: use `$fn/$fa/$fs` to tessellate outlines, honoring `r`/`d` precedence.  
  - **`polygon([points], [paths])`**: support both implicit outer path and explicit `paths` with hole semantics.  
  - Introduce a reusable 2D outline representation in `openscad-mesh` that downstream extruders (`linear_extrude`, `rotate_extrude`) can consume.  
  - Tests: parity fixtures comparing vertex sequences, winding, and error messages vs. OpenSCAD.

- **Phase 5 – Cylinders, Polyhedra & Transformations**  
  - Support `translate`, `rotate`, `scale`, `mirror`, `multmatrix`, `resize`, `color` transformations in IR and `openscad-mesh`.  
  - Ensure transformations preserve and update Spans for diagnostics.  
  - **Cylinder**: `cylinder(h, r|d, center)` and `cylinder(h, r1|d1, r2|d2, center)` with `$fn/$fa/$fs` driven fragment counts.  
  - **Polyhedron**: `polyhedron(points, faces, convexity)` with validation rules (face reversal, convexity flag, strict error logging).  
  - Update docs/tests whenever OpenSCAD introduces changes so parity remains explicit.

- **Phase 6 – Boolean Operations (Priority: HIGH)**  
  - Implement robust `union`, `difference`, and `intersection` using BSP tree algorithms (browser-safe).  
  - Use robust predicates for numerical stability.  
  - Add fuzz testing to validate mesh invariants.
  - **Skip performance optimization for now**; focus on correctness and feature coverage.

- **Phase 7 – Extrusions & Advanced Operations**  
  - **`linear_extrude(height, center, convexity, twist, slices)`**: extrude 2D shapes along Z axis.  
  - **`rotate_extrude(angle, convexity)`**: revolve 2D shapes around Z axis.  
  - **`hull()`**: convex hull using QuickHull algorithm.  
  - **`minkowski(convexity)`**: Minkowski sum.  
  - **`offset(r|delta, chamfer)`**: 2D offset operation.

### 4.1 OpenSCAD Feature Coverage Matrix

This matrix documents **100% OpenSCAD API compatibility** with exact parameter signatures from `openscad/src/core/*.cc`:

| OpenSCAD Feature | Parameters | Status | Rust Algorithm |
|------------------|-----------|--------|----------------|
| **3D Primitives** | | | |
| `cube()` | `size` (scalar or `[x,y,z]`), `center` | 🔲 Pending | Vertex + index buffer |
| `sphere()` | `r` or `d`, `$fn/$fa/$fs` | 🔲 Pending | Lat/long tessellation |
| `cylinder()` | `h`, `r`/`d` or `r1`/`d1`+`r2`/`d2`, `center`, `$fn/$fa/$fs` | 🔲 Pending | Circle extrusion |
| `polyhedron()` | `points`, `faces`, `convexity` | 🔲 Pending | Face validation + triangulation |
| **2D Primitives** | | | |
| `circle()` | `r` or `d`, `$fn/$fa/$fs` | 🔲 Pending | Polygon tessellation |
| `square()` | `size` (scalar or `[x,y]`), `center` | 🔲 Pending | 4-vertex polygon |
| `polygon()` | `points`, `paths` (optional for holes) | 🔲 Pending | Ear clipping triangulation |
| **Transformations** | | | |
| `translate()` | `v=[x,y,z]` | 🔲 Pending | `glam::DMat4::from_translation` |
| `rotate()` | `a=[x,y,z]` (Euler) or `a`, `v=[x,y,z]` (axis-angle) | 🔲 Pending | `glam` rotation matrices |
| `scale()` | `v` (scalar or `[x,y,z]`) | 🔲 Pending | `glam::DMat4::from_scale` |
| `mirror()` | `v=[x,y,z]` | 🔲 Pending | Reflection matrix |
| `multmatrix()` | `m` (4×4 matrix) | 🔲 Pending | Direct `DMat4` |
| `resize()` | `newsize=[x,y,z]`, `auto=[bool...]`, `convexity` | 🔲 Pending | Bounding box scale |
| `color()` | `"name"`/`"#hex"`/`[r,g,b,a]`, `alpha` | 🔲 Pending | Per-vertex RGBA metadata |
| **Boolean Operations** | | | |
| `union()` | children | 🔲 Pending | BSP tree: `A.clipTo(B); B.clipTo(A); ...` |
| `difference()` | children (first - rest) | 🔲 Pending | BSP tree: `~(~A \| B)` |
| `intersection()` | children | 🔲 Pending | BSP tree: `~(~A \| ~B)` |
| **Extrusions** | | | |
| `linear_extrude()` | `height`/`v`, `center`, `convexity`, `twist`, `slices`, `scale`, `$fn/$fa/$fs` | 🔲 Pending | Slice-based mesh generation |
| `rotate_extrude()` | `angle`, `start`, `convexity`, `$fn/$fa/$fs` | 🔲 Pending | Revolution mesh generation |
| **Advanced Operations** | | | |
| `hull()` | children | 🔲 Pending | QuickHull algorithm |
| `minkowski()` | `convexity`, children | 🔲 Pending | Convex sum / vertex-face iteration |
| `offset()` | `r` or `delta`, `chamfer`, `$fn/$fa/$fs` | 🔲 Pending | Clipper2-style offset |
| `fill()` | children | 🔲 Pending | Polygon filling |
| **Special Variables** | | | |
| `$fn` | Fragment count override | 🔲 Pending | `EvaluationContext` |
| `$fa` | Minimum fragment angle | 🔲 Pending | `resolution::compute_segments` |
| `$fs` | Minimum fragment size | 🔲 Pending | `resolution::compute_segments` |

**Compatibility Notes:**
- All parameter names and precedence rules match OpenSCAD (e.g., `d` takes precedence over `r`).
- Warning/error messages mirror OpenSCAD's `LOG()` output where applicable.
- Face winding and vertex ordering match OpenSCAD's `PolySet` output.
- `$fn/$fa/$fs` resolution rules: if `$fn > 0`, use it; else `ceil(min(360/$fa, 2πr/$fs))` with min 5 fragments.

A detailed breakdown of tasks, subtasks, and acceptance criteria for each phase lives in `tasks.md`.

---

## 5. Coding Standards & Global Guidelines

### 5.1 Rust & Crate-Level Standards

- **SRP & Folder Layout**  
  - Each SRP unit resides in its own folder with `mod.rs` and `tests.rs`.  
  - Example: `libs/openscad-mesh/src/primitives/cube/{mod.rs, tests.rs}`.

- **File Length**  
  - Maximum 500 lines per file; refactor and split as needed.

- **Error Handling Policy**  
  - No silent fallbacks or best-effort behaviour on failure.  
  - Return explicit errors, with diagnostics and spans, for all failure modes.

- **Configuration**  
  - All constants and tunable parameters go into `config.rs`.  
  - Cross-cutting precision parameters (like `EPSILON`) can be toggled via Cargo features (e.g. `high_precision`).

- **Testing & TDD**  
  - Write tests first for every new behaviour.  
  - Use unit tests for small units and integration tests for cross-crate flows.  
  - Add **visual regression tests** (golden `.scad` → `.stl`/`.obj` hashes).  
  - Add **fuzz tests** for boolean operations using `proptest`.

- **Performance & Safety**  
  - Prefer safe Rust; use `unsafe` only in small, well-audited sections encapsulated behind safe APIs.  
  - Enable overflow checks in builds that guard geometry math (e.g. `overflow-checks = true` in appropriate profiles), especially in debug and fuzzing configurations.  
  - Use `rayon` for safe data-parallel operations when beneficial (browser-safe when WASM threads are available).

### 5.2 WASM & TypeScript Standards

- **TypeScript Types**  
  - No `any` types. Always use precise TypeScript types or generics.  
  - Model WASM handles and messages with clear interfaces.

- **WASM Error Propagation Contract**  
  - The primary geometry entry point `compile_and_render(source: &str) -> Result<MeshHandle, JsValue>` must not stringify diagnostics; all failures are surfaced as a structured JavaScript object.  
  - In browser code, an error thrown by `compile_and_render` is expected to have a `diagnostics` property containing an array of wasm `Diagnostic` objects, each exposing `severity()`, `message()`, `start()`, `end()`, and `hint()`.  
  - Typical usage pattern in TypeScript:

    ```ts
    try {
      const mesh = await compile_and_render(source);
      // Success: use mesh.vertex_count(), mesh.vertices(), mesh.indices(), etc.
    } catch (error: unknown) {
      const payload = error as { diagnostics?: Diagnostic[] };
      const diagnostics = payload.diagnostics ?? [];
      for (const d of diagnostics) {
        console.error(d.severity(), d.message(), d.start(), d.end(), d.hint());
      }
    }
    ```

  - Any pipeline error produced by Rust that lacks a `diagnostics` array is treated as a bug and must be fixed, not as an acceptable fallback.

- **WASM Memory Lifecycle**  
  - Pattern: **Alloc → View → Upload → Free**.
  - Rust allocates mesh buffers and returns a handle.  
  - JS creates a typed view (`Float32Array`/`Uint32Array`) over WASM memory and uploads to GPU.  
  - JS **must** call a Rust-exported `free_*` function (or invoke `.free()` on a wrapper) immediately after upload.

- **Panic Debugging**  
  - `console_error_panic_hook` must be active for debug builds to avoid opaque `unreachable` panics.

- **Playground Architecture**  
  - Heavy computation must occur in a Web Worker.  
  - Main thread only handles UI and rendering.

- **Playground Tooling Stack**  
  - `apps/playground` uses Svelte 5 with SvelteKit, Vite 7, Vitest 4, TypeScript 5.9, ESLint 9, and plain `three` (no Svelte wrapper library).  
  - The worker uses a structured message protocol (`CompileSuccess`, `CompileError` with `DiagnosticData[]`) to communicate with the UI.
  - All `pnpm` scripts (dev, test, lint) must complete with zero TypeScript and ESLint errors.

- **Async & Long-Running Operations**  
  - Expose long-running operations (e.g. boolean CSG) as async WASM functions (via `wasm-bindgen-futures`) and `await` them in the worker to avoid blocking its event loop.

- **TypeScript Wrapper for WASM**  
  - All raw pointer logic lives in a small, well-tested TS wrapper module.  
  - The wrapper returns higher-level objects (e.g. `Mesh`) with explicit `dispose()`/`free()` methods and/or `FinalizationRegistry` for safety.

- **WASM Runtime Bundle**  
  - The only browser-facing WASM entrypoint is the wasm-bindgen glue in `libs/wasm/pkg/wasm.js` and its `wasm_bg.wasm` binary, consumed via the `$wasm` alias in `apps/playground`.  
  - Build tooling such as `scripts/build-wasm.sh` and any Node-based helper (for example `build-wasm.js`) is strictly CLI-only and must never be imported into browser bundles, mirroring Tree-sitter's separation between its `binding_web` runtime and `script/build.js`.

- **WASM Parallelism**  
  - Where browser support allows, enable WASM threads + shared memory so `rayon` can run in parallel inside `openscad-mesh` for heavy operations.

- **Local WASM Build & Distribution**  
  - `libs/wasm` is built for the `wasm32-unknown-unknown` target via dedicated helpers (`scripts/build-wasm.sh` on Unix-like systems and a Node CLI equivalent on Windows, such as `build-wasm.js`).  
  - These helpers ensure the `wasm32-unknown-unknown` target and a compatible `wasm-bindgen` CLI are installed, respect `WASI_SDK_PATH` when compiling any C/C++ dependencies, run `cargo build --release -p wasm --target wasm32-unknown-unknown`, and invoke `wasm-bindgen --target web` to emit artifacts into `libs/wasm/pkg`.  
  - `apps/playground` pnpm scripts (for example `pnpm build:wasm`) call these helpers so that the `libs/wasm/pkg/wasm.js` + `wasm_bg.wasm` bundle used in the browser is always up to date.

### 5.3 Documentation & Project Hygiene

- Keep `overview-plan.md` and `tasks.md` up to date with implementation changes.  
- Remove obsolete sections instead of letting them drift.  
- Ensure examples in comments and tests remain correct and relevant.

### 5.4 Performance Targets

- For typical interactive models (~10k triangles), aim for end-to-end compile times under ~100 ms on a mid-tier 2025 laptop browser, and **≤ 50 ms** for boolean-heavy ~10k triangle scenes when WASM threads + `rayon` are available.  
- Use tools such as `wasm-opt -O4`, dead-code elimination, and profiling to keep the WASM bundle lean.  
- Benchmark with synthetic scenes generated via property/fuzz tests to capture worst-case CSG patterns.

---

## 6. References

These internal resources are the primary references for the implementation:

- **OpenSCAD Syntax: Grammar Definition and Test Coverage**  
  - `libs/openscad-parser/src/grammar.json` (canonical Tree-sitter grammar)  
  - `libs/openscad-parser/test/corpus/**`  
  - Optionally, an upstream OpenSCAD grammar such as `holistic-stack/tree-sitter-openscad` may be consulted for reference or additional test cases, but `grammar.json` remains the source of truth.

- **OpenSCAD Reference Implementation**  
  - The OpenSCAD C++ source under `openscad/` serves as the reference for feature parity (parameters, output shapes, diagnostics).  
  - `libs/openscad-mesh` implements equivalent functionality using browser-safe algorithms.

- **Browser-Safe Algorithms**  
  - **BSP Trees**: For boolean operations (union, difference, intersection) - based on csg.js by Evan Wallace.  
  - **Ear Clipping**: For polygon triangulation.  
  - **QuickHull**: For convex hull computation.  
  - **Robust Predicates**: The `robust` crate for exact geometric predicates.

All new work should keep this overview in sync, and `tasks.md` should always reflect the current state of the actionable backlog.

---

## 7. CI/CD & Testing Pipeline

- Use GitHub Actions (or an equivalent CI system) to run on every PR:
  - `cargo fmt`, `cargo clippy`, and `cargo test` for all crates.  
  - WASM tests (e.g. `wasm-pack test --headless --chrome` for `libs/wasm` or related crates) run **inside the same Dockerized Rust+WASM image** used for builds, not directly on the CI host.  
  - Playwright end-to-end tests for the Playground.  
  - Golden regression tests and periodic fuzz tests (at least nightly).
- Treat failing CI as a blocker; update this plan and `tasks.md` alongside behavioural changes.

# Rust OpenSCAD Pipeline – Task Breakdown

> This file is the **actionable backlog** for the Rust OpenSCAD pipeline.  
> It is structured into small, test-driven tasks and subtasks.  
> See `overview-plan.md` for goals, architecture, and coding standards.

---

## Conventions

- Each task is **small**, **TDD-first**, and adheres to the **SRP**.
- No mocks, except for filesystem or other external I/O boundaries.
- All failures must return explicit errors; no hidden fallbacks.
- All geometry code uses `f64` internally; `f32` is only for GPU export.
- Keep files under **500 lines**; split as soon as they grow too large.
 - For every new public API (functions, structs, modules), add Rust doc-comments with at least one minimal usage example, and keep examples compiling as part of tests.

For each task, we list:

- **Goal** – What we want to achieve.  
- **Steps** – Concrete actions to perform.  
- **Acceptance Criteria** – How we know the task is done.

---

## Phase 0 – Pre-Bootstrap Evaluation

### Task 0.1 – Confirm Geometry Kernel Strategy (Direct Port)

**Goal**  
Confirm and document that `libs/manifold-rs` will be a direct port of the local C++ Manifold algorithms (index-based half-edge), not a thin wrapper around external kernels like `manifold3d` or `csgrs`.

**Steps**

1. Survey `manifold/` C++ sources to outline the main data structures and algorithms to port into `libs/manifold-rs`.  
2. Optionally review `manifold3d` Rust bindings and the `csgrs` crate **only as references** (for ideas, tests, or benchmarks), not as runtime geometry kernels.  
3. Write a short design note describing:
   - The chosen approach: direct port into `libs/manifold-rs` with an index-based half-edge.  
   - Which parts of C++ Manifold are in scope for the first vertical slices.  
   - An explicit statement that `manifold3d` and `csgrs` will not be used as runtime geometry kernels behind `libs/manifold-rs`.

**Acceptance Criteria**

- A design note exists in the repo (e.g. `specs/pipeline/kernel-decision.md`) clearly stating that `libs/manifold-rs` is a direct C++ Manifold port and that `manifold3d`/`csgrs` are not used as interim kernels.  
- `overview-plan.md` and the dependency graph remain valid for this approach.

### Task 0.2 – Select Tree-sitter Grammar Variant

**Goal**  
Confirm which OpenSCAD grammar to use for Tree-sitter (local grammar vs upstream alternatives).

**Steps**

1. Evaluate the existing `libs/openscad-parser/grammar.js` vs any upstream grammars (e.g. `holistic-stack/tree-sitter-openscad`).  
2. Confirm coverage for primitives, transformations, booleans, control flow, and advanced features.  
3. Decide whether to:
   - Keep the current local grammar, or  
   - Adopt/track an upstream grammar and build its WASM via `tree-sitter-cli`.

**Acceptance Criteria**

- The chosen grammar variant is explicitly documented in `overview-plan.md` and this task file.  
- Parser tests pass for the supported OpenSCAD syntax subset.

---

## Phase 1 – Infrastructure & "Tracer Bullet"

### Task 1.1 – Workspace & Crate Setup

**Goal**  
Initialize the Cargo workspace and core crates, with proper dependencies and configuration.

**Steps**

1. **Workspace Configuration**
   - Update root `Cargo.toml` to include:
     - `libs/openscad-parser`
     - `libs/openscad-ast`
     - `libs/openscad-eval`
     - `libs/manifold-rs`
     - `libs/wasm`

2. **Create `libs/manifold-rs`**
   - Create crate structure:
     - `src/lib.rs`
     - `src/config.rs`
     - `src/core/vec3/mod.rs` (type alias `pub type Vec3 = glam::DVec3;`).
   - Add dependencies in `Cargo.toml`:
     - `glam` (f64 support)
     - `thiserror`
     - `robust`
     - `rayon`
   - Define `config.rs` with:
     - `EPSILON`
     - `DEFAULT_SEGMENTS`
     - Document purpose and defaults.

3. **Create `libs/openscad-eval`**
   - Create structure:
     - `src/lib.rs`
     - `src/ir/mod.rs`
     - `src/evaluator/mod.rs`
     - `src/filesystem.rs`
   - Add dependencies:
     - `glam`
     - `stacker`
   - Add basic `GeometryNode` enum scaffold under `ir/` (cube variant can initially be a stub, to be filled in Phase 2).
   - Implement **recursion depth guards** and configure `stacker` for deeper recursion in the evaluator entry points.

4. **Create `libs/wasm`**
   - Crate type: `cdylib`.
   - Dependencies:
     - `wasm-bindgen`
     - `console_error_panic_hook`
   - In `lib.rs`:
     - Initialize `console_error_panic_hook` in a dedicated `init` function called by JS in debug builds.
     - Add a placeholder `greet()` function and export it via `#[wasm_bindgen]`.
   - Ensure the workspace builds for the `wasm32-unknown-unknown` target and, if using `rayon` on WASM, that appropriate thread/atomics features, bulk-memory, and linker flags are enabled for future WASM threads.

**Acceptance Criteria**

- `cargo build` at workspace root succeeds.
- `cargo test` (no-op tests) succeeds for all new crates.
- Crate dependency graph matches the overview (no unexpected cycles).

---

### Task 1.2 – Playground Setup (Svelte + Three.js + Worker)

**Goal**  
Set up the Playground with a Web Worker and Three.js scene, ready to call WASM.

**Steps**

1. **Project Initialization**
   - Under `playground/`, initialize or confirm a SvelteKit/Vite setup.
   - Use **pnpm** as the package manager (all commands below assume `pnpm dev`, `pnpm lint`, etc.).
   - Ensure TypeScript is enabled and strict mode is on.
   - Enforce `kebab-case` for TypeScript file and folder names (e.g. `mesh-wrapper.ts`, `pipeline.worker.ts`).

2. **Web Worker for Pipeline**
   - Create `src/worker/pipeline.worker.ts`.
   - Responsibilities:
     - Load the WASM bundle.  
     - Expose a message protocol for `compile(source: string)`.
     - Forward diagnostics and mesh handles back to the main thread.

3. **TypeScript Wrapper for WASM (Glue Code)**
   - Create a dedicated module (e.g. `src/lib/wasm/mesh-wrapper.ts`) that:
     - Encapsulates raw pointer handling (`ptr`, `len`) from `MeshHandle`.
     - Provides a `Mesh` class or interface with:
       - Typed views over WASM memory (`Float32Array`, `Uint32Array`).
       - A `dispose()` or `free()` method that calls the Rust `free_*` entry point.
     - Optionally uses `FinalizationRegistry` to guard against leaks (but still require explicit `dispose()` in critical paths).
   - Enforce **no `any` types**; use exact TypeScript definitions for all WASM interactions.
   - Define explicit TS interfaces/types for `MeshHandle`, diagnostics, and worker messages instead of relying on inference from `any`.

4. **Three.js Scene Manager**
   - Implement `src/components/viewer/scene-manager.ts` with SRP:
     - Set up renderer, camera, lights, and controls (`OrbitControls`).
     - Expose functions to:
       - Attach to a canvas.
       - Update geometry from provided buffers.

**Acceptance Criteria**

- `pnpm dev` in `playground/` starts without errors.
- A stub pipeline can send dummy geometry buffers from the worker to the main thread and render a simple mesh (e.g. hard-coded triangle).
- All TypeScript code compiles under strict mode with **no `any` usages**.

---

### Task 1.4 – Parser Infrastructure (Rust/WASM)

**Goal**  
Ensure the existing Rust `libs/openscad-parser` (Tree-sitter bindings) is wired through the Rust/WASM pipeline, so parsing occurs entirely inside Rust and the Playground never imports `web-tree-sitter` or parser WASM directly.

**Steps**

1. **Parser Crate Wiring**
   - Confirm that `libs/openscad-parser` is part of the Cargo workspace and that its Rust bindings under `libs/openscad-parser/bindings/rust/lib.rs` are used by `libs/openscad-ast` to build typed AST nodes from CST.  
   - Ensure no `web-tree-sitter` or `tree-sitter.wasm` assets are referenced from the Playground.

2. **WASM Entry Point for Parsing**
   - In `libs/wasm`, expose a small, synchronous or async Rust function (e.g. `parse_only(source: &str) -> Result<(), Vec<Diagnostic>>` or similar) that:
     - Calls into `libs/openscad-parser` (via `openscad-ast` where appropriate) to parse the source into CST/AST.  
     - Returns either success or a list of diagnostics describing parse errors.

3. **Worker Integration (Rust-Backed Parsing)**
   - In the Playground worker, call the `parse_only` (or equivalent) function exported from the `libs/wasm` bundle as the **only** way to parse OpenSCAD source.  
   - Do not load or initialize `web-tree-sitter` in TypeScript.

**Acceptance Criteria**

- Given a basic OpenSCAD snippet (e.g. `cube(10);`), the worker can call the `libs/wasm` parse entry point and receive either success or structured diagnostics.  
- No `web-tree-sitter` dependency or Tree-sitter WASM assets exist in the Playground; parsing happens entirely in Rust/WASM through `libs/wasm`.

---

## Phase 2 – First Primitive (Cube)

#### Porting Guidelines for `libs/manifold-rs` (C++ → Rust)

Before implementing specific primitives or boolean operations in `libs/manifold-rs`, apply these guidelines to make the port from the C++ Manifold codebase mechanical and consistent:

- **Half-Edge Representation**  
  - Replace raw pointers or index fields that point into C++ arrays with **index-based handles** in Rust (`u32` indices into `Vec` arenas).  
  - Keep ownership in central arenas (e.g. `Vec<Vertex>`, `Vec<HalfEdge>`, `Vec<Face>`); pass indices between functions instead of references with complex lifetimes.

- **Parallelism (thrust/TBB → `rayon`)**  
  - For C++ code that uses `thrust`/TBB to parallelize loops over faces/edges, map them to `par_iter()`/`par_iter_mut()` over the corresponding `Vec`s in Rust.  
  - Keep side effects confined to data local to each loop iteration, or use scoped parallelism patterns to avoid shared mutable state.

- **Memory & Safety**  
  - Eliminate manual memory management patterns from C++ (new/delete, raw pointer arithmetic).  
  - Use `unsafe` only in small, well-audited sections where performance demands it, and always expose a safe API on top.  
  - Replace C++ `assert`/`abort` with explicit `Result`-based errors or internal debug assertions that never leak to the public API as panics.

- **Error Handling**  
  - Convert C++ failure paths (error codes, special values) into typed Rust errors using `thiserror`.  
  - Ensure all public `manifold-rs` operations used by the Evaluator return `Result<Self, Error>` or similar, never relying on panics.

- **Testing Strategy**  
  - Where possible, mirror C++ test cases/fixtures in Rust, comparing topological invariants (e.g. Euler characteristic, manifold validity) rather than relying only on exact floating-point equality.  
  - Add new tests that exercise edge cases surfaced by fuzzing and visual regression.

- **Robust Predicates Initialization**  
  - If the chosen robust predicates library requires a one-time initialization (for example an `exactinit()` call), invoke this once at WASM startup (e.g. in a Rust `init` function or lazy static) so all downstream geometry code benefits from correct predicate behaviour.

**Example: Porting a Boolean Union Loop (Conceptual)**

- **C++ pattern (conceptual)**  
  - A typical C++ Manifold boolean operation iterates over `std::vector<Halfedge>` and `std::vector<Face>` collections, sometimes in parallel using `thrust`/TBB:
    - Build or update arrays of half-edges and faces.  
    - For each face/edge, classify it against a plane or another mesh and mark results in-place.  
    - Use parallel for-loops to accelerate classification and merging.

- **Rust pattern (mechanical translation)**  
  - Model the same data as arenas in `manifold-rs`:
    - `vertices: Vec<Vertex>`, `half_edges: Vec<HalfEdge>`, `faces: Vec<Face>` with `u32` indices connecting them.  
    - Each face stores indices into `half_edges`, each half-edge stores indices into `vertices` and adjacent faces.
  - Replace C++ loops with safe Rust iteration:
    - Sequential: `for face_idx in 0..faces.len() { let face = &mut faces[face_idx]; /* classify/update */ }`.  
    - Parallel: `faces.par_iter_mut().for_each(|face| { /* classify/update */ });` when no cross-face mutation is required (or when shared state is carefully managed).
  - Instead of mutating global flags or using raw pointers, have classification functions return explicit enums/results and update the arenas through indices.  
  - Any failure (numerical issues, invalid topology) must surface as a `Result` with a typed error, never as a silent assertion or abort.

**Template: C++ `Manifold::Boolean` → Rust `Manifold::boolean`**

- **C++ signature (from `manifold/src/manifold.cpp`)**  
  - `Manifold Manifold::Boolean(const Manifold& second, OpType op) const;`

- **Rust public API shape (in `libs/manifold-rs`)**  
  - Provide a safe, explicit API instead of operator overloading:
    - `pub fn boolean(&self, other: &Manifold, op: BooleanOp) -> Result<Manifold, BooleanError>;`  
    - `pub fn union(&self, other: &Manifold) -> Result<Manifold, BooleanError>;`  
    - `pub fn difference(&self, other: &Manifold) -> Result<Manifold, BooleanError>;`  
    - `pub fn intersection(&self, other: &Manifold) -> Result<Manifold, BooleanError>;`
  - `BooleanOp` is a small Rust enum mirroring `OpType` (`Add`, `Subtract`, `Intersect`).  
  - `BooleanError` is a `thiserror`-based type capturing triangulation failures, invalid topology, or precision/pathological cases.

- **Rust internal delegation pattern**  
  - Keep the user-facing `Manifold` as a thin handle around an internal implementation (e.g. `struct Manifold { impl_: Arc<Impl> }`).  
  - Implement the heavy boolean logic on an internal `Impl`/`Node` type that owns the index-based half-edge arenas.  
  - `Manifold::boolean` should:
    - Validate inputs (e.g. basic sanity checks, bounding boxes).  
    - Construct an internal CSG tree node (similar to `CsgNode`/`CsgOpNode` in C++) using indices and `rayon` for parallel phases.  
    - Execute the boolean operation, returning either a new `Impl` (wrapped in `Manifold`) or a `BooleanError`.

- **Testing pattern**  
  - Mirror the intention of `operator+`, `operator-`, `operator^` by:
    - Adding tests for `union/difference/intersection` that validate:  
      - Topological invariants (`validate()` passes, expected Euler characteristic).  
      - Expected behaviour on disjoint vs overlapping bounding boxes.  
    - Ensuring the Rust `boolean` operations never panic and always return a `Result`.

These guidelines, together with the examples above, apply to all `manifold-rs` tasks below (cube, sphere, transforms, booleans) and should be followed consistently for each ported algorithm.

### Task 2.1 – Manifold-RS Cube Primitive (TDD)

**Goal**  
Implement a robust cube primitive in `libs/manifold-rs` with TDD.

**Steps**

1. **Test First**
   - In `libs/manifold-rs/src/primitives/cube/tests.rs`:
     - Add tests asserting:
       - 8 vertices and 12 triangles for a simple cube.  
       - Correct bounding box for given `size` and `center` flag.  
       - `Manifold::validate()` passes.

2. **Implementation**
   - In `libs/manifold-rs/src/primitives/cube/mod.rs`:
     - Implement `pub fn cube(size: DVec3, center: bool) -> Manifold`.
     - Use the index-based half-edge representation.
   - Ensure cube construction relies on `Vec` arenas and u32 indices, not pointer-based structures.

3. **Robustness**
   - Where predicates are needed (e.g. coplanarity checks), use `robust`-style predicates from the beginning rather than ad-hoc epsilon comparisons.

4. **Validation**
   - Implement `Manifold::validate()` (if not already) to run Euler checks and topology invariants.

**Acceptance Criteria**

- `cargo test -p manifold-rs` passes with cube tests.
- `Manifold::Cube` (or equivalent) produces a valid manifold for typical sizes.

---

### Task 2.4 – Pipeline Integration & Error Reporting

**Goal**  
Connect source → AST → IR → (stub) Manifold and introduce structured diagnostics.

**Steps**

1. **Diagnostic Type**
   - Define in a shared crate or module:

     ```rust
     struct Diagnostic {
         severity: Severity,
         message: String,
         span: Span,
         hint: Option<String>,
     }
     ```

2. **WASM Interface**
   - In `libs/wasm`:
     - Expose `async fn compile_and_render(source: &str) -> Result<MeshHandle, Vec<Diagnostic>>` (exported via `wasm-bindgen` and `wasm-bindgen-futures`).
     - For now, `MeshHandle` can be a stub or dummy when rendering is not fully implemented.

3. **Playground Diagnostics**
   - In the worker, forward diagnostics back to the main thread.  
   - In the Playground, highlight errors using editor squiggles and an error panel.

**Acceptance Criteria**

- Intentionally invalid OpenSCAD code produces a list of `Diagnostic` entries with correct spans and messages.
- The Playground shows helpful error messages instead of crashes.

---

## Phase 3 – Filesystem & Parameters

### Task 3.1 – Named Arguments

**Goal**  
Support OpenSCAD-style named arguments (e.g. `cube(size=[10,10,10], center=true)`).

**Steps**

1. **Parser & AST Checks**
   - Ensure the parser/AST already supports named arguments; extend if necessary.

2. **Evaluator Argument Mapping**
   - Implement a helper to map positional + named args into a canonical structure for builtins like `cube`.

3. **Tests**
   - Add evaluator tests confirming that combinations of positional and named args produce the expected `GeometryNode::Cube` values.

**Acceptance Criteria**

- `cube(size=[10,10,10], center=true)` and `cube([10,10,10], true)` behave identically where appropriate.

---

### Task 3.2 – Resolution Special Variables ($fn, $fa, $fs)

**Goal**  
Represent and evaluate OpenSCAD’s resolution variables `$fn`, `$fa`, and `$fs` in a dedicated evaluation context.

**Steps**

1. **EvaluationContext Struct**
   - Define an `EvaluationContext` (or similar) in `libs/openscad-eval` that tracks `$fn`, `$fa`, `$fs` along with other global/built-in parameters.

2. **Propagation**
   - Ensure parser/AST nodes that set these variables update the context correctly.  
   - Ensure primitives like `sphere` and any resolution-sensitive operations read from this context when constructing IR.

3. **Tests**
   - Add evaluator tests confirming that adjusting `$fn`, `$fa`, `$fs` leads to the expected change in resolution for spheres and other affected primitives.

**Acceptance Criteria**

- `$fn`, `$fa`, `$fs` are represented explicitly in a context struct and used consistently by resolution-sensitive primitives.

---

## Phase 4 – Sphere & Resolution

### Task 4.1 – Sphere (Icosphere) with Resolution Controls

**Goal**  
Implement a robust `sphere()` primitive with resolution managed by `$fn`, `$fa`, `$fs`.

**Steps**

1. **Manifold-RS Sphere**
   - Implement `Sphere` construction using an octahedron base and subdivision.  
   - Use `DEFAULT_FN` from `config.rs` when no explicit resolution is supplied.

2. **Evaluator Context**
   - Track `$fn`, `$fa`, `$fs` in an evaluation context struct.  
   - Ensure they are respected when creating `GeometryNode::Sphere`.

3. **Tests**
   - Add tests for various radius/segment combinations.  
   - Verify `Manifold::validate()` passes.

**Acceptance Criteria**

- `sphere(r=10);` produces a valid manifold with reasonable tessellation.  
- Changing `$fn` influences the sphere resolution as expected.

---

## Phase 5 – Transformations

### Task 5.1 – Transform Nodes & Application

**Goal**  
Support `translate`, `rotate`, and `scale` transformations end-to-end.

**Steps**

1. **IR Representation**
   - Add `GeometryNode::Transform(Mat4, Box<GeometryNode>)` to Geometry IR.

2. **Evaluator Mapping**
   - Map AST transformation constructs (`translate(...)`, `rotate(...)`, `scale(...)`) into IR nodes.

3. **Manifold-RS Operations**
   - Implement transform methods that apply matrices to underlying vertices.

4. **Span Propagation**
   - Ensure spans for transformed geometry still map back to originating nodes for diagnostics.

**Acceptance Criteria**

- Complex transform chains (e.g. `translate([1,2,3]) rotate([0,90,0]) cube(5);`) render correctly.  
- Diagnostics still point to the correct source spans.

---

## Phase 6 – Boolean Operations

### Task 6.1 – Robust Predicates

**Goal**  
Introduce robust predicates for geometric computations.

**Steps**

1. **`robust` Integration**
   - Use the `robust` crate for orientation tests (e.g. `orient3d`).

2. **Replace Epsilon Checks**
   - Audit existing predicate code (e.g. `dot > EPSILON`) and replace with robust predicates where correctness is critical.

**Acceptance Criteria**

- Predicates behave correctly for nearly coplanar and nearly parallel cases in tests.

---

### Task 6.2 – Boolean Logic (CSG)

**Goal**  
Implement robust `union`, `difference`, and `intersection` operations.

**Steps**

1. **Broad Phase (Spatial Index)**
   - Implement an R-Tree/BVH for triangle bounding boxes.  
   - Use it to find candidate triangle pairs, using `rayon` where appropriate for parallel partitioning (and WASM threads + shared memory when available).

2. **Exact Phase (Intersection)**
   - Implement edge-plane and edge-edge intersection logic, carefully handling precision.

3. **Classification & Retriangulation**
   - Classify triangles as inside/outside relative to other manifolds.  
   - Re-triangulate intersection regions.

4. **Sanitation**
   - Remove degenerate triangles; ensure final mesh is watertight.

**Acceptance Criteria**

- Boolean examples from the test corpus produce valid manifolds.  
- `Manifold::validate()` passes after each boolean operation.

---

### Task 6.3 – Fuzz Testing

**Goal**  
Catch edge cases in boolean operations using property-based tests.

**Steps**

1. **Fuzz Harness**
   - Use `proptest` to generate random primitives (cubes, spheres) with random transforms, favouring strategies that are SIMD-friendly where beneficial so fuzzing can exercise many cases quickly.

2. **Operation Under Test**
   - Perform `union` (and if feasible, `difference`, `intersection`) on random pairs.

3. **Invariant Checks**
   - Assert that `Manifold::validate()` always returns `true`.  
   - Assert no panics occur.

**Acceptance Criteria**

- Fuzz tests run regularly in CI (or at least locally) and catch regressions in boolean logic.

---

## 7. Global Ongoing Tasks

These are continuous practices rather than one-time tasks:

- **Follow Current Rust Best Practices**  
  - Use modern Rust patterns (2021+/2024 editions), idiomatic error handling, and ownership patterns.

- **Browser & WASM Testing**  
  - Regularly run `wasm-pack test --headless --chrome` to validate behaviour in a browser-like environment.

- **Keep Docs in Sync**  
  - Update `overview-plan.md` and `tasks.md` whenever architecture or workflows change.  
  - Remove outdated sections instead of letting them drift.

- **Enforce SRP & File Size Limits**  
  - Refactor aggressively when a module grows too large or takes on multiple responsibilities.  
  - After each larger task, audit files approaching ~300–500 lines and, if needed, split logic into SRP-friendly submodules (e.g. `evaluator/calls`, `evaluator/transforms`).

- **Project Cleanup**  
  - Regularly remove unused code, APIs, and comments.  
  - Maintain consistent design patterns across crates.

- **Error Handling & Tests**  
  - In tests, assert on explicit error variants (from `thiserror`) instead of relying on panics or `unwrap`/`expect`.  
  - Treat any panic in library code paths as a bug to be removed.

- **Benchmarking & Fuzzing**  
  - Periodically benchmark end-to-end compile times and WASM size, adjusting algorithms and build flags (e.g. `wasm-opt`) as needed.  
  - Run fuzz tests for boolean operations regularly (e.g. nightly) and treat regressions as high severity.

This `tasks.md` should evolve with the project; treat it as a living backlog, always aligned with the codebase and the principles described in `overview-plan.md`.

# Rust OpenSCAD Pipeline – Task Breakdown

> This file is the **actionable backlog** for the Rust OpenSCAD pipeline.  
> It is structured into small, test-driven tasks and subtasks.  
> See `overview-plan.md` for goals, architecture, and coding standards.

---

## Implementation Progress (Updated: 2025-01-27, Edge Highlighting Fix)

### Recent Changes

- **Fixed sphere edge highlighting**: Changed from `EdgesGeometry` to `WireframeGeometry` to show ALL triangle edges, not just sharp edges. This matches OpenSCAD's behavior where smooth surfaces like spheres show their tessellation wireframe.
- **Fixed $fn parameter parsing for sphere/cylinder**: Local `$fn` overrides in primitive calls (e.g., `sphere(r=10, $fn=8)`) now correctly affect tessellation
- **Fixed WASM build script**: Now copies WASM files to playground automatically
- **Added CST parsing tests**: Comprehensive tests for `$fn` parameter parsing from browser CST

### Completed ✅

| Phase | Feature | Status | Tests |
|-------|---------|--------|-------|
| 1.1 | Workspace & Crate Setup | ✅ Complete | - |
| 1.2 | Config crate with constants | ✅ Complete | 29 tests |
| 1.3 | openscad-ast crate | ✅ Complete | 32 tests |
| 1.4 | openscad-eval crate | ✅ Complete | 33 tests |
| 1.5 | openscad-mesh crate | ✅ Complete | 136 tests |
| 1.6 | openscad-wasm crate | ✅ Complete | 3 tests |
| 1.7 | openscad-lsp crate (skeleton) | ✅ Complete | - |
| 2.1 | Cube primitive | ✅ Complete | 8 tests |
| 2.2 | Sphere primitive | ✅ Complete | 6 tests |
| 2.3 | Cylinder primitive | ✅ Complete | 8 tests |
| 3.1 | Transforms (translate, rotate, scale) | ✅ Complete | - |
| 3.2 | Mirror transform | ✅ Complete | - |
| 3.3 | Color modifier | ✅ Complete | - |
| 4.1 | $fn/$fa/$fs resolution | ✅ Complete | 9 tests |
| 6.1 | BSP tree data structure | ✅ Complete | 8 tests |
| 6.2 | BSP boolean operations | ✅ Complete | 50 tests |
| 7.1 | linear_extrude | ✅ Complete | 10 tests |
| 7.2 | rotate_extrude | ✅ Complete | 8 tests |
| 7.3 | Hull (QuickHull) | ✅ Complete | 9 tests |
| 7.4 | Minkowski sum | ✅ Complete | 6 tests |
| 7.5 | Offset (2D polygon) | ✅ Complete | 6 tests |
| 8.1 | Wire operations into from_ir | ✅ Complete | 11 tests |

### Feature Support Matrix

| Category | Feature | AST | CST Parser | Evaluator | Mesh |
|----------|---------|-----|------------|-----------|------|
| **2D Primitives** |||||
| | `circle(r\|d)` | ✅ | ✅ | ✅ | ✅ |
| | `square(size, center)` | ✅ | ✅ | ✅ | ✅ |
| | `polygon(points, paths)` | ✅ | ✅ | ✅ | ✅ |
| **3D Primitives** |||||
| | `cube(size, center)` | ✅ | ✅ | ✅ | ✅ |
| | `sphere(r\|d)` | ✅ | ✅ | ✅ | ✅ |
| | `cylinder(h, r, r1, r2)` | ✅ | ✅ | ✅ | ✅ |
| | `polyhedron(points, faces)` | ✅ | ✅ | ✅ | ✅ |
| **Extrusions** |||||
| | `linear_extrude(...)` | ✅ | ✅ | ✅ | ✅ |
| | `rotate_extrude(...)` | ✅ | ✅ | ✅ | ✅ |
| **Transforms** |||||
| | `translate([x,y,z])` | ✅ | ✅ | ✅ | ✅ |
| | `rotate([x,y,z])` | ✅ | ✅ | ✅ | ✅ |
| | `rotate(a, [x,y,z])` | ✅ | ✅ | ✅ | ✅ |
| | `scale([x,y,z])` | ✅ | ✅ | ✅ | ✅ |
| | `mirror([x,y,z])` | ✅ | ✅ | ✅ | ✅ |
| | `multmatrix(m)` | ✅ | ✅ | ✅ | ✅ |
| | `resize(newsize, auto)` | ✅ | ✅ | ✅ | ✅ |
| | `color(...)` | ✅ | ✅ | ✅ | ✅ |
| | `offset(r\|delta)` | ✅ | ✅ | ✅ | ✅ |
| | `hull()` | ✅ | ✅ | ✅ | ✅ |
| | `minkowski()` | ✅ | ✅ | ✅ | ✅ |
| **Booleans** |||||
| | `union()` | ✅ | ✅ | ✅ | ✅ |
| | `difference()` | ✅ | ✅ | ✅ | ✅ |
| | `intersection()` | ✅ | ✅ | ✅ | ✅ |
| **Syntax** |||||
| | `var = value;` | ✅ | ✅ | ✅ | - |
| | `var = cond ? a : b;` | ✅ | ✅ | ✅ | - |
| | `module name() {}` | ✅ | ✅ | ✅ | - |
| | `function name() = ...` | ✅ | ✅ | ✅ | - |
| | `for (var = range) {}` | ✅ | ✅ | ✅ | - |
| | `if (cond) {}` | ✅ | ✅ | ✅ | - |
| **Operators** |||||
| | Arithmetic (`+ - * / % ^`) | ✅ | ✅ | ✅ | - |
| | Comparison (`< <= == != >= >`) | ✅ | ✅ | ✅ | - |
| | Logical (`&& \|\| !`) | ✅ | ✅ | ✅ | - |
| **Special Variables** |||||
| | `$fn, $fa, $fs` | ✅ | ✅ | ✅ | - |
| | `$t` (animation) | ✅ | ✅ | ✅ | - |
| | `$children` | ✅ | ✅ | ✅ | - |
| | `$vpr, $vpt, $vpd, $vpf` | ✅ | ⚠️ | ✅ | - |
| **Modifiers** |||||
| | `*` (disable) | ✅ | ⚠️ | ✅ | - |
| | `!` (show only) | ✅ | ⚠️ | ✅ | - |
| | `#` (highlight) | ✅ | ⚠️ | ✅ | - |
| | `%` (transparent) | ✅ | ⚠️ | ✅ | - |

Legend: ✅ Implemented | ⚠️ Partial | ❌ Not Implemented

### Completed 🎉

| Phase | Feature | Status |
|-------|---------|--------|
| 9.1 | Playground UI implementation | ✅ Complete |
| 9.2 | WASM browser safety verification | ✅ Complete |
| 9.3 | Performance optimization | ✅ Complete |
| 9.4 | Browser-safe CST pipeline | ✅ Complete |

### Performance Benchmarks (Browser)

| Test Case | Time | Vertices | Triangles |
|-----------|------|----------|-----------|
| Target validation (3 boolean ops) | ~106ms | 9,788 | 4,172 |
| Simple cube | ~0.4ms | 8 | 12 |
| Sphere ($fn=32) | ~1.9ms | 450 | 896 |

Performance targets:
- Target validation: < 500ms ✅ (achieved 106ms)
- Simple primitives: < 10ms ✅

### Next Steps (Immediate)

| Priority | Feature | Description |
|----------|---------|-------------|
| ~~**Critical**~~ | ~~2D Primitives CST~~ | ✅ Added `parse_circle_call`, `parse_square_call`, `parse_polygon_call` |
| ~~**Critical**~~ | ~~Additional Transforms~~ | ✅ Added `multmatrix`, `resize`, `offset`, `minkowski` parsing |
| ~~**High**~~ | ~~User-defined functions with params~~ | ✅ FIXED - CST parameter node handling corrected |
| High | Expression evaluation | Add binary operators (`+ - * /`) and function calls in expressions |
| High | Control flow | Complete `for` loop and `if` statement CST parsing and evaluation |
| High | User modules | Support `module name() {}` definitions and calls |
| ~~Medium~~ | ~~Rotate extrude test~~ | ✅ `rotate_extrude { translate([x,0,0]) circle(); }` works |

### Latest Changes (2025-11-26)

**Viewport Variables Added:**
- `$vpr` - Viewport rotation (Euler angles) [55, 0, 25]
- `$vpt` - Viewport translation [0, 0, 0]
- `$vpd` - Viewport camera distance (140.0)
- `$vpf` - Viewport field of view (22.5°)

**Modifiers Added (AST & Evaluator):**
- `*` (Disable) - Object not rendered
- `!` (ShowOnly) - Only this object rendered
- `#` (Highlight) - Rendered in magenta
- `%` (Transparent) - Rendered semi-transparent

**Bug Fixes (Critical):**
1. **Function Parameter Parsing** ✅ FIXED
   - Issue: CST has `parameters -> parameter -> identifier` but code expected `parameters -> identifier`
   - Fix: Updated `parse_function_item` and `parse_module_definition` to handle "parameter" node type
   - Affected: `libs/openscad-ast/src/cst_parser.rs`
   - Now works: `function id(x) = x; cube(id(10));` ✓

2. **Context Isolation Bug** ✅ FIXED
   - Issue: `evaluate_expression_to_f64` created new empty context instead of using current context
   - Fix: Changed Assignment handling to use `eval_expr(value, ctx)` directly
   - Affected: `libs/openscad-eval/src/evaluator.rs`

3. **WASM Build Naming** ✅ FIXED
   - Issue: Build produced `wasm.js` but loader imported `openscad_wasm.js`
   - Fix: Updated `scripts/build-wasm.js` to use `--out-name openscad_wasm`

**All User-Defined Functions Now Work:**
- Single param: `cube(id(10))` ✓
- Multi-param: `sphere(r=add(3, 2))` ✓
- Arithmetic: `cube(double(5))` ✓
- Nested calls: `cube(double(id(5)))` ✓
- In loops: `for(i=[0:2]) translate([offset(i), 0, 0])` ✓
- In vectors: `translate([id(10), 0, 0])` ✓

### Next Steps (Future Work)

| Priority | Feature | Description |
|----------|---------|-------------|
| ~~High~~ | ~~Web Worker~~ | ✅ WASM execution moved to Web Worker for non-blocking UI |
| ~~High~~ | ~~Z-up Axis~~ | ✅ Camera and grid configured for Z-up (OpenSCAD/CAD standard) |
| ~~High~~ | ~~Edge Highlighting~~ | ✅ Black edges on geometry faces using EdgesGeometry |
| ~~High~~ | ~~OpenSCAD Defaults~~ | ✅ All special variables with correct defaults |
| High | Error highlighting | Show syntax errors inline in code editor |
| Medium | Mesh instancing | Use THREE.InstancedMesh for repeated geometry |
| Medium | LOD (Level of Detail) | Reduce polygon count for distant objects |
| Medium | Incremental updates | Only regenerate affected mesh components |
| Low | Export STL/OBJ | Add mesh export functionality |
| Low | Syntax highlighting | Add CodeMirror/Monaco editor with OpenSCAD syntax |
| Low | Share/Save | Allow saving and sharing models via URL |

### Web Worker Implementation (2025-11-26)

**Architecture:**
```
Main Thread                    Worker Thread
─────────────                  ─────────────
postMessage(COMPILE) ────────► onmessage
                                  │
                                  ▼
                               Parse CST (web-tree-sitter)
                                  │
                                  ▼
                               render_from_cst() (WASM)
                                  │
onmessage ◄──────────────────── postMessage(RESULT)
  │                              (transferable ArrayBuffers)
  ▼
Update Three.js scene
```

**Files Added:**
- `apps/playground/src/lib/worker/openscad-worker.ts` - Worker implementation
- `apps/playground/src/lib/worker/worker-client.ts` - Promise-based client
- `apps/playground/src/lib/worker/index.ts` - Module exports

**Benefits:**
- UI remains responsive during compilation
- Zero-copy data transfer via transferable ArrayBuffers
- Promise-based async API for clean error handling

### Z-Up Axis Configuration (2025-11-26)

**Changes to `scene-manager.ts`:**
- `camera.up.set(0, 0, 1)` - Z is up
- Grid rotated to XY plane: `gridHelper.rotation.x = Math.PI / 2`
- Camera positioned for isometric-like view

**Coordinate System:**
- X: Right
- Y: Forward  
- Z: Up (vertical)

This matches OpenSCAD and CAD/engineering conventions.

### Edge Highlighting (2025-11-26)

**Implementation:**
- Uses Three.js `EdgesGeometry` with configurable threshold angle (default: 30°)
- Only shows edges where face normals differ by more than threshold
- `polygonOffset` on mesh material prevents z-fighting with edge lines

**Configuration Options (SceneConfig):**
- `showEdges: boolean` - Enable/disable edge highlighting (default: true)
- `edgeColor: number` - Edge line color (default: 0x000000 - black)
- `edgeThreshold: number` - Angle threshold in degrees (default: 30)

### OpenSCAD Special Variables (2025-11-26)

**Complete list with defaults:**

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `$fn` | f64 | 0.0 | Fragment count override (0 = use $fa/$fs) |
| `$fa` | f64 | 12.0 | Minimum fragment angle (degrees) |
| `$fs` | f64 | 2.0 | Minimum fragment size |
| `$t` | f64 | 0.0 | Animation time (0.0 to 1.0) |
| `$children` | usize | 0 | Number of children in module scope |
| `$preview` | bool | true | True in preview mode (F5), false in render (F6) |
| `$vpr` | [f64; 3] | [55, 0, 25] | Viewport rotation (Euler angles) |
| `$vpt` | [f64; 3] | [0, 0, 0] | Viewport translation |
| `$vpd` | f64 | 140.0 | Viewport camera distance |
| `$vpf` | f64 | 22.5 | Viewport field of view (degrees) |

**Files:**
- `libs/openscad-eval/src/context.rs` - All special variables
- `config/src/constants.rs` - $fn, $fa, $fs defaults

### Test Summary

- **Total Tests**: 240+ passing
- **config**: 29 tests
- **openscad-ast**: 32 tests  
- **openscad-eval**: 33 tests
- **openscad-mesh**: 138 tests (includes 2 performance tests)
- **openscad-wasm**: 3 tests
- **tree-sitter-openscad-parser**: 5 tests

### Playground Implementation

The playground (`apps/playground`) is a SvelteKit + Three.js application:

**Components:**
- `src/routes/+page.svelte` - Main page with code editor and 3D viewer
- `src/routes/+page.ts` - Disables SSR (Three.js requires browser APIs)
- `src/lib/viewer/scene-manager.ts` - Three.js scene management (camera, lights, controls)
- `src/lib/wasm/loader.ts` - WASM module loader with singleton pattern
- `src/lib/wasm/types.ts` - TypeScript type definitions for WASM API
- `src/lib/wasm/env-shim.ts` - WASM environment stubs

**Features:**
- Real-time code editing with debounced auto-compile (500ms)
- 3D mesh rendering with OrbitControls
- Compilation status and timing display
- Error diagnostics panel
- Auto-fit camera to mesh bounds

**Dependencies:**
- `three` ^0.170.0 - 3D rendering
- `@types/three` ^0.170.0 - TypeScript types

**Build Commands:**
```bash
# Build WASM (from project root)
node scripts/build-wasm.js

# Copy WASM to static (from apps/playground)
Copy-Item -Path "../../libs/wasm/pkg/*" -Destination "static/wasm/" -Recurse -Force

# Install dependencies
pnpm install

# Start dev server
pnpm dev
```

**WASM Build Output:**
- `wasm.js` - 20.4 KB (JavaScript glue code)
- `wasm_bg.wasm` - 398.3 KB (WebAssembly binary)
- `wasm.d.ts` - 6.7 KB (TypeScript declarations)

### WASM Browser Safety Status

**Browser-Safe Crates** (Pure Rust, no C dependencies):
- **glam**: Pure Rust linear algebra (f64 mode) ✅
- **robust**: Pure Rust exact predicates ✅
- **thiserror**: Pure Rust error handling ✅
- **wasm-bindgen**: WASM bindings ✅
- **js-sys**: JavaScript interop ✅

**Problematic Crate** (C dependencies):
- **tree-sitter**: Has C code that requires C runtime functions (`fprintf`, `free`, `malloc`, etc.)

### Tree-sitter WASM Architecture Analysis

After reviewing `apps/tree-sitter/lib/binding_web/`, the official tree-sitter project uses a **completely separate approach** for browser support:

1. **web-tree-sitter** is a dedicated npm package that uses **Emscripten** to compile tree-sitter's C code to WASM
2. It provides its own `.wasm` binary (`tree-sitter.wasm`) that includes all C runtime functions
3. Language grammars are compiled separately to `.wasm` files using `tree-sitter build --wasm`
4. The JavaScript API (`Parser.init()`, `Language.load()`) handles WASM initialization

**Current Problem**: Our approach compiles the Rust `tree-sitter` crate to WASM, which includes C code that expects C runtime functions. These functions don't exist in the browser environment.

### Proposed Solutions

**Option A: Use web-tree-sitter (Recommended)**
- Use the official `web-tree-sitter` npm package in the playground
- Build the OpenSCAD grammar to `.wasm` using `tree-sitter build --wasm`
- Parse in JavaScript, then pass the CST to Rust for AST conversion
- Pros: Official support, well-tested, no C runtime stubs needed
- Cons: Parsing happens in JS, need to serialize CST across WASM boundary

**Option B: Provide C Runtime Stubs**
- Create comprehensive C runtime stubs in JavaScript
- Provide all functions: `fprintf`, `free`, `malloc`, `memcpy`, etc.
- Pros: Keep current architecture
- Cons: Complex, error-prone, may have subtle bugs

**Option C: Pre-parse on Server**
- Move parsing to a server/worker that runs native code
- Send parsed AST to browser for mesh generation
- Pros: Full native performance for parsing
- Cons: Requires server infrastructure

### Current Status

### Option A Implementation - COMPLETE ✅

1. ✅ Installed `web-tree-sitter` npm package in playground
2. ✅ Built OpenSCAD grammar to WASM using Docker
3. ✅ Created TypeScript parser wrapper (`openscad-parser.ts`)
4. ✅ Refactored Rust WASM to accept serialized CST
   - Created `cst.rs` with `SerializedNode` type for JSON deserialization
   - Created `cst_parser.rs` with `parse_from_cst()` function
   - Added `evaluate_from_cst()` in openscad-eval
   - Added `render_from_cst()` in openscad-wasm
   - Made native tree-sitter optional via `native-parser` feature
5. ✅ WASM build succeeds without C dependencies (287KB)
6. ✅ Full pipeline tested in browser
   - Boolean operations demo: 9788 vertices, 4172 triangles in 106ms
   - Simple cube: 8 vertices, 12 triangles in 0.4ms
   - Sphere ($fn=32): 450 vertices, 896 triangles in 1.9ms
7. ✅ Documentation updated

### Architecture

```
Browser Pipeline:
  OpenSCAD Source
       ↓
  web-tree-sitter (JS) → CST
       ↓
  serializeTree() → JSON
       ↓
  render_from_cst() (Rust WASM)
       ↓
  parse_from_cst() → AST
       ↓
  evaluate_from_cst() → Geometry IR
       ↓
  geometry_to_mesh() → Mesh
       ↓
  Three.js Rendering
```

**Files Created:**
- `libs/openscad-ast/src/cst.rs` - SerializedNode type for JSON deserialization
- `libs/openscad-ast/src/cst_parser.rs` - CST to AST conversion
- `apps/playground/src/lib/parser/openscad-parser.ts` - TypeScript wrapper for web-tree-sitter
- `scripts/build-grammar-wasm.js` - Build script for grammar WASM

---

## Conventions

- Each task is **small**, **TDD-first**, and adheres to the **SRP**.
- No mocks, except for filesystem or other external I/O boundaries.
- All failures must return explicit errors; no hidden fallbacks.
- All geometry code uses `f64` internally; `f32` is only for GPU export.
- Keep files under **500 lines**; split as soon as they grow too large.
- For every new public API (functions, structs, modules), add Rust doc-comments with at least one minimal usage example, and keep examples compiling as part of tests.
- All Rust crates must be **browser-safe** (compile to `wasm32-unknown-unknown` without native dependencies).

## Architecture: CPU (Rust/WASM) vs GPU (WebGL)

All geometry operations are **CPU-bound** and run in Rust via WebAssembly:

| Layer | Role | Technology |
|-------|------|------------|
| **Rust (WASM)** | Mesh generation: vertices, normals, indices | `libs/openscad-mesh` |
| **WebGL** | Rendering only | Three.js `BufferGeometry` |

**Workflow:**
1. Rust computes mesh geometry (CPU-bound operations like `union()`, `linear_extrude()`).
2. Rust returns `Float32Array` (vertices) + `Uint32Array` (indices) to JavaScript.
3. Three.js creates `BufferGeometry` and renders via WebGL (GPU).

**Rust Libraries for Geometry Processing:**

| Operation | Rust Algorithm/Library |
|-----------|----------------------|
| Linear Algebra | `glam` (f64 `DMat4`, `DVec3`) |
| Boolean Ops | BSP trees (csg.js algorithm) |
| Triangulation | Ear clipping (custom) |
| Hull | QuickHull (custom) |
| Minkowski | Convex sum (custom) |
| Offset | Clipper2-style (custom) |
| Extrusions | Slice-based mesh generation |
| Robust Predicates | `robust` crate |

For each task, we list:

- **Goal** – What we want to achieve.  
- **Steps** – Concrete actions to perform.  
- **Acceptance Criteria** – How we know the task is done.

---

## Phase 0 – Pre-Bootstrap Evaluation

### Task 0.1 – Confirm Geometry Kernel Strategy (Browser-Safe Algorithms)

**Goal**  
Confirm and document that `libs/openscad-mesh` will use browser-safe algorithms for mesh generation and operations, not external geometry kernels.

**Steps**

1. Document the chosen algorithms for each operation:
   - **Boolean Operations**: BSP trees (based on csg.js by Evan Wallace)
   - **Triangulation**: Ear clipping algorithm
   - **Convex Hull**: QuickHull algorithm
   - **Robust Predicates**: `robust` crate for exact geometric predicates
2. Ensure all chosen algorithms are browser-safe (no native dependencies, compile to WASM).
3. Write a short design note describing:
   - The chosen approach: browser-safe Rust implementations in `libs/openscad-mesh`.  
   - The algorithms selected for each operation type.  
   - An explicit statement that external geometry kernels (manifold3d, csgrs, CGAL) will not be used as runtime dependencies.

**Acceptance Criteria**

- A design note exists in the repo (e.g. `specs/pipeline/kernel-decision.md`) clearly stating the browser-safe algorithm choices.  
- `overview-plan.md` and the dependency graph remain valid for this approach.

### Task 0.2 – Confirm Tree-sitter Grammar Integration

**Goal**  
Confirm that `libs/openscad-parser` is the canonical Tree-sitter grammar for OpenSCAD and that it is correctly wired into the Rust parser crate.

**Steps**

1. Verify that `libs/openscad-parser/src/grammar.json` builds successfully with Tree-sitter and covers primitives, transforms, booleans, control flow, and advanced features required for the initial slices.  
2. Ensure the Rust bindings under `libs/openscad-parser/bindings/rust/lib.rs` use this `grammar.json` when generating the Tree-sitter parser.  
3. Document `grammar.json` as the canonical grammar file in `overview-plan.md` and this task file.

**Acceptance Criteria**

- `libs/openscad-parser/src/grammar.json` is clearly documented as the canonical grammar in `overview-plan.md` and this task file.

---

## Phase 1 – Infrastructure & "Tracer Bullet"

### Task 1.1 – Workspace & Crate Setup 

**Goal**  
Initialize the Cargo workspace and core Rust crates from scratch, with proper dependencies and configuration.

**Steps**

1. **Workspace Configuration**  
   - Update root `Cargo.toml` to include these members:  
     - `libs/openscad-parser`  
     - `libs/openscad-ast`  
     - `libs/openscad-eval`  
     - `libs/openscad-mesh`  
     - `libs/wasm`  
     - `libs/openscad-lsp`  

2. **Create `libs/openscad-mesh`**  
   - Create crate structure:  
     - `src/lib.rs`  
     - `src/config.rs`  
     - `src/core/vec3/mod.rs` (type alias `pub type Vec3 = glam::DVec3;`).  
   - Add dependencies in `Cargo.toml`:  
     - `glam` (f64 support)  
     - `thiserror`  
     - `robust`  
     - `rayon` (optional, for parallel operations when WASM threads are available)
   - Define `config.rs` with core constants (e.g. `EPSILON`, `DEFAULT_SEGMENTS`) and document their purpose.
   - Ensure all dependencies are browser-safe (compile to `wasm32-unknown-unknown`).

3. **Create `libs/openscad-eval`**  
   - Create structure:  
     - `src/lib.rs`  
     - `src/ir/mod.rs`  
     - `src/evaluator/mod.rs`  
     - `src/filesystem.rs`  
   - Add dependencies:  
     - `glam`  
     - `stacker`  
   - Add a basic `GeometryNode` enum scaffold under `ir/` (cube variant can initially be a stub, to be filled in Phase 2).  
   - Implement recursion depth guards and configure `stacker` for deeper recursion in evaluator entry points.

4. **Create `libs/wasm`**  
   - Crate type: `cdylib`.  
   - Dependencies:  
     - `wasm-bindgen`  
     - `console_error_panic_hook`  
   - In `src/lib.rs`:  
     - Initialize `console_error_panic_hook` in an `init` function called by JS in debug builds.  
     - Add a small placeholder export (e.g. `greet()`), to be replaced by real pipeline functions in later tasks.  
   - Ensure the crate compiles for the `wasm32-unknown-unknown` target **inside a Dockerized Rust+WASM image** (see Task 1.3), and, if using `rayon` on WASM, that appropriate thread/atomics features, bulk-memory, and linker flags are enabled for future WASM threads.

**Acceptance Criteria**

- `cargo build` at workspace root succeeds without errors.  
- `cargo test` (with initial no-op tests) succeeds for all new crates.  
- Crate dependency graph matches the architecture in `overview-plan.md` (no unexpected cycles).

---

### Task 1.2 – Playground Setup (Svelte + Three.js + Worker) 

**Goal**  
Set up `apps/playground` with a Web Worker and Three.js scene, ready to call the real `libs/wasm` bundle (no mocks).

**Steps**

1. **Project Initialization**  
   - Under `apps/playground/`, initialize a SvelteKit project.  
   - Use **pnpm** as the package manager (`pnpm dev`, `pnpm test`, `pnpm lint`).  
   - Use Svelte 5 with SvelteKit, Vite 7, Vitest 4, TypeScript 5.9, ESLint 9, and plain `three` (no Svelte wrapper library).  
   - Enable strict TypeScript mode.  
   - Enforce `kebab-case` for TypeScript file and folder names (e.g. `mesh-wrapper.ts`, `pipeline.worker.ts`).  
   - Add a `build:wasm` script to `package.json` that runs `../scripts/build-wasm.sh`, which drives the local Rust+wasm-bindgen toolchain.

2. **Web Worker for Pipeline**  
   - Create `src/worker/pipeline.worker.ts`.  
   - Responsibilities:  
     - Load the actual WASM bundle produced from `libs/wasm` (built via `../scripts/build-wasm.sh`), not a mock module.  
     - Expose a message protocol for `compile(source: string)`.  
     - Forward diagnostics and mesh handles back to the main thread.

3. **TypeScript Wrapper for WASM (Glue Code)**  
   - Create `src/lib/wasm/mesh-wrapper.ts` that:  
     - Encapsulates raw pointer handling (`ptr`, `len`) from `MeshHandle`.  
     - Provides a `Mesh` class or interface with typed views over WASM memory (`Float32Array`, `Uint32Array`).  
     - Exposes a `dispose()`/`free()` method that calls the Rust `free_*` entry point.  
     - Uses precise TypeScript types (no `any`), and defines explicit interfaces for `MeshHandle`, diagnostics, and worker messages.

4. **Three.js Scene Manager**  
   - Implement `src/components/viewer/scene-manager.ts` with SRP:  
     - Set up renderer, camera, lights, and controls.  
     - Expose functions to attach to a canvas and update geometry from provided buffers.

**Acceptance Criteria**

- `pnpm dev` in `apps/playground/` starts without runtime errors.  
- The worker loads the **real** `libs/wasm` bundle built via `../scripts/build-wasm.sh` and calls an exported function that returns geometry buffers (initially a trivial constant mesh such as a single triangle), with **no mocked WASM modules** in TypeScript.  
- TypeScript compiles in strict mode with **no `any` usages**, and ESLint runs cleanly (zero lint errors).

---

### Task 1.3 – Local WASM Build Pipeline

**Goal**  
Build `libs/wasm` (and any other `wasm32-unknown-unknown` artifacts) locally using the Rust toolchain so developers can iterate without Docker.

**Steps**

1. **Use Local Rust Toolchain**  
   - Install the `wasm32-unknown-unknown` target and `wasm-bindgen-cli` locally (via `rustup` / `cargo install`).  
   - Ensure `WASI_SDK_PATH` is set when cross-compiling C(++) dependencies so build scripts can find clang/llvm.  
   - Build `libs/wasm` for the wasm target in release mode and run `wasm-bindgen` to emit artifacts into `libs/wasm/pkg`.

2. **`scripts/build-wasm.sh` Helper**  
   - Provide a Bash script that encapsulates the steps above: installs targets, verifies `wasm-bindgen`, respects `WASI_SDK_PATH`, runs `cargo build`, and invokes `wasm-bindgen`.  
   - The script writes JS/TS glue files and `.wasm` output into `libs/wasm/pkg`.

3. **Wire pnpm Scripts**  
   - Ensure `apps/playground/package.json` defines a `build:wasm` script that runs `../scripts/build-wasm.sh`.  
   - Optionally, have the main app `build` script depend on `build:wasm` (e.g. `"build": "pnpm build:wasm && vite build"`) so that the WASM bundle is always up to date before SvelteKit/Vite compilation.

**Acceptance Criteria**

- Running `pnpm build:wasm` in `apps/playground/` builds `libs/wasm` locally using the Rust toolchain (with optional WASI SDK).  
- Developers need a local `rustup`, `cargo`, and `wasm-bindgen` installation; Docker is no longer required for wasm builds.  
- The generated `.wasm` and JS/TS glue files are written into a stable location that the SvelteKit/Vite build can import.

---

### Task 1.4 – Parser Infrastructure & Language Server (libs/openscad-lsp) 

**Goal**  
Provide a parse-only and structural analysis pipeline for OpenSCAD using `libs/openscad-parser`, and expose it via a Rust language server built with `tower-lsp`, without duplicating parser wiring in other crates.

**Steps**

1. **Create `libs/openscad-lsp` crate**  
   - Crate structure:  
     - `src/lib.rs` (public API for analysis).  
     - `src/server/mod.rs` (tower-lsp server setup).  
     - `src/document_store.rs` (in-memory text + versioning).  
     - `src/parser.rs` (Tree-sitter integration using `libs/openscad-parser` bindings).  
   - Add dependencies:  
     - `tower-lsp`.  
     - `tokio` (for async runtime).  
     - `libs/openscad-parser` as a workspace dependency.

2. **Tree-sitter Integration**  
   - Use `libs/openscad-parser/src/grammar.json` and the Rust bindings in `bindings/rust` (for example a `language()` function) to create and maintain a `tree_sitter::Parser` and `Tree`.  
   - Implement incremental parsing by applying `tree_sitter::InputEdit` on document changes and reparsing only affected regions.  
   - Add helpers to map positions between LSP `Position` and byte offsets/points used by Tree-sitter.

3. **High-Level Analysis API (parse-only)**  
   - Implement an internal API (for example `analyze_source(source: &str) -> Vec<Diagnostic>`) that:  
     - Parses the source with Tree-sitter.  
     - Collects syntax errors and basic structural issues into the shared `Diagnostic` type.  
     - Does **not** expose raw `tree_sitter::Node` values to callers; return only domain types (diagnostics, symbols, spans).

4. **tower-lsp Server Wiring**  
   - Implement an LSP server that:  
     - Handles `initialize`, `initialized`, `shutdown`, and `textDocument/*` requests.  
     - On `textDocument/didOpen` and `textDocument/didChange`, updates the document store and parser tree, then publishes diagnostics from `analyze_source`.  
   - Provide a thin `main.rs` (either in this crate or an `apps/openscad-lsp` binary) that starts the server over stdio.

5. **Parser Reuse Policy**  
   - Document that **all** IDE/editor-facing “parse-only” and structural analysis must go through `libs/openscad-lsp`.  
   - `libs/wasm` continues to own the runtime pipeline (`compile_and_render` etc.), but does **not** expose a separate `parse_only` entry point; avoid duplicating parser wiring there.  
   - Ensure the Playground and any external tools do **not** use `web-tree-sitter`; all parsing is Rust-based via `libs/openscad-parser`.

**Acceptance Criteria**

- A minimal `openscad-lsp` server binary can be launched (for example from an editor or CLI client) and responds correctly to `initialize` and `shutdown`.  
- Given a basic OpenSCAD snippet (e.g. `cube(10);`), the server publishes either zero diagnostics or a well-formed list of syntax diagnostics.  
- Parser integration lives only in `libs/openscad-parser` and `libs/openscad-lsp`; no other crate re-implements Tree-sitter wiring or depends on `web-tree-sitter`.

---

### Task 1.5 – Enforce Pipeline Boundaries 

**Goal**  
Ensure all future `libs/openscad-mesh` implementations follow consistent patterns (browser-safe algorithms, `rayon` parallelism when available, explicit errors, robust predicates), before any primitives or boolean operations are added.

**Steps**

1. **Mesh Representation**  
   - Use a simple, browser-safe mesh structure with vertices and triangle indices.  
   - Keep ownership in central `Vec`s (e.g. `Vec<DVec3>` for vertices, `Vec<[u32; 3]>` for triangles).  
   - Avoid complex pointer-based structures that are difficult to serialize across WASM boundaries.

2. **Parallelism (`rayon`)**  
   - Use `par_iter()`/`par_iter_mut()` for parallel operations when WASM threads are available.  
   - Keep side effects confined to data local to each loop iteration.  
   - Ensure algorithms work correctly in single-threaded mode (default for WASM without threads).

3. **Memory & Safety**  
   - Use safe Rust throughout; avoid `unsafe` except where absolutely necessary for performance.  
   - All public APIs return `Result<T, Error>` for fallible operations.  
   - No panics in library code paths.

4. **Error Handling**  
   - Use `thiserror` for typed error definitions.  
   - All public `openscad-mesh` operations return `Result<T, MeshError>` or similar, never relying on panics.

5. **Testing Strategy**  
   - Write tests that verify mesh validity (vertex counts, triangle counts, bounding boxes).  
   - Add fuzz tests using `proptest` to catch edge cases.  
   - Include integration tests that verify end-to-end pipeline correctness.

6. **Robust Predicates Initialization**  
   - If the `robust` crate requires initialization, invoke once at WASM startup.

**Boolean Operations (BSP Tree Algorithm)**

- **Algorithm**: Use BSP (Binary Space Partitioning) trees for boolean operations, based on csg.js by Evan Wallace.
- **Operations**:
  - `union(a, b)`: Combine two meshes, keeping exterior surfaces
  - `difference(a, b)`: Subtract b from a
  - `intersection(a, b)`: Keep only overlapping regions

- **Rust public API shape (in `libs/openscad-mesh`)**  
  ```rust
  pub fn union(a: &Mesh, b: &Mesh) -> Result<Mesh, BooleanError>;
  pub fn difference(a: &Mesh, b: &Mesh) -> Result<Mesh, BooleanError>;
  pub fn intersection(a: &Mesh, b: &Mesh) -> Result<Mesh, BooleanError>;
  pub fn union_all(meshes: &[Mesh]) -> Result<Mesh, BooleanError>;
  pub fn difference_all(meshes: &[Mesh]) -> Result<Mesh, BooleanError>;
  pub fn intersection_all(meshes: &[Mesh]) -> Result<Mesh, BooleanError>;
  ```
  - `BooleanError` is a `thiserror`-based type capturing triangulation failures, invalid topology, or numerical issues.

- **Testing pattern**  
  - Tests for `union/difference/intersection` that validate:  
    - Mesh validity (non-empty, correct vertex/triangle counts).  
    - Expected behaviour on disjoint vs overlapping bounding boxes.  
  - Ensure operations never panic and always return a `Result`.

**Acceptance Criteria**

- Code reviews for new `libs/openscad-mesh` features explicitly check against these guidelines (browser-safety, parallelism, safety, error handling, testing, robust predicates).
- New public boolean APIs in `libs/openscad-mesh` (`union`, `difference`, `intersection`) never panic in tests and always surface failures via `Result` with typed errors.
### Task 1.6 – WASM Runtime Packaging (Tree-sitter Style)

**Goal**  
Align the `libs/wasm` web distribution with Tree-sitter's `web-tree-sitter` pattern: a single JS entrypoint with a co-located `.wasm` binary and a clear async initialization API, while keeping build tooling Node-only.  

**Steps**

1. **Runtime Bundle Contract**  
   - Treat `libs/wasm/pkg` as the only browser-facing WASM distribution: it contains the wasm-bindgen glue (`wasm.js`) and the compiled binary (`wasm_bg.wasm`).  
   - Import this bundle in the Playground exclusively via the `$wasm` alias, never from build scripts or other ad-hoc paths.  

2. **TypeScript Wrapper Alignment**  
   - Ensure `apps/playground/src/lib/wasm/mesh-wrapper.ts` exposes an `initWasm(param?: WasmInitParameter)` function that mirrors `web-tree-sitter`'s `Parser.init()` pattern:  
     - Browser code calls `initWasm()` with no arguments and lets Vite resolve `wasm_bg.wasm`.  
     - Tests and Node code may pass explicit module bytes or a pre-fetched module for deterministic initialization.  

3. **Node-only Build Helpers**  
   - Keep `scripts/build-wasm.sh` and any Node-based helper (for example `build-wasm.js`) as CLI-only tools that are never imported into browser bundles.  
   - Document their intended usage in developer docs and verify no `apps/**` or `libs/**` runtime code imports these scripts.  

4. **Cross-platform Wiring**  
   - For Unix-like systems, ensure `pnpm build:wasm` in `apps/playground` calls `../../scripts/build-wasm.sh`.  
   - For Windows, provide an equivalent workflow (for example `pnpm build:wasm:win` calling `node ../../build-wasm.js`) if needed by contributors.  

**Acceptance Criteria**

- `libs/wasm/pkg/wasm.js` + `wasm_bg.wasm` are the only artifacts imported into browser-facing code, and they are initialized via `initWasm()` in a way analogous to `web-tree-sitter`.  
- All build helpers (`scripts/build-wasm.sh`, `build-wasm.js`, or equivalents) are clearly documented as Node-only and are not pulled into Vite/SvelteKit client bundles.  
- `pnpm build:wasm` (and any platform-specific variants) consistently regenerates `libs/wasm/pkg`, and `pnpm build` in `apps/playground` completes without bundler complaints about Node built-ins.  

### Task 2.1 – Cube Primitive (TDD)

**Goal**  
Implement a robust cube primitive in `libs/openscad-mesh` with TDD.

**Steps**

1. **Test First**
   - In `libs/openscad-mesh/src/primitives/cube/tests.rs`:
     - Add tests asserting:
       - 8 vertices and 12 triangles for a simple cube.  
       - Correct bounding box for given `size` and `center` flag.  
       - `Mesh::validate()` passes.

2. **Implementation**
   - In `libs/openscad-mesh/src/primitives/cube/mod.rs`:
     - Implement `pub fn cube(size: DVec3, center: bool) -> Result<Mesh, MeshError>`.
     - Use browser-safe mesh structure (vertices + triangle indices).
   - Ensure cube construction produces valid, watertight mesh.

3. **Robustness**
   - Where predicates are needed (e.g. coplanarity checks), use `robust`-style predicates from the beginning rather than ad-hoc epsilon comparisons.

4. **Validation**
   - Implement `Mesh::validate()` to check mesh integrity (no degenerate triangles, correct winding).

**Acceptance Criteria**

- `cargo test -p openscad-mesh` passes with cube tests.
- `cube()` produces a valid mesh for typical sizes.

---

### Task 2.4 – Pipeline Integration & Error Reporting

**Goal**  
Connect source → CST → AST → evaluated/flattened AST → Mesh through the full pipeline for a minimal `cube(10);` program, and introduce structured diagnostics.

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
   - This `Diagnostic` is the canonical error type used by `libs/openscad-parser`, `libs/openscad-ast`, `libs/openscad-eval`, and `libs/openscad-mesh` when reporting problems (syntax errors, unsupported primitives, evaluation issues, etc.).  
   - `libs/wasm::diagnostics` provides a WASM-compatible `Diagnostic` wrapper that implements `From<openscad_ast::Diagnostic>` and exposes `severity()`, `message()`, `start()`, `end()`, and `hint()` getters for JavaScript.  
   - Downstream consumers (the WASM boundary and the Playground) must never invent diagnostics; they always originate from this shared type.

2. **Minimal `cube(10);` Pipeline Wiring**  
   - Implement a tracer-bullet path that exercises **every layer** as described in `overview-plan.md` §3.0:
     - Playground sends the source string `cube(10);` to `libs/wasm`.
     - `libs/wasm` forwards `cube(10);` into a single entry point in `libs/openscad-mesh`.
     - `libs/openscad-mesh` calls `libs/openscad-eval` with the original source string.
     - `libs/openscad-eval` calls `libs/openscad-ast` with `cube(10);`.
     - `libs/openscad-ast` calls `libs/openscad-parser` with `cube(10);`, receives a CST, converts it to a typed AST, and returns the AST to `libs/openscad-eval`.
     - `libs/openscad-eval` decides whether evaluation is required, evaluates and resolves the AST, and returns an evaluated/flattened AST to `libs/openscad-mesh`.
     - `libs/openscad-mesh` transforms the evaluated AST into a mesh and returns it to `libs/wasm`.
     - `libs/wasm` returns the mesh to the Playground as typed buffers/handles, and the Playground converts it into Three.js geometry and renders the cube.

3. **WASM Interface**  
   - In `libs/wasm`:
     - Implement an internal helper that returns either a mesh or a list of Rust diagnostics:

       ```rust
       pub fn compile_and_render_internal(
           source: &str,
       ) -> Result<MeshHandle, Vec<openscad_ast::Diagnostic>> {
           // Calls openscad_mesh::compile_and_render(source) and converts the result
           // into a MeshHandle on success, or returns a Vec<Diagnostic> on error.
       }
       ```

     - Implement a mapping function that converts Rust diagnostics into WASM-visible diagnostics:

       ```rust
       fn map_rust_diagnostics(
           diagnostics: Vec<openscad_ast::Diagnostic>,
       ) -> Vec<wasm::Diagnostic> {
           diagnostics
               .into_iter()
               .map(wasm::Diagnostic::from)
               .collect()
       }
       ```

     - Implement a helper that builds the JavaScript error payload containing a `diagnostics` array:

       ```rust
       fn build_diagnostics_error_payload(
           diagnostics: Vec<wasm::Diagnostic>,
       ) -> JsValue {
           use js_sys::{Array, Object, Reflect};
           use wasm_bindgen::JsValue;

           let array = Array::new();
           for diag in diagnostics {
               array.push(&JsValue::from(diag));
           }

           let obj = Object::new();
           Reflect::set(&obj, &JsValue::from_str("diagnostics"), &array)
               .expect("set diagnostics property");
           JsValue::from(obj)
       }
       ```

     - Expose the WASM entry point that uses `Result<MeshHandle, JsValue>` and the helpers above, aligning with the project’s Option B decision:

       ```rust
       #[wasm_bindgen]
       pub fn compile_and_render(source: &str) -> Result<MeshHandle, JsValue> {
           match compile_and_render_internal(source) {
               Ok(mesh) => Ok(mesh),
               Err(rust_diags) => {
                   let wasm_diags = map_rust_diagnostics(rust_diags);
                   let payload = build_diagnostics_error_payload(wasm_diags);
                   Err(payload)
               }
           }
       }
       ```

     - Ensure `MeshHandle` carries counts and typed vertex/index buffers suitable for building a `THREE.BufferGeometry` in the Playground, so the worker can return the real mesh to the renderer.  
     - Do **not** add string-only or fallback error paths; all pipeline failures must flow through a structured `diagnostics` array.

4. **Playground Diagnostics**  
   - In the worker (`pipeline.worker.ts`), catch exceptions from the WASM wrapper and normalize them into a `compile_error` message that always carries a `diagnostics` array:

     ```ts
     try {
       const mesh = compile(source);
       (self as DedicatedWorkerGlobalScope).postMessage({
         type: 'compile_success',
         payload: mesh,
       });
     } catch (error: unknown) {
       const payload = error as { diagnostics?: Diagnostic[] };
       const diagnostics = payload.diagnostics ?? [];
       (self as DedicatedWorkerGlobalScope).postMessage({
         type: 'compile_error',
         payload: diagnostics,
       });
     }
     ```

   - In the Playground route (`+page.svelte`), handle `compile_error` messages by:
     - Updating status to an error state (for example `"Error"`).  
     - Storing the diagnostics in component state.  
     - Rendering a diagnostics panel that lists at least severity and message, and later uses `start`/`end` for source mapping.  
   - Do not crash or silently ignore diagnostics; invalid code must always produce visible, structured error information.

**Acceptance Criteria**

- Intentionally invalid OpenSCAD code produces a `Vec<Diagnostic>` with correct spans and messages in Rust, and the public `compile_and_render` binding throws a JavaScript error object of the form `{ diagnostics: Diagnostic[] }`.  
- The worker converts this error into a `compile_error` message whose `payload` is a non-empty diagnostics array, and the Playground renders these diagnostics in a panel instead of crashing.  
- A `cube(10);` snippet traverses the **full minimal pipeline** documented in `overview-plan.md` §3.0 (Playground → `libs/wasm` → `libs/openscad-mesh` → `libs/openscad-eval` → `libs/openscad-ast` → `libs/openscad-parser` → back up to `libs/openscad-mesh` → `libs/wasm` → Playground), verified by integration tests or targeted logging.

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
   - `libs/openscad-eval::evaluator::context::EvaluationContext` tracks `$fn`, `$fa`, `$fs` alongside generic variables, using defaults from `config::constants`.
   - Expose getters/setters with doc comments and examples so future primitives can reuse the centralized configuration.

2. **Propagation**  
   - Parser/AST already emit `Statement::Assignment { name: "$fn" | "$fa" | "$fs" }`; the evaluator must route these into `EvaluationContext::set_variable`.  
   - Resolution-sensitive primitives (currently `sphere`) read from the context when constructing IR, applying the OpenSCAD formula:  
     - If `$fn > 0`, fragments = max($fn, 3).  
     - Otherwise, fragments = `ceil(min(360/$fa, 2πr/$fs))` with a lower bound of 5. *(References: OpenSCAD User Manual, Wikibooks “Circle resolution” section, and community write-ups on $fn/$fa/$fs.)*

3. **Tests**  
   - Evaluator regression tests now cover:  
     - Global `$fn` assignments affecting subsequent spheres.  
     - Per-call `$fn` overrides.  
     - `$fa/$fs` fallback when `$fn = 0`.  
   - Extend tests to future primitives as they gain resolution controls.

**Acceptance Criteria**

- `$fn`, `$fa`, `$fs` live in a documented context struct and are consumed by primitives per the OpenSCAD formulas cited above.  
- Automated tests prove changes in `$fn/$fa/$fs` alter generated `GeometryNode::Sphere` segments accordingly.

---

## Phase 4 – Sphere & Resolution

### Task 4.1 – Sphere with Resolution Controls

**Goal**  
Implement a robust `sphere()` primitive with resolution managed by `$fn`, `$fa`, `$fs`.

**Steps**

1. **openscad-mesh Sphere**
   - Implement `sphere()` using latitude/longitude tessellation matching OpenSCAD's algorithm:
     - Use `$fn/$fa/$fs` to compute `num_fragments` and `num_rings`.  
     - Generate vertices via latitude/longitude rings.  
     - Emit triangles in correct winding order.  
     - Maintain deterministic vertex ordering so downstream boolean ops produce consistent results.  
   - Add fixtures that compare meshes against expected vertex/triangle counts for various radius + resolution combos.

2. **Evaluator Context**
   - `$fn`, `$fa`, `$fs` are already tracked; the new `evaluator::resolution::compute_segments` helper converts those values into fragment counts exactly as described in the OpenSCAD docs (if `$fn>0` use it, else `ceil(min(360/$fa, 2πr/$fs))` with a minimum of five).  
   - `Statement::Sphere` calls the helper so context assignments, per-call overrides, and defaults all produce deterministic `segments` passed into `GeometryNode::Sphere`.

3. **Tests**
   - `libs/openscad-mesh` includes regression tests for sphere validation, bounding boxes, and resolution scaling.  
   - `libs/openscad-eval` contains unit tests for the resolution helper plus evaluator scenarios where `$fn`, `$fa`, `$fs` override each other.  
   - Doc tests capture the helper's formula for future reference.
### Task 5.1 – Transform Nodes & Application

**Goal**  
Support `translate`, `rotate`, and `scale` transformations end-to-end.

**Steps**

1. **Evaluator (libs/openscad-eval)**  
   - Ensure `Statement::Translate/Rotate/Scale` wrap child nodes in `GeometryNode::Transform { matrix, child, span }`, composing glam matrices in OpenSCAD’s inside-out order (comment + example in code). Use column-vector math so `translate([tx,ty,tz]) rotate([rx,ry,rz]) cube(1);` becomes `T * R * cube`, meaning the cube is rotated first, then translated, matching [OpenSCAD Transformations Manual](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Transformations).  
   - Add evaluator tests documenting: (a) translate-only offset affecting bounding box, (b) rotate+translate order (rotate applied before translate), (c) scale anchored at origin vs. centered geometry. Each test must include inline comments and doc examples.

2. **IR + mesh bridge**  
   - Extend `libs/openscad-mesh::from_ir` with a dedicated transform applicator that multiplies vertex positions (and recomputes normals) by the evaluator-provided 4×4 matrix.  
   - Add SRP helper (e.g., `MeshTransform`) with comments explaining matrix usage, plus unit tests verifying translated, rotated, and scaled cubes keep vertex/face counts and pass `validate()`.

3. **End-to-end tests + docs**  
   - Add integration test: `translate([10,0,0]) sphere(5);` verifying bounding box shift; add rotate/scale combos plus a compound snippet such as:

     ```
     translate([1,2,3]) rotate([0,90,0]) scale([2,1,1]) cube(4);
     ```

     Document in comments that evaluation applies scale → rotate → translate even though the code is written translate → rotate → scale.  
   - Update `specs/pipeline/overview-plan.md` and this task section with diagrams / code snippets showing matrix order, referencing OpenSCAD manual links.  
   - Document acceptance criteria in `tasks.md`: transforms compose correctly, evaluator/mesh tests cover ordering and pivot semantics, and diagnostics stay explicit (no silent fallbacks).

4. **Span Propagation**
   - Ensure spans for transformed geometry still map back to originating nodes for diagnostics.

**Acceptance Criteria**

- Transformations can be layered; child nodes are evaluated with the correct matrix composition.  
- Bounding boxes and vertex counts remain consistent after transformations.  
- Test coverage: evaluator unit tests, mesh integration tests, and documentation updates.
- Complex transform chains (e.g. `translate([1,2,3]) rotate([0,90,0]) cube(5);`) render correctly.  
- Diagnostics still point to the correct source spans.

---

### Task 5.2 – Cylinder Parity (OpenSCAD-Compatible)

**Goal**  
Implement `cylinder()` (including cones and inverted cones) matching OpenSCAD's behavior, honoring `$fn`, `$fa`, `$fs`, and all parameter permutations (`h`, `r`, `r1`, `r2`, `d`, `d1`, `d2`, `center`).

**Steps**

1. **Evaluator Support**  
   - Extend the evaluator to parse cylinder statements into a new `GeometryNode::Cylinder { radius_bottom, radius_top, height, centered, segments, span }`.  
   - Reuse `resolution::compute_segments` with `max(r1, r2)` so `$fn/$fa/$fs` match OpenSCAD's fragment calculation.  
   - Validate parameters (positive height, non-negative radii, at least one non-zero radius) and emit diagnostics on failure—no silent fallbacks.

2. **openscad-mesh Primitive**  
   - Create `libs/openscad-mesh/src/primitives/cylinder/{mod.rs, tests.rs}` (each <500 lines, documented).  
   - Generate vertices: two circles (or single apex for cones) at `z1/z2` depending on `center`, with fragments determined above.  
   - Build faces for frustum, cone, and inverted cone cases with correct winding.  
   - Produce a valid mesh with proper triangle indices.

3. **Integration & Tests**  
   - Wire `GeometryNode::Cylinder` through `from_ir` and add regression tests verifying:  
     1. Centered vs non-centered bounding boxes.  
     2. `$fn` overrides fragment counts; `$fa/$fs` fallback when `$fn=0`.  
     3. Cones/inverted cones produce the expected vertex/triangle totals.  
     4. Invalid parameters return explicit `MeshError` or evaluator diagnostics.

**Acceptance Criteria**

- `cylinder()` (and `cone` variants) produce meshes matching OpenSCAD for representative `$fn/$fa/$fs` settings.  
- Evaluator + mesh tests cover parameter parsing, centering, cone cases, and fragment math; `cargo test -p openscad-mesh` passes.

---

### Task 5.3 – Polyhedron Parity (OpenSCAD-Compatible)

**Goal**  
Port OpenSCAD's `polyhedron(points, faces, convexity)` semantics into evaluator + `openscad-mesh`, matching point/face validation, winding reversal, and convexity bookkeeping.

**Steps**

1. **Evaluator & Diagnostics**  
   - Introduce `GeometryNode::Polyhedron { points, faces, convexity, span }`.  
   - Validate that `points` is a vector of finite triplets and `faces` is a vector of integer vectors with ≥3 entries, mirroring OpenSCAD log messages (converted into structured diagnostics).  
   - Reject out-of-range indices and non-numeric values with clear errors; no face auto-fixes beyond what upstream does.

2. **openscad-mesh Primitive**  
   - Add `libs/openscad-mesh/src/primitives/polyhedron/{mod.rs, tests.rs}` responsible for:  
     - Copying vertices, reversing face winding, and splitting polygons >3 into triangles using fan triangulation.  
     - Validating topology (duplicate vertex indices, degenerate faces) and returning `MeshError::InvalidTopology` when issues arise.  
     - Preserving `convexity` metadata for downstream consumers.

3. **Testing & Integration**  
   - Add unit tests for simple tetrahedron, cube, and invalid face cases (too few vertices, out-of-range indices).  
   - Extend integration tests to ensure evaluator diagnostics propagate through WASM (matching existing pipeline error flow).  
   - Document sample `polyhedron()` snippets in tests with comments explaining expectations per project guidelines.

**Acceptance Criteria**

- `polyhedron()` inputs that succeed in OpenSCAD yield identical meshes/diagnostics in Rust, including winding and convexity flags.  
- Invalid input scenarios produce explicit diagnostics identical in spirit to upstream logging.  
- `cargo test -p openscad-mesh` and evaluator/WASM suites include coverage for tetrahedron, indexed face errors, and documentation tests.

---

## Phase 6 – Boolean Operations & Target Validation (HIGH PRIORITY)

> **Goal**: Complete the pipeline to render the target validation test case from `overview-plan.md §1.1`.
> All implementations use browser-safe algorithms (BSP trees for booleans) with OpenSCAD-compatible parameters and output.

---

### Task 6.0 – Target Validation Test Case Setup

**Goal**  
Create the integration test fixture that validates the complete pipeline against the target OpenSCAD program.

**Target Program** (must render correctly):
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

**Steps**

1. **Create Test Fixture**
   - Add `libs/openscad-mesh/src/integration/target_validation/{mod.rs, tests.rs}` (SRP structure).
   - Store the target OpenSCAD source as a constant string.
   - Define expected outcomes: 3 separate meshes, each passing `validate()`.

2. **Pipeline Integration Test**
   - Wire end-to-end test: Source → Parser → AST → Eval → Mesh.
   - Assert: all three shapes produce valid, non-empty meshes.
   - Assert: bounding boxes are positioned at x=-24, x=0, x=24 respectively.

3. **WASM Integration Test**
   - Add corresponding test in `apps/playground` that calls `compile_and_render(source)`.
   - Verify no runtime errors; mesh buffers are non-empty.

**Acceptance Criteria**

- Integration test exists and documents expected behavior.
- Test currently fails (red) until Tasks 6.1–6.4 are complete, then passes (green).

---

### Task 6.1 – Translate Transform

**Goal**  
Implement `translate([x,y,z])` transformation.

**Steps**

1. **Parser/AST (if not done)**
   - Ensure `Statement::Translate { vector: DVec3, children: Vec<Statement>, span }` is parsed correctly.
   - Handle block syntax: `translate([x,y,z]) { children }` and single-child: `translate([x,y,z]) child;`.

2. **Evaluator (libs/openscad-eval)**
   - Convert `Statement::Translate` into `GeometryNode::Transform { matrix: DMat4, children, span }`.
   - Build translation matrix: `DMat4::from_translation(vector)`.
   - Document matrix semantics in comments with examples.

3. **openscad-mesh Transform Application**
   - Create `libs/openscad-mesh/src/ops/transform/{mod.rs, tests.rs}`.
   - Implement `pub fn translate(mesh: &Mesh, offset: DVec3) -> Result<Mesh, MeshError>`.
   - Internally: apply translation matrix to all vertices; update bounding box.
   - Add comprehensive doc comments with usage examples.

4. **IR Bridge (from_ir)**
   - Handle `GeometryNode::Transform` in `from_ir`: recursively evaluate children, then apply transform.
   - Compose multiple nested transforms by matrix multiplication.

5. **Tests**
   - Unit test: `translate([10,0,0]) cube(5);` produces bounding box shifted by +10 on X.
   - Unit test: nested transforms compose correctly.
   - Integration test: validate against expected vertex positions.

**Acceptance Criteria**

- `translate([x,y,z])` works for single and multiple children.
- Bounding box assertions pass in tests.
- `Mesh::validate()` passes after translation.

---

### Task 6.1b – Rotate Transform

**Goal**  
Implement `rotate([x,y,z])` transformation.

**Steps**

1. **Parser/AST**
   - Ensure `Statement::Rotate { angles: DVec3, children, span }` is parsed.
   - Support both `rotate([x,y,z])` and `rotate(a, [vx,vy,vz])` (axis-angle) forms.

2. **Evaluator**
   - Convert to `GeometryNode::Transform { matrix: DMat4, children, span }`.
   - Build rotation matrix using `glam` Euler angles (degrees → radians).
   - OpenSCAD order: rotate around X, then Y, then Z.

3. **openscad-mesh Implementation**
   - Add `pub fn rotate(mesh: &Mesh, angles: DVec3) -> Result<Mesh, MeshError>`.
   - Apply rotation matrix to all vertices and normals.

4. **Tests**
   - Test: `rotate([0,0,90]) cube(10);` rotates cube 90° around Z.
   - Test: `rotate([90,0,0])` rotates around X axis.
   - Test: Composed rotations follow correct order.

**Acceptance Criteria**

- `rotate([x,y,z])` produces correctly oriented geometry.
- Normals are rotated along with vertices.
- `validate()` passes after rotation.

---

### Task 6.1c – Scale Transform

**Goal**  
Implement `scale([x,y,z])` transformation.

**Steps**

1. **Parser/AST**
   - Ensure `Statement::Scale { factors: DVec3, children, span }` is parsed.
   - Support scalar: `scale(2)` = `scale([2,2,2])`.

2. **Evaluator**
   - Convert to `GeometryNode::Transform { matrix: DMat4, children, span }`.
   - Build scale matrix: `DMat4::from_scale(factors)`.
   - Handle negative scale factors (mirror operation).

3. **openscad-mesh Implementation**
   - Add `pub fn scale(mesh: &Mesh, factors: DVec3) -> Result<Mesh, MeshError>`.
   - Apply scale matrix to vertices.
   - **Handle negative scales**: flip normals if any factor is negative (odd number of negative = flip winding).

4. **Tests**
   - Test: `scale([2,1,1]) cube(5);` doubles X dimension.
   - Test: `scale(0.5)` shrinks uniformly.
   - Test: `scale([-1,1,1])` mirrors across YZ plane.

**Acceptance Criteria**

- `scale([x,y,z])` correctly scales geometry.
- Negative scales mirror correctly with proper winding.
- `validate()` passes after scaling.

---

### Task 6.2 – Union Boolean Operation (BSP Tree)

**Goal**  
Implement `union() { children }` using BSP tree boolean union algorithm.

**Steps**

1. **Parser/AST**
   - Ensure `Statement::Union { children: Vec<Statement>, span }` is parsed from `union() { ... }` blocks.
   - Grammar rule: `union_block` in Tree-sitter grammar.

2. **Evaluator**
   - Convert `Statement::Union` into `GeometryNode::Boolean { op: BooleanOp::Union, children, span }`.
   - `BooleanOp` enum: `Union`, `Difference`, `Intersection`.

3. **openscad-mesh Boolean Operations**
   - Create `libs/openscad-mesh/src/ops/boolean/{mod.rs, bsp.rs, polygon.rs, tests.rs}`.
   - Implement using BSP tree algorithm (based on csg.js by Evan Wallace):
     ```rust
     /// Performs boolean union of two meshes.
     /// 
     /// # Example
     /// ```
     /// let result = union(&cube, &sphere)?;
     /// assert!(result.validate());
     /// ```
     pub fn union(a: &Mesh, b: &Mesh) -> Result<Mesh, BooleanError>;
     
     /// Performs boolean union of multiple meshes.
     pub fn union_all(meshes: &[Mesh]) -> Result<Mesh, BooleanError>;
     ```
   - Handle edge cases: empty inputs, disjoint meshes, coincident faces.

4. **IR Bridge**
   - In `from_ir`, handle `GeometryNode::Boolean { op: Union, children }`:
     - Evaluate all children to `Vec<Mesh>`.
     - Call `union_all(children)`.

5. **Tests**
   - Test: `union() { cube(10); sphere(5); }` produces valid mesh.
   - Test: union of disjoint shapes = both shapes preserved.
   - Test: union of overlapping shapes = merged correctly.
   - Test: empty union returns empty mesh or error (match OpenSCAD behavior).

**Acceptance Criteria**

- `union()` with 2+ children produces valid mesh.
- No panics; all errors return `Result`.
- Tests cover overlapping and disjoint cases.

---

### Task 6.3 – Difference Boolean Operation (BSP Tree)

**Goal**  
Implement `difference() { a; b; c; }` using BSP tree boolean difference algorithm.  
OpenSCAD semantics: `a - b - c - ...` (subtract all subsequent children from the first).

**Steps**

1. **Parser/AST**
   - Ensure `Statement::Difference { children: Vec<Statement>, span }` is parsed.

2. **Evaluator**
   - Convert to `GeometryNode::Boolean { op: BooleanOp::Difference, children, span }`.

3. **openscad-mesh Implementation**
   - Add to `libs/openscad-mesh/src/ops/boolean/mod.rs`:
     ```rust
     /// Boolean difference: a - b.
     /// 
     /// # Example
     /// ```
     /// // Subtract sphere from cube
     /// let result = difference(&cube, &sphere)?;
     /// ```
     pub fn difference(a: &Mesh, b: &Mesh) -> Result<Mesh, BooleanError>;
     
     /// Boolean difference: first - rest[0] - rest[1] - ...
     pub fn difference_all(meshes: &[Mesh]) -> Result<Mesh, BooleanError>;
     ```
   - Implement using BSP tree algorithm: `A - B = ~(~A | B)`.

4. **IR Bridge**
   - Handle `BooleanOp::Difference` in `from_ir`.

5. **Tests**
   - Test: `difference() { cube(15, center=true); sphere(10); }` creates hollow cube.
   - Test: single child = identity (no subtraction).
   - Test: first child empty = error or empty result.
   - Validate mesh is valid after difference.

**Acceptance Criteria**

- `difference()` correctly subtracts children from first child.
- Output is valid mesh (correct normals).
- Edge cases handled with explicit errors.

---

### Task 6.4 – Intersection Boolean Operation (BSP Tree)

**Goal**  
Implement `intersection() { children }` using BSP tree boolean intersection algorithm.

**Steps**

1. **Parser/AST**
   - Ensure `Statement::Intersection { children: Vec<Statement>, span }` is parsed.

2. **Evaluator**
   - Convert to `GeometryNode::Boolean { op: BooleanOp::Intersection, children, span }`.

3. **openscad-mesh Implementation**
   - Add to `libs/openscad-mesh/src/ops/boolean/mod.rs`:
     ```rust
     /// Boolean intersection of two meshes.
     /// 
     /// # Example
     /// ```
     /// // Keep only overlapping volume
     /// let result = intersection(&cube, &sphere)?;
     /// ```
     pub fn intersection(a: &Mesh, b: &Mesh) -> Result<Mesh, BooleanError>;
     
     /// Boolean intersection of multiple meshes.
     pub fn intersection_all(meshes: &[Mesh]) -> Result<Mesh, BooleanError>;
     ```
   - Implement using BSP tree algorithm: `A & B = ~(~A | ~B)`.

4. **IR Bridge**
   - Handle `BooleanOp::Intersection` in `from_ir`.

5. **Tests**
   - Test: `intersection() { cube(15, center=true); sphere(10); }` creates rounded cube.
   - Test: disjoint shapes = empty mesh.
   - Test: identical shapes = original shape.

**Acceptance Criteria**

- `intersection()` correctly computes shared volume.
- Empty intersection returns empty mesh (not error).
- Valid mesh output in all cases.

---

### Task 6.5 – Robust Predicates

**Goal**  
Introduce robust predicates for geometric computations used by boolean operations.

**Steps**

1. **`robust` Integration**
   - Use the `robust` crate for orientation tests (e.g. `orient3d`).

2. **Replace Epsilon Checks**
   - Audit existing predicate code (e.g. `dot > EPSILON`) and replace with robust predicates where correctness is critical.

3. **Initialization**
   - If `robust` requires initialization, call once at WASM startup.

**Acceptance Criteria**

- Predicates behave correctly for nearly coplanar and nearly parallel cases in tests.

---

### Task 6.6 – Boolean Operations Core (BSP Tree Algorithm)

**Goal**  
Implement the core BSP tree algorithm powering union/difference/intersection.

**Steps**

1. **BSP Tree Structure**
   - Implement BSP tree in `libs/openscad-mesh/src/ops/boolean/bsp.rs`.
   - Each node holds a plane and lists of coplanar, front, and back polygons.
   - Use `rayon` for parallel operations where appropriate.

2. **Polygon Operations**
   - Implement polygon splitting in `libs/openscad-mesh/src/ops/boolean/polygon.rs`.
   - Use robust predicates for plane classification.

3. **CSG Operations**
   - `clip_to()`: Remove polygons on wrong side of BSP tree.
   - `invert()`: Flip all polygon normals.
   - `all_polygons()`: Collect all polygons from tree.

4. **Mesh Reconstruction**
   - Convert resulting polygons back to triangle mesh.
   - Remove degenerate triangles; optimize vertex deduplication.

**Acceptance Criteria**

- Boolean examples from the test corpus produce valid meshes.
- `Mesh::validate()` passes after each boolean operation.

---

### Task 6.7 – End-to-End Target Validation

**Goal**  
Verify the complete target validation test case passes.

**Steps**

1. **Run Target Test**
   - Execute the integration test from Task 6.0.
   - All assertions must pass.

2. **Visual Verification**
   - Render in Playground and compare with OpenSCAD reference.
   - Three shapes visible: union (left), intersection (center), difference (right).

3. **Performance Baseline**
   - Record compile time for future optimization reference.
   - Note: Skip optimization for now per project guidelines.

**Acceptance Criteria**

- Target validation test passes (green).
- Visual output matches OpenSCAD reference.
- No runtime errors in browser.

---

### Task 6.8 – Fuzz Testing

**Goal**  
Catch edge cases in boolean operations using property-based tests.

**Steps**

1. **Fuzz Harness**
   - Use `proptest` to generate random primitives (cubes, spheres) with random transforms.

2. **Operation Under Test**
   - Perform `union`, `difference`, `intersection` on random pairs.

3. **Invariant Checks**
   - Assert that `Mesh::validate()` always returns `true`.
   - Assert no panics occur.

**Acceptance Criteria**

- Fuzz tests run regularly in CI (or at least locally) and catch regressions in boolean logic.

---

## Phase 7 – Extrusions & Advanced Operations

> **Goal**: Implement extrusions, hull, minkowski, and offset operations using browser-safe algorithms.
> All operations are CPU-bound (Rust/WASM); WebGL is used only for rendering.

---

### Task 7.1 – Linear Extrude

**Goal**  
Implement `linear_extrude()` to extrude 2D shapes along the Z axis.

**OpenSCAD API (from `LinearExtrudeNode.cc`):**
```openscad
linear_extrude(height=100, v=[0,0,1], center=false, convexity=1, twist=0, slices=undef, segments=undef, scale=1)
```

**Steps**

1. **Evaluator Support**
   - Add `GeometryNode::LinearExtrude { height, direction, center, twist, slices, scale, children, span }`.
   - Parse all parameters with OpenSCAD precedence rules.

2. **openscad-mesh Implementation**
   - Create `libs/openscad-mesh/src/ops/extrude/linear/{mod.rs, tests.rs}`.
   - Algorithm:
     - Take 2D polygon outline from child geometry.
     - Generate slices at intervals along extrusion axis.
     - Apply twist rotation per slice if `twist != 0`.
     - Apply scale interpolation per slice if `scale != 1`.
     - Generate side faces connecting slices.
     - Cap top and bottom with triangulated polygons.

3. **Tests**
   - Test: `linear_extrude(10) square(5);` produces box mesh.
   - Test: `linear_extrude(10, twist=90) square(5);` produces twisted box.
   - Test: `linear_extrude(10, scale=0.5) circle(5);` produces cone.

**Acceptance Criteria**

- `linear_extrude()` works with all parameter combinations.
- Mesh is valid and watertight.
- Tests cover twist, scale, and centering.

---

### Task 7.2 – Rotate Extrude

**Goal**  
Implement `rotate_extrude()` to revolve 2D shapes around the Z axis.

**OpenSCAD API (from `RotateExtrudeNode.cc`):**
```openscad
rotate_extrude(angle=360, start=180, convexity=2, $fn, $fa, $fs)
```

**Steps**

1. **Evaluator Support**
   - Add `GeometryNode::RotateExtrude { angle, start, convexity, children, span }`.

2. **openscad-mesh Implementation**
   - Create `libs/openscad-mesh/src/ops/extrude/rotate/{mod.rs, tests.rs}`.
   - Algorithm:
     - Take 2D polygon outline from child geometry.
     - Rotate outline around Z axis at intervals determined by `$fn/$fa/$fs`.
     - Generate quads/triangles connecting adjacent rotated outlines.
     - Cap ends if `angle < 360`.

3. **Tests**
   - Test: `rotate_extrude() circle(5);` produces torus.
   - Test: `rotate_extrude(angle=180) square([5,10]);` produces half-cylinder.

**Acceptance Criteria**

- `rotate_extrude()` handles full and partial rotations.
- Fragment count respects `$fn/$fa/$fs`.
- Mesh is valid.

---

### Task 7.3 – Hull (QuickHull Algorithm)

**Goal**  
Implement `hull()` to compute the convex hull of child geometries.

**OpenSCAD API (from `CgalAdvNode.cc`):**
```openscad
hull() { children }
```

**Steps**

1. **Evaluator Support**
   - Add `GeometryNode::Hull { children, span }`.

2. **openscad-mesh Implementation**
   - Create `libs/openscad-mesh/src/ops/hull/{mod.rs, quickhull.rs, tests.rs}`.
   - Algorithm: **QuickHull**
     - Find extreme points to form initial simplex.
     - For each face, find furthest point outside.
     - Add point to hull, update faces.
     - Repeat until no points outside.

3. **Tests**
   - Test: `hull() { cube(5); translate([10,0,0]) cube(5); }` produces elongated box.
   - Test: `hull() { sphere(5); translate([10,0,0]) sphere(5); }` produces capsule shape.

**Acceptance Criteria**

- `hull()` produces valid convex mesh.
- Works with any number of children.
- QuickHull algorithm is O(n log n) average case.

---

### Task 7.4 – Minkowski (Convex Sum)

**Goal**  
Implement `minkowski()` for Minkowski sum of child geometries.

**OpenSCAD API (from `CgalAdvNode.cc`):**
```openscad
minkowski(convexity=1) { children }
```

**Steps**

1. **Evaluator Support**
   - Add `GeometryNode::Minkowski { convexity, children, span }`.

2. **openscad-mesh Implementation**
   - Create `libs/openscad-mesh/src/ops/minkowski/{mod.rs, tests.rs}`.
   - Algorithm: **Vertex-Face Iteration** (for convex shapes)
     - For each vertex of shape A, translate shape B to that vertex.
     - Compute union of all translated shapes.
   - For non-convex shapes, decompose into convex parts first.

3. **Tests**
   - Test: `minkowski() { cube(10); sphere(2); }` produces rounded cube.
   - Test: `minkowski() { square(10); circle(2); }` produces 2D rounded square.

**Acceptance Criteria**

- `minkowski()` works for convex shapes.
- Produces valid mesh.
- Performance note: This is inherently expensive; correctness over speed.

---

### Task 7.5 – Offset (2D Polygon Offset)

**Goal**  
Implement `offset()` for 2D polygon expansion/contraction.

**OpenSCAD API (from `OffsetNode.cc`):**
```openscad
offset(r=1)           // Round joins (default)
offset(delta=1)       // Miter joins
offset(delta=1, chamfer=true)  // Square joins
```

**Steps**

1. **Evaluator Support**
   - Add `GeometryNode::Offset { amount, join_type, children, span }`.
   - `join_type`: `Round`, `Miter`, `Square`.

2. **openscad-mesh Implementation**
   - Create `libs/openscad-mesh/src/ops/offset/{mod.rs, tests.rs}`.
   - Algorithm: **Clipper2-style polygon offset**
     - Offset each edge by the specified amount.
     - Handle joins according to join type.
     - Handle self-intersections.

3. **Tests**
   - Test: `offset(r=2) square(10);` produces rounded square outline.
   - Test: `offset(delta=-2) square(10);` produces smaller square.

**Acceptance Criteria**

- `offset()` handles positive and negative values.
- Join types produce correct geometry.
- Works with complex polygons.

---

## 8. Global Ongoing Tasks

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

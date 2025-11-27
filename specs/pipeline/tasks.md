# Rust OpenSCAD Pipeline – Task Breakdown

> This file is the **actionable backlog** for the Rust OpenSCAD pipeline.  
> It is structured into small, test-driven tasks and subtasks.  
> See `overview-plan.md` for goals, architecture, and coding standards.

---

## Implementation Progress (Updated: 2025-11-27, Manifold Algorithm & Optimizations)

### 🚀 MAJOR PERFORMANCE IMPROVEMENT: Manifold-like CSG Algorithm

**Implemented native Manifold-like CSG algorithm with:**
- Intersection-based boolean operations (not BSP)
- Spatial hashing for O(1) triangle queries
- Ray casting for inside/outside classification
- Lazy evaluation with CSG tree
- Tree rewriting optimization
- Mesh caching for repeated module calls

**Performance Results** (same $fs=0.1 test case):

| Metric | Before (BSP) | After (Manifold) | Improvement |
|--------|--------------|------------------|-------------|
| Total Time | 10,509ms | **575ms** | **18x faster** |
| WASM Mesh Time | 9,048ms | **10ms** | **900x faster** |
| Vertices | 237,443 | **11,414** | **95% less** |
| Triangles | 106,169 | **10,280** | **90% less** |

**New Modules Created:**
- `ops/boolean/manifold/` - Intersection-based CSG algorithm
- `ops/boolean/manifold/spatial_index.rs` - Spatial hashing for fast queries
- `ops/boolean/halfedge/` - Halfedge mesh data structure
- `ops/boolean/csg_tree/` - Lazy evaluation and tree rewriting
- `ops/boolean/cache/` - LRU mesh cache with hit/miss stats

### Previous Analysis (for reference)

**High-resolution models** (`$fs=0.1`) create extremely detailed meshes:
- Sphere r=10 with `$fa=5, $fs=0.1` → 72 segments (from $fa, since min(72, 628)=72)
- Complex CSG with debug helpers: 237k vertices, 106k triangles, ~10.5s (BSP)
- With Manifold: 11k vertices, 10k triangles, ~575ms

**Recommendations for users**:
- Use `$fs = 0.5` to `$fs = 1.0` for 3D printing (not `$fs = 0.1`)
- Use `$fa = 1` for smooth curves
- Disable debug helpers for faster iteration

### Recent Changes

- **Performance Timing**: Added WASM performance breakdown logging (parse/eval/mesh)
- **BSP Performance Optimization**: Improved `BspNode::new()` from O(N²) to O(N) complexity
  - Replaced `remove(0)` with `swap_remove` for O(1) splitter extraction
  - Added pre-allocation for front/back vectors to reduce reallocations
  - **Result**: ~3.5x speedup on high-resolution boolean tests (17s → 5s)
- **Built-in Functions**: Added OpenSCAD-compatible built-in functions
  - `version()` - Returns [year, month, day] vector
  - `version_num()` - Returns YYYYMMDD number
  - `str()` - Converts values to string representation
  - `concat()` - Concatenates vectors
  - `lookup()` - Linear interpolation lookup in tables
- **Iterative BSP Operations**: Converted ALL BSP tree operations from recursive to iterative using explicit stacks. This fixes WASM stack overflow with complex boolean operations involving high-resolution meshes.
  - `BspNode::new()` - Iterative tree construction
  - `invert()` - Iterative tree inversion
  - `clip_polygons()` - Iterative polygon clipping
  - `clip_to()` - Iterative tree clipping
  - `all_polygons()` - Iterative polygon collection
  - `polygon_count()` / `depth()` - Iterative counting
  - **CRITICAL**: Implemented iterative `Drop` for `BspNode` to prevent stack overflow during destruction
  - Verified with high-resolution stress tests (`$fs=0.1`)
- **Expression Wrapper Types**: Added CST parser support for `literal` and `expression` supertype wrappers that delegate to child nodes. This fixes parsing of float literals like `$fs = 0.1;`.
- **Undef Literal**: Added support for `undef` expression type in CST parser.
- **Ternary Expressions**: Added `ternary_expression` parsing for `cond ? then : else` syntax.
- **Index Expressions**: Added `index_expression` and `dot_index_expression` for array/object access.
- **Let Expressions**: Added `let_expression` parsing (delegates to body expression).
- **Echo/Assert Statements**: Added `echo_statement` and `assert_statement` handling (pass through to following statement).
- **Echo/Assert Expressions**: Added `assert_expression` and `echo_expression` for inline assert/echo.
- **Test Coverage**: Added 4 new tests for wrapper types plus 50 boolean tests all pass with iterative BSP.
- **Children Scoping Cleanup**: Implemented scope popping/restoration for correct children evaluation.
- **Enhanced Expression Support**: Added dynamic `Value` enum for OpenSCAD types with binary operators for strings, vectors, and comparison.

### Completed ✅

| Phase | Feature | Status | Tests |
|-------|---------|--------|-------|
| 1.1 | Workspace & Crate Setup | ✅ Complete | - |
| 1.2 | Config crate with constants | ✅ Complete | 29 tests |
| 1.3 | openscad-ast crate | ✅ Complete | 45 tests |
| 1.4 | openscad-eval crate | ✅ Complete | 57 tests |
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

## Current Backlog

| Priority | Task | Status | Notes |
|----------|------|--------|-------|
| **High** | **Verify Playground with Complex Code** | 🚧 In Progress | Test provided OpenSCAD code with modules, functions, booleans |
| **High** | **Debug `version()` Function** | ⏳ Pending | Implement `version()` built-in function |
| High | Cleanup Legacy Code | ⏳ Pending | Remove old evaluator code, ensure consistent naming |
| Medium | Add `let` Block Support | ⏳ Pending | Implement `let(var=val)` in evaluator with proper scoping |
| Medium | Add Error Highlighting | ⏳ Pending | Show syntax errors inline in editor |
| Low | Implement `use`/`include` | ⏳ Pending | Support file imports (requires virtual FS) |
| Low | Performance Optimization | ⏳ Pending | Profile and optimize BSP operations for large meshes |

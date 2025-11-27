# Rust OpenSCAD Pipeline – Overview Plan

_Last updated: 2025-11-26 — **Iterative BSP Operations & Stack Safety!** Converted all BSP tree operations from recursive to iterative using explicit stacks. This fixes WASM stack overflow with complex boolean operations. Also added CST parser support for expression wrappers, undef, ternary/index expressions, and echo/assert statements. All tests pass (280+)._

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

**Rendering Details:**
- **Mesh Material**: `MeshStandardMaterial` with metalness/roughness for realistic lighting
- **Edge Highlighting**: `WireframeGeometry` overlay showing ALL triangle edges (not just sharp edges)
  - Uses `LineSegments` with semi-transparent black lines
  - Matches OpenSCAD's behavior where smooth surfaces show tessellation wireframe
  - `EdgesGeometry` was rejected because it only shows edges where face normals differ by threshold angle
- **Coordinate System**: Z-up axis to match OpenSCAD/CAD conventions

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

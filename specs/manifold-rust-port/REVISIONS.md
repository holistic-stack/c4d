# Specification Revisions - OpenSCAD Alignment

## Date: November 17, 2024

## Purpose

This document tracks revisions made to align the Manifold Rust port specification with OpenSCAD's actual feature set, removing Manifold-specific features not present in OpenSCAD.

---

## Research Summary

Used Perplexity AI to research OpenSCAD's complete feature set:
- All 3D primitives: cube, sphere, cylinder, polyhedron
- All 2D primitives: square, circle, polygon, text
- All transformations: translate, rotate, scale, mirror, resize, multmatrix, color
- All boolean operations: union, difference, intersection
- Special operations: hull, minkowski, linear_extrude, rotate_extrude, offset, projection
- File operations: import, surface, render
- Control flow: for, intersection_for, if/else

---

## Major Changes

### 1. Removed Manifold-Specific Features

**From Overview (00-OVERVIEW.md)**:
- ❌ Removed: Level set (SDF-based construction)
- ❌ Removed: Warp (custom deformation functions)
- ❌ Removed: Slice, Split, SplitByPlane, TrimByPlane operations
- ❌ Removed: Vertex property manipulation features
- ❌ Removed: Mesh refinement and simplification
- ❌ Removed: OriginalID tracking for materials

**From Architecture (01-ARCHITECTURE.md)**:
- ❌ Simplified ManifoldImpl: removed `tolerance`, `original_id`, `properties`
- ❌ Simplified HalfEdgeMesh: removed `vert_properties`
- ❌ Simplified MeshGL: removed all Manifold-specific metadata (merge vectors, runs, face IDs, tangents)
- ❌ Removed smoothing module (normals, refine, simplify)
- ❌ Removed warp transformation

---

### 2. Added Missing OpenSCAD Features

**Phase 2 - Primitives & Transforms:**
- ✅ Added: Polyhedron primitive (replaces tetrahedron)
- ✅ Added: Resize transform (with auto aspect ratio)
- ✅ Renamed: "Arbitrary Matrix Transform" → "Multmatrix Transform" (4x4 matrix)

**Phase 4 - 2D Operations:**
- ✅ Added: offset() operation for 2D polygon offset
- ✅ Added: text() primitive (DEFERRED - complex, requires font rendering)
- ✅ Added: projection() operation for 3D to 2D conversion
- ✅ Added: surface() import (DEFERRED - not critical for MVP)
- ✅ Added: Minkowski sum operation (marked as complex, consider DEFER)

**Phase 5 - Integration:**
- ✅ Added: import() operation with STL file import
- ✅ Added: render() operation (no-op in our implementation)
- ✅ Added: STL import functionality

---

## Updated Module Organization

**Transforms Module:**
- `translate.rs`
- `rotate.rs`
- `scale.rs`
- `mirror.rs`
- `resize.rs` ⭐ NEW
- `multmatrix.rs` ⭐ RENAMED

**Special Operations Module (NEW):**
- `hull.rs` - Convex hull (2D and 3D)
- `minkowski.rs` - Minkowski sum
- `projection.rs` - 3D to 2D projection
- `surface.rs` - Height map import

**Polygon Module:**
- `triangulate.rs`
- `clipper.rs` (optional)
- ~~`convex_hull.rs`~~ → moved to special_ops

**Removed:**
- ~~`smoothing/`~~ module (not in OpenSCAD)
- ~~`warp.rs`~~ (not in OpenSCAD)

---

## Feature Status

### Core OpenSCAD Features - INCLUDED

| Feature | Status | Phase |
|---------|--------|-------|
| cube | ✅ Included | 2 |
| sphere | ✅ Included | 2 |
| cylinder | ✅ Included | 2 |
| polyhedron | ✅ Included | 2 |
| square | ✅ Included | 4 |
| circle | ✅ Included | 4 |
| polygon | ✅ Included | 4 |
| translate | ✅ Included | 2 |
| rotate | ✅ Included | 2 |
| scale | ✅ Included | 2 |
| mirror | ✅ Included | 2 |
| resize | ✅ Included | 2 |
| multmatrix | ✅ Included | 2 |
| union | ✅ Included | 3 |
| difference | ✅ Included | 3 |
| intersection | ✅ Included | 3 |
| hull | ✅ Included | 4 |
| linear_extrude | ✅ Included | 4 |
| rotate_extrude | ✅ Included | 4 |
| offset | ✅ Included | 4 |
| projection | ✅ Included | 4 |
| import | ✅ Included | 5 |
| render | ✅ Included | 5 |

### OpenSCAD Features - DEFERRED

| Feature | Reason | Future Phase |
|---------|--------|--------------|
| text() | Complex, requires font rendering library | Phase 6+ |
| surface() | Not critical for MVP | Phase 6+ |
| minkowski() | Very complex algorithm | Phase 6+ |

### Manifold-Specific Features - REMOVED

| Feature | Reason |
|---------|--------|
| Level sets / SDF | Not in OpenSCAD |
| Warp | Not in OpenSCAD |
| Slice / Split / Trim | Not in OpenSCAD |
| Mesh refinement | Not in OpenSCAD |
| Vertex properties (advanced) | Not needed for OpenSCAD |
| Original ID tracking | Not in OpenSCAD |
| Merge vectors | Internal Manifold feature |

---

## Data Structure Changes

### Before:
```rust
struct ManifoldImpl {
    mesh: HalfEdgeMesh,
    bbox: BoundingBox,
    tolerance: f64,
    original_id: u32,
    properties: VertexProperties,
}

pub struct MeshGL {
    pub num_prop: usize,
    pub vert_properties: Vec<f32>,
    pub tri_verts: Vec<u32>,
    pub merge_from_vert: Vec<u32>,
    pub merge_to_vert: Vec<u32>,
    pub run_index: Vec<u32>,
    pub run_original_id: Vec<u32>,
    pub run_transform: Vec<f32>,
    pub face_id: Vec<u32>,
    pub halfedge_tangent: Vec<f32>,
    pub tolerance: f32,
}
```

### After (Simplified):
```rust
struct ManifoldImpl {
    mesh: HalfEdgeMesh,
    bbox: BoundingBox,
}

pub struct MeshGL {
    pub num_prop: usize,
    pub vert_properties: Vec<f32>,
    pub tri_verts: Vec<u32>,
}
```

---

## Task Updates

### Phase 2
- **Task 2.4**: Changed from "Tetrahedron" to "Polyhedron"
- **Task 2.9**: Changed from "Arbitrary Matrix" to "Resize"
- **Task 2.10**: NEW - "Multmatrix Transform"

### Phase 4
- **Task 4.2b**: NEW - Text primitive (DEFERRED)
- **Task 4.3**: NEW - Offset operation
- **Task 4.7**: Existing - Convex hull
- **Task 4.8**: NEW - Minkowski sum (complex, consider DEFER)
- **Task 4.9**: NEW - Projection operation
- **Task 4.10**: NEW - Surface import (DEFERRED)

### Phase 5
- **Task 5.6b**: NEW - STL Import
- **Task 5.6c**: NEW - render() and import() operations

---

## Impact on Timeline

**No significant impact** on overall timeline:
- Removed features were complex and time-consuming
- Added features are mostly straightforward
- Deferred text() and surface() to future phases
- Minkowski may be deferred if too complex

**Estimated Effort Changes:**
- Removed: ~40-60 hours (SDF, warp, mesh refinement, properties)
- Added: ~40-50 hours (offset, projection, import, polyhedron)
- Net: Similar total effort, better OpenSCAD alignment

---

## Documentation Updates

All specification documents updated:
- ✅ 00-OVERVIEW.md - Updated feature lists
- ✅ 01-ARCHITECTURE.md - Simplified data structures
- ✅ 04-TASKS-PHASE2.md - Updated primitives and transforms
- ✅ 04-TASKS-PHASE4.md - Added 2D operations
- ✅ 04-TASKS-PHASE5.md - Added import/render
- ✅ 05-TESTING-STRATEGY.md - No changes needed
- ✅ 06-ROADMAP.md - No changes needed (timeline similar)

---

## Additional Revisions (November 2025)

### 4. Crate Pipeline and Web Viewer Integration

- Updated 00-OVERVIEW.md, 01-ARCHITECTURE.md, and README.md to explicitly document the
  crate pipeline:
  - `libs/openscad-parser` → CST
  - `libs/openscad-ast` → AST from CST
  - `libs/openscad-eval` → evaluated geometry IR (no remaining vars/loops)
  - `libs/manifold-rs` → manifold geometries + MeshGL, including an OpenSCAD helper API
  - `libs/wasm` → WebAssembly wrapper exposing the parse-and-evaluate API
  - `playground/` (Svelte + Three.js) → full-window 3D web viewer

- Updated 04-TASKS-PHASE5.md to include tasks for `libs/wasm` and the Svelte + Three.js playground
  and to make Phase 5 completion dependent on the entire Rust→WASM→web pipeline.

- Updated 05-TESTING-STRATEGY.md, 06-ROADMAP.md, 07-THIRD-PARTY-LIBRARIES.md, and
  IMPLEMENTATION-STRATEGY.md so that testing, schedule, library choices, and implementation phases
  all explicitly reference the full OpenSCAD → CST → AST → IR → Manifold/MeshGL → WASM → web
  pipeline.

---

## Next Steps

1. ✅ Specifications aligned with OpenSCAD
2. ⏭️ Begin implementation with Phase 1
3. ⏭️ Revisit deferred features (text, surface, minkowski) in Phase 6+

---

## References

- OpenSCAD Cheat Sheet: https://openscad.org/cheatsheet/
- OpenSCAD User Manual: https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/
- OpenSCAD Transformations: https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/Transformations
- Manifold GitHub: https://github.com/elalish/manifold

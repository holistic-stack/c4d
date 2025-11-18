# Manifold Rust Port - Architecture Design

## Table of Contents

1. [Core Data Structures](#core-data-structures)
2. [Module Organization](#module-organization)
3. [Algorithm Design](#algorithm-design)
4. [Parallelization Strategy](#parallelization-strategy)
5. [Error Handling](#error-handling)
6. [Memory Management](#memory-management)

## Core Data Structures

### Manifold

The primary type representing a solid 3D object.

```rust
pub struct Manifold {
    impl_: Arc<ManifoldImpl>,
}

struct ManifoldImpl {
    mesh: HalfEdgeMesh,
    bbox: BoundingBox,
}
```

**Design Rationale**:
- `Arc` allows cheap cloning for CSG tree construction
- Immutable operations return new `Manifold` instances
- Internal mesh representation optimized for queries

### HalfEdgeMesh

Internal representation using half-edge data structure for efficient manifold operations.

```rust
struct HalfEdgeMesh {
    // Vertex positions (3 floats per vertex)
    vert_pos: Vec<Vec3>,
    
    // Triangle indices (3 indices per triangle, CCW from outside)
    tri_verts: Vec<[u32; 3]>,
    
    // Half-edge pairing (maps each halfedge to its pair)
    // halfedge i is edge from tri_verts[i] to tri_verts[(i+1)%3]
    halfedge_pair: Vec<u32>,
    
    // Face normals (computed lazily)
    face_normals: Option<Vec<Vec3>>,
}
```

**Design Rationale**:
- Half-edge structure enables O(1) topology queries
- Compact representation for memory efficiency
- Lazy computation of derived data

### MeshGL

Public-facing mesh representation compatible with graphics APIs.

```rust
pub struct MeshGL {
    /// Number of properties per vertex (always >= 3 for x, y, z)
    pub num_prop: usize,
    
    /// Interleaved vertex properties [x, y, z, ...]
    pub vert_properties: Vec<f32>,
    
    /// Triangle vertex indices (3 per triangle, CCW)
    pub tri_verts: Vec<u32>,

    /// Per-triangle source/material ID for OpenSCAD color() and materials
    ///
    /// Each triangle stores an ID that indicates which input object/material
    /// it originated from. This is required to preserve `color()` information
    /// through boolean operations (`union()`, `difference()`, `intersection()`).
    pub tri_original_id: Vec<u32>,
}
```

**Design Rationale**:
- Directly usable with OpenGL/WebGL/wgpu
- Simple, standard mesh format
- Easy export to STL and other formats
- Supports OpenSCAD `color()` inheritance by tracking per-triangle source IDs

### CrossSection

2D polygon representation for extrusion operations.

```rust
pub struct CrossSection {
    impl_: Arc<CrossSectionImpl>,
}

struct CrossSectionImpl {
    polygons: Polygons,
    bbox: BoundingBox2D,
}

pub struct Polygons {
    // Each polygon is a list of simple polygons (outer + holes)
    contours: Vec<Vec<Vec2>>,
}
```

## Module Organization

**Design Principles**:
- Each module in its own folder with `mod.rs` + `tests.rs`
- Maximum 500 lines per file
- Tests alongside implementation (not separate directory)
- Single Responsibility: one logical component per module
- See [08-CODING-STANDARDS.md](./08-CODING-STANDARDS.md) for details

```
libs/manifold-rs/
├── src/
│   ├── lib.rs                          # Public API exports
│   ├── config.rs                       # Centralized constants
│   ├── error.rs                        # Error types
│   │
│   ├── core/
│   │   ├── mod.rs                      # Re-exports
│   │   ├── vec3/
│   │   │   ├── mod.rs                  # Vec3 implementation
│   │   │   └── tests.rs                # Vec3 tests
│   │   ├── bounding_box/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   ├── mesh_gl/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   └── half_edge_mesh/
│   │       ├── mod.rs
│   │       ├── builder.rs              # Split if >500 lines
│   │       ├── validation.rs
│   │       └── tests.rs
│   │
│   ├── primitives/
│   │   ├── mod.rs                      # Re-exports
│   │   ├── cube/
│   │   │   ├── mod.rs                  # Cube implementation
│   │   │   └── tests.rs                # Cube tests
│   │   ├── sphere/
│   │   │   ├── mod.rs
│   │   │   ├── icosphere.rs            # Subdivision
│   │   │   └── tests.rs
│   │   ├── cylinder/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   └── polyhedron/
│   │       ├── mod.rs
│   │       └── tests.rs
│   │
│   ├── boolean/
│   │   ├── mod.rs
│   │   ├── collision/
│   │   │   ├── mod.rs                  # R-tree collision
│   │   │   └── tests.rs
│   │   ├── intersection/
│   │   │   ├── mod.rs                  # Edge-triangle
│   │   │   └── tests.rs
│   │   ├── topology/
│   │   │   ├── mod.rs                  # Construction
│   │   │   └── tests.rs
│   │   └── operations/
│   │       ├── mod.rs
│   │       ├── union.rs
│   │       ├── difference.rs
│   │       ├── intersection.rs
│   │       └── tests.rs
│   │
│   ├── transforms/
│   │   ├── mod.rs
│   │   ├── translate/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   ├── rotate/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   ├── scale/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   ├── mirror/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   ├── resize/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   └── multmatrix/
│   │       ├── mod.rs
│   │       └── tests.rs
│   │
│   ├── cross_section/
│   │   ├── mod.rs
│   │   ├── primitives/
│   │   │   ├── mod.rs                  # 2D shapes
│   │   │   └── tests.rs
│   │   ├── boolean/
│   │   │   ├── mod.rs                  # 2D booleans
│   │   │   └── tests.rs
│   │   └── offset/
│   │       ├── mod.rs
│   │       └── tests.rs
│   │
│   ├── extrude/
│   │   ├── mod.rs
│   │   ├── linear/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   └── revolve/
│   │       ├── mod.rs
│   │       └── tests.rs
│   │
│   ├── polygon/
│   │   ├── mod.rs
│   │   ├── triangulate/
│   │   │   ├── mod.rs                  # Uses geo crate
│   │   │   └── tests.rs
│   │   └── clipper/
│   │       ├── mod.rs                  # Clipper2 wrapper
│   │       └── tests.rs
│   │
│   ├── special_ops/
│   │   ├── mod.rs
│   │   ├── hull/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   ├── minkowski/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   ├── projection/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   └── surface/
│   │       ├── mod.rs
│   │       └── tests.rs
│   │
│   ├── io/
│   │   ├── mod.rs
│   │   ├── stl/
│   │   │   ├── mod.rs                  # Uses stl_io crate
│   │   │   ├── reader.rs
│   │   │   ├── writer.rs
│   │   │   └── tests.rs
│   │   └── obj/
│   │       ├── mod.rs
│   │       └── tests.rs
│   │
│   └── utils/
│       ├── mod.rs
│       ├── math/
│       │   ├── mod.rs
│       │   └── tests.rs
│       └── spatial/
│           ├── mod.rs                  # Uses rstar crate
│           └── tests.rs
│
├── tests/                              # Integration tests
│   ├── primitives_test.rs
│   ├── boolean_operations_test.rs
│   ├── transformations_test.rs
│   └── end_to_end_test.rs
│
├── benches/                            # Criterion benchmarks
│   └── benchmarks.rs
│
└── examples/                           # Example usage
    ├── basic_primitives.rs
    ├── boolean_operations.rs
    └── openscad_pipeline.rs
```

### libs/openscad-eval Module Organization

The `libs/openscad-eval` crate evaluates the typed OpenSCAD AST into a fully resolved
geometry IR (no remaining variables, loops, or control flow) that can be consumed by
`libs/manifold-rs`. Its layout mirrors the same SRP, 500-line-per-file, and co-located
tests rules:

```
libs/openscad-eval/
├── src/
│   ├── lib.rs              # Public Evaluator API
│   ├── config.rs           # Evaluator-wide constants ($fn, $fa, $fs, limits)
│   ├── error.rs            # EvalError and Result<T> alias
│   │
│   ├── value/              # Value enum, GeometryId, helpers
│   │   ├── mod.rs
│   │   └── tests.rs
│   ├── geometry_ir/        # Fully evaluated geometry tree (IR)
│   │   ├── mod.rs
│   │   └── tests.rs
│   ├── context/            # EvalContext, scopes, module/function tables
│   │   ├── mod.rs
│   │   └── tests.rs
│   ├── builtins/           # Built-in functions and modules (3D, 2D, transforms, booleans, special)
│   │   ├── mod.rs
│   │   └── ... tests.rs
│   ├── eval_expr/          # Expression evaluation (literals, operators, lists, calls)
│   │   ├── mod.rs
│   │   └── ... tests.rs
│   ├── eval_stmt/          # Statement evaluation (primitives, transforms, booleans, control flow, modules)
│   │   ├── mod.rs
│   │   └── ... tests.rs
│   ├── frontend/           # High-level Evaluator, AST bridge
│   │   ├── mod.rs
│   │   └── tests.rs
│   ├── loader/             # include/use resolution and other file-based loading (I/O only)
│   │   ├── mod.rs
│   │   └── tests.rs
│   └── utils/              # Shared small helpers (no domain logic)
│       ├── mod.rs
│       └── tests.rs
│
├── tests/                  # Integration & e2e tests (OpenSCAD → eval → manifold-rs)
└── benches/                # Criterion benchmarks for evaluator hot paths
```

All modules follow the same standards as `libs/manifold-rs`: one responsibility per
folder, `mod.rs + tests.rs`, and files kept under 500 lines. See
`08-CODING-STANDARDS.md` and `05-TESTING-STRATEGY.md` for details.

## OpenSCAD→Geometry→Web Pipeline and Public APIs

This project defines a clear, layered pipeline from OpenSCAD source code to manifold geometry and
eventually to a WebAssembly-powered web viewer.

### Crate Responsibilities

- **libs/openscad-parser**
  - Tree-sitter wrapper that exposes a **CST parsing API**.
  - Example shape:
    ```rust
    pub fn parse_to_cst(source: &str) -> Result<CstRoot>;
    ```

- **libs/openscad-ast**
  - Builds a **typed AST** from the CST produced by `libs/openscad-parser`.
  - Responsible for OpenSCAD language structure (expressions, statements, modules, functions, etc.).
  - Public API (conceptual):
    ```rust
    pub fn parse_source(source: &str) -> Result<AstRoot> {
        let cst = openscad_parser::parse_to_cst(source)?;
        Ok(from_cst(&cst)?)
    }

    pub fn from_cst(cst: &CstRoot) -> Result<AstRoot>;
    ```

- **libs/openscad-eval**
  - Evaluates the typed AST, resolving all OpenSCAD semantics into a **fully evaluated geometry IR**
    (no variables, loops, or conditionals remain).
  - Public API (conceptual):
    ```rust
    pub fn evaluate_ast(ast: &AstRoot) -> Result<GeometryIr>;

    pub fn evaluate_source(source: &str) -> Result<GeometryIr> {
        let ast = openscad_ast::parse_source(source)?;
        evaluate_ast(&ast)
    }
    ```

- **libs/manifold-rs**
  - Core manifold kernel (primitives, booleans, transforms, 2D operations, extrusion, MeshGL).
  - Can be used directly for geometry construction without any OpenSCAD involvement.
  - Additionally, exposes an **OpenSCAD integration helper** (e.g. feature-gated `openscad` module)
    that wires together `openscad-ast` and `openscad-eval`:
    ```rust
    pub fn parse_and_evaluate_openscad(source: &str) -> Result<MeshGL> {
        let ast = openscad_ast::parse_source(source)?;
        let ir  = openscad_eval::evaluate_ast(&ast)?;
        let manifold = manifold_from_ir(&ir)?;
        Ok(manifold.to_meshgl())
    }
    ```
    where `manifold_from_ir` walks the geometry IR and invokes primitives, transforms and boolean
    operations defined in `libs/manifold-rs`.

- **libs/wasm**
  - WebAssembly glue crate that uses `wasm-bindgen` to expose the high-level
    `parse_and_evaluate_openscad`-style API from `libs/manifold-rs` to JavaScript:
    ```rust
    #[wasm_bindgen]
    pub fn parse_openscad_to_mesh(source: &str) -> JsValue {
        let mesh = manifold_rs::parse_and_evaluate_openscad(source)
            .map_err(to_js_error)?;
        mesh_to_js(mesh)
    }
    ```
  - This crate is the only place that knows about `wasm-bindgen` and JS types.

- **playground/** (Svelte + Three.js)
  - Frontend application that loads the `libs/wasm` WebAssembly bundle.
  - Calls the exported `parse_openscad_to_mesh` API whenever the user edits OpenSCAD code.
  - Renders the resulting mesh using Three.js in a 3D viewport that fills **100% of the browser
    window** (both width and height), ensuring a full-window modeling experience.

This separation keeps the core manifold kernel decoupled from OpenSCAD and web concerns, while still
providing a simple end-to-end API for parsing and visualizing OpenSCAD models.

## Algorithm Design

### Boolean Operations

Implementing Julian Smith's approach from his dissertation:

#### Phase 1: Broad-Phase Collision Detection

```rust
// Use R-tree to find potentially intersecting triangle pairs
fn find_intersections(mesh_a: &Manifold, mesh_b: &Manifold) -> Vec<(TriIdx, TriIdx)> {
    let tree_a = build_rtree(mesh_a);
    let tree_b = build_rtree(mesh_b);
    
    // Query overlapping bounding boxes
    tree_a.intersection_candidates_with(&tree_b)
}
```

#### Phase 2: Edge-Triangle Intersection

```rust
// Find exact intersection points between edges and triangles
struct Intersection {
    edge: HalfEdge,
    triangle: TriIdx,
    point: Vec3,
    parameter: f64,  // Position along edge
}

fn compute_intersections(pairs: &[(TriIdx, TriIdx)]) -> Vec<Intersection>
```

#### Phase 3: Topology Construction

```rust
// Build new topology by:
// 1. Splitting edges at intersection points
// 2. Classifying triangles (inside/outside/boundary)
// 3. Constructing new manifold mesh
```

**Key Insight**: Never ask the same geometric question twice. Store results and reuse them to avoid floating-point inconsistencies.

### Primitive Construction

#### Sphere via Icosphere Subdivision

```rust
pub fn sphere(radius: f64, circular_segments: usize) -> Manifold {
    // Start with icosahedron
    let mut mesh = icosahedron(radius);
    
    // Calculate subdivision levels
    let subdivisions = calculate_subdivisions(circular_segments);
    
    // Subdivide and project to sphere
    for _ in 0..subdivisions {
        mesh = subdivide_and_project(mesh, radius);
    }
    
    Manifold::from_mesh(mesh)
}
```

#### Cylinder Construction

```rust
pub fn cylinder(height: f64, radius_low: f64, radius_high: f64, 
                circular_segments: usize, center: bool) -> Manifold {
    let segments = resolve_circular_segments(radius_low.max(radius_high), 
                                             circular_segments);
    
    // Generate top and bottom circles
    let bottom = generate_circle(radius_low, segments, 0.0);
    let top = generate_circle(radius_high, segments, height);
    
    // Generate side triangles connecting circles
    let side = generate_cylinder_sides(&bottom, &top);
    
    // Cap top and bottom
    let mesh = combine_meshes(vec![
        triangulate_circle(bottom),
        side,
        triangulate_circle(top),
    ]);
    
    if center {
        mesh.translate(Vec3::new(0.0, 0.0, -height / 2.0))
    } else {
        mesh
    }
}
```

### Transformation Application

Transformations create new Manifold instances with modified vertex positions:

```rust
impl Manifold {
    pub fn translate(&self, offset: Vec3) -> Self {
        let mut new_impl = (*self.impl_).clone();
        for v in &mut new_impl.mesh.vert_pos {
            *v += offset;
        }
        new_impl.bbox = new_impl.bbox.translate(offset);
        Manifold { impl_: Arc::new(new_impl) }
    }
    
    pub fn transform(&self, matrix: Mat3x4) -> Self {
        let mut new_impl = (*self.impl_).clone();
        for v in &mut new_impl.mesh.vert_pos {
            *v = matrix.transform_point(*v);
        }
        // Recompute bbox
        new_impl.bbox = compute_bbox(&new_impl.mesh.vert_pos);
        // Transform vertex properties (normals, tangents)
        if let Some(props) = &mut new_impl.properties {
            props.transform(matrix);
        }
        Manifold { impl_: Arc::new(new_impl) }
    }
}
```

## Parallelization Strategy

### Levels of Parallelism

1. **Operation Level**: Parallel boolean operations on independent objects
2. **Triangle Level**: Parallel processing of non-interfering triangles
3. **Batch Level**: Batch boolean operations (union of many objects)

### Using Rayon

```rust
use rayon::prelude::*;

// Parallel triangle processing
let intersections: Vec<_> = candidate_pairs
    .par_iter()
    .flat_map(|(tri_a, tri_b)| {
        compute_edge_triangle_intersections(tri_a, tri_b)
    })
    .collect();

// Parallel vertex transformation
impl Manifold {
    pub fn transform_parallel(&self, matrix: Mat3x4) -> Self {
        let mut new_impl = (*self.impl_).clone();
        new_impl.mesh.vert_pos
            .par_iter_mut()
            .for_each(|v| *v = matrix.transform_point(*v));
        // ... rest of transformation
    }
}

// Parallel batch boolean
pub fn batch_union(manifolds: &[Manifold]) -> Manifold {
    if manifolds.is_empty() {
        return Manifold::empty();
    }
    
    // Parallel divide-and-conquer
    manifolds.par_iter()
        .cloned()
        .reduce(|| Manifold::empty(), |a, b| a.union(&b))
}
```

### Performance Considerations

- Use parallel processing only when:
  - Number of items > threshold (e.g., 1000 triangles)
  - Operation cost > overhead (avoid for cheap operations)
- Maintain work-stealing for load balancing
- Profile to identify bottlenecks

## Error Handling

### Error Type Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum ManifoldError {
    #[error("Mesh is not manifold: {reason}")]
    NotManifold { reason: String },
    
    #[error("Non-finite vertex at index {index}: {point:?}")]
    NonFiniteVertex { index: usize, point: Vec3 },
    
    #[error("Vertex index {index} out of bounds (max: {max})")]
    VertexOutOfBounds { index: usize, max: usize },
    
    #[error("Invalid mesh construction: {reason}")]
    InvalidConstruction { reason: String },
    
    #[error("Result mesh too large: {num_triangles} triangles")]
    ResultTooLarge { num_triangles: usize },
    
    #[error("Boolean operation failed: {operation}")]
    BooleanFailed { operation: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ManifoldError>;
```

### Error Handling Strategy

1. **Construction Errors**: Validate mesh topology on construction
2. **Operation Errors**: Return `Result` for fallible operations
3. **Internal Invariants**: Use debug assertions, not panics
4. **Recovery**: Provide `Merge()` function to fix slightly non-manifold meshes

```rust
impl Manifold {
    /// Create manifold from mesh, returns error if not manifold
    pub fn from_mesh(mesh: MeshGL) -> Result<Self> {
        validate_manifold(&mesh)?;
        Ok(Self::from_mesh_unchecked(mesh))
    }
    
    /// Create manifold without validation (unsafe)
    pub fn from_mesh_unchecked(mesh: MeshGL) -> Self {
        // Assume mesh is valid
    }
    
    /// Attempt to fix non-manifold mesh
    pub fn merge(mesh: &mut MeshGL) -> Result<bool> {
        // Try to merge vertices along open edges
    }
}
```

## Memory Management

### Ownership Strategy

1. **Immutable Operations**: Return new `Manifold` instances
2. **Copy-on-Write**: Use `Arc` for sharing, clone only when modifying
3. **Move Semantics**: Consume input where appropriate

```rust
// Good: Immutable, returns new instance
pub fn union(&self, other: &Manifold) -> Manifold { ... }

// Good: Consumes inputs for batch operations
pub fn batch_union(manifolds: Vec<Manifold>) -> Manifold { ... }

// Good: Builder pattern for construction
pub struct ManifoldBuilder {
    vertices: Vec<Vec3>,
    triangles: Vec<[u32; 3]>,
}

impl ManifoldBuilder {
    pub fn add_vertex(&mut self, pos: Vec3) -> u32 { ... }
    pub fn add_triangle(&mut self, indices: [u32; 3]) { ... }
    pub fn build(self) -> Result<Manifold> { ... }
}
```

### Memory Pools

For performance-critical paths, consider using memory pools:

```rust
struct BooleanContext {
    intersection_pool: Vec<Intersection>,
    vertex_pool: Vec<Vec3>,
    triangle_pool: Vec<[u32; 3]>,
}

impl BooleanContext {
    fn clear(&mut self) {
        self.intersection_pool.clear();
        self.vertex_pool.clear();
        self.triangle_pool.clear();
    }
}
```

### Resource Limits

```rust
const MAX_TRIANGLES: usize = 100_000_000;  // 100M triangles
const MAX_VERTICES: usize = 50_000_000;    // 50M vertices

fn check_size_limits(num_tris: usize, num_verts: usize) -> Result<()> {
    if num_tris > MAX_TRIANGLES {
        return Err(ManifoldError::ResultTooLarge { num_triangles: num_tris });
    }
    if num_verts > MAX_VERTICES {
        return Err(ManifoldError::InvalidConstruction { 
            reason: format!("Too many vertices: {}", num_verts) 
        });
    }
    Ok(())
}
```

## Type Safety

### Phantom Types for Units

```rust
struct Vec3<Unit = WorldSpace> {
    x: f64,
    y: f64,
    z: f64,
    _phantom: PhantomData<Unit>,
}

struct WorldSpace;
struct LocalSpace;

// Can't accidentally mix coordinate spaces
fn transform(local: Vec3<LocalSpace>, matrix: Mat3x4) -> Vec3<WorldSpace> {
    // ...
}
```

### Index Types

```rust
// Newtype for type-safe indices
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertIdx(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TriIdx(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct HalfEdgeIdx(pub u32);

impl HalfEdgeIdx {
    pub fn next(self) -> Self {
        Self((self.0 / 3) * 3 + (self.0 + 1) % 3)
    }
    
    pub fn tri(self) -> TriIdx {
        TriIdx(self.0 / 3)
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cube_manifold() {
        let cube = Manifold::cube(Vec3::new(1.0, 1.0, 1.0), false);
        assert!(cube.is_manifold());
        assert_eq!(cube.num_tri(), 12);  // 6 faces * 2 triangles
        assert_eq!(cube.num_vert(), 8);
    }
    
    #[test]
    fn test_union_cubes() {
        let a = Manifold::cube(Vec3::splat(1.0), false);
        let b = a.translate(Vec3::new(0.5, 0.0, 0.0));
        let result = a.union(&b);
        
        assert!(result.is_manifold());
        assert!(result.volume() > a.volume());
        assert!(result.volume() < a.volume() * 2.0);
    }
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn union_is_commutative(
        size_a in 0.1f64..10.0,
        size_b in 0.1f64..10.0,
    ) {
        let a = Manifold::cube(Vec3::splat(size_a), false);
        let b = Manifold::cube(Vec3::splat(size_b), false);
        
        let ab = a.union(&b);
        let ba = b.union(&a);
        
        prop_assert!((ab.volume() - ba.volume()).abs() < 1e-6);
    }
}
```

## Benchmark Strategy

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_boolean_ops(c: &mut Criterion) {
    c.bench_function("union_cubes", |b| {
        let a = Manifold::cube(Vec3::splat(1.0), false);
        let b = Manifold::cube(Vec3::splat(1.0), false)
            .translate(Vec3::new(0.5, 0.0, 0.0));
        
        b.iter(|| {
            black_box(a.union(&b))
        });
    });
}

criterion_group!(benches, bench_boolean_ops);
criterion_main!(benches);
```

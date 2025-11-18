# Testing Strategy

## Overview

Comprehensive testing is critical for a geometry library where correctness and robustness are paramount.

## Testing Philosophy

Our testing approach prioritizes:
- **Test-Driven Development (TDD)**: Write tests BEFORE implementation
- **Real Implementations**: No mocks except for I/O operations
- **Correctness**: Ensuring geometric operations are mathematically correct
- **Robustness**: Handling edge cases and invalid input gracefully
- **Performance**: Identifying bottlenecks and regressions
- **Maintainability**: Tests should be clear and easy to update
- **Explicit Failures**: All errors must be explicit, no silent failures

### TDD Workflow

**Red-Green-Refactor Cycle**:
1. **Red**: Write a failing test that specifies desired behavior
2. **Green**: Write minimal code to make the test pass
3. **Refactor**: Improve code while keeping tests green
4. **Repeat**: Continue with next feature

### No Mocks Policy

**Use real implementations** for all internal components:
- Use actual `Manifold` objects in tests
- Use actual mesh structures
- Use actual boolean operations
- Do NOT mock internal components

**Mock only external I/O**:
- File system operations
- Network requests (if any)
- External service calls

## Test Categories

### 1. Unit Tests

**Location**: `src/**/*.rs` (inline with code)

**Purpose**: Test individual functions and methods in isolation.

**Examples**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vec3_is_finite() {
        assert!(vec3(1.0, 2.0, 3.0).is_finite());
        assert!(!vec3(f64::NAN, 2.0, 3.0).is_finite());
    }
    
    #[test]
    fn test_bbox_intersection() {
        let a = BoundingBox::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
        let b = BoundingBox::new(vec3(0.5, 0.5, 0.5), vec3(1.5, 1.5, 1.5));
        assert!(a.intersects(&b));
    }
}
```

**Coverage Goals**: >90% line coverage for all modules

---

### 2. Integration Tests

**Location**: `tests/*.rs`

**Purpose**: Test complete workflows and component interactions.

**Examples**:
```rust
// tests/boolean_operations.rs
use manifold_rs::*;

#[test]
fn test_cube_union() {
    let a = Manifold::cube(vec3(1.0, 1.0, 1.0), false);
    let b = Manifold::cube(vec3(1.0, 1.0, 1.0), false)
        .translate(vec3(0.5, 0.0, 0.0));
    
    let result = a.union(&b);
    
    assert!(result.is_manifold());
    assert!(result.volume() > 1.0);
    assert!(result.volume() < 2.0);
}

#[test]
fn test_sphere_difference() {
    let sphere = Manifold::sphere(2.0, 64);
    let hole = Manifold::cylinder(4.0, 0.5, 0.5, 32, true);
    
    let result = sphere.difference(&hole);
    
    assert!(result.is_manifold());
    assert!(result.volume() < sphere.volume());
}
```

---

### 3. Property-Based Tests

**Location**: `tests/property_tests.rs`

**Purpose**: Test invariants and properties that should hold for all inputs.

**Tool**: `proptest` crate

**Examples**:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn union_is_commutative(
        size_a in 0.1f64..10.0,
        size_b in 0.1f64..10.0,
    ) {
        let a = Manifold::cube(vec3(size_a, size_a, size_a), false);
        let b = Manifold::cube(vec3(size_b, size_b, size_b), false);
        
        let ab = a.clone().union(&b);
        let ba = b.union(&a);
        
        // Volumes should be equal (within tolerance)
        prop_assert!((ab.volume() - ba.volume()).abs() < 1e-6);
    }
    
    #[test]
    fn transformation_preserves_manifoldness(
        tx in -10.0f64..10.0,
        ty in -10.0f64..10.0,
        tz in -10.0f64..10.0,
    ) {
        let cube = Manifold::cube(vec3(1.0, 1.0, 1.0), false);
        let translated = cube.translate(vec3(tx, ty, tz));
        
        prop_assert!(translated.is_manifold());
        prop_assert!((translated.volume() - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn scale_affects_volume_correctly(
        sx in 0.1f64..5.0,
        sy in 0.1f64..5.0,
        sz in 0.1f64..5.0,
    ) {
        let cube = Manifold::cube(vec3(1.0, 1.0, 1.0), false);
        let scaled = cube.scale(vec3(sx, sy, sz));
        
        let expected_volume = sx * sy * sz;
        prop_assert!((scaled.volume() - expected_volume).abs() < 1e-6);
    }
}
```

**Properties to Test**:
- Boolean commutativity: A∪B = B∪A
- Boolean associativity: (A∪B)∪C = A∪(B∪C)
- Boolean idempotence: A∪A = A
- Volume conservation in unions
- Manifoldness preservation
- Bounding box containment

---

### 4. Regression Tests

**Location**: `tests/regression/*.rs`

**Purpose**: Prevent previously fixed bugs from reappearing.

**Process**:
1. When a bug is found, create a minimal reproduction
2. Add as a test (initially marked as `#[ignore]` or failing)
3. Fix the bug
4. Ensure test passes
5. Keep test in suite permanently

**Example**:
```rust
// tests/regression/issue_123.rs
/// Regression test for issue #123: Boolean fails on nearly coplanar faces
#[test]
fn test_issue_123_coplanar_boolean() {
    let a = Manifold::cube(vec3(1.0, 1.0, 1.0), false);
    let b = Manifold::cube(vec3(1.0, 1.0, 1.0), false)
        .translate(vec3(0.0, 0.0, 0.9999)); // Nearly touching
    
    let result = a.union(&b);
    assert!(result.is_manifold());
}
```

---

### 5. Performance Benchmarks

**Location**: `benches/*.rs`

**Tool**: `criterion` crate

**Purpose**: Track performance over time, identify regressions.

**Examples**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use manifold_rs::*;

fn bench_cube_creation(c: &mut Criterion) {
    c.bench_function("create_cube", |b| {
        b.iter(|| {
            black_box(Manifold::cube(vec3(1.0, 1.0, 1.0), false))
        });
    });
}

fn bench_sphere_creation(c: &mut Criterion) {
    c.bench_function("create_sphere_64", |b| {
        b.iter(|| {
            black_box(Manifold::sphere(1.0, 64))
        });
    });
}

fn bench_boolean_union(c: &mut Criterion) {
    let a = Manifold::cube(vec3(1.0, 1.0, 1.0), false);
    let b = Manifold::cube(vec3(1.0, 1.0, 1.0), false)
        .translate(vec3(0.5, 0.0, 0.0));
    
    c.bench_function("union_cubes", |b| {
        b.iter(|| {
            black_box(a.union(&b))
        });
    });
}

fn bench_boolean_union_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("boolean_union_many");
    
    for count in [10, 50, 100, 200].iter() {
        let cubes: Vec<_> = (0..*count)
            .map(|i| {
                Manifold::cube(vec3(0.5, 0.5, 0.5), false)
                    .translate(vec3(i as f64 * 0.3, 0.0, 0.0))
            })
            .collect();
        
        group.bench_with_input(
            criterion::BenchmarkId::from_parameter(count),
            count,
            |b, _| {
                b.iter(|| {
                    black_box(Manifold::batch_union(cubes.clone()))
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_cube_creation,
    bench_sphere_creation,
    bench_boolean_union,
    bench_boolean_union_complex,
);
criterion_main!(benches);
```

**Metrics to Track**:
- Primitive creation time
- Boolean operation time (vs number of triangles)
- Memory usage
- Parallel speedup

---

### 6. End-to-End Tests

**Location**:
- Core Rust pipeline: `tests/e2e/*.rs` in the relevant crates
- WASM binding: `libs/wasm/tests/` or `wasm-bindgen-test` suites
- Web playground: manual / visual tests plus optional automated checks

**Purpose**: Test the complete OpenSCAD → CST → AST → geometry IR → Manifold/MeshGL → WASM → web viewer
pipeline.

#### 6.1 Core Rust Pipeline (openscad-parser → openscad-ast → openscad-eval → manifold-rs)

```rust
// tests/e2e/openscad_to_mesh.rs
use openscad_parser::parse_to_cst;
use openscad_ast::from_cst;
use openscad_eval::evaluate_ast;
use manifold_rs::{manifold_from_ir, MeshGL};

#[test]
fn test_simple_cube_pipeline() {
    let source = "cube([2, 3, 4]);";

    let cst = parse_to_cst(source).unwrap();
    let ast = from_cst(&cst).unwrap();
    let ir  = evaluate_ast(&ast).unwrap();
    let manifold = manifold_from_ir(&ir).unwrap();
    let mesh: MeshGL = manifold.to_meshgl();

    assert!(manifold.is_manifold());
    assert_eq!(manifold.volume(), 24.0);
    assert_eq!(mesh.tri_verts.len(), 12 * 3);
}
```

Alternatively, tests can use a high-level helper in `libs/manifold-rs`:

```rust
#[test]
fn test_simple_cube_helper() {
    let mesh = manifold_rs::parse_and_evaluate_openscad("cube([2, 3, 4]);").unwrap();
    // Assert on vertex/triangle counts, volume, etc.
}
```

#### 6.2 WASM API (libs/wasm)

```rust
// libs/wasm/tests/web_api.rs (conceptual)
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_parse_openscad_to_mesh_js_api() {
    // Call the exported WASM function and verify it returns a mesh object.
    let mesh_js = crate::parse_openscad_to_mesh("cube([1,1,1]);");

    // Inspect fields via JS bindings / serde and ensure triangle/vertex counts are correct.
}
```

#### 6.3 Web Playground (Svelte + Three.js)

- Load the `libs/wasm` bundle and call `parse_openscad_to_mesh` from the Svelte app when the user
  edits OpenSCAD code.
- Render the resulting mesh with Three.js in a viewport that fills **100% of the browser window**
  (both width and height).
- Use manual and (optional) visual regression tests to ensure:
  - The scene renders without errors.
  - Camera controls work as expected.
  - The viewport resizes correctly with the browser window.

---

### 7. Visual Regression Tests (Optional)

**Purpose**: Detect visual changes in rendered output.

**Process**:
1. Generate reference images for test models
2. Re-render after changes
3. Compare images pixel-by-pixel
4. Flag differences for review

**Tools**: Consider using image comparison libraries

---

## Test Data

### Golden Test Files

Create reference test files in `tests/data/`:
- `tests/data/primitives/*.scad` - Simple primitives
- `tests/data/booleans/*.scad` - Boolean operations
- `tests/data/transforms/*.scad` - Transformations
- `tests/data/complex/*.scad` - Real-world models

### Expected Outputs

Store expected mesh data:
- Triangle counts
- Vertex counts
- Volumes
- Surface areas
- Bounding boxes

---

## Continuous Integration

### GitHub Actions Workflow

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          override: true
      
      - name: Run tests
        run: cargo test --all-features --verbose
      
      - name: Run property tests
        run: cargo test --test property_tests -- --ignored
      
      - name: Run benchmarks
        run: cargo bench --no-run
  
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Generate coverage
        run: cargo tarpaulin --out Xml --all-features
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

## Test Execution Strategy

### Development Workflow

1. **Pre-commit**: Run fast unit tests
   ```bash
   cargo test --lib
   ```

2. **Pre-push**: Run all tests
   ```bash
   cargo test --all-features
   ```

3. **CI**: Run full suite including property tests
   ```bash
   cargo test --all-features
   cargo test --test property_tests -- --ignored
   cargo bench
   ```

### Test Organization

```
manifold-rs/
├── src/
│   ├── lib.rs           # Unit tests inline
│   ├── manifold.rs      # Unit tests inline
│   └── ...
├── tests/
│   ├── integration_tests.rs
│   ├── property_tests.rs
│   ├── regression/
│   │   ├── issue_001.rs
│   │   └── ...
│   ├── e2e/
│   │   └── openscad_examples.rs
│   └── data/
│       ├── primitives/
│       ├── booleans/
│       └── complex/
└── benches/
    └── benchmarks.rs
```

---

## Coverage Goals

| Component | Unit Test Coverage | Integration Tests | Property Tests |
|-----------|-------------------|-------------------|----------------|
| Vec3/BBox | 100% | N/A | Yes |
| MeshGL | 100% | Yes | No |
| HalfEdgeMesh | 95%+ | Yes | No |
| Primitives | 100% | Yes | Yes |
| Transformations | 100% | Yes | Yes |
| Booleans | 90%+ | Yes | Yes |
| Extrusion | 90%+ | Yes | No |
| Evaluator | 85%+ | Yes | No |

---

## Test Success Criteria

A test suite is considered successful when:

- [ ] All unit tests pass on all platforms
- [ ] Integration tests cover all major workflows
- [ ] Property tests verify key invariants
- [ ] Regression tests prevent known bugs
- [ ] Benchmarks track performance trends
- [ ] Code coverage exceeds 80% overall
- [ ] CI pipeline passes consistently
- [ ] Test execution time is reasonable (<5 min for quick tests)

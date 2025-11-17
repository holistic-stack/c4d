# Coding Standards and Development Principles

## Overview

This document defines coding standards, architectural principles, and best practices for the Manifold Rust port project. These standards ensure maintainability, testability, and code quality.

---

## Core Development Principles

### 1. Test-Driven Development (TDD)

**Principle**: Write tests before implementation

**Practice**:
```rust
// Step 1: Write the test first (it will fail)
#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that Vec3 addition works correctly
    /// 
    /// # Examples
    /// ```
    /// let a = Vec3::new(1.0, 2.0, 3.0);
    /// let b = Vec3::new(4.0, 5.0, 6.0);
    /// assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
    /// ```
    #[test]
    fn test_vec3_addition() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        let result = a + b;
        
        assert_eq!(result, Vec3::new(5.0, 7.0, 9.0));
    }
}

// Step 2: Implement minimal code to pass
// Step 3: Refactor while keeping tests green
```

**Rules**:
- ✅ Write test first
- ✅ Make it pass with minimal code
- ✅ Refactor
- ✅ Repeat

### 2. No Mocks (Except I/O)

**Principle**: Use real implementations, mock only external I/O

**Good** ✅:
```rust
#[test]
fn test_cube_volume() {
    // Use real Manifold, not a mock
    let cube = Manifold::cube(2.0, 2.0, 2.0, false);
    assert_eq!(cube.volume(), 8.0);
}
```

**Bad** ❌:
```rust
#[test]
fn test_boolean_union() {
    // DON'T mock Manifold
    let mock_manifold = MockManifold::new();
    // ...
}
```

**Acceptable** ✅ (I/O operations):
```rust
#[test]
fn test_stl_import() {
    // Mock file system is OK
    let mock_fs = MockFileSystem::new();
    mock_fs.add_file("test.stl", STL_CONTENT);
    // ...
}
```

### 3. Single Responsibility Principle (SRP)

**Principle**: Each module/function has ONE reason to change

**File Organization**:
```
libs/manifold-rs/src/
├── primitives/
│   ├── cube/
│   │   ├── mod.rs              # Cube implementation
│   │   └── tests.rs            # Cube tests
│   ├── sphere/
│   │   ├── mod.rs              # Sphere implementation
│   │   └── tests.rs            # Sphere tests
│   └── mod.rs                  # Re-exports
├── transforms/
│   ├── translate/
│   │   ├── mod.rs
│   │   └── tests.rs
│   └── mod.rs
└── lib.rs
```

**Function-Level SRP**:
```rust
// ✅ GOOD: One responsibility
/// Calculates the volume of a manifold mesh.
///
/// # Arguments
/// * `mesh` - The mesh to calculate volume for
///
/// # Returns
/// The signed volume of the mesh
///
/// # Examples
/// ```
/// let cube = create_cube_mesh(2.0);
/// assert_eq!(calculate_volume(&cube), 8.0);
/// ```
fn calculate_volume(mesh: &HalfEdgeMesh) -> f64 {
    // Single responsibility: volume calculation
    mesh.triangles()
        .map(|tri| triangle_signed_volume(tri))
        .sum()
}

// ❌ BAD: Multiple responsibilities
fn process_mesh(mesh: &HalfEdgeMesh) -> (f64, BoundingBox, Vec<Vec3>) {
    // Too many things: volume, bbox, normals
    // Should be 3 separate functions
}
```

### 4. File Size Limit: 500 Lines

**Principle**: Keep files manageable and focused

**Rules**:
- Maximum 500 lines per file (including tests if inline)
- If over 500 lines, split into smaller modules
- Tests can be in separate files for clarity

**Example Split**:
```rust
// Before: primitives.rs (800 lines) ❌
// After: Split into modules ✅
primitives/
├── mod.rs          (50 lines - re-exports)
├── cube.rs         (150 lines)
├── sphere.rs       (200 lines)
├── cylinder.rs     (180 lines)
└── polyhedron.rs   (220 lines)
```

### 5. DRY (Don't Repeat Yourself)

**Principle**: Avoid code duplication

**Bad** ❌:
```rust
pub fn translate_x(manifold: &Manifold, dx: f64) -> Manifold {
    let matrix = Mat4::from_translation(Vec3::new(dx, 0.0, 0.0));
    manifold.transform(&matrix)
}

pub fn translate_y(manifold: &Manifold, dy: f64) -> Manifold {
    let matrix = Mat4::from_translation(Vec3::new(0.0, dy, 0.0));
    manifold.transform(&matrix)
}

pub fn translate_z(manifold: &Manifold, dz: f64) -> Manifold {
    let matrix = Mat4::from_translation(Vec3::new(0.0, 0.0, dz));
    manifold.transform(&matrix)
}
```

**Good** ✅:
```rust
/// Translates a manifold by the given offset vector.
///
/// # Arguments
/// * `manifold` - The manifold to translate
/// * `offset` - The translation vector (x, y, z)
///
/// # Returns
/// A new translated manifold
///
/// # Examples
/// ```
/// let cube = Manifold::cube(1.0, 1.0, 1.0, false);
/// let translated = translate(&cube, Vec3::new(5.0, 0.0, 0.0));
/// ```
pub fn translate(manifold: &Manifold, offset: Vec3) -> Manifold {
    let matrix = Mat4::from_translation(offset);
    manifold.transform(&matrix)
}
```

### 6. KISS (Keep It Simple, Stupid)

**Principle**: Favor simple, readable solutions over clever ones

**Bad** ❌:
```rust
// Clever but unreadable
fn f(v: &[f64]) -> f64 {
    v.iter().fold(0.0, |a, &b| a + b) / v.len() as f64
}
```

**Good** ✅:
```rust
/// Calculates the average of a slice of values.
///
/// # Arguments
/// * `values` - Non-empty slice of values
///
/// # Returns
/// The arithmetic mean
///
/// # Panics
/// Panics if the slice is empty
///
/// # Examples
/// ```
/// let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// assert_eq!(calculate_average(&values), 3.0);
/// ```
fn calculate_average(values: &[f64]) -> f64 {
    let sum: f64 = values.iter().sum();
    let count = values.len() as f64;
    sum / count
}
```

### 7. Explicit Error Handling (No Silent Failures)

**Principle**: All errors must be explicit, no fallbacks

**Bad** ❌:
```rust
fn load_mesh(path: &Path) -> Option<Manifold> {
    match read_stl(path) {
        Ok(mesh) => Some(mesh),
        Err(_) => None, // Silent failure! ❌
    }
}
```

**Good** ✅:
```rust
/// Loads a mesh from an STL file.
///
/// # Arguments
/// * `path` - Path to the STL file
///
/// # Returns
/// The loaded manifold mesh
///
/// # Errors
/// Returns `ManifoldError::IoError` if file cannot be read
/// Returns `ManifoldError::InvalidMesh` if STL is malformed
///
/// # Examples
/// ```
/// let mesh = load_mesh(Path::new("cube.stl"))?;
/// ```
pub fn load_mesh(path: &Path) -> Result<Manifold, ManifoldError> {
    let stl_data = read_stl(path)
        .map_err(|e| ManifoldError::IoError {
            path: path.to_owned(),
            source: e,
        })?;
    
    validate_mesh(&stl_data)?;
    
    Ok(Manifold::from_mesh(stl_data))
}
```

### 8. Comprehensive Documentation

**Principle**: All public items must be documented with examples

**Requirements**:
- ✅ Doc comment for every public function/struct/trait
- ✅ Describe purpose
- ✅ Document parameters
- ✅ Document return values
- ✅ Document errors
- ✅ Provide examples
- ✅ Add inline comments for complex logic

**Example**:
```rust
/// Represents a 3D manifold mesh with guaranteed topological correctness.
///
/// A manifold is a mesh where every edge is shared by exactly two triangles,
/// ensuring it represents a valid solid object.
///
/// # Examples
/// ```
/// use manifold_rs::Manifold;
///
/// // Create a simple cube
/// let cube = Manifold::cube(2.0, 2.0, 2.0, false);
/// assert_eq!(cube.volume(), 8.0);
///
/// // Boolean union
/// let sphere = Manifold::sphere(1.5, 32);
/// let result = cube.union(&sphere);
/// ```
pub struct Manifold {
    /// Internal mesh representation using half-edge structure
    mesh: HalfEdgeMesh,
    
    /// Axis-aligned bounding box for spatial queries
    bbox: BoundingBox,
}

impl Manifold {
    /// Creates a new axis-aligned box (cube/cuboid).
    ///
    /// # Arguments
    /// * `width` - Size in X dimension (must be positive)
    /// * `depth` - Size in Y dimension (must be positive)
    /// * `height` - Size in Z dimension (must be positive)
    /// * `center` - If true, center at origin; if false, place at positive octant
    ///
    /// # Returns
    /// A new manifold representing the box
    ///
    /// # Panics
    /// Panics if any dimension is not positive
    ///
    /// # Examples
    /// ```
    /// // Create centered 2x2x2 cube
    /// let cube = Manifold::cube(2.0, 2.0, 2.0, true);
    /// assert_eq!(cube.bounding_box().center(), Vec3::ZERO);
    ///
    /// // Create corner-aligned 1x2x3 box
    /// let box = Manifold::cube(1.0, 2.0, 3.0, false);
    /// ```
    pub fn cube(width: f64, depth: f64, height: f64, center: bool) -> Self {
        // Validate inputs
        assert!(width > 0.0, "Width must be positive");
        assert!(depth > 0.0, "Depth must be positive");
        assert!(height > 0.0, "Height must be positive");
        
        // Calculate vertex positions based on center flag
        let (x_min, y_min, z_min) = if center {
            (-width / 2.0, -depth / 2.0, -height / 2.0)
        } else {
            (0.0, 0.0, 0.0)
        };
        
        // ... implementation
    }
}
```

---

## Rust-Specific Standards

### 1. Naming Conventions

**File Names**: `snake_case`
```
✅ half_edge_mesh.rs
✅ boolean_operations.rs
❌ HalfEdgeMesh.rs
❌ BooleanOperations.rs
```

**Module Names**: `snake_case`
```rust
mod half_edge_mesh;
mod boolean_operations;
```

**Type Names**: `PascalCase`
```rust
struct Manifold { }
enum ManifoldError { }
trait MeshOperations { }
```

**Function/Variable Names**: `snake_case`
```rust
fn calculate_volume() -> f64 { }
let mesh_count = 5;
```

**Constants**: `SCREAMING_SNAKE_CASE`
```rust
const DEFAULT_SEGMENTS: usize = 32;
const EPSILON: f64 = 1e-10;
```

### 2. Centralized Configuration

**Location**: `libs/manifold-rs/src/config.rs`

```rust
/// Global configuration constants for the Manifold library.
///
/// These values control default behavior and numerical precision
/// throughout the library.

/// Default number of segments for circular primitives (sphere, cylinder)
///
/// Higher values produce smoother meshes but increase polygon count.
/// Must be at least 3.
pub const DEFAULT_SEGMENTS: usize = 32;

/// Numerical tolerance for floating-point comparisons
///
/// Used for checking mesh validity and geometric predicates.
/// Two values are considered equal if their difference is less than EPSILON.
pub const EPSILON: f64 = 1e-10;

/// Minimum edge length for mesh validity
///
/// Edges shorter than this are considered degenerate and may cause
/// numerical instability in boolean operations.
pub const MIN_EDGE_LENGTH: f64 = 1e-8;

/// Maximum number of boolean operation iterations
///
/// Prevents infinite loops in edge cases. If exceeded, operation fails
/// with an error.
pub const MAX_BOOLEAN_ITERATIONS: usize = 10000;

/// Default R-tree node capacity for spatial indexing
///
/// Higher values use more memory but may improve query performance
/// for large meshes.
pub const RTREE_NODE_CAPACITY: usize = 16;
```

### 3. Error Handling

**Use `thiserror` for error types**:
```rust
use thiserror::Error;

/// Errors that can occur during manifold operations.
///
/// All operations that can fail return `Result<T, ManifoldError>`.
/// No silent failures are allowed.
#[derive(Error, Debug)]
pub enum ManifoldError {
    /// Mesh is not manifold (has boundary edges or non-manifold vertices)
    #[error("Mesh is not manifold: {reason}")]
    NonManifoldMesh {
        reason: String,
    },
    
    /// Boolean operation failed due to numerical issues
    #[error("Boolean operation failed: {operation} - {reason}")]
    BooleanOperationFailed {
        operation: String,
        reason: String,
    },
    
    /// File I/O error
    #[error("IO error reading {path}: {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    /// Invalid mesh data (degenerate triangles, etc.)
    #[error("Invalid mesh: {reason}")]
    InvalidMesh {
        reason: String,
    },
}

/// Result type alias for manifold operations
pub type Result<T> = std::result::Result<T, ManifoldError>;
```

### 4. Type Safety (No `Any` equivalent)

**Rules**:
- ✅ Use concrete types
- ✅ Use generics when appropriate
- ✅ Use enums for variants
- ❌ Never use `Box<dyn Any>`

**Good** ✅:
```rust
/// Geometry can be either 2D or 3D
pub enum Geometry {
    /// 2D polygon cross-section
    CrossSection(CrossSection),
    /// 3D manifold mesh
    Manifold(Manifold),
}
```

**Bad** ❌:
```rust
// Don't do this
pub fn process(data: Box<dyn Any>) { }
```

---

## Project Structure Standards

### Directory Layout

```
libs/manifold-rs/
├── src/
│   ├── lib.rs                          # Public API exports
│   ├── config.rs                       # Centralized constants
│   ├── error.rs                        # Error types
│   │
│   ├── core/
│   │   ├── mod.rs                      # Core types re-exports
│   │   ├── vec3/
│   │   │   ├── mod.rs                  # Vec3 implementation (<500 lines)
│   │   │   └── tests.rs                # Vec3 tests
│   │   ├── bounding_box/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   ├── mesh_gl/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   └── half_edge_mesh/
│   │       ├── mod.rs
│   │       ├── builder.rs              # Mesh construction
│   │       ├── validation.rs           # Mesh validation
│   │       └── tests.rs
│   │
│   ├── primitives/
│   │   ├── mod.rs
│   │   ├── cube/
│   │   │   ├── mod.rs                  # Cube implementation
│   │   │   └── tests.rs                # Cube tests
│   │   ├── sphere/
│   │   │   ├── mod.rs                  # Sphere via icosphere
│   │   │   ├── icosphere.rs            # Subdivision algorithm
│   │   │   └── tests.rs
│   │   ├── cylinder/
│   │   │   ├── mod.rs
│   │   │   └── tests.rs
│   │   └── polyhedron/
│   │       ├── mod.rs
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
│   │   └── ... (other transforms)
│   │
│   ├── boolean/
│   │   ├── mod.rs
│   │   ├── collision/
│   │   │   ├── mod.rs                  # Broad-phase collision
│   │   │   └── tests.rs
│   │   ├── intersection/
│   │   │   ├── mod.rs                  # Edge-triangle intersection
│   │   │   └── tests.rs
│   │   ├── topology/
│   │   │   ├── mod.rs                  # Topology construction
│   │   │   └── tests.rs
│   │   └── operations/
│   │       ├── mod.rs
│   │       ├── union.rs
│   │       ├── difference.rs
│   │       ├── intersection.rs
│   │       └── tests.rs
│   │
│   ├── cross_section/
│   │   ├── mod.rs                      # 2D polygon operations
│   │   └── tests.rs
│   │
│   ├── io/
│   │   ├── mod.rs
│   │   ├── stl/
│   │   │   ├── mod.rs
│   │   │   ├── reader.rs               # STL reading
│   │   │   ├── writer.rs               # STL writing
│   │   │   └── tests.rs
│   │   └── ... (other formats)
│   │
│   └── utils/
│       ├── mod.rs
│       ├── math/
│       │   ├── mod.rs
│       │   └── tests.rs
│       └── geometry/
│           ├── mod.rs
│           └── tests.rs
│
├── tests/                              # Integration tests
│   ├── primitives_test.rs
│   ├── boolean_operations_test.rs
│   └── end_to_end_test.rs
│
├── benches/                            # Benchmarks
│   └── benchmarks.rs
│
└── examples/                           # Example usage
    ├── basic_primitives.rs
    ├── boolean_operations.rs
    └── openscad_pipeline.rs
```

### Module Organization Rules

1. **Each logical component in its own folder**
2. **Tests alongside implementation** (`mod.rs` + `tests.rs`)
3. **Large modules split into sub-modules** (keep under 500 lines each)
4. **Clear hierarchy**: core → primitives → boolean → integration

---

## Testing Standards

### Test Organization

```rust
// In src/primitives/cube/mod.rs
/// Implementation
pub fn cube(width: f64, depth: f64, height: f64, center: bool) -> Manifold {
    // ... implementation
}

// In src/primitives/cube/tests.rs
use super::*;
use crate::config::EPSILON;
use approx::assert_abs_diff_eq;

/// Tests that a centered cube has the correct volume.
///
/// # Test Case
/// - 2x2x2 cube should have volume of 8.0
#[test]
fn test_cube_volume() {
    let cube = cube(2.0, 2.0, 2.0, true);
    assert_abs_diff_eq!(cube.volume(), 8.0, epsilon = EPSILON);
}

/// Tests that a centered cube is centered at origin.
///
/// # Test Case
/// - Centered cube's bounding box center should be at (0,0,0)
#[test]
fn test_cube_centered() {
    let cube = cube(2.0, 2.0, 2.0, true);
    let center = cube.bounding_box().center();
    
    assert_abs_diff_eq!(center.x, 0.0, epsilon = EPSILON);
    assert_abs_diff_eq!(center.y, 0.0, epsilon = EPSILON);
    assert_abs_diff_eq!(center.z, 0.0, epsilon = EPSILON);
}

/// Tests that non-centered cube is positioned correctly.
///
/// # Test Case
/// - Non-centered 2x2x2 cube should have min corner at origin
#[test]
fn test_cube_not_centered() {
    let cube = cube(2.0, 2.0, 2.0, false);
    let bbox = cube.bounding_box();
    
    assert_abs_diff_eq!(bbox.min.x, 0.0, epsilon = EPSILON);
    assert_abs_diff_eq!(bbox.min.y, 0.0, epsilon = EPSILON);
    assert_abs_diff_eq!(bbox.min.z, 0.0, epsilon = EPSILON);
}

/// Tests that cube panics with invalid dimensions.
///
/// # Test Case
/// - Zero width should panic
#[test]
#[should_panic(expected = "Width must be positive")]
fn test_cube_invalid_width() {
    cube(0.0, 2.0, 2.0, false);
}
```

### Test Coverage Requirements

- ✅ **Unit tests**: Every function
- ✅ **Integration tests**: End-to-end workflows
- ✅ **Property tests**: Invariants (using `proptest`)
- ✅ **Boundary tests**: Edge cases and limits
- ✅ **Error tests**: All error paths
- ✅ **Performance tests**: Benchmarks for critical paths

**Target**: 80%+ code coverage

---

## Code Review Checklist

Before committing, verify:

- [ ] All functions under 50 lines
- [ ] All files under 500 lines
- [ ] All public items documented with examples
- [ ] All functions have tests
- [ ] Tests pass (`cargo test`)
- [ ] No compiler warnings (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Error handling is explicit (no silent failures)
- [ ] No code duplication (DRY)
- [ ] Single responsibility per module/function
- [ ] Constants centralized in `config.rs`
- [ ] Meaningful, self-explanatory names
- [ ] Comments explain WHY, not WHAT

---

## Incremental Development Workflow

### 1. Start with Test

```rust
#[test]
fn test_sphere_volume() {
    let sphere = Manifold::sphere(1.0, 32);
    let expected_volume = 4.0 / 3.0 * PI;
    assert_abs_diff_eq!(sphere.volume(), expected_volume, epsilon = 0.01);
}
```

### 2. Implement Minimally

```rust
pub fn sphere(radius: f64, segments: usize) -> Manifold {
    // Minimal implementation to pass test
    let mesh = create_icosphere(radius, 2); // subdivision level 2
    Manifold::from_mesh(mesh)
}
```

### 3. Refactor

```rust
/// Creates a sphere using icosphere subdivision.
///
/// # Arguments
/// * `radius` - Radius of the sphere (must be positive)
/// * `segments` - Number of segments (determines subdivision level)
///
/// # Returns
/// A new manifold representing the sphere
///
/// # Examples
/// ```
/// let sphere = Manifold::sphere(1.0, 32);
/// ```
pub fn sphere(radius: f64, segments: usize) -> Manifold {
    assert!(radius > 0.0, "Radius must be positive");
    
    let subdivision_level = calculate_subdivision_level(segments);
    let mesh = create_icosphere(radius, subdivision_level);
    
    Manifold::from_mesh(mesh)
}
```

### 4. Add More Tests

```rust
#[test]
fn test_sphere_is_centered() {
    let sphere = Manifold::sphere(1.0, 32);
    let center = sphere.bounding_box().center();
    assert_abs_diff_eq!(center, Vec3::ZERO, epsilon = EPSILON);
}
```

---

## Documentation Maintenance

### Keep Docs Current

**After each phase**:
1. Update relevant task documents
2. Mark completed tasks with ✅
3. Remove obsolete information
4. Update timeline estimates
5. Document any deviations from plan

### Clean Up Old Content

**Regular cleanup**:
- Remove outdated TODOs
- Archive superseded designs
- Consolidate similar sections
- Keep only relevant context

---

## Summary Checklist

### Code Quality
- [ ] TDD: Tests first, then implementation
- [ ] No mocks (except I/O)
- [ ] SRP: One responsibility per module/function
- [ ] Files under 500 lines
- [ ] DRY: No duplication
- [ ] KISS: Simple over clever
- [ ] Explicit errors (no silent failures)

### Documentation
- [ ] All public items documented
- [ ] Examples provided
- [ ] Comments explain WHY
- [ ] Docs updated with changes
- [ ] Old content cleaned up

### Testing
- [ ] Unit tests for all functions
- [ ] Integration tests for workflows
- [ ] Property tests for invariants
- [ ] 80%+ coverage

### Organization
- [ ] Modules in own folders with tests
- [ ] Constants in `config.rs`
- [ ] snake_case filenames
- [ ] Clear naming
- [ ] Logical hierarchy

---

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)
- [SOLID Principles](https://en.wikipedia.org/wiki/SOLID)
- [Test-Driven Development](https://martinfowler.com/bliki/TestDrivenDevelopment.html)

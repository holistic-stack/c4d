# Phase 1: Project Setup and Core Infrastructure

## Overview

Phase 1 establishes the foundation for the Manifold Rust port with core data structures, error handling, and project infrastructure.

**Duration**: 2-3 weeks  
**Dependencies**: None

---

## Task 1.1: Initialize Manifold Rust Crate

**Description**: Set up the basic Rust project structure for the Manifold library.

**Why**: Foundation for all development. Proper setup ensures smooth workflow and CI/CD integration.

**Subtasks**:

1. **Create library crate**
   - Command: `cd libs && cargo new --lib manifold-rs`
   - Initialize Git if needed
   - Add README.md with project overview

2. **Configure Cargo.toml**
   ```toml
   [package]
   name = "manifold-rs"
   version = "0.1.0"
   edition = "2021"
   authors = ["Your Name <email@example.com>"]
   description = "Rust port of the Manifold 3D geometry kernel"
   license = "Apache-2.0"
   repository = "https://github.com/username/rust-openscad"
   
   [dependencies]
   nalgebra = "0.32"
   rayon = { version = "1.8", optional = true }
   thiserror = "1.0"
   serde = { version = "1.0", features = ["derive"], optional = true }
   
   [dev-dependencies]
   criterion = "0.5"
   proptest = "1.4"
   approx = "0.5"
   
   [features]
   default = ["parallel"]
   parallel = ["rayon"]
   serde = ["dep:serde", "nalgebra/serde-serialize"]
   
   [[bench]]
   name = "benchmarks"
   harness = false
   ```

3. **Create module structure**
   ```
   src/
   ├── lib.rs              # Public API exports
   ├── error.rs            # Error types
   ├── vec.rs              # Vector/matrix utilities
   ├── bbox.rs             # Bounding box
   ├── mesh.rs             # MeshGL
   ├── halfedge.rs         # HalfEdgeMesh
   ├── manifold.rs         # Manifold type
   ├── primitives/mod.rs   # Geometric primitives
   ├── boolean/mod.rs      # Boolean operations
   ├── transforms/mod.rs   # Transformations
   └── utils/mod.rs        # Utilities
   ```

4. **Set up CI/CD (.github/workflows/ci.yml)**
   ```yaml
   name: CI
   on: [push, pull_request]
   jobs:
     test:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - uses: actions-rs/toolchain@v1
           with:
             toolchain: stable
         - run: cargo test --all-features
         - run: cargo clippy -- -D warnings
         - run: cargo fmt -- --check
         - run: cargo doc --no-deps
   ```

**Acceptance Criteria**:
- ✅ Crate compiles without errors
- ✅ All modules declared and stubbed
- ✅ `cargo test` runs (even with no tests)
- ✅ `cargo clippy` passes with no warnings
- ✅ `cargo fmt --check` passes
- ✅ `cargo doc` generates docs
- ✅ CI pipeline configured and passing

**Effort**: 4-6 hours

---

## Task 1.2: Core Data Structures - Vec3 and BoundingBox

**Description**: Implement fundamental vector and bounding box types.

**Why**: These are used throughout the codebase for positions, directions, and spatial queries.

**Subtasks**:

1. **Implement Vec3 utilities (src/vec.rs)**
   ```rust
   pub use nalgebra::{Vector2, Vector3, Matrix3x4};
   
   pub type Vec3 = Vector3<f64>;
   pub type Vec2 = Vector2<f64>;
   pub type Mat3x4 = Matrix3x4<f64>;
   
   pub fn vec3(x: f64, y: f64, z: f64) -> Vec3 {
       Vec3::new(x, y, z)
   }
   
   pub fn vec2(x: f64, y: f64) -> Vec2 {
       Vec2::new(x, y)
   }
   
   pub trait Vec3Ext {
       fn is_finite(&self) -> bool;
       fn approx_eq(&self, other: &Vec3, epsilon: f64) -> bool;
   }
   
   impl Vec3Ext for Vec3 {
       fn is_finite(&self) -> bool {
           self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
       }
       
       fn approx_eq(&self, other: &Vec3, epsilon: f64) -> bool {
           (self - other).norm() < epsilon
       }
   }
   ```

2. **Implement BoundingBox (src/bbox.rs)**
   ```rust
   use crate::vec::*;
   
   #[derive(Clone, Debug, PartialEq)]
   pub struct BoundingBox {
       pub min: Vec3,
       pub max: Vec3,
   }
   
   impl BoundingBox {
       pub fn new(min: Vec3, max: Vec3) -> Self {
           Self { min, max }
       }
       
       pub fn from_points(points: &[Vec3]) -> Self {
           let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
           let mut max = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
           
           for p in points {
               min.x = min.x.min(p.x);
               min.y = min.y.min(p.y);
               min.z = min.z.min(p.z);
               max.x = max.x.max(p.x);
               max.y = max.y.max(p.y);
               max.z = max.z.max(p.z);
           }
           
           Self { min, max }
       }
       
       pub fn contains(&self, point: Vec3) -> bool {
           point.x >= self.min.x && point.x <= self.max.x &&
           point.y >= self.min.y && point.y <= self.max.y &&
           point.z >= self.min.z && point.z <= self.max.z
       }
       
       pub fn intersects(&self, other: &BoundingBox) -> bool {
           self.min.x <= other.max.x && self.max.x >= other.min.x &&
           self.min.y <= other.max.y && self.max.y >= other.min.y &&
           self.min.z <= other.max.z && self.max.z >= other.min.z
       }
       
       pub fn union(&self, other: &BoundingBox) -> Self {
           Self {
               min: vec3(
                   self.min.x.min(other.min.x),
                   self.min.y.min(other.min.y),
                   self.min.z.min(other.min.z),
               ),
               max: vec3(
                   self.max.x.max(other.max.x),
                   self.max.y.max(other.max.y),
                   self.max.z.max(other.max.z),
               ),
           }
       }
       
       pub fn center(&self) -> Vec3 {
           (self.min + self.max) / 2.0
       }
       
       pub fn size(&self) -> Vec3 {
           self.max - self.min
       }
       
       pub fn volume(&self) -> f64 {
           let s = self.size();
           s.x * s.y * s.z
       }
       
       pub fn translate(&self, offset: Vec3) -> Self {
           Self {
               min: self.min + offset,
               max: self.max + offset,
           }
       }
       
       pub fn transform(&self, matrix: &Mat3x4) -> Self {
           // Transform all 8 corners and find new bbox
           let corners = [
               self.min,
               vec3(self.max.x, self.min.y, self.min.z),
               vec3(self.max.x, self.max.y, self.min.z),
               vec3(self.min.x, self.max.y, self.min.z),
               vec3(self.min.x, self.min.y, self.max.z),
               vec3(self.max.x, self.min.y, self.max.z),
               self.max,
               vec3(self.min.x, self.max.y, self.max.z),
           ];
           
           let transformed: Vec<Vec3> = corners.iter()
               .map(|&p| matrix.transform_point(&p.into()).into())
               .collect();
           
           Self::from_points(&transformed)
       }
   }
   
   // Tests
   #[cfg(test)]
   mod tests {
       use super::*;
       use approx::assert_relative_eq;
       
       #[test]
       fn bbox_from_points() {
           let points = vec![
               vec3(0.0, 0.0, 0.0),
               vec3(1.0, 2.0, 3.0),
               vec3(-1.0, 1.0, 2.0),
           ];
           let bbox = BoundingBox::from_points(&points);
           assert_eq!(bbox.min, vec3(-1.0, 0.0, 0.0));
           assert_eq!(bbox.max, vec3(1.0, 2.0, 3.0));
       }
       
       #[test]
       fn bbox_contains() {
           let bbox = BoundingBox::new(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0));
           assert!(bbox.contains(vec3(0.5, 0.5, 0.5)));
           assert!(!bbox.contains(vec3(1.5, 0.5, 0.5)));
       }
       
       #[test]
       fn bbox_intersection() {
           let a = BoundingBox::new(vec3(0.0, 0.0, 0.0), vec3(2.0, 2.0, 2.0));
           let b = BoundingBox::new(vec3(1.0, 1.0, 1.0), vec3(3.0, 3.0, 3.0));
           assert!(a.intersects(&b));
           
           let c = BoundingBox::new(vec3(3.0, 3.0, 3.0), vec3(4.0, 4.0, 4.0));
           assert!(!a.intersects(&c));
       }
   }
   ```

**Acceptance Criteria**:
- ✅ Vec3/Vec2 types work correctly
- ✅ Vec3Ext trait methods work
- ✅ BoundingBox operations are correct
- ✅ All tests pass
- ✅ Documentation complete

**Effort**: 4-6 hours

---

## Task 1.3: Mesh Data Structures

**Description**: Implement MeshGL (public) and HalfEdgeMesh (internal) representations.

**Why**: Core data structures that represent 3D geometry throughout the library.

**Context**: 
- **MeshGL**: Public-facing, GPU-compatible format
- **HalfEdgeMesh**: Internal format optimized for topology queries

**Subtasks**: See detailed spec in 04-TASKS-PHASE1-DETAILED.md

**Acceptance Criteria**:
- ✅ MeshGL can be constructed and validated
- ✅ HalfEdgeMesh can be built from MeshGL
- ✅ Conversions between formats work
- ✅ Manifold validation works
- ✅ All tests pass

**Effort**: 12-16 hours

---

## Task 1.4: Manifold Type

**Description**: Implement the main Manifold type that wraps the internal representation.

**Why**: This is the primary public API type users interact with.

**Acceptance Criteria**:
- ✅ Manifold can be created from MeshGL
- ✅ Query methods work (num_vert, num_tri, bbox, etc.)
- ✅ Arc-based cloning is cheap
- ✅ Tests cover basic functionality

**Effort**: 6-8 hours

---

## Task 1.5: Error Handling

**Description**: Implement comprehensive error types and validation.

**Why**: Good error handling is crucial for debugging and user experience.

**Acceptance Criteria**:
- ✅ All error types defined with thiserror
- ✅ Error messages are clear and actionable
- ✅ Validation helpers exist
- ✅ Tests cover all error types

**Effort**: 4-6 hours

---

## Phase 1 Complete When:

- [ ] All tasks in Phase 1 completed
- [ ] All tests passing
- [ ] Documentation complete
- [ ] CI/CD passing
- [ ] Code review approved

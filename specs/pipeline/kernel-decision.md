# Geometry Kernel Strategy Decision

**Date**: 2025-11-18  
**Status**: Confirmed  
**Decision**: Direct Port of C++ Manifold Algorithms

## Executive Summary

This document confirms that `libs/manifold-rs` will be a **direct port** of the local C++ Manifold algorithms using an index-based half-edge data structure, not a thin wrapper around external kernels like `manifold3d` or `csgrs`.

## Rationale

### Why Direct Port?

1. **Algorithmic Control**: Direct porting gives us complete control over the geometry algorithms, ensuring consistency with OpenSCAD's behavior and performance characteristics.

2. **Index-Based Half-Edge Design**: The C++ Manifold library uses an efficient index-based half-edge structure that maps naturally to Rust's memory model and safety guarantees.

3. **WASM Optimization**: Direct porting allows us to optimize specifically for WebAssembly, including memory layout and parallel processing using `rayon`.

4. **Zero External Dependencies**: Avoiding external kernel dependencies reduces WASM bundle size and eliminates potential licensing issues.

### Architecture Overview

The Rust implementation will follow these core principles:

#### Index-Based Half-Edge Structure
```rust
// Core data structures using Vec arenas + u32 indices
pub struct HalfEdgeMesh {
    vertices: Vec<Vertex>,        // Position data (f64 precision)
    half_edges: Vec<HalfEdge>,      // Connectivity information
    faces: Vec<Face>,               // Face metadata
    edges: Vec<Edge>,               // Edge properties
}

// All references use indices, not pointers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HalfEdgeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceId(u32);
```

#### Memory Layout Optimization
- **Vec Arenas**: Centralized storage for each entity type
- **u32 Indices**: Compact references instead of pointers
- **Cache-Friendly**: Sequential memory access patterns
- **Parallel Processing**: `rayon` for data-parallel operations

### Porting Strategy

#### Phase 1: Core Infrastructure
1. **Basic Half-Edge Structure**: Implement vertices, half-edges, and faces
2. **Topology Validation**: Euler characteristic and manifold checks
3. **Memory Management**: Efficient allocation and deallocation

#### Phase 2: Basic Primitives
1. **Cube Implementation**: Simple hexahedron construction
2. **Sphere Implementation**: Icosphere with subdivision
3. **Cylinder Implementation**: Extruded circle with tessellation

#### Phase 3: Transformations
1. **Matrix Operations**: Affine transformations on vertices
2. **Normal Computation**: Face and vertex normal calculations
3. **Bounding Boxes**: Axis-aligned bounding box computation

#### Phase 4: Boolean Operations
1. **Broad Phase**: Spatial indexing (R-Tree/BVH)
2. **Exact Intersection**: Edge-plane and edge-edge intersections
3. **Classification**: Inside/outside triangle classification
4. **Retriangulation**: Constrained Delaunay triangulation

### Key Algorithms to Port

#### Boolean Operations (CSG)
- **Union**: Combine two manifolds
- **Difference**: Subtract one manifold from another
- **Intersection**: Find common volume between manifolds

#### Spatial Acceleration
- **R-Tree**: For triangle bounding box queries
- **BVH**: For ray-triangle intersection tests
- **Grid Hash**: For spatial hashing of vertices

#### Robust Predicates
- **Orientation Tests**: Using `robust` crate for exact arithmetic
- **Intersection Tests**: Consistent geometric computations
- **Winding Number**: Point-in-polyhedron tests

### External References (Not Runtime Dependencies)

The following external libraries may be consulted **only as references**:

1. **manifold3d**: Rust bindings to the C++ Manifold library
   - Useful for understanding API design patterns
   - Benchmarking and test case comparison
   - **Not used as runtime dependency**

2. **csgrs**: Pure Rust CSG implementation
   - Algorithm inspiration for boolean operations
   - Test case validation and comparison
   - **Not used as runtime dependency**

### Implementation Guidelines

#### C++ to Rust Translation Patterns

1. **Pointer → Index**: Replace raw pointers with u32 indices into Vec arenas
2. **thrust/TBB → rayon**: Map parallel loops to `par_iter()` operations
3. **Manual Memory → RAII**: Replace new/delete with Rust ownership
4. **Error Codes → Results**: Convert error codes to `Result<T, Error>`

#### Safety Considerations

1. **No Unsafe by Default**: Prefer safe Rust implementations
2. **Audited Unsafe Blocks**: When performance demands unsafe code
3. **Debug Assertions**: Extensive internal consistency checks
4. **Fuzz Testing**: Property-based testing for robustness

### Performance Targets

- **Compile Time**: <100ms for 10k triangle models
- **Memory Usage**: <50MB for typical interactive models
- **Parallel Speedup**: 2-4x on multi-core systems
- **WASM Bundle**: <2MB compressed size

### Conclusion

This direct port approach provides:
- ✅ Complete algorithmic control
- ✅ Optimal WASM performance
- ✅ Zero external dependencies
- ✅ Consistent OpenSCAD compatibility
- ✅ Modern Rust safety guarantees

The implementation will serve as a robust foundation for the OpenSCAD-to-mesh pipeline while maintaining the performance and reliability standards required for interactive browser-based modeling.
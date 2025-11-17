# Manifold Rust Port - Project Overview

## Executive Summary

This project aims to create a comprehensive Rust port of the Manifold geometric kernel and integrate it with the existing OpenSCAD parser to create a complete OpenSCAD-to-3D-geometry pipeline in pure Rust.

## Project Goals

### Primary Goals

1. **Create a Rust port of the Manifold library** that provides guaranteed-manifold mesh Boolean operations and geometric primitives
2. **Integrate with OpenSCAD parser** to convert OpenSCAD AST into Manifold geometry operations
3. **Build a complete rendering pipeline** from OpenSCAD source code to 3D mesh output
4. **Maintain performance** comparable to the C++ implementation
5. **Ensure reliability** with comprehensive testing and validation

### Secondary Goals

1. Support parallel processing using Rust's native concurrency primitives (Rayon)
2. Provide WebAssembly support for browser-based rendering
3. Enable interoperability with existing 3D file formats (STL, 3MF, glTF)
4. Create idiomatic Rust APIs that are safe and ergonomic

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     OpenSCAD Source Code                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Tree-sitter OpenSCAD Parser                     │
│              (libs/openscad-parser)                          │
└────────────────────────┬────────────────────────────────────┘
                         │ CST (Concrete Syntax Tree)
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                 OpenSCAD AST Builder                         │
│                 (libs/openscad-ast)                          │
└────────────────────────┬────────────────────────────────────┘
                         │ Typed AST
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              OpenSCAD Evaluator/Interpreter                  │
│              (NEW: libs/openscad-eval)                       │
└────────────────────────┬────────────────────────────────────┘
                         │ Evaluated Geometry Tree (IR, no vars/loops)
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Manifold Rust Port                              │
│              (NEW: libs/manifold-rs)                         │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Core Manifold (Boolean Operations, CSG)            │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │  Primitives (Cube, Sphere, Cylinder, etc.)          │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │  Transformations (Translate, Rotate, Scale, etc.)   │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │  2D CrossSection (Polygon operations)               │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │  Extrusion (Linear, Rotational)                     │   │
│  ├─────────────────────────────────────────────────────┤   │
│  │  Mesh I/O (Import/Export)                           │   │
│  └─────────────────────────────────────────────────────┘   │
└────────────────────────┬────────────────────────────────────┘
                         │ MeshGL output
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              3D Mesh Output (STL, 3MF, glTF)                │
└─────────────────────────────────────────────────────────────┘
```

## Technology Stack

### Core Technologies

- **Rust** (Edition 2021): Systems programming language with memory safety
- **glam** (0.28): Linear algebra for vectors, matrices, and transformations
- **clipper2** (1.0): 2D polygon boolean operations and offset
- **geo** (0.28): Polygon triangulation (earcut algorithm)
- **rstar** (0.12): R-tree spatial indexing for collision detection
- **stl_io** (0.7): STL file import/export
- **thiserror** (1.0): Error handling

### Optional Dependencies

- **rayon** (1.10): Data parallelism for performance-critical operations
- **iOverlay**: Pure Rust alternative to clipper2
- **baby_shark** or **csgrs**: Potential for 3D boolean operations (evaluate)
- **parry3d**: Potential for convex hull (evaluate)
- **wasm-bindgen**: WebAssembly support

## Key Features from Manifold

### Guaranteed Manifold Output

The core innovation of Manifold is its guaranteed-manifold mesh Boolean algorithm, which ensures:
- Every output is a valid manifold mesh (represents a solid object)
- No edge cases that cause failures
- Reliable results even with complex geometry

### Robust Boolean Operations

- **Union** (OR): Combine two solids
- **Difference** (AND NOT): Subtract one solid from another
- **Intersection** (AND): Keep only overlapping volume

### Geometric Primitives

- Cube (box)
- Sphere
- Cylinder (with different top/bottom radii)
- Polyhedron (arbitrary convex solid)

### Transformations

- Translate, Rotate, Scale
- Mirror
- Resize (with optional auto aspect ratio)
- Multmatrix (arbitrary 4x4 transformation matrix)

### 2D/3D Operations

- Linear extrude (with twist, scale)
- Rotate extrude (revolve)
- Projection (3D → 2D)
- Convex hull (2D and 3D)
- Minkowski sum
- Offset (2D polygon offset)

### File I/O

- Import (STL, OFF, AMF, 3MF formats)
- Surface (height map from DAT/PNG files)
- Export to STL, 3MF, glTF

## OpenSCAD Language Coverage

### Primitives to Implement

- `cube([x, y, z], center=false)`
- `sphere(r=radius, d=diameter, $fn, $fa, $fs)`
- `cylinder(h, r/r1, r2, d/d1, d2, center, $fn, $fa, $fs)`
- `polyhedron(points, faces, convexity)`

### Transformations to Implement

- `translate([x, y, z])`
- `rotate([x, y, z])` and `rotate(a, [x, y, z])`
- `scale([x, y, z])`
- `mirror([x, y, z])`
- `multmatrix(m)`
- `resize([x, y, z], auto)`
- `color([r, g, b, a])`

### Boolean Operations to Implement

- `union()`
- `difference()`
- `intersection()`
- `hull()`
- `minkowski()`

### 2D/3D Operations to Implement

- `linear_extrude(height, center, twist, slices, scale)`
- `rotate_extrude(angle, convexity, $fn, $fa, $fs)`

### 2D Primitives to Implement

- `circle(r/d, $fn, $fa, $fs)`
- `square([x, y], center)`
- `polygon(points, paths, convexity)`
- `text(text, size, font, ...)`

### 2D Operations to Implement

- `offset(r/delta, chamfer)`

### Other Operations to Implement

- `projection(cut)`
- `surface(file, center, convexity)`
- `import(file, convexity)`
- `render(convexity)`

### Control Flow (Handled by Evaluator)

- `for` loops
- `intersection_for` loops
- `if/else` conditionals
- Module definitions and calls
- Variable assignments

## Project Structure

```
rust-openscad/
├── libs/
│   ├── openscad-parser/       # Existing tree-sitter parser
│   ├── openscad-ast/          # Existing AST representation
│   ├── openscad-eval/         # NEW: AST evaluator/interpreter
│   └── manifold-rs/           # NEW: Manifold Rust port
│       ├── src/
│       │   ├── lib.rs
│       │   ├── manifold.rs    # Core Manifold type
│       │   ├── mesh.rs        # MeshGL representation
│       │   ├── primitives/    # Geometric primitives
│       │   │   ├── mod.rs
│       │   │   ├── cube.rs
│       │   │   ├── sphere.rs
│       │   │   └── cylinder.rs
│       │   ├── boolean.rs     # Boolean operations
│       │   ├── transforms.rs  # Transformations
│       │   ├── cross_section/ # 2D operations
│       │   │   ├── mod.rs
│       │   │   └── ...
│       │   ├── collider/      # Broad-phase collision
│       │   ├── polygon.rs     # Polygon triangulation
│       │   └── utils/         # Utilities
│       ├── tests/
│       └── Cargo.toml
├── specs/
│   └── manifold-rust-port/    # This directory
└── ...
```

## Success Criteria

### Functional Criteria

1. Can parse and evaluate all common OpenSCAD primitives
2. Boolean operations produce correct, manifold output
3. Transformations apply correctly to geometry
4. Can export to STL and 3MF formats
5. Passes comprehensive test suite

### Performance Criteria

1. Within 2-3x of C++ Manifold performance for single-threaded operations
2. Achieves near-linear speedup with parallelization
3. Memory usage comparable to C++ implementation

### Quality Criteria

1. 100% safe Rust (no unsafe blocks in public API)
2. Comprehensive documentation
3. >80% test coverage
4. No panics in production code

## Risks and Mitigations

### Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Boolean algorithm complexity | High | Medium | Start with simpler algorithms, iterate |
| Performance not meeting goals | Medium | Medium | Profile early, use SIMD where appropriate |
| Numerical stability issues | High | Medium | Study Manifold's approach carefully, use robust predicates |
| Integration complexity | Medium | Low | Clear interface contracts, comprehensive tests |

### Resource Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Underestimated effort | Medium | High | Phased approach, MVP first |
| Missing domain knowledge | Medium | Medium | Study Manifold source, consult literature |

## Timeline Estimate

This is a substantial project. Rough estimates:

- **Phase 1** (Core Infrastructure): 2-3 weeks
- **Phase 2** (Primitives & Transforms): 2-3 weeks
- **Phase 3** (Boolean Operations): 4-6 weeks
- **Phase 4** (2D Operations): 2-3 weeks
- **Phase 5** (Integration & Polish): 2-3 weeks

**Total**: 12-18 weeks for MVP

## Next Steps

1. Review and approve this specification
2. Set up basic Rust project structure
3. Implement core data structures (Manifold, MeshGL)
4. Begin with simple primitives (cube, sphere)
5. Implement transformations
6. Tackle boolean operations
7. Build evaluator bridge
8. Integration testing

## References

- [Manifold GitHub Repository](https://github.com/elalish/manifold)
- [Manifold Library Wiki](https://github.com/elalish/manifold/wiki/Manifold-Library)
- [Julian Smith's Dissertation on Robust Boolean Operations](https://github.com/elalish/manifold/blob/master/docs/RobustBoolean.pdf)
- [OpenSCAD User Manual](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual)
- [OpenSCAD Cheat Sheet](https://openscad.org/cheatsheet/)

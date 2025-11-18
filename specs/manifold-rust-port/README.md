# Manifold Rust Port - Specification Documents

This directory contains comprehensive planning and specification documents for porting the Manifold 3D geometry kernel to Rust and integrating it with the OpenSCAD parser.

## Recent Updates

**November 17, 2024**: Comprehensive specification updates

**Major Updates**:
1. **OpenSCAD Alignment**: See [REVISIONS.md](./REVISIONS.md)
   - Removed Manifold-specific features not in OpenSCAD
   - Added missing OpenSCAD operations
   - Simplified data structures

2. **Third-Party Library Analysis**: See [07-THIRD-PARTY-LIBRARIES.md](./07-THIRD-PARTY-LIBRARIES.md)
   - Identified libraries to simplify implementation
   - Potential **2-4 weeks time savings**
   - Recommended hybrid approach (libraries + custom)

3. **Coding Standards**: See [08-CODING-STANDARDS.md](./08-CODING-STANDARDS.md) ⭐ NEW
   - TDD workflow (Red-Green-Refactor)
   - No mocks policy (real implementations only)
   - 500-line file limit
   - SRP, DRY, KISS principles
   - Explicit error handling
   - Comprehensive documentation requirements

## Document Index

### Core Specifications

1. **[00-OVERVIEW.md](./00-OVERVIEW.md)** - Project overview, goals, and architecture
   - Executive summary
   - High-level architecture diagram
   - Technology stack
   - Success criteria
   - Timeline estimates

2. **[01-ARCHITECTURE.md](./01-ARCHITECTURE.md)** - Detailed technical architecture
   - Core data structures
   - Module organization
   - Algorithm designs
   - Parallelization strategy
   - Error handling approach
   - Memory management

### Task Breakdown

3. **[04-TASKS-PHASE1.md](./04-TASKS-PHASE1.md)** - Phase 1: Core Infrastructure
   - Project setup
   - Core data structures (Vec3, BoundingBox, MeshGL, HalfEdgeMesh, Manifold)
   - Error handling
   - Duration: 2-3 weeks

4. **[04-TASKS-PHASE2.md](./04-TASKS-PHASE2.md)** - Phase 2: Primitives & Transforms
   - Geometric primitives (cube, sphere, cylinder, tetrahedron)
   - Transformations (translate, rotate, scale, mirror, matrix)
   - Duration: 2-3 weeks

5. **[04-TASKS-PHASE3.md](./04-TASKS-PHASE3.md)** - Phase 3: Boolean Operations
   - Collision detection (R-tree)
   - Edge-triangle intersection
   - Topology construction
   - Union, difference, intersection operations
   - Duration: 4-6 weeks

6. **[04-TASKS-PHASE4.md](./04-TASKS-PHASE4.md)** - Phase 4: 2D Operations & Extrusion
   - CrossSection (2D polygons)
   - 2D primitives
   - Polygon triangulation
   - Linear and rotational extrusion
   - Convex hull
   - Duration: 2-3 weeks

7. **[04-TASKS-PHASE5.md](./04-TASKS-PHASE5.md)** - Phase 5: Integration & Polish
   - OpenSCAD evaluator design
   - Expression and statement evaluation
   - File I/O (STL export)
   - End-to-end testing
   - Documentation
   - Duration: 2-3 weeks

### Supporting Documents

8. **[05-TESTING-STRATEGY.md](./05-TESTING-STRATEGY.md)** - Comprehensive testing plan
   - Unit tests
   - Integration tests
   - Property-based tests
   - Regression tests
   - Benchmarks
   - Coverage goals

9. **[06-ROADMAP.md](./06-ROADMAP.md)** - Development roadmap and timeline
   - Month-by-month breakdown
   - Milestones
   - Success metrics
   - Risk mitigation
   - Progress tracking

10. **[REVISIONS.md](./REVISIONS.md)** - Specification revision history
   - OpenSCAD alignment changes
   - Removed Manifold-specific features
   - Added missing OpenSCAD operations
   - Deferred features
   - Data structure simplifications

11. **[07-THIRD-PARTY-LIBRARIES.md](./07-THIRD-PARTY-LIBRARIES.md)** - Third-party library analysis
   - Comprehensive library research
   - Recommendations by category
   - Effort savings analysis
   - Updated dependencies
   - Revised timeline (10-16 weeks)

12. **[IMPLEMENTATION-STRATEGY.md](./IMPLEMENTATION-STRATEGY.md)** - Executive implementation guide ⭐ START HERE
   - Quick summary of hybrid approach
   - Selected libraries with rationale
   - Complete Cargo.toml
   - Phase-by-phase strategy
   - Timeline comparison

13. **[08-CODING-STANDARDS.md](./08-CODING-STANDARDS.md)** - Coding standards and best practices ⭐ REQUIRED READING
   - TDD workflow (Red-Green-Refactor)
   - No mocks policy (except I/O)
   - Single Responsibility Principle
   - File size limits (500 lines)
   - DRY and KISS principles
   - Explicit error handling
   - Documentation requirements
   - Project structure standards

## Quick Start Guide

### For Implementers

**⭐ START HERE**: [IMPLEMENTATION-STRATEGY.md](./IMPLEMENTATION-STRATEGY.md) - Executive summary of recommended approach

1. **Read in order**:
   - Start with IMPLEMENTATION-STRATEGY.md for quick overview
   - **Read 08-CODING-STANDARDS.md** (REQUIRED - defines how we code)
   - Read 00-OVERVIEW.md for big picture
   - Review 07-THIRD-PARTY-LIBRARIES.md for library decisions
   - Review 01-ARCHITECTURE.md for technical details
   - Follow phase documents in sequence (04-TASKS-PHASE*.md)

2. **Implementation approach**:
   - Complete phases in order (dependencies!)
   - Follow task order within each phase
   - Write tests as you implement
   - Document as you go

3. **Reference materials**:
   - [Manifold C++ Source](https://github.com/elalish/manifold)
   - [Manifold Wiki](https://github.com/elalish/manifold/wiki/Manifold-Library)
   - [OpenSCAD Manual](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual)

### For Reviewers

- **Overview**: Read 00-OVERVIEW.md
- **Architecture**: Review 01-ARCHITECTURE.md
- **Roadmap**: Check 06-ROADMAP.md for timeline
- **Testing**: Review 05-TESTING-STRATEGY.md

## Project Structure

After implementation, the project will have this structure:

```
rust-openscad/
├── libs/
│   ├── openscad-parser/       # Existing: Tree-sitter parser
│   ├── openscad-ast/          # Existing: AST representation
│   ├── openscad-eval/         # NEW: AST evaluator
│   └── manifold-rs/           # NEW: Manifold Rust port
│       ├── src/
│       │   ├── lib.rs
│       │   ├── manifold.rs
│       │   ├── primitives/
│       │   ├── boolean/
│       │   ├── transforms/
│       │   ├── cross_section/
│       │   └── ...
│       ├── tests/
│       ├── benches/
│       └── examples/
├── specs/
│   └── manifold-rust-port/    # This directory
└── ...
```

## Key Concepts

### Manifold Mesh
A triangle mesh that represents a valid solid object:
- Every edge connects exactly two triangles
- Consistent orientation (CCW winding from outside)
- No holes, gaps, or self-intersections

### Boolean Operations (CSG)
Constructive Solid Geometry operations:
- **Union (∪)**: Combine two solids
- **Difference (−)**: Subtract one solid from another
- **Intersection (∩)**: Keep only overlapping volume

### OpenSCAD Integration
The evaluator and integration crates convert OpenSCAD source all the way to file output and a web
viewer:
```
OpenSCAD source
  → libs/openscad-parser   (Tree-sitter CST)
  → libs/openscad-ast      (typed AST)
  → libs/openscad-eval     (evaluated geometry IR, no vars/loops)
  → libs/manifold-rs       (Manifold + MeshGL)
  → file export            (STL/3MF/glTF) and libs/wasm (WebAssembly)
  → playground/ (Svelte + Three.js, full-window viewport)
```

## Dependencies

### Required Rust Crates
- `nalgebra` or `glam` - Linear algebra
- `thiserror` - Error handling
- `rayon` (optional) - Parallelization

### Optional Crates
- `rstar` - R-tree spatial indexing
- `clipper2` - 2D polygon operations
- `criterion` - Benchmarking
- `proptest` - Property-based testing

## Timeline Summary

**Total Duration**: 12-18 weeks for MVP

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| 1 | 2-3 weeks | Core data structures, error handling |
| 2 | 2-3 weeks | Primitives, transformations |
| 3 | 4-6 weeks | Boolean operations (CSG) |
| 4 | 2-3 weeks | 2D operations, extrusion |
| 5 | 2-3 weeks | OpenSCAD integration, file I/O |

## Success Criteria

### Functional
- ✅ Parse and render OpenSCAD files
- ✅ All operations produce manifold meshes
- ✅ Export to STL format
- ✅ Support all common OpenSCAD features

### Performance
- ✅ Within 2-3x of C++ Manifold performance
- ✅ Handle 100k+ triangle meshes
- ✅ Parallel speedup with Rayon

### Quality
- ✅ >80% test coverage
- ✅ Comprehensive documentation
- ✅ Zero critical bugs

## Contributing

When implementing:
1. Follow the task order in phase documents
2. Write tests before or alongside implementation
3. Document public APIs thoroughly
4. Run `cargo fmt` and `cargo clippy`
5. Update progress in 06-ROADMAP.md

## Research References

### Manifold Library
- [Main Repository](https://github.com/elalish/manifold)
- [Algorithm Wiki](https://github.com/elalish/manifold/wiki/Manifold-Library)
- [Julian Smith's Dissertation on Robust Booleans](https://github.com/elalish/manifold/blob/master/docs/RobustBoolean.pdf)

### OpenSCAD
- [User Manual](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual)
- [Cheat Sheet](https://openscad.org/cheatsheet/)
- [Language Reference](https://en.wikibooks.org/wiki/OpenSCAD_User_Manual/The_OpenSCAD_Language)

### Computational Geometry
- CGAL library documentation
- Geometric Tools for Computer Graphics (Book)
- Real-Time Collision Detection (Book)

## Notes

### All Task Details Include:
- **Description**: What needs to be done
- **Context**: Why it's important
- **Subtasks**: Step-by-step breakdown
- **Acceptance Criteria**: How to verify completion
- **Dependencies**: Prerequisites
- **Estimated Effort**: Time estimate
- **Implementation Notes**: Technical details

### Task Format Example:
Each task is self-contained with all necessary context, so you can implement without constantly referencing external resources. Code examples and detailed specifications are provided inline.

## Status

**Current Status**: 📝 Planning Complete, Implementation Not Started

**Last Updated**: November 16, 2025

**Next Steps**:
1. Review all specification documents
2. Set up development environment
3. Begin Phase 1, Task 1.1: Initialize Manifold Rust Crate

---

## Questions?

If you have questions about any aspect of this project:
1. Review the relevant specification document
2. Check the architecture document for design decisions
3. Consult the Manifold C++ source code
4. Open a discussion issue

## License

This specification follows the same license as the main project (Apache-2.0).

# Manifold Rust Port - Development Roadmap

## Project Summary

**Goal**: Create a complete OpenSCAD-to-3D-mesh pipeline in Rust by porting the Manifold geometry kernel and integrating it with the existing OpenSCAD parser.

**Total Estimated Duration**: 12-18 weeks for MVP

---

## Quick Reference

**Note**: Updated timeline reflects use of third-party libraries (see [07-THIRD-PARTY-LIBRARIES.md](./07-THIRD-PARTY-LIBRARIES.md))

| Phase | Focus | Original | With Libraries | Status |
|-------|-------|----------|----------------|--------|
| Phase 0 | Library Evaluation | - | 1 week | 📝 Not Started |
| Phase 1 | Core Infrastructure | 2-3 weeks | 2-3 weeks | 📝 Not Started |
| Phase 2 | Primitives & Transforms | 2-3 weeks | 2-3 weeks | 📝 Not Started |
| Phase 3 | Boolean Operations | 4-6 weeks | 3-5 weeks | 📝 Not Started |
| Phase 4 | 2D & Extrusion | 2-3 weeks | 1-2 weeks | 📝 Not Started |
| Phase 5 | Integration & Polish | 2-3 weeks | 2-3 weeks | 📝 Not Started |
| **Total** | **MVP** | **12-18 weeks** | **11-17 weeks** | - |

---

## Detailed Timeline

### Week 0: Library Evaluation (NEW)

**Phase 0 - Prototype & Validate Third-Party Libraries**

- [ ] Create quick prototype with baby_shark boolean operations
- [ ] Test baby_shark with simple meshes (cube union, difference)
- [ ] Evaluate manifoldness of output
- [ ] Compare with csgrs library
- [ ] Test clipper2 for 2D operations
- [ ] Test geo triangulation
- [ ] Document findings and make decision
- [ ] Update dependencies based on evaluation

**Deliverable**: Library selection decision document

---

### Month 1: Foundation

**Weeks 1-2: Phase 1 - Core Infrastructure**

- [ ] Set up project structure and CI/CD
- [ ] Implement core data structures (Vec3, BoundingBox, MeshGL, HalfEdgeMesh)
- [ ] Implement Manifold wrapper type
- [ ] Set up error handling
- [ ] Write comprehensive unit tests
- [ ] Documentation for core types

**Deliverable**: Working data structures with full test coverage

**Weeks 3-4: Phase 2 - Primitives (Part 1)**

- [ ] Implement cube primitive
- [ ] Implement sphere primitive (icosphere)
- [ ] Implement cylinder primitive
- [ ] Implement tetrahedron primitive
- [ ] Verify all primitives produce manifold meshes
- [ ] Write tests for all primitives

**Deliverable**: All basic primitives working and tested

---

### Month 2: Transformations and Booleans

**Week 5: Phase 2 - Transformations**

- [ ] Implement translate, rotate, scale
- [ ] Implement mirror and matrix transformations
- [ ] Test transformation composition
- [ ] Verify manifoldness preservation

**Deliverable**: Complete transformation system

**Weeks 6-9: Phase 3 - Boolean Operations (Part 1)**

- [ ] Implement R-tree spatial indexing
- [ ] Implement edge-triangle intersection
- [ ] Implement topology construction
- [ ] Implement union operation
- [ ] Write extensive tests

**Deliverable**: Working union operation

---

### Month 3: Boolean Operations Complete

**Weeks 10-11: Phase 3 - Boolean Operations (Part 2)**

- [ ] Implement difference operation
- [ ] Implement intersection operation
- [ ] Implement batch boolean operations
- [ ] Add operator overloading
- [ ] Comprehensive boolean testing
- [ ] Performance optimization

**Deliverable**: Complete CSG system

**Week 12: Phase 3 - Polish and Testing**

- [ ] Fix edge cases in booleans
- [ ] Property-based testing
- [ ] Performance benchmarking
- [ ] Documentation

---

### Month 4: 2D Operations and Integration

**Weeks 13-14: Phase 4 - 2D and Extrusion**

- [ ] Implement CrossSection data structure
- [ ] Implement 2D primitives (circle, square)
- [ ] Implement polygon triangulation
- [ ] Implement linear_extrude (with twist, scale)
- [ ] Implement rotate_extrude
- [ ] Implement convex hull (2D and 3D)

**Deliverable**: Complete 2D/3D conversion system

**Weeks 15-16: Phase 5 - OpenSCAD Integration (Part 1)**

- [ ] Design evaluator architecture
- [ ] Implement expression evaluator
- [ ] Implement primitive evaluation
- [ ] Implement transformation evaluation
- [ ] Basic OpenSCAD scripts working

**Deliverable**: Basic OpenSCAD evaluation working

---

### Month 5: Integration Complete

**Weeks 17-18: Phase 5 - OpenSCAD Integration (Part 2)**

- [ ] Implement boolean statement evaluation
- [ ] Implement control flow (for, if/else)
- [ ] Implement module definitions
- [ ] Implement function definitions
- [ ] Complete OpenSCAD coverage

**Deliverable**: Full OpenSCAD language support

---

### Month 6: Polish and Release

**Week 19: File I/O and Examples**

- [ ] Implement STL export
- [ ] Implement 3MF export (optional)
- [ ] Create example gallery
- [ ] End-to-end integration tests

**Week 20: Documentation and Release**

- [ ] Complete API documentation
- [ ] Write user guide
- [ ] Write contributor guide
- [ ] Create tutorial examples
- [ ] Prepare for v0.1.0 release

**Deliverable**: Production-ready library

---

## Milestones

### Milestone 1: Core Foundation (End of Week 2)
- ✅ Project structure set up
- ✅ Core data structures implemented
- ✅ Test framework in place

### Milestone 2: Primitives Ready (End of Week 4)
- ✅ All basic primitives working
- ✅ Transformations implemented
- ✅ Can create and manipulate simple geometry

### Milestone 3: CSG Complete (End of Week 11)
- ✅ All boolean operations working
- ✅ Guaranteed manifold output
- ✅ Performance acceptable

### Milestone 4: 2D/3D Integration (End of Week 14)
- ✅ Extrusion operations working
- ✅ Can create complex 3D shapes from 2D

### Milestone 5: OpenSCAD Working (End of Week 18)
- ✅ Can parse and execute OpenSCAD files
- ✅ Full language support
- ✅ Examples working

### Milestone 6: Production Ready (End of Week 20)
- ✅ Complete documentation
- ✅ File I/O working
- ✅ Ready for release

---

## Success Metrics

### Functional Metrics
- [ ] 100% of OpenSCAD primitives supported
- [ ] 100% of OpenSCAD transformations supported
- [ ] 100% of OpenSCAD boolean operations supported
- [ ] All operations produce manifold meshes
- [ ] Can export to STL format

### Quality Metrics
- [ ] >80% code coverage
- [ ] All property tests passing
- [ ] Zero critical bugs
- [ ] Documentation complete

### Performance Metrics
- [ ] Within 3x of C++ Manifold for single-threaded
- [ ] Parallel speedup >2x for large meshes
- [ ] Can handle 100k+ triangle meshes

---

## Risk Mitigation

### Technical Risks

| Risk | Mitigation | Status |
|------|------------|--------|
| Boolean algorithm too complex | Start simple, iterate | Planned |
| Performance not meeting goals | Profile early, optimize hot paths | Planned |
| Numerical stability issues | Use robust predicates, study Manifold approach | Planned |
| Integration complexity | Clear interfaces, comprehensive tests | Planned |

### Schedule Risks

| Risk | Mitigation | Status |
|------|------------|--------|
| Underestimated effort | Phased approach, MVP first | Planned |
| Blocking dependencies | Parallel work where possible | Planned |
| Scope creep | Strict MVP definition | Planned |

---

## Phase Details

See individual phase documents for detailed task breakdowns:
- [Phase 1: Core Infrastructure](./04-TASKS-PHASE1.md)
- [Phase 2: Primitives & Transforms](./04-TASKS-PHASE2.md)
- [Phase 3: Boolean Operations](./04-TASKS-PHASE3.md)
- [Phase 4: 2D & Extrusion](./04-TASKS-PHASE4.md)
- [Phase 5: Integration & Polish](./04-TASKS-PHASE5.md)

---

## Beyond MVP (Future Work)

### Phase 6: Advanced Features (Optional)
- Minkowski sum
- Level set (SDF) operations
- Mesh refinement and smoothing
- Mesh simplification
- Advanced vertex properties

### Phase 7: Optimization
- SIMD optimizations
- GPU acceleration (optional)
- Memory pool allocators
- Cache optimization

### Phase 8: Ecosystem Integration
- CAD file format support (STEP, IGES)
- Mesh repair tools
- Visualization tools
- Web assembly builds

---

## Getting Started

To begin implementation:

1. **Review** all specification documents
2. **Set up** development environment
3. **Start** with Phase 1, Task 1.1
4. **Follow** the task order in each phase
5. **Test** thoroughly as you go
6. **Document** as you implement

---

## Questions or Issues?

If anything is unclear or needs discussion:
1. Review the architecture document
2. Check task details in phase documents
3. Consult Manifold C++ source code
4. Open discussion issue

---

## Progress Tracking

Update this file as you complete milestones:
- Change status from 📝 Not Started → 🏗️ In Progress → ✅ Complete
- Note actual vs. estimated time
- Document any deviations from plan
- Adjust future estimates accordingly

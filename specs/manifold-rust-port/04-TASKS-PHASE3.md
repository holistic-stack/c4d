# Phase 3: Boolean Operations (CSG)

## Overview

Phase 3 implements the core boolean operations that enable Constructive Solid Geometry (CSG). This is the most complex phase.

**Duration**: 4-6 weeks  
**Dependencies**: Phase 1, Phase 2

---

## Task 3.1: Broad-Phase Collision Detection

**Description**: Implement R-tree spatial indexing for finding potentially intersecting triangles.

**Why**: Boolean operations need to find which triangles from two meshes might intersect. Naive O(n²) is too slow.

**Subtasks**:

1. **Integrate or implement R-tree**
   - Option A: Use `rstar` crate
   - Option B: Implement simple bounding box hierarchy
   - Build spatial index for triangle bounding boxes

2. **Implement triangle bounding box calculation**
   ```rust
   fn triangle_bbox(v0: Vec3, v1: Vec3, v2: Vec3) -> BoundingBox {
       BoundingBox::from_points(&[v0, v1, v2])
   }
   ```

3. **Implement intersection candidate query**
   ```rust
   pub fn find_intersection_candidates(
       mesh_a: &HalfEdgeMesh,
       mesh_b: &HalfEdgeMesh,
   ) -> Vec<(TriIdx, TriIdx)> {
       // Build R-trees for both meshes
       // Query overlapping triangle pairs
   }
   ```

4. **Write tests**
   - Test R-tree construction
   - Test query correctness
   - Benchmark performance

**Acceptance Criteria**:
- ✅ R-tree builds correctly
- ✅ Finds all overlapping pairs
- ✅ No false negatives (misses actual intersections)
- ✅ Acceptable false positive rate
- ✅ Performance is O(n log n) or better
- ✅ Tests pass

**Effort**: 8-12 hours

---

## Task 3.2: Edge-Triangle Intersection

**Description**: Compute exact intersection points between edges and triangles.

**Why**: Core geometric computation for boolean operations.

**Subtasks**:

1. **Implement edge-triangle intersection test**
   ```rust
   pub struct EdgeTriIntersection {
       pub edge: HalfEdgeIdx,
       pub triangle: TriIdx,
       pub point: Vec3,
       pub parameter: f64,  // Position along edge [0, 1]
       pub barycentric: Vec3,  // Position in triangle
   }
   
   pub fn edge_triangle_intersection(
       edge_start: Vec3,
       edge_end: Vec3,
       tri_v0: Vec3,
       tri_v1: Vec3,
       tri_v2: Vec3,
   ) -> Option<EdgeTriIntersection>
   ```

2. **Handle degenerate cases**
   - Edge parallel to triangle
   - Edge endpoint on triangle
   - Edge on triangle plane
   - Use epsilon for tolerance

3. **Implement batch processing**
   ```rust
   pub fn compute_all_intersections(
       candidates: &[(TriIdx, TriIdx)],
       mesh_a: &HalfEdgeMesh,
       mesh_b: &HalfEdgeMesh,
   ) -> Vec<EdgeTriIntersection>
   ```

4. **Parallelize with rayon**
   - Process candidates in parallel
   - Collect results
   - Benchmark speedup

**Acceptance Criteria**:
- ✅ Intersection test is correct
- ✅ Handles degenerate cases
- ✅ Numerically stable
- ✅ Parallel implementation works
- ✅ Tests cover edge cases
- ✅ Benchmarks show good performance

**Effort**: 16-24 hours

---

## Task 3.3: Topology Construction

**Description**: Build new manifold mesh from intersection data.

**Why**: Takes geometric intersections and constructs output mesh topology.

**Subtasks**:

1. **Split edges at intersection points**
   - Insert new vertices where edges are cut
   - Update edge connectivity
   - Maintain manifold property

2. **Classify triangles (inside/outside/boundary)**
   ```rust
   enum TriangleClass {
       Inside,
       Outside,
       Boundary,
   }
   
   fn classify_triangle(
       tri: TriIdx,
       mesh: &HalfEdgeMesh,
       other: &HalfEdgeMesh,
   ) -> TriangleClass
   ```
   - Use winding number or ray casting
   - Mark boundary triangles

3. **Stitch boundary edges**
   - Connect triangles along intersection curve
   - Ensure manifold connectivity
   - Handle edge pairing correctly

4. **Remove duplicate vertices**
   - Merge vertices within tolerance
   - Update triangle indices

5. **Validate output mesh**
   - Check manifoldness
   - Verify no degenerate triangles
   - Check bounding boxes

**Acceptance Criteria**:
- ✅ Edge splitting works correctly
- ✅ Triangle classification is accurate
- ✅ Output mesh is manifold
- ✅ No duplicate vertices
- ✅ Tests verify correctness
- ✅ Handles complex cases

**Effort**: 24-32 hours

---

## Task 3.4: Union Operation

**Description**: Implement A ∪ B (combine two solids).

**Why**: Most common boolean operation.

**Subtasks**:

1. **Implement union logic**
   ```rust
   impl Manifold {
       pub fn union(&self, other: &Manifold) -> Manifold {
           // Keep triangles from A that are outside B
           // Keep triangles from B that are outside A
           // Stitch boundary triangles
       }
   }
   ```

2. **Handle special cases**
   - Empty meshes
   - Non-overlapping meshes
   - Fully contained meshes

3. **Optimize for common patterns**
   - Non-intersecting → simple combine
   - Share bounding box test

4. **Write comprehensive tests**
   - Test with primitives
   - Test volume conservation
   - Test commutativity: A∪B = B∪A
   - Test associativity: (A∪B)∪C = A∪(B∪C)
   - Test idempotence: A∪A = A

**Acceptance Criteria**:
- ✅ Union produces correct result
- ✅ Output is manifold
- ✅ Volume is correct
- ✅ All tests pass
- ✅ Performance acceptable

**Effort**: 12-16 hours

---

## Task 3.5: Difference Operation

**Description**: Implement A - B (subtract B from A).

**Why**: Essential for creating holes and cavities.

**Subtasks**:

1. **Implement difference logic**
   ```rust
   impl Manifold {
       pub fn difference(&self, other: &Manifold) -> Manifold {
           // Keep triangles from A that are outside B
           // Keep triangles from B that are inside A (flipped)
           // Stitch boundary
       }
   }
   ```

2. **Handle triangle flipping**
   - Invert triangles from B
   - Reverse winding order

3. **Write tests**
   - Test with primitives
   - Test volume: Vol(A-B) = Vol(A) - Vol(A∩B)
   - Test non-commutativity: A-B ≠ B-A
   - Test A-A = empty

**Acceptance Criteria**:
- ✅ Difference produces correct result
- ✅ Output is manifold
- ✅ Winding order correct
- ✅ Tests pass

**Effort**: 8-12 hours

---

## Task 3.6: Intersection Operation

**Description**: Implement A ∩ B (keep only overlapping volume).

**Why**: Useful for finding common volumes.

**Subtasks**:

1. **Implement intersection logic**
   ```rust
   impl Manifold {
       pub fn intersection(&self, other: &Manifold) -> Manifold {
           // Keep triangles from A that are inside B
           // Keep triangles from B that are inside A
           // Stitch boundary
       }
   }
   ```

2. **Write tests**
   - Test with primitives
   - Test commutativity: A∩B = B∩A
   - Test associativity
   - Test A∩A = A
   - Test A∩(B∪C) = (A∩B)∪(A∩C)

**Acceptance Criteria**:
- ✅ Intersection works correctly
- ✅ Output is manifold
- ✅ Tests pass

**Effort**: 8-12 hours

---

## Task 3.7: Batch Boolean Operations

**Description**: Optimize boolean operations on many objects at once.

**Why**: Efficient for union/intersection of many objects.

**Subtasks**:

1. **Implement batch union**
   ```rust
   pub fn batch_union(manifolds: Vec<Manifold>) -> Manifold {
       // Parallel divide-and-conquer
       manifolds.par_iter()
           .cloned()
           .reduce(|| Manifold::empty(), |a, b| a.union(&b))
   }
   ```

2. **Implement batch intersection**

3. **Write tests**
   - Test with many cubes
   - Verify result matches sequential
   - Benchmark speedup

**Acceptance Criteria**:
- ✅ Batch operations work
- ✅ Faster than sequential
- ✅ Correct results
- ✅ Tests pass

**Effort**: 6-8 hours

---

## Task 3.8: Operator Overloading

**Description**: Add operator overloads for boolean operations.

**Why**: Ergonomic API: `a + b` for union, `a - b` for difference, `a ^ b` for intersection.

**Subtasks**:

1. **Implement operators**
   ```rust
   impl std::ops::Add for Manifold {
       type Output = Manifold;
       fn add(self, other: Manifold) -> Manifold {
           self.union(&other)
       }
   }
   
   impl std::ops::Sub for Manifold { /* ... */ }
   impl std::ops::BitXor for Manifold { /* ... */ }
   
   impl std::ops::AddAssign for Manifold { /* ... */ }
   impl std::ops::SubAssign for Manifold { /* ... */ }
   impl std::ops::BitXorAssign for Manifold { /* ... */ }
   ```

2. **Write tests**

**Acceptance Criteria**:
- ✅ All operators work
- ✅ Tests pass

**Effort**: 2-3 hours

---

## Phase 3 Complete When:

- [ ] All boolean operations implemented and tested
- [ ] Collision detection is efficient
- [ ] Intersection computation is robust
- [ ] Output meshes are always manifold
- [ ] Comprehensive test suite passes
- [ ] Performance benchmarks meet goals
- [ ] Documentation complete with examples

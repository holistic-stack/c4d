# Phase 4: 2D Operations and Extrusion

## Overview

Phase 4 implements 2D polygon operations (CrossSection) and extrusion to 3D.

**Duration**: 2-3 weeks  
**Dependencies**: Phase 1, Phase 2, Phase 3

---

## Task 4.1: CrossSection Data Structure

**Description**: Implement 2D polygon representation for extrusion operations.

**Why**: Need 2D geometry for linear_extrude and rotate_extrude operations.

**Subtasks**:

1. **Implement Polygons type**
   ```rust
   // In src/cross_section/mod.rs
   
   pub type SimplePolygon = Vec<Vec2>;
   
   pub struct Polygons {
       /// Outer contours and holes
       pub contours: Vec<SimplePolygon>,
   }
   
   impl Polygons {
       pub fn new() -> Self;
       pub fn add_contour(&mut self, contour: SimplePolygon);
       pub fn is_empty(&self) -> bool;
       pub fn bbox(&self) -> BoundingBox2D;
   }
   ```

2. **Implement CrossSection type**
   ```rust
   #[derive(Clone)]
   pub struct CrossSection {
       impl_: Arc<CrossSectionImpl>,
   }
   
   struct CrossSectionImpl {
       polygons: Polygons,
       bbox: BoundingBox2D,
   }
   
   impl CrossSection {
       pub fn empty() -> Self;
       pub fn from_polygons(polygons: Polygons) -> Self;
       pub fn to_polygons(&self) -> &Polygons;
       pub fn is_empty(&self) -> bool;
       pub fn bbox(&self) -> &BoundingBox2D;
       pub fn area(&self) -> f64;
   }
   ```

3. **Implement BoundingBox2D**
   ```rust
   #[derive(Clone, Debug)]
   pub struct BoundingBox2D {
       pub min: Vec2,
       pub max: Vec2,
   }
   // Similar to BoundingBox but 2D
   ```

**Acceptance Criteria**:
- ✅ CrossSection structure compiles
- ✅ Can construct from polygons
- ✅ Tests pass

**Effort**: 6-8 hours

---

## Task 4.2: 2D Primitives

**Description**: Implement 2D primitive shapes (circle, square, polygon).

**Why**: Building blocks for extrusion.

**Subtasks**:

1. **Implement square**
   ```rust
   pub fn square(size: Vec2, center: bool) -> CrossSection {
       let (x, y) = if center {
           (-size.x/2.0, -size.y/2.0)
       } else {
           (0.0, 0.0)
       };
       
       let points = vec![
           vec2(x, y),
           vec2(x + size.x, y),
           vec2(x + size.x, y + size.y),
           vec2(x, y + size.y),
       ];
       
       CrossSection::from_polygons(Polygons { contours: vec![points] })
   }
   ```

2. **Implement circle**
   ```rust
   pub fn circle(radius: f64, segments: usize) -> CrossSection {
       let n = resolve_segments(segments, radius);
       let points: Vec<Vec2> = (0..n)
           .map(|i| {
               let angle = 2.0 * PI * (i as f64) / (n as f64);
               vec2(radius * angle.cos(), radius * angle.sin())
           })
           .collect();
       
       CrossSection::from_polygons(Polygons { contours: vec![points] })
   }
   ```

3. **Implement polygon from points**
   ```rust
   pub fn polygon(points: Vec<Vec2>, paths: Option<Vec<Vec<usize>>>) -> CrossSection {
       // If paths provided, use them to define holes
       // Otherwise, treat points as single outer contour
   }
   ```

**Acceptance Criteria**:
- ✅ All 2D primitives work
- ✅ Area calculations correct
- ✅ Tests pass

**Effort**: 6-8 hours

---

## Task 4.2b: Text Primitive (Optional/Deferred)

**Description**: Implement text() 2D primitive for rendering text.

**Why**: OpenSCAD text() operation for creating text geometry.

**Note**: This is complex and may require external font rendering library.

**Decision**: **DEFER to future phase** - not critical for MVP.

**Future Subtasks**:
1. Research font rendering options (freetype-rs, rusttype, etc.)
2. Implement basic text rendering to polygons
3. Handle font, size, halign, valign parameters
4. Convert to CrossSection

**Effort**: 16-24 hours (complex) - DEFERRED

---

## Task 4.3: Offset Operation

**Description**: Implement 2D offset operation for polygons.

**Why**: OpenSCAD offset() for expanding/contracting 2D shapes.

**Subtasks**:
1. **Integrate offset library**
   - Option A: Use Clipper2 for robust offset
   - Option B: Implement simple offset algorithm

2. **Implement offset with radius**
   ```rust
   impl CrossSection {
       pub fn offset(&self, radius: f64, chamfer: bool) -> CrossSection {
           // Positive radius: expand
           // Negative radius: contract
       }
   }
   ```

3. **Handle chamfer vs round**
   - Round: create smooth curves
   - Chamfer: create straight edges

4. **Write tests**

**Acceptance Criteria**:
- ✅ Offset expands polygons correctly
- ✅ Offset contracts polygons correctly
- ✅ Chamfer option works
- ✅ Tests pass

**Effort**: 12-16 hours

---

## Task 4.4: Polygon Triangulation

**Description**: Implement algorithm to convert polygons to triangles.

**Why**: Needed for extrusion - must cap extruded shapes.

**Subtasks**:

1. **Choose triangulation algorithm**
   - Option A: Ear clipping (simple, O(n²))
   - Option B: Use `earcutr` crate
   - Option C: Implement constrained Delaunay

2. **Implement triangulation**
   ```rust
   // In src/polygon/triangulate.rs
   pub fn triangulate(polygon: &SimplePolygon) -> Vec<[usize; 3]> {
       // Returns triangle indices into polygon vertices
   }
   
   pub fn triangulate_with_holes(
       outer: &SimplePolygon,
       holes: &[SimplePolygon],
   ) -> Vec<[usize; 3]>
   ```

3. **Handle degenerate cases**
   - Collinear points
   - Self-intersecting polygons
   - Very small triangles

4. **Write tests**
   - Simple convex polygon
   - Concave polygon
   - Polygon with holes
   - Verify triangle winding

**Acceptance Criteria**:
- ✅ Triangulation works for various polygons
- ✅ Handles concave polygons
- ✅ Handles holes
- ✅ Winding order consistent
- ✅ Tests pass

**Effort**: 12-16 hours

---

## Task 4.4: Linear Extrusion

**Description**: Implement linear_extrude operation.

**Why**: Core operation to convert 2D to 3D in OpenSCAD.

**Subtasks**:

1. **Implement basic extrusion**
   ```rust
   impl Manifold {
       pub fn extrude(
           cross_section: &CrossSection,
           height: f64,
       ) -> Manifold {
           // Extrude along Z axis
           // Create top and bottom caps
           // Create side faces
       }
   }
   ```

2. **Add twist parameter**
   ```rust
   pub fn extrude_twist(
       cross_section: &CrossSection,
       height: f64,
       twist_degrees: f64,
       slices: usize,
   ) -> Manifold {
       // Subdivide height into slices
       // Rotate each slice progressively
   }
   ```

3. **Add scale parameter**
   ```rust
   pub fn extrude_scale(
       cross_section: &CrossSection,
       height: f64,
       scale_top: Vec2,
       slices: usize,
   ) -> Manifold {
       // Scale cross-section from 1.0 at bottom to scale_top at top
   }
   ```

4. **Combine all parameters**
   ```rust
   pub fn extrude_full(
       cross_section: &CrossSection,
       height: f64,
       slices: usize,
       twist_degrees: f64,
       scale_top: Vec2,
       center: bool,
   ) -> Manifold
   ```

5. **Write tests**
   - Extrude square → cube
   - Extrude circle → cylinder
   - Verify volume
   - Test twist
   - Test scale

**Acceptance Criteria**:
- ✅ Basic extrusion works
- ✅ Twist works correctly
- ✅ Scale works correctly
- ✅ Combined parameters work
- ✅ Output is manifold
- ✅ Tests pass

**Effort**: 16-20 hours

---

## Task 4.5: Rotate Extrusion

**Description**: Implement rotate_extrude operation (revolve).

**Why**: Create rotationally symmetric objects (bowls, vases, etc.).

**Subtasks**:

1. **Implement basic revolve**
   ```rust
   pub fn revolve(
       cross_section: &CrossSection,
       segments: usize,
   ) -> Manifold {
       // Revolve around Z axis
       // Create triangular faces connecting rotated profiles
   }
   ```

2. **Add angle parameter**
   ```rust
   pub fn revolve_angle(
       cross_section: &CrossSection,
       segments: usize,
       degrees: f64,
   ) -> Manifold {
       // Partial revolution (e.g., 180° for half)
   }
   ```

3. **Handle edge cases**
   - Points on axis (no triangles)
   - Negative X coordinates (hole in middle)

4. **Write tests**
   - Revolve rectangle → cylinder
   - Revolve semicircle → sphere
   - Partial revolution
   - Verify volume

**Acceptance Criteria**:
- ✅ Revolve works correctly
- ✅ Partial angles work
- ✅ Handles points on axis
- ✅ Output is manifold
- ✅ Tests pass

**Effort**: 12-16 hours

---

## Task 4.6: 2D Boolean Operations (Optional with Clipper2)

**Description**: Implement 2D boolean operations on CrossSection if using Clipper2.

**Why**: Allows complex 2D shape construction before extrusion.

**Subtasks**:

1. **Integrate Clipper2 library**
   - Add dependency
   - Create FFI bindings if needed

2. **Implement 2D union**
   ```rust
   impl CrossSection {
       pub fn union(&self, other: &CrossSection) -> CrossSection;
   }
   ```

3. **Implement 2D difference and intersection**

4. **Write tests**

**Acceptance Criteria**:
- ✅ 2D booleans work
- ✅ Tests pass
- ✅ OR mark as optional feature

**Effort**: 12-16 hours (if implemented)

---

## Task 4.7: Convex Hull

**Description**: Implement 2D and 3D convex hull operations.

**Why**: OpenSCAD hull() operation is commonly used.

**Subtasks**:

1. **Implement 2D convex hull**
   - Use Graham scan or Jarvis march
   - Returns smallest convex polygon containing points

2. **Implement 3D convex hull**
   - Use QuickHull algorithm
   - Returns smallest convex polyhedron

3. **Implement hull for multiple Manifolds**
   ```rust
   impl Manifold {
       pub fn hull(manifolds: &[Manifold]) -> Manifold {
           // Compute convex hull of all vertices
       }
   }
   ```

4. **Write tests**
   - Test with points
   - Test with primitives
   - Verify convexity

**Acceptance Criteria**:
- ✅ 2D hull works
- ✅ 3D hull works
- ✅ Output is manifold and convex
- ✅ Tests pass

**Effort**: 16-20 hours

---

## Task 4.8: Minkowski Sum

**Description**: Implement Minkowski sum operation.

**Why**: OpenSCAD minkowski() for sweeping one shape through another.

**Subtasks**:

1. **Understand Minkowski algorithm**
   - For each vertex in A, translate all of B by that vertex
   - Take convex hull of result
   - Or use more efficient algorithms

2. **Implement basic Minkowski**
   ```rust
   impl Manifold {
       pub fn minkowski(a: &Manifold, b: &Manifold) -> Manifold {
           // Simplified: take convex hull of pairwise sums
       }
   }
   ```

3. **Optimize for common cases**
   - Minkowski with sphere → rounded edges
   - Minkowski with cube → thickening

4. **Write tests**

**Acceptance Criteria**:
- ✅ Basic Minkowski works
- ✅ Output is manifold
- ✅ Tests pass
- ✅ OR mark as optional/future if too complex for MVP

**Effort**: 20-24 hours (complex) - consider DEFER

---

## Task 4.9: Projection Operation

**Description**: Implement projection() to convert 3D to 2D.

**Why**: OpenSCAD projection(cut) operation.

**Subtasks**:

1. **Implement projection without cut**
   ```rust
   impl Manifold {
       pub fn projection(&self, cut: bool) -> CrossSection {
           if cut {
               // Slice at Z=0, return cross-section
           } else {
               // Project all geometry onto XY plane
           }
       }
   }
   ```

2. **Implement cut mode**
   - Slice mesh at Z=0 plane
   - Return 2D outline

3. **Implement shadow mode (no cut)**
   - Project all triangles onto XY plane
   - Take union of projected shapes

4. **Write tests**

**Acceptance Criteria**:
- ✅ Projection cut mode works
- ✅ Projection shadow mode works
- ✅ Tests pass

**Effort**: 12-16 hours

---

## Task 4.10: Surface Import (Optional/Deferred)

**Description**: Implement surface() for importing height map files.

**Why**: OpenSCAD surface() operation for terrain/height maps.

**Decision**: **DEFER to future phase** - not critical for MVP.

**Future Subtasks**:
1. Parse DAT/PNG height map files
2. Convert to 3D mesh
3. Handle center, convexity parameters

**Effort**: 12-16 hours - DEFERRED

---

## Phase 4 Complete When:

- [ ] CrossSection data structure implemented
- [ ] 2D primitives work (circle, square, polygon)
- [ ] Offset operation works
- [ ] Polygon triangulation works
- [ ] Linear extrusion works (with twist and scale)
- [ ] Rotate extrusion works
- [ ] Convex hull implemented
- [ ] Projection operation works
- [ ] 2D boolean operations work (if using Clipper2)
- [ ] All tests pass
- [ ] Documentation complete

**Deferred to Future**:
- [ ] text() primitive (complex, requires font rendering)
- [ ] surface() import (not critical)
- [ ] Minkowski sum (very complex, consider for later)

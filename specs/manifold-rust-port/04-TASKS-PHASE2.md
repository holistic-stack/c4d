# Phase 2: Geometric Primitives and Transformations

## Overview

Phase 2 implements all geometric primitive constructors (cube, sphere, cylinder) and transformation operations.

**Duration**: 2-3 weeks  
**Dependencies**: Phase 1

---

## Task 2.0: OpenSCAD Evaluator Design (Geometry IR)

**Description**: Design the interpreter that converts OpenSCAD AST to Manifold operations.

**Why**: Moved from Phase 5. Defining the Geometry IR early acts as the contract between the Language layer and the Geometry layer. This ensures `manifold-rs` builds exactly what `openscad-eval` needs to output.

**Subtasks**:

1. **Define Value type**
   ```rust
   // In libs/openscad-eval/src/value.rs
   #[derive(Clone, Debug)]
   pub enum Value {
       Undef,
       Bool(bool),
       Number(f64),
       String(String),
       List(Vec<Value>),
       // Geometry values are REFERENCES into the fully-evaluated geometry IR.
       Geometry(GeometryId),        // 3D geometry handle
       CrossSection(CrossSectionId),// 2D geometry handle
   }
   ```

2. **Define Geometry IR**
   ```rust
   // In libs/openscad-eval/src/geometry_ir.rs
   pub enum GeometryNode {
       Primitive { kind: PrimitiveKind, params: PrimitiveParams },
       Transform { kind: TransformKind, params: TransformParams, child: GeometryId },
       Boolean { kind: BooleanKind, children: Vec<GeometryId> },
       Special { kind: SpecialKind, params: SpecialParams, children: Vec<GeometryId> },
   }
   ```

**Acceptance Criteria**:
- ✅ GeometryIR enum defined
- ✅ Value type defined
- ✅ Contract between Evaluator and Manifold established

**Effort**: 4-6 hours

---

## Task 2.1: Cube Primitive

**Description**: Generate cube/rectangular box meshes.

**Why**: Simplest 3D primitive, validates mesh construction pipeline.

**Acceptance Criteria**:
- ✅ Generates 8 vertices, 12 triangles
- ✅ Correct winding order (CCW from outside)
- ✅ Volume calculation accurate
- ✅ Center option works
- ✅ Mesh is manifold
- ✅ Tests pass

**Implementation Details**:
- 8 vertices forming corners
- 6 faces, 2 triangles each
- Support centered or origin-based positioning
- Validate all size components are positive

**Effort**: 6-8 hours

---

## Task 2.2: Sphere Primitive

**Description**: Generate sphere meshes using icosphere subdivision.

**Why**: Common primitive, tests subdivision algorithms.

**Acceptance Criteria**:
- ✅ Volume approximates 4/3πr³ within 1%
- ✅ Surface area approximates 4πr² within 1%
- ✅ Mesh is manifold
- ✅ circular_segments parameter works
- ✅ Tests pass

**Implementation Details**:
- Start with icosahedron (12 vertices, 20 faces)
- Subdivide edges, project to sphere
- Each subdivision multiplies triangles by 4
- Support circular_segments parameter to control detail

**Effort**: 8-10 hours

---

## Task 2.3: Cylinder Primitive

**Description**: Generate cylinder/cone meshes with optional different radii.

**Why**: Essential for mechanical parts, supports tapered forms.

**Acceptance Criteria**:
- ✅ Generates correct cylinder mesh
- ✅ Supports different top/bottom radii (cone)
- ✅ Center option works
- ✅ circular_segments parameter works
- ✅ Mesh is manifold
- ✅ Tests pass

**Implementation Details**:
- Generate top and bottom circles
- Connect with side triangles
- Cap top and bottom
- Support $fn, $fa, $fs parameters (OpenSCAD compatibility)

**Effort**: 8-10 hours

---

## Task 2.4: Polyhedron Primitive

**Description**: Create arbitrary convex polyhedron from points and faces.

**Why**: OpenSCAD's general-purpose primitive for creating any convex solid.

**Subtasks**:
1. Parse points array (Vec3 vertices)
2. Parse faces array (triangle indices)
3. Validate convexity (optional, warn if non-convex)
4. Build manifold mesh from points and faces
5. Validate mesh is manifold
6. Write tests

**Acceptance Criteria**:
- ✅ Creates valid manifold from points and faces
- ✅ Handles various polyhedra (tetrahedron, octahedron, etc.)
- ✅ Mesh is manifold
- ✅ Tests pass

**Effort**: 8-12 hours

---

## Task 2.5: Translation Transform

**Description**: Implement translation transformation.

**Why**: Most basic transformation operation.

**Subtasks**:
1. Implement `translate` method on Manifold
2. Apply offset to all vertices
3. Update bounding box
4. Write tests

**Acceptance Criteria**:
- ✅ Translation works correctly
- ✅ Bounding box updates
- ✅ Multiple translations compose correctly
- ✅ Tests pass

**Effort**: 4-6 hours

---

## Task 2.6: Rotation Transform

**Description**: Implement rotation transformations (Euler angles and axis-angle).

**Why**: Essential for orienting objects.

**Subtasks**:
1. Implement rotation from Euler angles [x, y, z]
2. Implement rotation from axis-angle
3. Apply rotation matrix to vertices
4. Transform vertex properties (normals)
5. Update bounding box
6. Write tests

**Acceptance Criteria**:
- ✅ Euler angle rotation works
- ✅ Axis-angle rotation works
- ✅ Rotations compose correctly
- ✅ Tests verify correct angles

**Effort**: 6-8 hours

---

## Task 2.7: Scale Transform

**Description**: Implement non-uniform scaling.

**Why**: Essential for resizing objects.

**Subtasks**:
1. Implement `scale` method
2. Apply scale to vertices
3. Update bounding box
4. Handle negative scales (reflection)
5. Write tests

**Acceptance Criteria**:
- ✅ Uniform scaling works
- ✅ Non-uniform scaling works
- ✅ Negative scales work
- ✅ Volume scales correctly
- ✅ Tests pass

**Effort**: 4-6 hours

---

## Task 2.8: Mirror Transform

**Description**: Implement mirror/reflection transformation.

**Why**: Common CAD operation for symmetric parts.

**Subtasks**:
1. Implement `mirror` method with plane normal
2. Apply reflection matrix
3. Flip triangle winding order
4. Write tests

**Acceptance Criteria**:
- ✅ Mirroring works correctly
- ✅ Winding order corrected
- ✅ Tests pass

**Effort**: 4-6 hours

---

## Task 2.9: Resize Transform

**Description**: Implement resize transformation with optional auto aspect ratio.

**Why**: OpenSCAD resize operation for fitting objects to specific dimensions.

**Subtasks**:
1. Implement `resize` method with size and auto parameters
2. Calculate scale factors based on current bounding box
3. Handle auto parameter (maintain aspect ratios)
4. Apply scaling
5. Write tests

**Acceptance Criteria**:
- ✅ Resize to exact dimensions works
- ✅ Auto aspect ratio works
- ✅ Tests pass

**Effort**: 4-6 hours

---

## Task 2.10: Multmatrix Transform

**Description**: Implement transformation with arbitrary 4x4 matrix (OpenSCAD multmatrix).

**Why**: Allows custom transformations as in OpenSCAD.

**Subtasks**:
1. Implement `multmatrix` method
2. Apply 4x4 matrix to vertices
3. Update bounding box
4. Write tests

**Acceptance Criteria**:
- ✅ Matrix transformation works
- ✅ Compatible with OpenSCAD multmatrix format
- ✅ Tests pass

**Effort**: 4-6 hours

---

## Phase 2 Complete When:

- [ ] All primitives implemented (cube, sphere, cylinder, polyhedron)
- [ ] All transformations implemented (translate, rotate, scale, mirror, resize, multmatrix)
- [ ] Comprehensive tests for all operations
- [ ] Documentation complete
- [ ] Integration tests verify primitives + transforms work together

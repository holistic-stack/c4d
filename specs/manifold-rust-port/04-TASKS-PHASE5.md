# Phase 5: OpenSCAD Integration and Finalization

## Overview

Phase 5 creates the bridge between OpenSCAD AST and Manifold operations, plus file I/O and polish.

**Duration**: 2-3 weeks  
**Dependencies**: Phase 1-4

---

## Task 5.1: OpenSCAD Evaluator Design

**Description**: Design the interpreter that converts OpenSCAD AST to Manifold operations.

**Why**: This is the bridge that makes the entire pipeline work.

**Context**: The evaluator walks the OpenSCAD AST and:
- Evaluates expressions to values
- Executes statements to produce geometry
- Manages variable scopes
- Handles modules and functions

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
       Geometry(Manifold),  // 3D geometry
       CrossSection(CrossSection),  // 2D geometry
   }
   
   impl Value {
       pub fn as_number(&self) -> Result<f64>;
       pub fn as_vec3(&self) -> Result<Vec3>;
       pub fn as_bool(&self) -> Result<bool>;
       pub fn to_geometry(&self) -> Result<Manifold>;
   }
   ```

2. **Define evaluation context**
   ```rust
   pub struct EvalContext {
       scopes: Vec<Scope>,
       modules: HashMap<String, ModuleDef>,
       functions: HashMap<String, FunctionDef>,
       special_vars: SpecialVars,
   }
   
   struct SpecialVars {
       fn_: f64,  // $fn
       fa: f64,  // $fa
       fs: f64,  // $fs
       t: f64,   // $t (animation)
   }
   
   impl EvalContext {
       pub fn new() -> Self;
       pub fn push_scope(&mut self);
       pub fn pop_scope(&mut self);
       pub fn define_var(&mut self, name: &str, value: Value);
       pub fn lookup_var(&self, name: &str) -> Option<&Value>;
   }
   ```

3. **Design evaluator trait**
   ```rust
   pub trait Evaluator {
       fn eval_expr(&mut self, expr: &Expr, ctx: &mut EvalContext) -> Result<Value>;
       fn eval_stmt(&mut self, stmt: &Stmt, ctx: &mut EvalContext) -> Result<Vec<Manifold>>;
       fn eval_item(&mut self, item: &Item, ctx: &mut EvalContext) -> Result<Vec<Manifold>>;
   }
   ```

**Acceptance Criteria**:
- ✅ Design is complete
- ✅ Types defined
- ✅ Interfaces clear

**Effort**: 4-6 hours

---

## Task 5.2: Expression Evaluator

**Description**: Implement expression evaluation.

**Why**: Convert OpenSCAD expressions to runtime values.

**Subtasks**:

1. **Implement literal evaluation**
   ```rust
   fn eval_literal(lit: &Literal) -> Value {
       match lit {
           Literal::Integer(n, _) => Value::Number(*n as f64),
           Literal::Float(f, _) => Value::Number(*f),
           Literal::String(s, _) => Value::String(s.clone()),
           Literal::Boolean(b, _) => Value::Bool(*b),
           Literal::Undef(_) => Value::Undef,
       }
   }
   ```

2. **Implement binary operators**
   ```rust
   fn eval_binary_op(op: BinaryOp, left: Value, right: Value) -> Result<Value> {
       match (op, left, right) {
           (BinaryOp::Add, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
           (BinaryOp::Sub, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
           // ... all operators
       }
   }
   ```

3. **Implement list operations**
   - List construction
   - List indexing
   - List comprehensions
   - Range expressions

4. **Implement function calls**
   - Built-in functions (sin, cos, len, etc.)
   - User-defined functions

5. **Write tests**

**Acceptance Criteria**:
- ✅ All expression types evaluate correctly
- ✅ Operators work
- ✅ Lists work
- ✅ Function calls work
- ✅ Tests pass

**Effort**: 16-20 hours

---

## Task 5.3: Statement Evaluator - Primitives

**Description**: Evaluate primitive geometry statements (cube, sphere, cylinder).

**Why**: Core geometry generation.

**Subtasks**:

1. **Implement module call dispatcher**
   ```rust
   fn eval_module_call(call: &ModuleCall, ctx: &EvalContext) -> Result<Vec<Manifold>> {
       match call.name.name.as_str() {
           "cube" => eval_cube(call, ctx),
           "sphere" => eval_sphere(call, ctx),
           "cylinder" => eval_cylinder(call, ctx),
           _ => eval_user_module(call, ctx),
       }
   }
   ```

2. **Implement cube evaluation**
   ```rust
   fn eval_cube(call: &ModuleCall, ctx: &EvalContext) -> Result<Vec<Manifold>> {
       let size = extract_arg(call, "size", ctx)?
           .unwrap_or(Value::Number(1.0));
       let center = extract_arg(call, "center", ctx)?
           .map(|v| v.as_bool())
           .transpose()?
           .unwrap_or(false);
       
       let size_vec = size.as_vec3()?;
       Ok(vec![Manifold::cube(size_vec, center)])
   }
   ```

3. **Implement sphere evaluation with $fn, $fa, $fs**
   ```rust
   fn eval_sphere(call: &ModuleCall, ctx: &EvalContext) -> Result<Vec<Manifold>> {
       let radius = extract_radius_or_diameter(call, ctx)?;
       let segments = resolve_circular_segments(call, ctx, radius)?;
       Ok(vec![Manifold::sphere(radius, segments)])
   }
   
   fn resolve_circular_segments(call: &ModuleCall, ctx: &EvalContext, radius: f64) -> Result<usize> {
       if let Some(fn_val) = extract_arg(call, "$fn", ctx)? {
           return Ok(fn_val.as_number()? as usize);
       }
       
       // Use $fa and $fs to calculate segments
       let fa = extract_arg(call, "$fa", ctx)?
           .or(ctx.special_vars.fa)
           .unwrap_or(12.0);
       let fs = extract_arg(call, "$fs", ctx)?
           .or(ctx.special_vars.fs)
           .unwrap_or(2.0);
       
       // Calculate from fragment angle and size
       let angle_segments = (360.0 / fa).ceil() as usize;
       let size_segments = (2.0 * PI * radius / fs).ceil() as usize;
       Ok(angle_segments.min(size_segments).max(3))
   }
   ```

4. **Implement cylinder evaluation**

5. **Write tests**

**Acceptance Criteria**:
- ✅ All primitives work
- ✅ Parameters parsed correctly
- ✅ $fn, $fa, $fs work
- ✅ Tests pass

**Effort**: 12-16 hours

---

## Task 5.4: Statement Evaluator - Transformations

**Description**: Evaluate transformation statements.

**Why**: Apply transformations to geometry.

**Subtasks**:

1. **Implement transformation evaluation**
   ```rust
   fn eval_transform_chain(chain: &TransformChain, ctx: &mut EvalContext) -> Result<Vec<Manifold>> {
       let transform = parse_transform(&chain.call, ctx)?;
       let children = eval_stmt(&chain.tail, ctx)?;
       
       Ok(children.into_iter()
           .map(|m| apply_transform(m, &transform))
           .collect())
   }
   
   enum Transform {
       Translate(Vec3),
       Rotate(Vec3),  // Euler angles
       Scale(Vec3),
       Mirror(Vec3),
       Multmatrix(Mat3x4),
       Color(Vec4),  // Store but don't apply
   }
   
   fn parse_transform(call: &ModuleCall, ctx: &EvalContext) -> Result<Transform> {
       match call.name.name.as_str() {
           "translate" => {
               let v = extract_vec3(call, "v", ctx)?;
               Ok(Transform::Translate(v))
           }
           "rotate" => {
               // Handle both rotate([x,y,z]) and rotate(a, [x,y,z])
               // ...
           }
           // ... other transforms
       }
   }
   ```

2. **Handle transformation composition**
   - Multiple transforms on same object
   - Correct order of application

3. **Write tests**
   - Test each transformation
   - Test composition
   - Verify correctness

**Acceptance Criteria**:
- ✅ All transformations work
- ✅ Composition works
- ✅ Tests pass

**Effort**: 12-16 hours

---

## Task 5.5: Statement Evaluator - Booleans and Control Flow

**Description**: Evaluate boolean operations and control flow.

**Why**: Enable CSG and conditionals.

**Subtasks**:

1. **Implement union block**
   ```rust
   fn eval_union_block(block: &Block, ctx: &mut EvalContext) -> Result<Vec<Manifold>> {
       let geometries: Vec<Manifold> = block.items.iter()
           .flat_map(|item| eval_item(item, ctx))
           .collect::<Result<Vec<_>>>()?
           .into_iter()
           .flatten()
           .collect();
       
       if geometries.is_empty() {
           return Ok(vec![]);
       }
       
       Ok(vec![Manifold::batch_union(geometries)])
   }
   ```

2. **Implement difference**
   ```rust
   fn eval_difference(items: &[Item], ctx: &mut EvalContext) -> Result<Vec<Manifold>> {
       let mut iter = items.iter();
       let first = iter.next()
           .ok_or_else(|| EvalError::EmptyDifference)?;
       
       let base = eval_item(first, ctx)?
           .into_iter()
           .reduce(|a, b| a.union(&b))
           .ok_or_else(|| EvalError::EmptyGeometry)?;
       
       let subtracts: Vec<Manifold> = iter
           .flat_map(|item| eval_item(item, ctx))
           .collect::<Result<Vec<_>>>()?
           .into_iter()
           .flatten()
           .collect();
       
       let subtract_union = if subtracts.is_empty() {
           return Ok(vec![base]);
       } else {
           Manifold::batch_union(subtracts)
       };
       
       Ok(vec![base.difference(&subtract_union)])
   }
   ```

3. **Implement intersection**

4. **Implement for loops**
   ```rust
   fn eval_for_block(for_block: &ForBlock, ctx: &mut EvalContext) -> Result<Vec<Manifold>> {
       let mut results = Vec::new();
       
       match &for_block.binds {
           ForBinds::Assigns(assigns) => {
               // Iterate over ranges/lists
               for values in iterate_assignments(assigns, ctx)? {
                   ctx.push_scope();
                   for (name, value) in values {
                       ctx.define_var(&name, value);
                   }
                   let geoms = eval_stmt(&for_block.body, ctx)?;
                   results.extend(geoms);
                   ctx.pop_scope();
               }
           }
           ForBinds::CondUpdate(_) => {
               // C-style for loop
               // ...
           }
       }
       
       Ok(results)
   }
   ```

5. **Implement if/else**

6. **Write tests**

**Acceptance Criteria**:
- ✅ Boolean operations work
- ✅ For loops work
- ✅ If/else works
- ✅ Tests pass

**Effort**: 16-20 hours

---

## Task 5.6: File I/O - STL Export

**Description**: Implement STL file export.

**Why**: Most common 3D file format for 3D printing.

**Subtasks**:

1. **Implement ASCII STL writer**
   ```rust
   // In libs/manifold-rs/src/io/stl.rs
   pub fn write_stl_ascii(manifold: &Manifold, path: &Path) -> Result<()> {
       let mesh = manifold.to_mesh();
       let mut file = File::create(path)?;
       
       writeln!(file, "solid manifold")?;
       
       for tri_idx in 0..mesh.num_tri() {
           let [v0, v1, v2] = mesh.get_tri_verts(tri_idx);
           let p0 = mesh.get_vert_pos(v0 as usize);
           let p1 = mesh.get_vert_pos(v1 as usize);
           let p2 = mesh.get_vert_pos(v2 as usize);
           
           // Compute normal
           let normal = (p1 - p0).cross(&(p2 - p0)).normalize();
           
           writeln!(file, "  facet normal {} {} {}", normal.x, normal.y, normal.z)?;
           writeln!(file, "    outer loop")?;
           writeln!(file, "      vertex {} {} {}", p0.x, p0.y, p0.z)?;
           writeln!(file, "      vertex {} {} {}", p1.x, p1.y, p1.z)?;
           writeln!(file, "      vertex {} {} {}", p2.x, p2.y, p2.z)?;
           writeln!(file, "    endloop")?;
           writeln!(file, "  endfacet")?;
       }
       
       writeln!(file, "endsolid manifold")?;
       Ok(())
   }
   ```

2. **Implement binary STL writer**
   - More efficient format
   - 80-byte header
   - uint32 triangle count
   - 50 bytes per triangle

3. **Write tests**

**Acceptance Criteria**:
- ✅ ASCII STL works
- ✅ Binary STL works
- ✅ Files can be imported by standard tools
- ✅ Tests pass

**Effort**: 6-8 hours

---

## Task 5.6b: File I/O - STL Import

**Description**: Implement STL file import for OpenSCAD import() operation.

**Why**: OpenSCAD can import existing STL files.

**Subtasks**:

1. **Implement ASCII STL reader**
   ```rust
   pub fn read_stl_ascii(path: &Path) -> Result<Manifold> {
       // Parse "solid ... facet ... vertex ... endsolid"
       // Build MeshGL from vertices and triangles
       // Convert to Manifold
   }
   ```

2. **Implement binary STL reader**
   - Parse binary format
   - More efficient than ASCII

3. **Auto-detect format**
   ```rust
   pub fn import_stl(path: &Path) -> Result<Manifold> {
       // Check first bytes to determine ASCII vs binary
       // Call appropriate reader
   }
   ```

4. **Write tests**

**Acceptance Criteria**:
- ✅ Can read ASCII STL files
- ✅ Can read binary STL files
- ✅ Auto-detection works
- ✅ Tests pass

**Effort**: 8-12 hours

---

## Task 5.6c: Special Operations - render() and import()

**Description**: Implement render() and import() OpenSCAD operations.

**Why**: Complete OpenSCAD language support.

**Subtasks**:

1. **Implement render() evaluation**
   ```rust
   fn eval_render(call: &ModuleCall, ctx: &mut EvalContext) -> Result<Vec<Manifold>> {
       // In our implementation, render() doesn't need to do anything special
       // We always fully evaluate geometry
       // Just evaluate children
       eval_children(call, ctx)
   }
   ```
   - OpenSCAD render() forces CGAL evaluation
   - In our case, we always produce manifold meshes
   - render() becomes a no-op, just evaluate children

2. **Implement import() evaluation**
   ```rust
   fn eval_import(call: &ModuleCall, ctx: &EvalContext) -> Result<Vec<Manifold>> {
       let file = extract_arg(call, "file", ctx)?
           .ok_or(EvalError::MissingArg("file"))?
           .as_string()?;
       
       let path = Path::new(&file);
       let ext = path.extension()
           .and_then(|e| e.to_str())
           .ok_or(EvalError::UnknownFileFormat)?;
       
       match ext.to_lowercase().as_str() {
           "stl" => Ok(vec![import_stl(path)?]),
           "off" => Ok(vec![import_off(path)?]),  // Optional
           _ => Err(EvalError::UnsupportedFileFormat(ext.to_string())),
       }
   }
   ```

3. **Write tests**

**Acceptance Criteria**:
- ✅ render() evaluates correctly
- ✅ import() loads STL files
- ✅ Tests pass

**Effort**: 4-6 hours

---

## Task 5.7: Integration Testing

**Description**: Test complete OpenSCAD → Manifold → STL pipeline.

**Why**: Verify everything works together.

**Subtasks**:

1. **Create end-to-end tests**
   ```rust
   #[test]
   fn test_simple_cube() {
       let source = "cube([2, 3, 4]);";
       let ast = openscad_ast::parse(source).unwrap();
       let mut eval = Evaluator::new();
       let geometries = eval.evaluate(&ast).unwrap();
       
       assert_eq!(geometries.len(), 1);
       let cube = &geometries[0];
       assert_eq!(cube.volume(), 24.0);
   }
   
   #[test]
   fn test_union() {
       let source = r#"
           union() {
               cube([1, 1, 1]);
               translate([0.5, 0, 0])
                   cube([1, 1, 1]);
           }
       "#;
       // ...
   }
   ```

2. **Test complex examples**
   - Difference with cylinder (hole)
   - For loop with rotations
   - Extrusion examples
   - Nested transformations

3. **Performance tests**
   - Measure time for various operations
   - Profile hot paths

**Acceptance Criteria**:
- ✅ Can parse and render example OpenSCAD files
- ✅ Output matches expected geometry
- ✅ Performance is acceptable
- ✅ Tests pass

**Effort**: 12-16 hours

---

## Task 5.8: Documentation and Examples

**Description**: Create comprehensive documentation and examples.

**Why**: Make the library usable.

**Subtasks**:

1. **Write API documentation**
   - Document all public types and methods
   - Include examples in doc comments
   - Generate docs with `cargo doc`

2. **Create examples directory**
   - Simple primitives
   - Boolean operations
   - Transformations
   - Extrusion
   - Complex models

3. **Write usage guide**
   - Getting started
   - Basic concepts
   - API reference
   - Performance tips

**Acceptance Criteria**:
- ✅ All public APIs documented
- ✅ Examples compile and run
- ✅ Guide is clear and complete

**Effort**: 12-16 hours

---

## Phase 5 Complete When:

- [ ] Evaluator fully implemented
- [ ] All OpenSCAD operations supported
- [ ] File I/O works
- [ ] Integration tests pass
- [ ] Documentation complete
- [ ] Ready for production use

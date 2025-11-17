/**
 * @file ast-types.ts
 * @description This file defines all Abstract Syntax Tree (AST) node types for the OpenSCAD parser.
 * These types provide a structured, semantic representation of OpenSCAD code, which is more abstract
 * and easier to work with than the Concrete Syntax Tree (CST) from Tree-sitter.
 *
 * @architectural_decision
 * The AST types are designed to be a clean, language-agnostic representation of OpenSCAD constructs.
 * Each node type is a separate interface, promoting type safety and making the AST easier to traverse and manipulate.
 * All nodes share a `BaseNode` interface, ensuring that essential properties like `type` and `location` are always present.
 * This structured approach is fundamental for all downstream processing, including validation, transformation, and 3D model generation.
 * The types are intentionally kept separate from the Babylon.js-specific nodes to maintain a clean separation of concerns
 * between parsing and rendering, as outlined in the `refactory-architecture.md` document.
 */

/**
 * @interface BaseNode
 * @description The fundamental interface for all AST nodes in the OpenSCAD parser.
 * It ensures that every node has a `type` identifier and an optional `location` in the source code.
 * This common structure is crucial for polymorphic processing of the AST.
 *
 * @property {string} type - A string literal that uniquely identifies the type of the node (e.g., 'cube', 'translate').
 * @property {SourceLocation} [location] - An optional object containing the node's start and end position in the source file.
 *
 * @example
 * ```typescript
 * const genericNode: BaseNode = {
 *   type: 'generic',
 *   location: {
 *     start: { line: 0, column: 0, offset: 0 },
 *     end: { line: 0, column: 10, offset: 10 },
 *     text: 'some_code'
 *   }
 * };
 * ```
 */
export interface BaseNode {
  type: string;
  location?: SourceLocation;
}

/**
 * @interface SourceLocation
 * @description Represents the range of a node in the source code, from its start to its end position.
 * This information is vital for error reporting, enabling tools to highlight the exact code causing an issue.
 *
 * @property {Position} start - The starting position of the node.
 * @property {Position} end - The ending position of the node.
 * @property {string} [text] - The original source code text corresponding to this location. Useful for debugging.
 *
 * @example
 * ```typescript
 * const location: SourceLocation = {
 *   start: { line: 1, column: 0, offset: 12 },
 *   end: { line: 1, column: 8, offset: 20 },
 *   text: 'cube(5);'
 * };
 * ```
 */
export interface SourceLocation {
  start: Position;
  end: Position;
  text?: string;
}

/**
 * @interface Position
 * @description Represents a specific point in the source code, with line, column, and offset.
 * All coordinates are zero-based, following the common convention in developer tools.
 *
 * @property {number} line - The 0-based line number.
 * @property {number} column - The 0-based column number within the line.
 * @property {number} offset - The 0-based character offset from the beginning of the file.
 *
 * @example
 * ```typescript
 * // Position of the 'c' in 'cube(10);'
 * const pos: Position = { line: 0, column: 0, offset: 0 };
 * ```
 */
export interface Position {
  line: number;
  column: number;
  offset: number;
}

/**
 * @type Vector3D
 * @description Represents a 3D vector as a tuple of three numbers `[x, y, z]`.
 */
export type Vector3D = [number, number, number];

/**
 * @type Vector2D
 * @description Represents a 2D vector as a tuple of two numbers `[x, y]`.
 */
export type Vector2D = [number, number];

/**
 * @type Vector4D
 * @description Represents a 4D vector as a tuple of four numbers `[x, y, z, w]`.
 * Often used for colors (RGBA) or homogeneous coordinates.
 */
export type Vector4D = [number, number, number, number];

/**
 * @interface Value
 * @description A generic interface representing a value extracted from the parser,
 * which can be a literal or a more complex type like a vector or range.
 */
export interface Value {
  type: 'number' | 'boolean' | 'string' | 'identifier' | 'vector' | 'range';
  value: string | number | boolean | Value[];
  start?: string;
  end?: string;
  step?: string;
}

/**
 * @interface VectorValue
 * @description Represents a vector value, which is an array of other `Value` objects.
 */
export interface VectorValue extends Value {
  type: 'vector';
  value: Value[];
}

/**
 * @interface RangeValue
 * @description Represents a range value with start, end, and optional step.
 */
export interface RangeValue extends Value {
  type: 'range';
  start?: string;
  end?: string;
  step?: string;
}

/**
 * @type ParameterValue
 * @description A union type representing the possible values of a parameter in OpenSCAD.
 * It can be a primitive literal, a vector, an expression, or an error.
 */
export type ParameterValue =
  | number
  | boolean
  | string
  | Vector2D
  | Vector3D
  | ExpressionNode
  | ErrorNode
  | null
  | undefined;

/**
 * @type BinaryOperator
 * @description Union type for all binary operators supported in OpenSCAD expressions.
 */
export type BinaryOperator =
  | '+'
  | '-'
  | '*'
  | '/'
  | '%'
  | '=='
  | '!='
  | '<'
  | '>'
  | '<='
  | '>='
  | '&&'
  | '||'
  | '&'
  | '|'
  | '^'
  | '<<'
  | '>>';

/**
 * @type UnaryOperator
 * @description Union type for all unary operators supported in OpenSCAD expressions.
 */
export type UnaryOperator = '+' | '-' | '!' | '~';

/**
 * @interface Parameter
 * @description Represents a parameter passed to a module or function, with an optional name.
 *
 * @property {string | undefined} name - The name of the parameter (for named arguments).
 * @property {ParameterValue} value - The value of the parameter.
 */
export interface Parameter {
  name: string | undefined;
  value: ParameterValue;
}

/**
 * @interface ExpressionNode
 * @description The base interface for all expression nodes in the AST.
 * Expressions are parts of the code that evaluate to a value.
 */
export interface ExpressionNode extends BaseNode {
  type: 'expression';
  expressionType: string;
  value?: number | string | boolean;
  name?: string;
  operator?: BinaryOperator | UnaryOperator | string;
  left?: ExpressionNode;
  right?: ExpressionNode;
  condition?: ExpressionNode;
  thenBranch?: ExpressionNode;
  elseBranch?: ExpressionNode;
  items?: ExpressionNode[];
  operand?: ExpressionNode;
  args?: Parameter[];
}

/**
 * @interface LiteralNode
 * @description Represents a literal value in the AST (number, string, boolean).
 */
export interface LiteralNode extends ExpressionNode {
  expressionType: 'literal';
  value: number | string | boolean;
}

/**
 * @interface LiteralExpressionNode
 * @description Alias for LiteralNode to maintain compatibility with existing code.
 * Represents a literal expression (number, string, boolean) in OpenSCAD.
 */
export interface LiteralExpressionNode extends LiteralNode {
  // Inherits all properties from LiteralNode
}

/**
 * @interface BinaryExpressionNode
 * @description Represents a binary expression with two operands and an operator.
 */
export interface BinaryExpressionNode extends ExpressionNode {
  expressionType: 'binary';
  operator: BinaryOperator;
  left: ExpressionNode;
  right: ExpressionNode;
}

/**
 * @interface UnaryExpressionNode
 * @description Represents a unary expression with one operand and an operator.
 */
export interface UnaryExpressionNode extends ExpressionNode {
  expressionType: 'unary';
  operator: UnaryOperator;
  operand: ExpressionNode;
}

/**
 * @interface ConditionalExpressionNode
 * @description Represents a conditional (ternary) expression.
 */
export interface ConditionalExpressionNode extends ExpressionNode {
  expressionType: 'conditional';
  condition: ExpressionNode;
  thenBranch: ExpressionNode;
  elseBranch: ExpressionNode;
}

/**
 * @interface ArrayExpressionNode
 * @description Represents an array expression (vector literal).
 */
export interface ArrayExpressionNode extends ExpressionNode {
  expressionType: 'array';
  items: ExpressionNode[];
}

/**
 * @interface IdentifierNode
 * @description Represents an identifier (variable name, function name, etc.).
 */
export interface IdentifierNode extends ExpressionNode {
  expressionType: 'identifier';
  name: string;
}

/**
 * @interface FunctionCallNode
 * @description Represents a function call expression.
 */
export interface FunctionCallNode extends ExpressionNode {
  expressionType: 'function_call';
  functionName: string;
  args: Parameter[];
}

/**
 * @interface RangeExpressionNode
 * @description Represents a range expression like [start:step:end] or [start:end].
 */
export interface RangeExpressionNode extends ExpressionNode {
  expressionType: 'range_expression';
  start: ExpressionNode;
  end: ExpressionNode;
  step?: ExpressionNode;
}

/**
 * @interface LetExpressionNode
 * @description Represents a let expression with variable assignments and body.
 */
export interface LetExpressionNode extends ExpressionNode {
  expressionType: 'let_expression';
  assignments: AssignmentNode[];
  expression: ExpressionNode;
}

/**
 * @interface AssignmentNode
 * @description Represents a variable assignment in a let expression.
 */
export interface AssignmentNode extends BaseNode {
  type: 'assignment';
  variable: IdentifierNode;
  value: ExpressionNode;
}

/**
 * @interface VectorExpressionNode
 * @description Represents a vector expression (same as ArrayExpressionNode but different semantic context).
 */
export interface VectorExpressionNode extends ExpressionNode {
  expressionType: 'vector';
  elements: ExpressionNode[];
}

// ... and so on for all the other interfaces and types ...

/**
 * @interface CubeNode
 * @description Represents a `cube()` primitive in OpenSCAD.
 *
 * @property {'cube'} type - The node type.
 * @property {ParameterValue} size - The size of the cube, which can be a single number or a 3D vector.
 * @property {boolean} [center] - Whether the cube is centered at the origin.
 */
export interface CubeNode extends BaseNode {
  type: 'cube';
  size: ParameterValue;
  center?: boolean;
}

/**
 * @interface SphereNode
 * @description Represents a `sphere()` primitive in OpenSCAD.
 *
 * @property {'sphere'} type - The node type.
 * @property {number} [r] - The radius of the sphere.
 * @property {number} [d] - The diameter of the sphere.
 * @property {number} [radius] - Alternative radius property for compatibility.
 * @property {number} [diameter] - Alternative diameter property for compatibility.
 * @property {number} [$fn] - Number of fragments for sphere resolution.
 * @property {number} [$fa] - Fragment angle for sphere resolution.
 * @property {number} [$fs] - Fragment size for sphere resolution.
 */
export interface SphereNode extends BaseNode {
  type: 'sphere';
  r?: number | string; // Support parameter references
  d?: number | string; // Support parameter references
  radius?: number | string; // Support parameter references
  diameter?: number | string; // Support parameter references
  $fn?: number;
  $fa?: number;
  $fs?: number;
}

/**
 * @interface CylinderNode
 * @description Represents a `cylinder()` primitive in OpenSCAD.
 *
 * @property {'cylinder'} type - The node type.
 * @property {number | string} h - The height of the cylinder. Can be a parameter reference.
 * @property {number | string} [r] - The radius of the cylinder. Can be a parameter reference.
 * @property {number | string} [r1] - The bottom radius of the cylinder. Can be a parameter reference.
 * @property {number | string} [r2] - The top radius of the cylinder. Can be a parameter reference.
 * @property {number | string} [d] - The diameter of the cylinder. Can be a parameter reference.
 * @property {number | string} [d1] - The bottom diameter of the cylinder. Can be a parameter reference.
 * @property {number | string} [d2] - The top diameter of the cylinder. Can be a parameter reference.
 * @property {boolean} [center] - Whether the cylinder is centered at the origin.
 * @property {number} [$fn] - Number of fragments for cylinder resolution.
 * @property {number} [$fa] - Fragment angle for cylinder resolution.
 * @property {number} [$fs] - Fragment size for cylinder resolution.
 */
export interface CylinderNode extends BaseNode {
  type: 'cylinder';
  h: number | string;
  r?: number | string;
  r1?: number | string;
  r2?: number | string;
  d?: number | string;
  d1?: number | string;
  d2?: number | string;
  center?: boolean;
  $fn?: number;
  $fa?: number;
  $fs?: number;
}

/**
 * @interface CircleNode
 * @description Represents a `circle()` 2D primitive in OpenSCAD.
 *
 * @property {'circle'} type - The node type.
 * @property {number} [r] - The radius of the circle.
 * @property {number} [d] - The diameter of the circle.
 * @property {number} [$fn] - Number of fragments for circle resolution.
 * @property {number} [$fa] - Fragment angle for circle resolution.
 * @property {number} [$fs] - Fragment size for circle resolution.
 */
export interface CircleNode extends BaseNode {
  type: 'circle';
  r?: number;
  d?: number;
  $fn?: number;
  $fa?: number;
  $fs?: number;
}

/**
 * @interface SquareNode
 * @description Represents a `square()` 2D primitive in OpenSCAD.
 *
 * @property {'square'} type - The node type.
 * @property {ParameterValue} size - The size of the square, which can be a single number or a 2D vector.
 * @property {boolean} [center] - Whether the square is centered at the origin.
 */
export interface SquareNode extends BaseNode {
  type: 'square';
  size: ParameterValue;
  center?: boolean;
}

/**
 * @interface PolygonNode
 * @description Represents a `polygon()` 2D primitive in OpenSCAD.
 *
 * @property {'polygon'} type - The node type.
 * @property {number[][]} points - The vertex points of the polygon.
 * @property {number[][]} [paths] - The path indices for the polygon.
 * @property {number} [convexity] - The convexity parameter.
 */
export interface PolygonNode extends BaseNode {
  type: 'polygon';
  points: number[][];
  paths?: number[][];
  convexity?: number;
}

/**
 * @interface TextNode
 * @description Represents a `text()` 2D primitive in OpenSCAD.
 *
 * @property {'text'} type - The node type.
 * @property {string} text - The text string to render.
 * @property {number} [size] - The font size.
 * @property {string} [font] - The font name.
 * @property {string} [halign] - Horizontal alignment: "left", "center", "right".
 * @property {string} [valign] - Vertical alignment: "top", "center", "baseline", "bottom".
 * @property {number} [spacing] - Character spacing.
 * @property {string} [direction] - Text direction: "ltr", "rtl", "ttb", "btt".
 * @property {string} [language] - Language for text rendering.
 * @property {string} [script] - Script for text rendering.
 * @property {number} [$fn] - Number of fragments for text resolution.
 */
export interface TextNode extends BaseNode {
  type: 'text';
  text: string;
  size?: number;
  font?: string;
  halign?: string;
  valign?: string;
  spacing?: number;
  direction?: string;
  language?: string;
  script?: string;
  $fn?: number;
}

/**
 * @interface VariableNode
 * @description Represents a variable reference in OpenSCAD code.
 *
 * @property {'variable'} type - The node type.
 * @property {string} name - The variable name.
 * @property {ExpressionNode} [value] - The variable value (for assignments).
 */
export interface VariableNode extends BaseNode {
  type: 'variable';
  name: string;
  value?: ExpressionNode;
}

/**
 * @interface SpecialVariableNode
 * @description Represents special variables in OpenSCAD like $fn, $fa, $fs, $t, etc.
 * @property {'special_variable'} type - The node type.
 * @property {string} name - The special variable name (e.g., '$fn', '$fa', '$fs').
 * @property {number | ExpressionNode} [value] - The variable value if assigned.
 */
export interface SpecialVariableNode extends BaseNode {
  type: 'special_variable';
  name: string;
  value?: number | ExpressionNode;
}

/**
 * @interface ParenthesizedExpressionNode
 * @description Represents a parenthesized expression in OpenSCAD.
 * @property {'parenthesized_expression'} type - The node type.
 * @property {ExpressionNode} expression - The wrapped expression.
 */
export interface ParenthesizedExpressionNode extends BaseNode {
  type: 'parenthesized_expression';
  expression: ExpressionNode;
}

/**
 * @interface ForLoopVariable
 * @description Represents a for loop variable in OpenSCAD.
 * @property {'for_loop_variable'} type - The node type.
 * @property {string} name - The variable name.
 * @property {ExpressionNode} iterable - The expression to iterate over.
 */
export interface ForLoopVariable extends BaseNode {
  type: 'for_loop_variable';
  name: string;
  iterable: ExpressionNode;
}

/**
 * @interface IdentifierExpressionNode
 * @description Represents an identifier expression in OpenSCAD.
 * @property {'identifier_expression'} type - The node type.
 * @property {string} name - The identifier name.
 */
export interface IdentifierExpressionNode extends BaseNode {
  type: 'identifier_expression';
  name: string;
}

/**
 * @interface ErrorHandler
 * @description Interface for handling errors during AST processing.
 * @property {(message: string, context?: unknown) => void} handleError - Method to handle errors.
 * @property {(message: string, context?: unknown) => void} handleWarning - Method to handle warnings.
 */
export interface ErrorHandler {
  readonly handleError: (message: string, context?: unknown) => void;
  readonly handleWarning: (message: string, context?: unknown) => void;
}

/**
 * @interface TranslateNode
 * @description Represents a `translate()` transformation in OpenSCAD.
 *
 * @property {'translate'} type - The node type.
 * @property {number[] | VectorExpressionNode} v - The translation vector [x, y, z] or a vector expression with identifiers.
 * @property {ASTNode[]} [children] - The child nodes to transform.
 */
export interface TranslateNode extends BaseNode {
  type: 'translate';
  v: number[] | VectorExpressionNode;
  children?: ASTNode[];
}

/**
 * @interface RotateNode
 * @description Represents a `rotate()` transformation in OpenSCAD.
 *
 * @property {'rotate'} type - The node type.
 * @property {number[]} v - The rotation vector [x, y, z] or angle.
 * @property {number} [a] - The rotation angle (when v is an axis).
 * @property {ASTNode[]} [children] - The child nodes to transform.
 */
export interface RotateNode extends BaseNode {
  type: 'rotate';
  v: number[];
  a?: number;
  children?: ASTNode[];
}

/**
 * @interface ScaleNode
 * @description Represents a `scale()` transformation in OpenSCAD.
 *
 * @property {'scale'} type - The node type.
 * @property {number[]} v - The scale factors [x, y, z].
 * @property {ASTNode[]} [children] - The child nodes to transform.
 */
export interface ScaleNode extends BaseNode {
  type: 'scale';
  v: number[];
  children?: ASTNode[];
}

/**
 * @interface MirrorNode
 * @description Represents a `mirror()` transformation in OpenSCAD.
 *
 * @property {'mirror'} type - The node type.
 * @property {number[]} v - The mirror plane normal vector [x, y, z].
 * @property {ASTNode[]} [children] - The child nodes to transform.
 */
export interface MirrorNode extends BaseNode {
  type: 'mirror';
  v: number[];
  children?: ASTNode[];
}

/**
 * @interface MultmatrixNode
 * @description Represents a `multmatrix()` transformation in OpenSCAD.
 *
 * @property {'multmatrix'} type - The node type.
 * @property {number[][]} m - The transformation matrix.
 * @property {ASTNode[]} [children] - The child nodes to transform.
 */
export interface MultmatrixNode extends BaseNode {
  type: 'multmatrix';
  m: number[][];
  children?: ASTNode[];
}

/**
 * @interface OffsetNode
 * @description Represents an `offset()` operation in OpenSCAD.
 *
 * @property {'offset'} type - The node type.
 * @property {number} r - The offset radius.
 * @property {number} [delta] - The offset delta.
 * @property {boolean} [chamfer] - Whether to use chamfer.
 * @property {ASTNode[]} [children] - The child nodes to offset.
 */
export interface OffsetNode extends BaseNode {
  type: 'offset';
  r: number;
  delta?: number;
  chamfer?: boolean;
  children?: ASTNode[];
}

/**
 * @interface ForNode
 * @description Represents a `for` loop in OpenSCAD.
 * @property {'for'} type - The node type.
 * @property {string} variable - The loop variable name.
 * @property {ExpressionNode} range - The range expression to iterate over.
 * @property {ASTNode[]} children - The child nodes to execute in each iteration.
 */
export interface ForNode extends BaseNode {
  type: 'for';
  variable: string;
  range: ExpressionNode;
  children: ASTNode[];
}

/**
 * @interface IfNode
 * @description Represents an `if` conditional statement in OpenSCAD.
 * @property {'if'} type - The node type.
 * @property {ExpressionNode} condition - The condition expression.
 * @property {ASTNode[]} children - The child nodes to execute if condition is true.
 */
export interface IfNode extends BaseNode {
  type: 'if';
  condition: ExpressionNode;
  children: ASTNode[];
}

/**
 * @interface LetNode
 * @description Represents a `let` variable assignment in OpenSCAD.
 * @property {'let'} type - The node type.
 * @property {{ [key: string]: ParameterValue }} assignments - The variable assignments as a dictionary.
 * @property {ASTNode[]} body - The child nodes with the new variable scope.
 */
export interface LetNode extends BaseNode {
  type: 'let';
  assignments: { [key: string]: ParameterValue };
  body: ASTNode[];
}

/**
 * @interface EachNode
 * @description Represents an `each` statement in OpenSCAD for iterating over collections.
 * @property {'each'} type - The node type.
 * @property {string} variable - The iteration variable name.
 * @property {ExpressionNode} collection - The collection to iterate over.
 * @property {ASTNode} expression - The expression to evaluate for each iteration.
 * @property {ASTNode[]} children - The child nodes to execute for each iteration.
 */
export interface EachNode extends BaseNode {
  type: 'each';
  variable?: string;
  collection?: ExpressionNode;
  expression?: ASTNode;
  children?: ASTNode[];
}

/**
 * @interface UnionNode
 * @description Represents a `union()` CSG operation in OpenSCAD.
 * @property {'union'} type - The node type.
 * @property {ASTNode[]} children - The child nodes to union together.
 */
export interface UnionNode extends BaseNode {
  type: 'union';
  children: ASTNode[];
}

/**
 * @interface DifferenceNode
 * @description Represents a `difference()` CSG operation in OpenSCAD.
 * @property {'difference'} type - The node type.
 * @property {ASTNode[]} children - The child nodes, first is base, rest are subtracted.
 */
export interface DifferenceNode extends BaseNode {
  type: 'difference';
  children: ASTNode[];
}

/**
 * @interface IntersectionNode
 * @description Represents an `intersection()` CSG operation in OpenSCAD.
 * @property {'intersection'} type - The node type.
 * @property {ASTNode[]} children - The child nodes to intersect.
 */
export interface IntersectionNode extends BaseNode {
  type: 'intersection';
  children: ASTNode[];
}

/**
 * @interface LinearExtrudeNode
 * @description Represents a `linear_extrude()` operation in OpenSCAD.
 * @property {'linear_extrude'} type - The node type.
 * @property {number} height - The extrusion height.
 * @property {boolean} [center] - Whether to center the extrusion.
 * @property {number} [twist] - The twist angle.
 * @property {number | Vector2D} [scale] - The scale factor.
 * @property {number} [convexity] - The convexity parameter.
 * @property {number} [slices] - The number of slices.
 * @property {number} [$fn] - The number of fragments.
 * @property {ASTNode[]} children - The 2D shapes to extrude.
 */
export interface LinearExtrudeNode extends BaseNode {
  type: 'linear_extrude';
  height: number;
  center?: boolean;
  twist?: number;
  scale?: number | Vector2D;
  convexity?: number;
  slices?: number;
  $fn?: number;
  children: ASTNode[];
}

/**
 * @interface RotateExtrudeNode
 * @description Represents a `rotate_extrude()` operation in OpenSCAD.
 * @property {'rotate_extrude'} type - The node type.
 * @property {number} [angle] - The rotation angle (default 360).
 * @property {number} [convexity] - The convexity parameter.
 * @property {number} [$fn] - The number of fragments.
 * @property {number} [$fa] - The minimum angle.
 * @property {number} [$fs] - The minimum size.
 * @property {ASTNode[]} children - The 2D shapes to extrude.
 */
export interface RotateExtrudeNode extends BaseNode {
  type: 'rotate_extrude';
  angle?: number;
  convexity?: number;
  $fn?: number;
  $fa?: number;
  $fs?: number;
  children: ASTNode[];
}

/**
 * @interface PolyhedronNode
 * @description Represents a `polyhedron()` primitive in OpenSCAD.
 * @property {'polyhedron'} type - The node type.
 * @property {number[][]} points - The vertex points.
 * @property {number[][]} faces - The face indices.
 * @property {number} [convexity] - The convexity parameter.
 */
export interface PolyhedronNode extends BaseNode {
  type: 'polyhedron';
  points: number[][];
  faces: number[][];
  convexity?: number;
}

/**
 * @interface ColorNode
 * @description Represents a `color()` operation in OpenSCAD.
 * @property {'color'} type - The node type.
 * @property {string | number[]} color - The color value (name or RGB/RGBA array).
 * @property {string | number[]} c - Alias for color property.
 * @property {number} [alpha] - The alpha transparency value.
 * @property {ASTNode[]} children - The child nodes to color.
 */
export interface ColorNode extends BaseNode {
  type: 'color';
  color?: string | number[];
  c: string | number[];
  alpha?: number;
  children: ASTNode[];
}

/**
 * @interface ModuleNode
 * @description Represents a module definition or instantiation in OpenSCAD.
 * @property {'module'} type - The node type.
 * @property {string} name - The module name.
 * @property {ParameterValue[]} [parameters] - The module parameters.
 * @property {ASTNode[]} [children] - The child nodes (for instantiation).
 * @property {ASTNode[]} [body] - The module body (for definition).
 */
export interface ModuleNode extends BaseNode {
  type: 'module';
  name: string;
  parameters?: ParameterValue[];
  children?: ASTNode[];
  body?: ASTNode[];
}

/**
 * @interface ModuleDefinitionNode
 * @description Represents a module definition in OpenSCAD.
 */
export interface ModuleDefinitionNode extends BaseNode {
  type: 'module_definition';
  name: IdentifierNode;
  parameters?: ModuleParameter[];
  body: ASTNode[];
}

/**
 * @interface ModuleInstantiationNode
 * @description Represents a module instantiation in OpenSCAD.
 */
export interface ModuleInstantiationNode extends BaseNode {
  type: 'module_instantiation';
  name: string | IdentifierNode;
  args: Parameter[];
  children?: ASTNode[];
}

/**
 * @interface ModuleParameter
 * @description Represents a parameter in a module definition.
 */
export interface ModuleParameter extends BaseNode {
  type: 'module_parameter';
  name: string;
  defaultValue?: ASTNode;
  parameterType?: string;
}

/**
 * @interface ChildrenNode
 * @description Represents the children() statement in OpenSCAD.
 */
export interface ChildrenNode extends BaseNode {
  type: 'children';
  indices?: number[];
}

/**
 * @interface HullNode
 * @description Represents a hull() operation in OpenSCAD.
 */
export interface HullNode extends BaseNode {
  type: 'hull';
  children?: ASTNode[];
}

/**
 * @interface MinkowskiNode
 * @description Represents a minkowski() operation in OpenSCAD.
 */
export interface MinkowskiNode extends BaseNode {
  type: 'minkowski';
  children?: ASTNode[];
}

/**
 * @interface FunctionDefinitionNode
 * @description Represents a function definition in OpenSCAD.
 */
export interface FunctionDefinitionNode extends BaseNode {
  type: 'function_definition';
  name: IdentifierNode;
  parameters?: ModuleParameter[];
  expression: ExpressionNode;
}

/**
 * @interface ForLoopNode
 * @description Represents a for loop statement in OpenSCAD.
 */
export interface ForLoopNode extends BaseNode {
  type: 'for_loop';
  variable: string;
  range: ExpressionNode;
  body: ASTNode[];
}

/**
 * @interface AssertStatementNode
 * @description Represents an assert statement in OpenSCAD.
 */
export interface AssertStatementNode extends BaseNode {
  type: 'assert_statement';
  condition: ExpressionNode;
  message?: ExpressionNode;
}

/**
 * @interface EchoStatementNode
 * @description Represents an echo statement in OpenSCAD.
 */
export interface EchoStatementNode extends BaseNode {
  type: 'echo_statement';
  arguments: ExpressionNode[];
}

/**
 * @interface AssignStatementNode
 * @description Represents an assignment statement in OpenSCAD.
 */
export interface AssignStatementNode extends BaseNode {
  type: 'assign_statement';
  variable: string;
  value: ExpressionNode;
}

/**
 * @interface ErrorNode
 * @description Represents an error in the AST. This is a placeholder for parts of the code that could not be parsed correctly.
 * @property {'error'} type - The node type.
 * @property {string} message - A description of the error.
 * @property {string} [errorCode] - A structured error code for categorization.
 * @property {string} [cstNodeText] - The original CST node text that caused the error.
 * @property {ErrorNode} [cause] - The underlying error that caused this error.
 * @property {string} [originalNodeType] - The original CST node type that caused the error.
 */
export interface ErrorNode extends BaseNode {
  type: 'error';
  message: string;
  errorCode?: string;
  cstNodeText?: string;
  cause?: ErrorNode;
  originalNodeType?: string;
}

/**
 * @type StatementNode
 * @description A union type of all possible AST nodes that can function as a statement in a block.
 * This includes control flow, assignments, module instantiations, and geometry-producing operations.
 */
export type StatementNode =
  | CubeNode
  | SphereNode
  | CylinderNode
  | CircleNode
  | SquareNode
  | PolygonNode
  | TextNode
  | PolyhedronNode
  | TranslateNode
  | RotateNode
  | ScaleNode
  | MirrorNode
  | MultmatrixNode
  | OffsetNode
  | UnionNode
  | DifferenceNode
  | IntersectionNode
  | HullNode
  | MinkowskiNode
  | LinearExtrudeNode
  | RotateExtrudeNode
  | ColorNode
  | ModuleNode
  | ModuleDefinitionNode
  | ModuleInstantiationNode
  | ModuleParameter
  | ChildrenNode
  | FunctionDefinitionNode
  | ForLoopNode
  | ForNode
  | IfNode
  | LetNode
  | EachNode
  | VariableNode
  | ExpressionNode
  | AssertStatementNode
  | EchoStatementNode
  | AssignStatementNode;

/**
 * @type ASTNode
 * @description A union type representing any possible node in the OpenSCAD AST.
 * This is the most general type for a node and is useful for functions that can operate on any part of the tree.
 */
export type ASTNode =
  | ExpressionNode
  | LiteralNode
  | LiteralExpressionNode
  | BinaryExpressionNode
  | UnaryExpressionNode
  | ConditionalExpressionNode
  | ArrayExpressionNode
  | IdentifierNode
  | FunctionCallNode
  | RangeExpressionNode
  | LetExpressionNode
  | AssignmentNode
  | VectorExpressionNode
  | SpecialVariableNode
  | ParenthesizedExpressionNode
  | ForLoopVariable
  | IdentifierExpressionNode
  | StatementNode
  | BaseNode
  | ErrorNode;

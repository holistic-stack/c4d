/**
 * # OpenSCAD Parser Module
 *
 * Provides OpenSCAD parsing using web-tree-sitter in the browser.
 *
 * ## Architecture
 *
 * ```text
 * OpenSCAD Source → web-tree-sitter → CST → JSON → Rust WASM
 * ```
 *
 * ## Usage
 *
 * ```typescript
 * import { initParser, parseToJson } from './lib/parser/openscad-parser';
 *
 * await initParser();
 * const cstJson = parseToJson('cube(10);');
 * ```
 */

import Parser from 'web-tree-sitter';

// =============================================================================
// TYPES
// =============================================================================

/**
 * CST Node - matches Rust struct for JSON serialization.
 *
 * Represents a node in the Concrete Syntax Tree.
 */
export interface CstNode {
  /** Node type from tree-sitter grammar (e.g., "module_call", "number") */
  nodeType: string;

  /** Text content of the node */
  text: string;

  /** Start byte offset in source */
  startByte: number;

  /** End byte offset in source */
  endByte: number;

  /** Start position */
  startPosition: { row: number; column: number };

  /** End position */
  endPosition: { row: number; column: number };

  /** Named child nodes */
  namedChildren: CstNode[];

  /** Whether this node has syntax errors */
  hasError: boolean;
}

/**
 * Parse result containing CST and diagnostics.
 */
export interface ParseResult {
  /** Root CST node */
  cst: CstNode;

  /** Whether parsing succeeded without errors */
  success: boolean;

  /** Error messages */
  errors: string[];

  /** Parse time in milliseconds */
  parseTimeMs: number;
}

// =============================================================================
// MODULE STATE
// =============================================================================

/** web-tree-sitter Parser instance */
let parser: Parser | null = null;

/** OpenSCAD language grammar */
let openscadLanguage: Parser.Language | null = null;

/** Initialization promise (for deduplication) */
let initPromise: Promise<void> | null = null;

// =============================================================================
// INITIALIZATION
// =============================================================================

/**
 * Initialize the tree-sitter parser.
 *
 * Loads web-tree-sitter WASM and the OpenSCAD grammar.
 * Safe to call multiple times - will only initialize once.
 *
 * @throws Error if initialization fails
 *
 * @example
 * ```typescript
 * await initParser();
 * console.log('Parser ready!');
 * ```
 */
export async function initParser(): Promise<void> {
  // Return existing promise if already initializing
  if (initPromise) {
    return initPromise;
  }

  // Already initialized
  if (parser && openscadLanguage) {
    return Promise.resolve();
  }

  initPromise = doInit();
  return initPromise;
}

/**
 * Perform actual parser initialization.
 *
 * @internal
 */
async function doInit(): Promise<void> {
  try {
    console.log('[Parser] Initializing web-tree-sitter...');

    // Initialize web-tree-sitter
    await Parser.init();

    // Create parser instance
    parser = new Parser();

    // Load OpenSCAD grammar WASM
    // Path is relative to public directory (copied by build script)
    const grammarPath = '/tree-sitter-openscad.wasm';
    console.log(`[Parser] Loading grammar from ${grammarPath}`);

    openscadLanguage = await Parser.Language.load(grammarPath);
    parser.setLanguage(openscadLanguage);

    console.log('[Parser] Initialized successfully');
  } catch (error) {
    initPromise = null;
    const message = error instanceof Error ? error.message : 'Unknown error';
    throw new Error(`Failed to initialize parser: ${message}`);
  }
}

// =============================================================================
// PARSING
// =============================================================================

/**
 * Check if parser is initialized and ready.
 *
 * @returns true if parser is ready to use
 */
export function isParserReady(): boolean {
  return parser !== null && openscadLanguage !== null;
}

/**
 * Parse OpenSCAD source code to CST.
 *
 * @param source - OpenSCAD source code
 * @returns Parse result with CST and diagnostics
 *
 * @example
 * ```typescript
 * const result = parseOpenSCAD('cube(10);');
 * if (result.success) {
 *   console.log('Parsed successfully');
 * }
 * ```
 */
export function parseOpenSCAD(source: string): ParseResult {
  if (!parser) {
    throw new Error('Parser not initialized. Call initParser() first.');
  }

  const startTime = performance.now();

  try {
    const tree = parser.parse(source);
    const rootNode = tree.rootNode;

    // Convert to CST
    const cst = convertToCst(rootNode);

    // Collect errors
    const errors: string[] = [];
    collectErrors(rootNode, errors);

    return {
      cst,
      success: !rootNode.hasError(),
      errors,
      parseTimeMs: performance.now() - startTime,
    };
  } catch (error) {
    return {
      cst: createEmptyCst(),
      success: false,
      errors: [error instanceof Error ? error.message : 'Unknown parse error'],
      parseTimeMs: performance.now() - startTime,
    };
  }
}

/**
 * Parse source and return CST as JSON string.
 *
 * This is the main function for passing CST to Rust WASM.
 *
 * @param source - OpenSCAD source code
 * @returns JSON string of CST
 *
 * @example
 * ```typescript
 * const cstJson = parseToJson('cube(10);');
 * const result = wasmModule.render_from_cst(cstJson);
 * ```
 */
export function parseToJson(source: string): string {
  const result = parseOpenSCAD(source);
  return JSON.stringify(result.cst);
}

// =============================================================================
// HELPERS
// =============================================================================

/**
 * Convert tree-sitter node to CST node.
 *
 * @param node - Tree-sitter syntax node
 * @returns CST node for JSON serialization
 */
function convertToCst(node: Parser.SyntaxNode): CstNode {
  return {
    nodeType: node.type,
    text: node.text,
    startByte: node.startIndex,
    endByte: node.endIndex,
    startPosition: {
      row: node.startPosition.row,
      column: node.startPosition.column,
    },
    endPosition: {
      row: node.endPosition.row,
      column: node.endPosition.column,
    },
    namedChildren: node.namedChildren.map(convertToCst),
    hasError: node.hasError(),
  };
}

/**
 * Collect error messages from syntax tree.
 *
 * @param node - Tree-sitter syntax node
 * @param errors - Array to collect error messages
 */
function collectErrors(node: Parser.SyntaxNode, errors: string[]): void {
  if (node.type === 'ERROR' || node.isMissing()) {
    const pos = node.startPosition;
    const type = node.isMissing() ? 'Missing' : 'Syntax error';
    errors.push(`${type} at line ${pos.row + 1}, column ${pos.column + 1}`);
  }

  for (const child of node.namedChildren) {
    collectErrors(child, errors);
  }
}

/**
 * Create an empty CST node for error cases.
 */
function createEmptyCst(): CstNode {
  return {
    nodeType: 'ERROR',
    text: '',
    startByte: 0,
    endByte: 0,
    startPosition: { row: 0, column: 0 },
    endPosition: { row: 0, column: 0 },
    namedChildren: [],
    hasError: true,
  };
}

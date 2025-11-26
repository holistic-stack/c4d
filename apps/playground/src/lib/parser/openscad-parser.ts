/**
 * # OpenSCAD Parser (Browser)
 *
 * Provides OpenSCAD parsing using web-tree-sitter in the browser.
 * This module handles initialization of tree-sitter and loading
 * the OpenSCAD grammar.
 *
 * ## Architecture
 *
 * The browser parsing pipeline:
 * 1. web-tree-sitter parses OpenSCAD source to CST
 * 2. CST is serialized and passed to Rust WASM
 * 3. Rust converts CST to AST and evaluates to mesh
 *
 * ## Usage
 *
 * ```typescript
 * import { initParser, parseOpenSCAD } from '$lib/parser/openscad-parser';
 *
 * await initParser();
 * const tree = parseOpenSCAD('cube(10);');
 * console.log(tree.rootNode.toString());
 * ```
 */

import { Parser, Language, Tree, Node } from 'web-tree-sitter';

/** Cached parser instance */
let parser: Parser | null = null;

/** Cached language instance */
let openscadLanguage: Language | null = null;

/** Promise for ongoing initialization */
let initPromise: Promise<void> | null = null;

/**
 * Parsed syntax tree result.
 */
export interface ParseResult {
	/** The root node of the syntax tree */
	tree: Tree;
	/** Any syntax errors found during parsing */
	errors: SyntaxError[];
}

/**
 * Syntax error information.
 */
export interface SyntaxError {
	/** Error message */
	message: string;
	/** Start byte offset */
	startIndex: number;
	/** End byte offset */
	endIndex: number;
	/** Start position (row, column) */
	startPosition: { row: number; column: number };
	/** End position (row, column) */
	endPosition: { row: number; column: number };
}

/**
 * Initializes the OpenSCAD parser.
 *
 * This function is idempotent - calling it multiple times will
 * return immediately after the first successful initialization.
 *
 * @throws Error if initialization fails
 *
 * @example
 * ```typescript
 * await initParser();
 * // Parser is now ready to use
 * ```
 */
export async function initParser(): Promise<void> {
	// Return if already initialized
	if (parser && openscadLanguage) {
		return;
	}

	// Return ongoing initialization if in progress
	if (initPromise) {
		return initPromise;
	}

	// Start initialization
	initPromise = doInit();

	try {
		await initPromise;
	} finally {
		initPromise = null;
	}
}

/**
 * Performs the actual initialization.
 */
async function doInit(): Promise<void> {
	// Initialize tree-sitter
	await Parser.init({
		locateFile(scriptName: string) {
			// Return the path to the WASM file
			// In development, this is served from node_modules
			// In production, it should be in the static directory
			if (scriptName === 'tree-sitter.wasm') {
				return '/tree-sitter.wasm';
			}
			return scriptName;
		}
	});

	// Create parser instance
	parser = new Parser();

	// Load the OpenSCAD language
	// The grammar WASM file should be in the static directory
	try {
		openscadLanguage = await Language.load('/tree-sitter-openscad_parser.wasm');
		parser.setLanguage(openscadLanguage);
	} catch (error) {
		// If the grammar WASM is not available, throw a helpful error
		throw new Error(
			`Failed to load OpenSCAD grammar. ` +
			`Make sure to build the grammar WASM with 'node scripts/build-grammar-wasm.js'. ` +
			`Original error: ${error instanceof Error ? error.message : String(error)}`
		);
	}
}

/**
 * Parses OpenSCAD source code.
 *
 * @param source - The OpenSCAD source code to parse
 * @returns The parse result containing the syntax tree and any errors
 * @throws Error if the parser is not initialized
 *
 * @example
 * ```typescript
 * await initParser();
 * const result = parseOpenSCAD('cube(10);');
 * console.log(result.tree.rootNode.toString());
 * ```
 */
export function parseOpenSCAD(source: string): ParseResult {
	if (!parser) {
		throw new Error('Parser not initialized. Call initParser() first.');
	}

	const tree = parser.parse(source);
	if (!tree) {
		throw new Error('Failed to parse source code');
	}
	const errors = collectErrors(tree.rootNode);

	return { tree, errors };
}

/**
 * Collects syntax errors from the parse tree.
 *
 * @param node - The root node to search for errors
 * @returns Array of syntax errors
 */
function collectErrors(node: Node): SyntaxError[] {
	const errors: SyntaxError[] = [];

	function visit(n: Node): void {
		if (n.hasError) {
			// Check if this node itself is an ERROR node
			if (n.type === 'ERROR' || n.isMissing) {
				errors.push({
					message: n.isMissing
						? `Missing ${n.type}`
						: `Syntax error: unexpected ${n.type}`,
					startIndex: n.startIndex,
					endIndex: n.endIndex,
					startPosition: n.startPosition,
					endPosition: n.endPosition
				});
			}
		}

		// Visit children
		for (let i = 0; i < n.childCount; i++) {
			const child = n.child(i);
			if (child) {
				visit(child);
			}
		}
	}

	visit(node);
	return errors;
}

/**
 * Checks if the parser is initialized.
 *
 * @returns True if the parser is ready to use
 */
export function isParserReady(): boolean {
	return parser !== null && openscadLanguage !== null;
}

/**
 * Gets the parser instance.
 *
 * @returns The parser instance
 * @throws Error if the parser is not initialized
 */
export function getParser(): Parser {
	if (!parser) {
		throw new Error('Parser not initialized. Call initParser() first.');
	}
	return parser;
}

/**
 * Serializes a syntax tree to a JSON-compatible format.
 *
 * This is used to pass the CST from JavaScript to Rust WASM.
 *
 * @param tree - The syntax tree to serialize
 * @returns Serialized tree data
 */
export function serializeTree(tree: Tree): SerializedNode {
	return serializeNode(tree.rootNode);
}

/**
 * Serialized syntax node for WASM transfer.
 */
export interface SerializedNode {
	/** Node type (e.g., 'source_file', 'module_call', 'number') */
	type: string;
	/** Node text content */
	text: string;
	/** Start byte offset */
	startIndex: number;
	/** End byte offset */
	endIndex: number;
	/** Start position */
	startPosition: { row: number; column: number };
	/** End position */
	endPosition: { row: number; column: number };
	/** Child nodes */
	children: SerializedNode[];
	/** Named children only */
	namedChildren: SerializedNode[];
	/** Whether this is a named node */
	isNamed: boolean;
	/** Field name if this node is a field */
	fieldName: string | null;
}

/**
 * Serializes a single syntax node.
 *
 * @param node - The node to serialize
 * @param fieldName - Optional field name
 * @returns Serialized node data
 */
function serializeNode(node: Node, fieldName: string | null = null): SerializedNode {
	const children: SerializedNode[] = [];
	const namedChildren: SerializedNode[] = [];

	for (let i = 0; i < node.childCount; i++) {
		const child = node.child(i);
		if (child) {
			const childFieldName = node.fieldNameForChild(i);
			const serialized = serializeNode(child, childFieldName);
			children.push(serialized);
			if (child.isNamed) {
				namedChildren.push(serialized);
			}
		}
	}

	return {
		type: node.type,
		text: node.text,
		startIndex: node.startIndex,
		endIndex: node.endIndex,
		startPosition: node.startPosition,
		endPosition: node.endPosition,
		children,
		namedChildren,
		isNamed: node.isNamed,
		fieldName
	};
}

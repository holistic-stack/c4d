let wasm;

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

let WASM_VECTOR_LEN = 0;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint32ArrayMemory0 = null;

function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

let cachedFloat32ArrayMemory0 = null;

function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}
/**
 * Initializes the WASM module.
 *
 * Call this once before using any other functions.
 * Sets up panic hooks for better error messages in debug builds.
 */
export function init() {
    wasm.init();
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
/**
 * Renders a mesh from a serialized CST (browser-safe).
 *
 * This function accepts a JSON-serialized CST from web-tree-sitter
 * and returns a mesh handle. This is the recommended API for browser use
 * as it avoids the C runtime dependencies of the native tree-sitter parser.
 *
 * # Arguments
 *
 * * `cst_json` - JSON string containing the serialized CST from web-tree-sitter
 *
 * # Returns
 *
 * A `MeshHandle` containing vertex and index buffers for rendering.
 *
 * # Errors
 *
 * Throws a JavaScript error with a `diagnostics` property if:
 * - The JSON is invalid
 * - The CST contains syntax errors
 * - Evaluation fails
 * - Mesh generation fails
 *
 * # Example (JavaScript)
 *
 * ```javascript
 * import { initParser, parseOpenSCAD, serializeTree } from './parser/openscad-parser';
 * import init, { render_from_cst } from './openscad-wasm';
 *
 * await init();
 * await initParser();
 *
 * try {
 *     const { tree, errors } = parseOpenSCAD("cube(10);");
 *     if (errors.length > 0) {
 *         console.error("Syntax errors:", errors);
 *         return;
 *     }
 *     const cst = serializeTree(tree);
 *     const mesh = render_from_cst(JSON.stringify(cst));
 *     console.log(`Vertices: ${mesh.vertex_count()}`);
 * } catch (error) {
 *     console.error("Render error:", error);
 * }
 * ```
 * @param {string} cst_json
 * @returns {MeshHandle}
 */
export function render_from_cst(cst_json) {
    const ptr0 = passStringToWasm0(cst_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.render_from_cst(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return MeshHandle.__wrap(ret[0]);
}

const DiagnosticFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_diagnostic_free(ptr >>> 0, 1));
/**
 * A diagnostic message that can be accessed from JavaScript.
 *
 * # Example (JavaScript)
 *
 * ```javascript
 * try {
 *     compile_and_render("invalid code");
 * } catch (error) {
 *     for (const diag of error.diagnostics) {
 *         console.error(`${diag.severity}: ${diag.message}`);
 *         console.error(`  at ${diag.start}..${diag.end}`);
 *         if (diag.hint) {
 *             console.error(`  hint: ${diag.hint}`);
 *         }
 *     }
 * }
 * ```
 */
export class Diagnostic {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DiagnosticFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_diagnostic_free(ptr, 0);
    }
    /**
     * Returns the end byte offset in the source.
     * @returns {number}
     */
    get end() {
        const ret = wasm.diagnostic_end(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the optional hint for fixing the issue.
     * @returns {string | undefined}
     */
    get hint() {
        const ret = wasm.diagnostic_hint(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Returns the start byte offset in the source.
     * @returns {number}
     */
    get start() {
        const ret = wasm.diagnostic_start(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the diagnostic message.
     * @returns {string}
     */
    get message() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.diagnostic_message(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Returns the severity level ("error" or "warning").
     * @returns {string}
     */
    get severity() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.diagnostic_severity(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) Diagnostic.prototype[Symbol.dispose] = Diagnostic.prototype.free;

const MeshHandleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_meshhandle_free(ptr >>> 0, 1));
/**
 * A handle to mesh data that can be accessed from JavaScript.
 *
 * Provides zero-copy access to vertex and index buffers via typed arrays.
 *
 * # Example (JavaScript)
 *
 * ```javascript
 * const mesh = compile_and_render("cube(10);");
 *
 * // Get counts
 * const vertexCount = mesh.vertex_count();
 * const triangleCount = mesh.triangle_count();
 *
 * // Get buffers for Three.js
 * const vertices = mesh.vertices();  // Float32Array
 * const indices = mesh.indices();    // Uint32Array
 *
 * // Create BufferGeometry
 * const geometry = new THREE.BufferGeometry();
 * geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
 * geometry.setIndex(new THREE.BufferAttribute(indices, 1));
 * ```
 */
export class MeshHandle {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(MeshHandle.prototype);
        obj.__wbg_ptr = ptr;
        MeshHandleFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MeshHandleFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_meshhandle_free(ptr, 0);
    }
    /**
     * Returns true if the mesh has colors.
     * @returns {boolean}
     */
    has_colors() {
        const ret = wasm.meshhandle_has_colors(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Returns true if the mesh has normals.
     * @returns {boolean}
     */
    has_normals() {
        const ret = wasm.meshhandle_has_normals(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Returns the number of vertices.
     * @returns {number}
     */
    get vertex_count() {
        const ret = wasm.meshhandle_vertex_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the number of triangles.
     * @returns {number}
     */
    get triangle_count() {
        const ret = wasm.meshhandle_triangle_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the vertex colors as a Float32Array, if available.
     *
     * Format: [r, g, b, a, r, g, b, a, ...]
     * Length: vertex_count * 4
     * @returns {Float32Array | undefined}
     */
    colors() {
        const ret = wasm.meshhandle_colors(this.__wbg_ptr);
        return ret;
    }
    /**
     * Returns the triangle indices as a Uint32Array.
     *
     * Format: [i0, i1, i2, i0, i1, i2, ...]
     * Length: triangle_count * 3
     * @returns {Uint32Array}
     */
    indices() {
        const ret = wasm.meshhandle_indices(this.__wbg_ptr);
        return ret;
    }
    /**
     * Returns the vertex normals as a Float32Array, if available.
     *
     * Format: [nx, ny, nz, nx, ny, nz, ...]
     * Length: vertex_count * 3
     * @returns {Float32Array | undefined}
     */
    normals() {
        const ret = wasm.meshhandle_normals(this.__wbg_ptr);
        return ret;
    }
    /**
     * Returns true if the mesh is empty.
     * @returns {boolean}
     */
    is_empty() {
        const ret = wasm.meshhandle_is_empty(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Returns the vertex positions as a Float32Array.
     *
     * Format: [x, y, z, x, y, z, ...]
     * Length: vertex_count * 3
     * @returns {Float32Array}
     */
    vertices() {
        const ret = wasm.meshhandle_vertices(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) MeshHandle.prototype[Symbol.dispose] = MeshHandle.prototype.free;

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg___wbindgen_debug_string_df47ffb5e35e6763 = function(arg0, arg1) {
        const ret = debugString(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg___wbindgen_throw_b855445ff6a94295 = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_error_7534b8e9a36f1ab4 = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_new_1acc0b6eea89d040 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
        const ret = new Error();
        return ret;
    };
    imports.wbg.__wbg_new_e17d9f43105b08be = function() {
        const ret = new Array();
        return ret;
    };
    imports.wbg.__wbg_new_from_slice_7943307099c96d15 = function(arg0, arg1) {
        const ret = new Uint32Array(getArrayU32FromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_from_slice_924249b74d07c449 = function(arg0, arg1) {
        const ret = new Float32Array(getArrayF32FromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_push_df81a39d04db858c = function(arg0, arg1) {
        const ret = arg0.push(arg1);
        return ret;
    };
    imports.wbg.__wbg_set_c2abbebe8b9ebee1 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.set(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_stack_0ed75d68575b0f3c = function(arg0, arg1) {
        const ret = arg1.stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
        // Cast intrinsic for `Ref(String) -> Externref`.
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_cast_d6cd19b81560fd6e = function(arg0) {
        // Cast intrinsic for `F64 -> Externref`.
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_externrefs;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };

    return imports;
}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;

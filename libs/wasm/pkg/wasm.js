let wasm;

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
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

let WASM_VECTOR_LEN = 0;

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
/**
 * Installs a panic hook that forwards Rust panics to the browser console.
 *
 * # Examples
 * ```no_run
 * // In JavaScript: import and call once at startup.
 * // import { init_panic_hook } from "wasm";
 * // init_panic_hook();
 * ```
 */
export function init_panic_hook() {
    wasm.init_panic_hook();
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
/**
 * Compiles OpenSCAD source and renders it to a mesh.
 *
 * This is the main entry point for the pipeline. It parses the source,
 * evaluates it, and generates a mesh suitable for GPU rendering.
 *
 * # Errors
 * Returns a JavaScript error containing diagnostics if compilation fails.
 *
 * # Examples
 * ```no_run
 * // In JavaScript:
 * // try {
 * //   const mesh = await compile_and_render("cube([2, 2, 2]);");
 * //   console.log("Vertices:", mesh.vertex_count());
 * // } catch (error) {
 * //   console.error("Compilation failed:", error);
 * // }
 * ```
 * @param {string} source
 * @returns {MeshHandle}
 */
export function compile_and_render(source) {
    const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.compile_and_render(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return MeshHandle.__wrap(ret[0]);
}

/**
 * Returns the default tessellation segment count used by the geometry
 * pipeline. This is currently a thin wrapper around a shared constant.
 *
 * # Examples
 * ```
 * let segments = wasm::default_segments();
 * assert!(segments >= 3);
 * ```
 * @returns {number}
 */
export function default_segments() {
    const ret = wasm.default_segments();
    return ret >>> 0;
}

/**
 * Compiles OpenSCAD source and returns the number of geometry nodes
 * produced by the current evaluator pipeline.
 *
 * This function is the primary entry point used from JavaScript. For Rust
 * tests, prefer `compile_and_count_nodes_internal`, which exposes Rust
 * error types directly.
 *
 * # Errors
 * Returns a JavaScript error value containing a human-readable message
 * when evaluation fails.
 *
 * # Examples
 * ```no_run
 * // In JavaScript: await compile_and_count_nodes("cube(1);");
 * ```
 * @param {string} source
 * @returns {number}
 */
export function compile_and_count_nodes(source) {
    const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.compile_and_count_nodes(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0] >>> 0;
}

/**
 * Diagnostic severity for JavaScript.
 * @enum {0 | 1 | 2}
 */
export const Severity = Object.freeze({
    Error: 0, "0": "Error",
    Warning: 1, "1": "Warning",
    Info: 2, "2": "Info",
});

const DiagnosticFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_diagnostic_free(ptr >>> 0, 1));
/**
 * A diagnostic message for JavaScript.
 *
 * # Examples
 * ```no_run
 * // In JavaScript:
 * // const diag = result.diagnostics[0];
 * // console.log(diag.message());
 * // console.log(diag.start(), diag.end());
 * ```
 */
export class Diagnostic {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Diagnostic.prototype);
        obj.__wbg_ptr = ptr;
        DiagnosticFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

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
     * Returns the end position in the source.
     * @returns {number}
     */
    end() {
        const ret = wasm.diagnostic_end(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the hint, if any.
     * @returns {string | undefined}
     */
    hint() {
        const ret = wasm.diagnostic_hint(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Returns the start position in the source.
     * @returns {number}
     */
    start() {
        const ret = wasm.diagnostic_start(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the diagnostic message.
     * @returns {string}
     */
    message() {
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
     * Returns the severity of the diagnostic.
     * @returns {Severity}
     */
    severity() {
        const ret = wasm.diagnostic_severity(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) Diagnostic.prototype[Symbol.dispose] = Diagnostic.prototype.free;

const DiagnosticListFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_diagnosticlist_free(ptr >>> 0, 1));
/**
 * A collection of diagnostics.
 */
export class DiagnosticList {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DiagnosticListFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_diagnosticlist_free(ptr, 0);
    }
    /**
     * Returns a diagnostic by index.
     * @param {number} index
     * @returns {Diagnostic | undefined}
     */
    get(index) {
        const ret = wasm.diagnosticlist_get(this.__wbg_ptr, index);
        return ret === 0 ? undefined : Diagnostic.__wrap(ret);
    }
    /**
     * Returns the number of diagnostics.
     * @returns {number}
     */
    len() {
        const ret = wasm.diagnosticlist_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns true if there are no diagnostics.
     * @returns {boolean}
     */
    is_empty() {
        const ret = wasm.diagnosticlist_is_empty(this.__wbg_ptr);
        return ret !== 0;
    }
}
if (Symbol.dispose) DiagnosticList.prototype[Symbol.dispose] = DiagnosticList.prototype.free;

const MeshHandleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_meshhandle_free(ptr >>> 0, 1));
/**
 * Mesh handle returned from compilation.
 *
 * Contains vertex and index counts for the rendered mesh.
 *
 * # Examples
 * ```no_run
 * // In JavaScript:
 * // const result = await compile_and_render("cube([1, 1, 1]);");
 * // console.log(result.vertex_count(), result.triangle_count());
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
     * Returns the number of vertices in the mesh.
     * @returns {number}
     */
    vertex_count() {
        const ret = wasm.meshhandle_vertex_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the number of triangles in the mesh.
     * @returns {number}
     */
    triangle_count() {
        const ret = wasm.meshhandle_triangle_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Returns the index buffer as a Uint32Array.
     * @returns {Uint32Array}
     */
    indices() {
        const ret = wasm.meshhandle_indices(this.__wbg_ptr);
        var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Returns the vertex buffer as a Float32Array.
     * @returns {Float32Array}
     */
    vertices() {
        const ret = wasm.meshhandle_vertices(this.__wbg_ptr);
        var v1 = getArrayF32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
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
    imports.wbg.__wbg_new_8a6f238a6ece86ea = function() {
        const ret = new Error();
        return ret;
    };
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

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

/**
 * @file Typed utilities to intercept fetch calls for WebAssembly assets during tests.
 * Provides deterministic, local loading of `.wasm` files while preserving original fetch behavior for all other requests.
 */

/**
 * Maps a Wasm file URL or filename to a local filesystem path that should be returned by the interceptor.
 */
export interface WasmUrlMap {
  readonly [urlOrFile: string]: string
}

/**
 * Options to configure the Wasm fetch interceptor.
 * @property map Optional explicit URL→path map for resolving Wasm assets.
 * @property allowPassThrough When true, non-wasm requests use the original fetch.
 */
export interface WasmFetchInterceptorOptions {
  readonly map?: WasmUrlMap
  readonly allowPassThrough?: boolean
}

/**
 * Handle to manage the interceptor lifecycle and inspect call counts for assertions.
 * @property restore Restores the original global fetch.
 * @property calls Number of intercepted Wasm fetch calls.
 */
export interface WasmFetchInterceptor {
  readonly restore: () => void
  readonly getCalls: () => number
}

/**
 * Resolves the local filesystem path to use for a Wasm request.
 * - If `input` is a `file://` URL pointing at a `.wasm` file, use that file path.
 * - Else, try explicit `map` by full URL string or basename.
 * - Else, default to the built wasm at `playground/src/lib/wasm/pkg/openscad_wasm_bg.wasm`.
 * @param input The original fetch input.
 * @param map Optional explicit mapping.
 * @returns Absolute path to the local `.wasm` file.
 */
export function resolveWasmLocalPath(input: string | URL | Request, map?: WasmUrlMap): string {
  const asUrl = input instanceof URL
    ? input
    : (typeof input === 'string'
        ? safeToURL(input)
        : (input instanceof Request ? safeToURL(input.url) : null))

  const key = asUrl?.toString() ?? (typeof input === 'string' ? input : '')
  const basename = key ? path.basename(key) : ''

  if (map) {
    if (key && map[key]) return path.resolve(map[key])
    if (basename && map[basename]) return path.resolve(map[basename])
  }

  if (asUrl && asUrl.protocol === 'file:' && asUrl.pathname.endsWith('.wasm')) {
    return fileURLToPath(asUrl)
  }

  return path.resolve('src', 'lib', 'wasm', 'pkg', 'openscad_wasm_bg.wasm')
}

/**
 * Installs a global fetch interceptor that serves local `.wasm` bytes for Wasm requests only.
 * Non-wasm requests are passed through to the original fetch when `allowPassThrough` is true.
 * @param options Interceptor configuration.
 * @returns A handle with `restore()` and `calls` for assertions.
 */
export function setupWasmFetchInterceptor(options: WasmFetchInterceptorOptions = {}): WasmFetchInterceptor {
  const originalFetch = globalThis.fetch?.bind(globalThis)
  let calls = 0

  const allowPassThrough = options.allowPassThrough !== false

  function isWasmRequest(input: unknown): input is string | URL | Request {
    if (typeof input === 'string') return input.endsWith('.wasm')
    if (input instanceof URL) return input.pathname.endsWith('.wasm')
    if (input instanceof Request) return input.url.endsWith('.wasm')
    return false
  }

  if (!originalFetch) {
    throw new Error('Global fetch is not available; cannot install Wasm interceptor')
  }

  const intercepted = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
    if (isWasmRequest(input)) {
      calls += 1
      const localPath = resolveWasmLocalPath(input as any, options.map)
      const bytes = readFileSync(localPath)
      return new Response(bytes, {
        status: 200,
        headers: { 'Content-Type': 'application/wasm' }
      })
    }

    if (!allowPassThrough) {
      return new Response('Blocked by Wasm interceptor', { status: 400 })
    }

    return originalFetch(input as any, init as any)
  }

  // @ts-expect-error allow mutation of global for test-only interceptor
  globalThis.fetch = intercepted as typeof fetch

  return {
    restore: () => {
      // @ts-expect-error revert mutation of global
      globalThis.fetch = originalFetch as typeof fetch
    },
    getCalls: () => calls
  }
}

function safeToURL(value: string): URL | null {
  try {
    return new URL(value)
  } catch {
    return null
  }
}
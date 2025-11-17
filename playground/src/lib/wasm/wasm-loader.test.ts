import { beforeEach, afterEach, describe, it, expect, vi } from 'vitest'

import { setupWasmFetchInterceptor, type WasmFetchInterceptor } from '$lib/test-utils/wasm-fetch'

let interceptor: WasmFetchInterceptor

beforeEach(() => {
  interceptor = setupWasmFetchInterceptor({
    map: { 'openscad_wasm_bg.wasm': 'src/lib/wasm/pkg/openscad_wasm_bg.wasm' },
    allowPassThrough: true
  })
})

afterEach(() => {
  interceptor.restore()
  vi.resetModules()
})

describe('wasm-loader success', () => {
  it('helloFromWasm returns expected hello string', async () => {
    const { helloFromWasm } = await import('./wasm-loader')
    const msg = await helloFromWasm()
    expect(msg).toBe('Hello from Rust WASM!')
  })

  it('loadWasm returns module and memoizes fetch (single fetch)', async () => {
    const { loadWasm } = await import('./wasm-loader')

    const m1 = await loadWasm()
    const m2 = await loadWasm()

    expect(typeof m1.hello_world).toBe('function')
    expect(typeof m2.hello_world).toBe('function')

    expect(interceptor.getCalls()).toBe(1)
  })
})

describe('wasm-loader error handling', () => {
  it('throws when hello_world export is missing', async () => {
    const { loadWasm, helloFromWasm } = await import('./wasm-loader')
    const mod = await loadWasm()

    // @ts-expect-error test-only deletion to exercise error branch
    delete (mod as any).hello_world

    await expect(helloFromWasm()).rejects.toThrowError('hello_world export not found')
  })
})

describe('cleanup and isolation', () => {
  it('restores fetch and reinitializes between tests', async () => {
    const { loadWasm } = await import('./wasm-loader')
    await loadWasm()
    expect(interceptor.getCalls()).toBe(1)

    interceptor.restore()
    vi.resetModules()
    interceptor = setupWasmFetchInterceptor({ allowPassThrough: true })

    const { loadWasm: loadWasm2 } = await import('./wasm-loader')
    await loadWasm2()
    expect(interceptor.getCalls()).toBe(1)
  })

  it('memoizes across parallel loadWasm() calls', async () => {
    const { loadWasm } = await import('./wasm-loader')
    const [a, b] = await Promise.all([loadWasm(), loadWasm()])
    expect(typeof a.hello_world).toBe('function')
    expect(typeof b.hello_world).toBe('function')
    expect(interceptor.getCalls()).toBe(1)
  })
})
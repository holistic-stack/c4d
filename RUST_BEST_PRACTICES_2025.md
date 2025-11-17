# Rust WASM Best Practices - November 2025

## 🔍 Research Summary (via Perplexity)

This document summarizes the latest Rust WASM best practices as of November 2025.

---

## 🚨 Major Changes in 2025

### wasm-pack Deprecation
- **wasm-pack was officially archived in July 2025**
- No longer maintained by the Rust WASM working group
- Migration away from wasm-pack is recommended for all new projects
- Existing projects should plan long-term transition

### Recommended Replacement
Use **direct Cargo + wasm-bindgen CLI** workflow:
1. Build with `cargo build --target wasm32-unknown-unknown`
2. Post-process with `wasm-bindgen` CLI
3. Integrate with modern bundlers (Vite, esbuild, webpack)

---

## 🛠️ Core Tooling (2025)

### wasm-bindgen
- **Still the standard** for Rust ↔ JavaScript/TypeScript interop
- Enhanced TypeScript binding generation
- Automatic `.d.ts` file generation with `--typescript` flag
- Supports advanced features:
  - Async function bindings
  - Typed exceptions
  - Class-like struct exports
  - Complex type mappings

### Build Process
```bash
# 1. Build WASM with Cargo
cargo build --target wasm32-unknown-unknown --release

# 2. Generate bindings with wasm-bindgen
wasm-bindgen target/wasm32-unknown-unknown/release/your_crate.wasm \
  --out-dir pkg \
  --target web \
  --typescript
```

### Integration Options
- **Vite**: Use `vite-plugin-wasm` for seamless integration
- **Webpack**: Use `wasm-loader` or `@wasm-tool/wasm-pack-plugin`
- **Rollup**: Native ES module imports work well
- **Custom scripts**: Build automation with Node.js (see `build-wasm.js`)

---

## 📂 Project Structure Best Practices

### Recommended Layout
```
project/
├── src/
│   └── lib.rs              # Core Rust library
├── Cargo.toml              # Rust dependencies
├── web/                    # Frontend project
│   ├── src/
│   │   └── lib/
│   │       └── wasm-loader.ts  # WASM loading utility
│   └── pkg/                # Generated WASM bindings (gitignored)
└── build-wasm.js           # Build automation script
```

### Modular Architecture
- **Core business logic**: Keep in internal Rust modules
- **Public API layer**: Thin layer with `#[wasm_bindgen]` annotations
- **Minimal exports**: Only expose what's needed for JS/TS
- **Type safety**: Leverage auto-generated `.d.ts` files

---

## 🎯 TypeScript Integration

### Automatic Type Generation
```rust
// Rust side
#[wasm_bindgen]
pub struct Point {
    x: f64,
    y: f64,
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
    
    pub fn distance_to(&self, other: &Point) -> f64 {
        // implementation
    }
}
```

Generates TypeScript:
```typescript
export class Point {
  constructor(x: number, y: number);
  distance_to(other: Point): number;
  readonly x: number;
  readonly y: number;
}
```

### Import in TypeScript
```typescript
import init, { Point, greet } from './wasm/pkg/your_crate';

// Initialize WASM
await init();

// Use with full type safety
const p = new Point(0, 0);
const greeting: string = greet("World");
```

---

## 🚀 Performance Optimizations

### Cargo.toml Settings
```toml
[profile.release]
opt-level = "z"         # Optimize for size
lto = true              # Link Time Optimization
codegen-units = 1       # Better optimization
panic = "abort"         # Smaller binary
strip = true            # Remove debug symbols
```

### Additional Techniques
- **Lazy loading**: Load WASM only when needed
- **Code splitting**: Split large WASM modules
- **Streaming compilation**: Use `WebAssembly.instantiateStreaming()`
- **Worker threads**: Offload heavy computation to Web Workers

---

## 🧪 Testing Best Practices

### Unit Tests in Rust
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
```

Run with: `cargo test`

### Integration Tests in TypeScript
```typescript
import { describe, it, expect, beforeAll } from 'vitest';
import { loadWasm } from './wasm-loader';

describe('WASM Integration', () => {
  beforeAll(async () => {
    await loadWasm();
  });

  it('should work correctly', async () => {
    const wasm = await loadWasm();
    expect(wasm.add(2, 3)).toBe(5);
  });
});
```

---

## 🌐 Deployment Platforms (2025)

### Edge Computing
- **Cloudflare Workers**: Excellent WASM support, V8 isolates
- **Fastly Compute**: Native WASM execution
- **AWS Lambda@Edge**: Limited but improving WASM support
- **Deno Deploy**: First-class WASM support

### Traditional Hosting
- **Netlify**: Static + WASM bundles work seamlessly
- **Vercel**: Edge Functions support WASM
- **GitHub Pages**: Static WASM hosting
- **S3 + CloudFront**: Custom CDN setup

---

## 🎨 Framework Ecosystem

### Pure Rust Frameworks (2025)
- **Leptos**: Full-stack framework, excellent DX
- **Yew**: Mature, React-like
- **Sycamore**: Fine-grained reactivity
- **Dioxus**: Cross-platform (web, desktop, mobile)

### Hybrid Approach (Rust + JS/TS)
- **Svelte + Rust**: Lightweight, performant (this project!)
- **React + Rust**: Large ecosystem
- **Vue + Rust**: Progressive integration
- **SolidJS + Rust**: Fine-grained reactivity

---

## 📦 Essential Crates

### Core WASM
- `wasm-bindgen`: JS/TS interop
- `serde-wasm-bindgen`: Serialize complex types
- `web-sys`: Browser API bindings
- `js-sys`: JavaScript standard library

### Utilities
- `console_error_panic_hook`: Better error messages
- `wee_alloc`: Smaller memory allocator
- `getrandom`: Random number generation
- `instant`: Time measurement in WASM

---

## 🔒 Security Considerations

1. **Validate all inputs** from JavaScript
2. **Avoid panics** in production code
3. **Use Result types** for error handling
4. **Sanitize data** crossing the JS/Rust boundary
5. **Keep WASM binary minimal** to reduce attack surface

---

## 📊 Comparison: 2024 vs 2025

| Aspect | 2024 | 2025 |
|--------|------|------|
| Build Tool | wasm-pack | Cargo + wasm-bindgen CLI |
| TS Bindings | Manual or basic | Auto-generated, comprehensive |
| Edge Support | Limited | Widespread |
| Frameworks | Maturing | Production-ready |
| Tooling | Fragmented | Consolidated |
| Best Practices | Emerging | Well-established |

---

## 📚 Key Resources

### Official Documentation
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [MDN WebAssembly](https://developer.mozilla.org/en-US/docs/WebAssembly)

### Community Resources
- [Life After wasm-pack](https://nickb.dev/blog/life-after-wasm-pack-an-opinionated-deconstruction/)
- [Rust WASM Working Group](https://github.com/rustwasm)
- [Awesome WASM](https://github.com/mbasso/awesome-wasm)

### Forums & Discussion
- [Rust Users Forum - WASM](https://users.rust-lang.org/c/wasm/29)
- [r/rust on Reddit](https://reddit.com/r/rust)
- [Discord: Rust WASM](https://discord.gg/rust-lang)

---

## ✅ Migration Checklist (from wasm-pack)

- [ ] Install `wasm-bindgen-cli`
- [ ] Create custom build script (or use existing template)
- [ ] Update CI/CD pipelines
- [ ] Test WASM loading in target environment
- [ ] Verify TypeScript bindings work correctly
- [ ] Update documentation for contributors
- [ ] Remove `wasm-pack` dependency
- [ ] Update `package.json` scripts

---

## 🎯 Summary

**Key Takeaways for November 2025:**

1. ✅ Use Cargo + wasm-bindgen CLI (not wasm-pack)
2. ✅ Leverage automatic TypeScript binding generation
3. ✅ Integrate with modern bundlers (Vite, esbuild)
4. ✅ Follow modular architecture principles
5. ✅ Optimize for production with proper Cargo settings
6. ✅ Test thoroughly in both Rust and TypeScript
7. ✅ Consider edge deployment for performance
8. ✅ Keep public API minimal and type-safe

---

*Last updated: November 2025*
*Research source: Perplexity AI with multiple authoritative citations*

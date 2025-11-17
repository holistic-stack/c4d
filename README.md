# Rust WASM + Svelte 5 + TypeScript Project

A modern WASM project built with Rust, Svelte 5, TypeScript 5.9, Vite 7, and Vitest 4, following **November 2025 best practices**.

## 🎯 Project Overview

This project demonstrates:
- ✅ Rust library compiled to WebAssembly with TypeScript bindings
- ✅ Svelte 5 with runes and modern reactive patterns
- ✅ TypeScript 5.9 for type safety
- ✅ Vite 7 for fast development and building
- ✅ Vitest 4 with WASM loading support
- ✅ Post-wasm-pack architecture (wasm-pack deprecated July 2025)

## 📚 Research Summary (November 2025)

Based on latest Rust WASM best practices:

### Key Changes in 2025
- **wasm-pack has been archived** (July 2025) - No longer maintained
- Use **Cargo + wasm-bindgen CLI** directly for building
- Auto-generate TypeScript definitions with `--typescript` flag
- Modular architecture with thin public API layer

### Tooling Stack
- **wasm-bindgen**: Primary bridge between Rust and TypeScript
- **Cargo**: Build WASM with `wasm32-unknown-unknown` target
- **vite-plugin-wasm**: Seamless WASM integration in Vite
- **Direct imports**: Load WASM as ES modules

## 🚀 Getting Started

### Prerequisites

Install required tools:
```bash
# Rust and Cargo (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli (matching Cargo.toml version)
cargo install wasm-bindgen-cli

# Node.js 18+ (for Vite 7 and Vitest 4)
# Install via nvm, fnm, or official installer
```

### Installation

```bash
# Clone the repository
git clone <your-repo>
cd rust-openscad

# Install Node dependencies
cd web
npm install
```

## 🔨 Build and Development

### Build WASM Module

```bash
# From project root
node build-wasm.js

# Or with npm script from web directory
cd web
npm run build:wasm
```

The build script:
1. Compiles Rust to WASM with Cargo
2. Generates TypeScript bindings with wasm-bindgen
3. Outputs to `web/wasm/pkg/` directory

### Development Server

```bash
cd web
npm run dev
```

Visit `http://localhost:5173` to see the app.

### Production Build

```bash
cd web
npm run build        # Builds WASM + Vite bundle
npm run preview      # Preview production build
```

## 🧪 Testing

Run tests with Vitest:

```bash
cd web
npm test                    # Run all tests
npm run test:ui            # Open Vitest UI
npm run test:coverage      # Generate coverage report
```

### Test Structure

- **`wasm-loader.test.ts`**: Tests WASM loading and Rust functions
- **`App.test.ts`**: Integration tests for Svelte component logic
- **`test-setup.ts`**: Global test configuration for WASM

## 📁 Project Structure

```
rust-openscad/
├── src/
│   └── lib.rs                 # Rust library with wasm-bindgen exports
├── web/
│   ├── src/
│   │   ├── lib/
│   │   │   ├── wasm-loader.ts      # WASM loading utility
│   │   │   └── wasm-loader.test.ts # WASM tests
│   │   ├── App.svelte              # Main Svelte 5 component
│   │   ├── App.test.ts             # Component tests
│   │   ├── main.ts                 # Entry point
│   │   └── test-setup.ts           # Vitest setup
│   ├── wasm/pkg/                   # Generated WASM bindings (gitignored)
│   ├── vite.config.ts              # Vite configuration
│   ├── vitest.config.ts            # Vitest configuration
│   ├── tsconfig.json               # TypeScript config
│   └── package.json                # Node dependencies
├── build-wasm.js              # WASM build script (post wasm-pack)
├── Cargo.toml                 # Rust dependencies
└── README.md
```

## 🛠️ Technology Stack

### Rust Side
- **Rust 2021 Edition**: Modern Rust features
- **wasm-bindgen 0.2**: JS/TS interop
- **serde**: JSON serialization
- **web-sys**: Browser API access

### Frontend Side
- **Svelte 5**: Runes-based reactivity
- **TypeScript 5.9**: Latest TS features
- **Vite 7**: Next-gen frontend tooling
- **Vitest 4**: Fast unit testing
- **vite-plugin-wasm**: WASM module support

## 🎨 Features Demonstrated

### Rust Exports
1. **Simple functions**: `greet(name)`, `add(a, b)`
2. **Class-like structs**: `Point` with methods
3. **JSON processing**: `process_json(json_str)`
4. **Browser APIs**: Console logging via `web-sys`

### Svelte 5 Features
- **Runes**: `$state`, `$derived` for reactivity
- **Modern syntax**: Cleaner component structure
- **TypeScript integration**: Full type safety
- **WASM loading**: Async module initialization

### Testing
- **Unit tests**: WASM function validation
- **Integration tests**: Component logic testing
- **WASM in tests**: Dynamic loading in test environment

## 📖 Learn More

### Official Documentation
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [Svelte 5 Documentation](https://svelte.dev/docs/svelte/overview)
- [Vite Guide](https://vitejs.dev/guide/)
- [Vitest Documentation](https://vitest.dev/)

### Nov 2025 Resources
- [Life After wasm-pack](https://nickb.dev/blog/life-after-wasm-pack-an-opinionated-deconstruction/)
- Rust WASM Working Group discussions
- Svelte 5 migration guides

## ⚡ Performance Tips

1. **Release builds**: Always use `--release` for production
2. **Size optimization**: Enabled in `Cargo.toml` (LTO, strip symbols)
3. **Lazy loading**: WASM loads only when needed
4. **Type safety**: TypeScript catches errors at compile time

## 🤝 Contributing

Contributions welcome! This project demonstrates modern patterns for:
- Rust → WASM → TypeScript workflows
- Svelte 5 component architecture
- Test-driven WASM development

## 📝 License

MIT License - See LICENSE file for details

---

**Built with ❤️ using Rust, Svelte 5, and modern web technologies**

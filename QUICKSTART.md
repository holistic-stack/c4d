# 🚀 Quick Start Guide

## Installation & Setup (5 minutes)

### Step 1: Install Rust and WASM target

```bash
# If you don't have Rust installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen CLI (must match version in Cargo.toml: 0.2)
cargo install wasm-bindgen-cli --version 0.2
```

### Step 2: Install Node.js dependencies

```bash
cd web
npm install
```

### Step 3: Build WASM and run dev server

```bash
# Build WASM module (from project root or web directory)
cd ..
node build-wasm.js

# OR from web directory
cd web
npm run build:wasm

# Start development server
npm run dev
```

Visit `http://localhost:5173` 🎉

## Running Tests

```bash
cd web
npm test              # Run all tests
npm run test:ui       # Visual test UI
npm run test:coverage # Coverage report
```

## Production Build

```bash
cd web
npm run build        # Builds optimized bundle
npm run preview      # Preview production build
```

## Troubleshooting

### wasm-bindgen not found
```bash
cargo install wasm-bindgen-cli
```

### Node modules errors
```bash
cd web
rm -rf node_modules package-lock.json
npm install
```

### WASM not loading
1. Ensure you ran `node build-wasm.js` first
2. Check that `web/wasm/pkg/` directory exists
3. Try rebuilding: `npm run build:wasm`

### TypeScript errors in IDE
- These are expected until you run `npm install`
- Install dependencies, then reload your IDE/editor

## Development Workflow

1. **Edit Rust code** → Run `npm run build:wasm` → Changes reflect in browser
2. **Edit Svelte/TS** → Hot reload (automatic)
3. **Write tests** → Run `npm test` for instant feedback

## Project Structure Quick Reference

```
rust-openscad/
├── src/lib.rs              # Rust WASM library (edit exports here)
├── web/src/
│   ├── App.svelte          # Main UI component
│   ├── lib/wasm-loader.ts  # WASM loading utility
│   └── *.test.ts           # Test files
├── build-wasm.js           # WASM build script
└── web/package.json        # Dependencies
```

## Next Steps

- ✅ Modify `src/lib.rs` to add Rust functions
- ✅ Update `App.svelte` to use your functions  
- ✅ Write tests in `*.test.ts` files
- ✅ Build and deploy!

Happy coding! 🦀✨

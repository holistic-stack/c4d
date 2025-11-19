# Rust Windows Development Guide 2025

> Comprehensive best practices, dos & don'ts, and pitfalls for Rust development on Windows 11 with monorepo setups.

## Table of Contents
- [Latest Rust Best Practices for 2025](#latest-rust-best-practices-for-2025)
- [Advanced Monorepo Setup Patterns](#advanced-monorepo-setup-patterns)
- [Windows 11 Development Environment Setup](#windows-11-development-environment-setup)
- [Rust WebAssembly Development](#rust-webassembly-development)
- [Common Pitfalls and How to Avoid Them](#common-pitfalls-and-how-to-avoid-them)
- [Performance Optimization](#performance-optimization)
- [Security Best Practices](#security-best-practices)

---

## Latest Rust Best Practices for 2025

### 🚀 Language Features to Adopt

- **Expanded const generics** and compile-time function evaluation for metaprogramming
- **Next-generation trait solver** with implied bounds and improved coherence
- **SIMD intrinsics** and ABI stability for low-level optimizations
- **Pattern matching** improvements and upcoming "Macros 2.0"
- **`try_blocks!` macro** for complex error handling scenarios

### 🛠️ Modern Tooling

**Required Tools:**
```bash
# Core toolchain
rustup default stable-msvc
rustup component add rustfmt clippy rust-src

# Performance optimization
cargo install cargo-watch cargo-audit cargo-deny
```

**IDE Setup:**
- **VS Code** with rust-analyzer (essential)
- **CodeLLDB** for debugging
- **Error Lens** for inline error display
- **GitLens** for source control integration

### 📦 Dependency Management

**Dos:**
- ✅ Use workspace dependencies with `[workspace.dependencies]`
- ✅ Centralize version management at workspace level
- ✅ Prune crate features aggressively
- ✅ Regular dependency audits with `cargo audit`

**Don'ts:**
- ❌ Mix different versions of the same crate across workspace
- ❌ Enable default features unless necessary
- ❌ Use many small dependencies for simple functionality
- ❌ Ignore dependency security advisories

**Example Workspace Setup:**
```toml
# Cargo.toml (workspace root)
[workspace]
resolver = "2"
members = ["libs/*", "apps/*"]

[workspace.dependencies]
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["rt-multi-thread"] }

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

### 🧪 Testing Strategies

**Property-Based Testing:**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_commutative_property(x in any::<i32>(), y in any::<i32>()) {
        assert_eq!(x + y, y + x);
    }
}
```

**Integration Testing:**
```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration

# Benchmark tests
cargo bench
```

---

## Advanced Monorepo Setup Patterns

### 📁 Organization Structure

```
project/
├── Cargo.toml          # Workspace configuration
├── libs/               # Internal libraries
│   ├── parser/         # Domain-specific libs
│   ├── database/
│   └── utils/
├── apps/               # Applications/services
│   ├── web-api/
│   └── cli-tool/
├── tools/              # Development tools
│   ├── xtask/
│   └── scripts/
└── docs/               # Documentation
```

### ⚡ Build Optimization

**Intelligent Build Triggering:**
```yaml
# .github/workflows/ci.yml
name: CI
on:
  pull_request:
    paths:
      - 'libs/parser/**'
      - 'libs/database/**'
jobs:
  test:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Test affected crates
        run: |
          cargo test -p parser
          cargo test -p database
```

**Cache Strategy:**
```yaml
- name: Cache cargo registry
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

### 🔄 Dependency Management Across Crates

**Workspace Dependencies:**
```toml
# libs/parser/Cargo.toml
[dependencies]
utils = { path = "../utils" }
thiserror = { workspace = true }
serde = { workspace = true }
```

**Version Policies:**
- Use `cargo tree -d` to detect duplicate dependencies
- Employ `cargo-deny` to enforce version consistency
- Regular `cargo update` with cautious review

### 🚀 CI/CD Integration

**Path-Based Triggers:**
```yaml
jobs:
  detect-changes:
    outputs:
      parser: ${{ steps.changes.outputs.parser }}
      database: ${{ steps.changes.outputs.database }}
    steps:
      - uses: actions/checkout@v4
      - uses: dorny/paths-filter@v3
        id: changes
        with:
          filters: |
            parser:
              - 'libs/parser/**'
            database:
              - 'libs/database/**'
```

---

## Windows 11 Development Environment Setup

### 🎯 Core Installation

**Step 1: Install Rust Toolchain**
```powershell
# Run as Administrator
Invoke-WebRequest -Uri https://win.rustup.rs -OutFile rustup-init.exe
.\rustup-init.exe

# Select MSVC toolchain (recommended for Windows)
rustup default stable-msvc
rustup component add rust-src rustfmt clippy
```

**Step 2: Install Visual Studio Build Tools**
```powershell
# Download Visual Studio Installer
# Install "Desktop development with C++" workload
# Ensure these components are included:
# - MSVC v143 build tools
# - Windows 10/11 SDK
# - CMake tools
```

**Step 3: Configure VS Code**
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.target": "x86_64-pc-windows-msvc",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.imports.granularity.group": "module",
    "editor.formatOnSave": true,
    "files.exclude": {
        "**/target": true
    }
}
```

### ⚡ Performance Optimization

**Build Performance:**
```toml
# .cargo/config.toml
[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"  # Faster linking

[build]
# Use more threads for parallel builds
jobs = 8  # Set to number of CPU cores
```

**Alternative Linkers:**
```bash
# Install mold for faster linking (if available)
cargo install mold

# Use in .cargo/config.toml
[target.x86_64-pc-windows-msvc]
linker = "mold"
```

### 🔧 Essential Extensions

**VS Code Extensions:**
```json
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "vadimcn.vscode-lldb",
        "usernamehw.errorlens",
        "eamodio.gitlens",
        "serayuzguz.git-streaks",
        "ms-vscode.cpptools"
    ]
}
```

### 🐛 Troubleshooting Common Issues

**Issue 1: Linker Errors**
```bash
# Solution: Ensure Visual Studio Build Tools are installed
# Check PATH includes Visual Studio tools
where cl.exe
```

**Issue 2: Slow Builds**
```bash
# Enable incremental compilation
# Add to Cargo.toml:
[profile.dev]
incremental = true

# Clear cache and rebuild
cargo clean
cargo build
```

**Issue 3: IDE Performance**
```json
// .vscode/settings.json
{
    "rust-analyzer.procMacro.enable": false,
    "rust-analyzer.cargo.loadOutDirsFromCheck": false,
    "files.watcherExclude": {
        "**/target/**": true
    }
}
```

---

## Common Pitfalls and How to Avoid Them

### 🚫 Monorepo Pitfalls

**Pitfall 1: Monolithic Crate Design**
- **Problem**: Single massive crate with unclear boundaries
- **Solution**: Split into focused domain crates with clear responsibilities
- **Sign**: Long compile times, difficult onboarding

**Pitfall 2: Dependency Hell**
- **Problem**: Multiple versions of the same crate
- **Solution**: Use workspace dependencies and regular `cargo tree -d` checks
- **Prevention**:
```bash
cargo install cargo-deny
cargo deny check
```

**Pitfall 3: Inefficient CI/CD**
- **Problem**: Building everything on every change
- **Solution**: Path-based triggers and affected crate detection
- **Tool**: `dorny/paths-filter` GitHub Action

### 💻 Windows-Specific Pitfalls

**Pitfall 1: Path Separator Issues**
```rust
// ❌ Wrong (Unix-style)
let path = "src/main.rs";

// ✅ Correct (cross-platform)
use std::path::PathBuf;
let path = PathBuf::from("src").join("main.rs");
```

**Pitfall 2: Line Ending Issues**
```gitignore
# .gitattributes
* text=auto eol=crlf
*.rs text eol=lf
```

**Pitfall 3: Case Sensitivity**
```rust
// ❌ Assumes case-insensitive filesystem
use std::fs::File;
File::open("README.md")?;  // Fails if file is "readme.md"

// ✅ Case-agnostic when needed
use std::path::Path;
let path = Path::new("README.md");
```

### 🧪 Testing Pitfalls

**Pitfall 1: Ignoring Windows-Specific Tests**
```rust
// ❌ Unix-only tests
#[test]
fn test_unix_permissions() {
    // Assumes Unix permissions model
}

// ✅ Cross-platform tests
#[cfg(unix)]
#[test]
fn test_unix_permissions() {
    // Unix-specific implementation
}

#[test]
fn test_cross_platform_logic() {
    // Platform-agnostic tests
}
```

**Pitfall 2: Hardcoded Paths**
```rust
// ❌ Hardcoded Windows paths
let config_path = "C:\\Program Files\\MyApp\\config.toml";

// ✅ Environment-aware paths
use dirs::config_dir;
let config_path = config_dir()
    .unwrap_or_else(|| PathBuf::from("."))
    .join("myapp")
    .join("config.toml");
```

---

## Rust WebAssembly Development

### 🚀 Why Rust for WebAssembly in 2025

Rust has solidified its position as the premier language for WebAssembly development, offering:
- **Performance**: Up to 40% improvement over JavaScript in computationally intensive tasks
- **Safety**: Compile-time checks prevent buffer overflows, use-after-free errors, and race conditions
- **Developer Experience**: First-class tooling with `wasm-bindgen` and `wasm-pack`

**Key Use Cases:**
- High-performance web applications requiring near-native execution speed
- Interactive gaming experiences and real-time simulations
- Video and audio processing tasks
- Cryptographic operations and secure computations

### 🛠️ WebAssembly Setup for Windows 11

**Step 1: Install WASM Target**
```bash
# Add WebAssembly compilation target
rustup target add wasm32-unknown-unknown

# Verify installation
rustup target list --installed
```

**Step 2: Install Essential Tools**
```bash
# Install wasm-pack for building and bundling
cargo install wasm-pack

# Alternative: Install via npm
npm install -g wasm-pack

# Install optimization tools
# Download Binaryen from: https://github.com/WebAssembly/binaryen/releases
# Add wasm-opt to your PATH
```

**Step 3: Configure Cargo.toml**
```toml
[package]
name = "my-wasm-project"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A WebAssembly module built with Rust"
license = "MIT"

[lib]
crate-type = ["cdylib"]  # Compile to dynamic system library

[dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
console_error_panic_hook = { version = "0.1", optional = true }
wee_alloc = { version = "0.4", optional = true }

[features]
default = ["console_error_panic_hook"]
console_error_panic_hook = ["dep:console_error_panic_hook"]
```

### 🏗️ Project Structure

**Recommended WASM Project Layout:**
```
my-wasm-project/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Main WASM entry point
│   ├── utils.rs        # Utility functions
│   └── tests.rs        # Unit tests
├── pkg/                # Generated by wasm-pack
│   ├── .gitignore
│   ├── my_wasm_project.d.ts
│   ├── my_wasm_project.js
│   ├── my_wasm_project_bg.wasm
│   └── package.json
├── www/                # Web frontend
│   ├── index.html
│   ├── index.js
│   └── style.css
└── build-wasm.js       # Custom build script (optional)
```

**Basic lib.rs Example:**
```rust
use wasm_bindgen::prelude::*;

// Import the browser's `console.log` function
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Export a JavaScript-accessible function
#[wasm_bindgen]
pub fn greet(name: &str) {
    log(&format!("Hello, {}!", name));
}

// Memory-intensive computation
#[wasm_bindgen]
pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// Async operation example
#[wasm_bindgen]
pub async fn fetch_data(url: &str) -> Result<String, String> {
    let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
    response.text().await.map_err(|e| e.to_string())
}
```

### 🔨 Build Strategies

**Development Build:**
```bash
# Quick development build
wasm-pack build --dev

# For web targets
wasm-pack build --target web --dev

# For bundler targets (webpack, rollup, etc.)
wasm-pack build --target bundler --dev
```

**Production Build:**
```bash
# Optimized production build
wasm-pack build --release

# Target-specific builds
wasm-pack build --target web --release
wasm-pack build --target bundler --release
wasm-pack build --target nodejs --release
```

**Advanced Build Pipeline:**
```bash
#!/bin/bash
# build-wasm.sh

echo "Building WebAssembly module..."
wasm-pack build --target web --release

echo "Optimizing WASM file..."
wasm-opt -O4 pkg/my_wasm_project_bg.wasm -o pkg/my_wasm_project_bg_opt.wasm

echo "Replacing optimized file..."
mv pkg/my_wasm_project_bg_opt.wasm pkg/my_wasm_project_bg.wasm

echo "Build complete!"
```

### ⚡ Performance Optimization

**Binary Size Reduction:**
```rust
// Use wee_alloc for smaller memory footprint
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Optimize for size over speed
#![allow(unused_imports)]
use wee_alloc;

// Remove debug symbols in release
#[cfg(not(debug_assertions))]
#[panic_handler]
#[no_mangle]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
```

**Memory Management Best Practices:**
```rust
use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;

// ✅ Use typed arrays for efficient data transfer
#[wasm_bindgen]
pub fn process_image_data(data: &Uint8Array) -> Uint8Array {
    let len = data.length() as usize;
    let mut buffer = vec![0u8; len];

    // Copy data efficiently
    data.copy_to(&mut buffer);

    // Process in-place to avoid allocations
    for pixel in buffer.chunks_mut(4) {
        if pixel.len() == 4 {
            // Simple brightness adjustment
            pixel[0] = (pixel[0] as u16 + 10).min(255) as u8;
            pixel[1] = (pixel[1] as u16 + 10).min(255) as u8;
            pixel[2] = (pixel[2] as u16 + 10).min(255) as u8;
        }
    }

    Uint8Array::from(&buffer[..])
}

// ✅ Avoid frequent allocations
#[wasm_bindgen]
pub fn compute_heavy(input: &[f32]) -> Vec<f32> {
    // Pre-allocate output buffer
    let mut output = Vec::with_capacity(input.len());

    // Process efficiently
    for &value in input {
        output.push(value.sqrt() * 2.0);
    }

    output
}
```

### 🔧 JavaScript Integration

**HTML Integration:**
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Rust WebAssembly Demo</title>
</head>
<body>
    <h1>Rust WebAssembly Demo</h1>
    <input type="text" id="nameInput" placeholder="Enter your name">
    <button onclick="greet()">Greet</button>
    <div id="output"></div>

    <!-- Load the generated JS glue code -->
    <script type="module">
        import init, { greet } from './pkg/my_wasm_project.js';

        // Initialize the WASM module
        async function run() {
            await init();

            // Make greet available globally
            window.greet = function() {
                const name = document.getElementById('nameInput').value;
                const result = greet(name);
                document.getElementById('output').textContent = result;
            };
        }

        run().catch(console.error);
    </script>
</body>
</html>
```

**Advanced JavaScript Integration:**
```javascript
import init, {
    process_image_data,
    fibonacci,
    compute_heavy
} from './pkg/my_wasm_project.js';

class WasmProcessor {
    constructor() {
        this.initialized = false;
    }

    async initialize() {
        if (!this.initialized) {
            await init();
            this.initialized = true;
        }
    }

    async processImage(imageData) {
        if (!this.initialized) {
            await this.initialize();
        }

        try {
            const startTime = performance.now();
            const result = process_image_data(imageData);
            const endTime = performance.now();

            console.log(`Processing took ${endTime - startTime}ms`);
            return result;
        } catch (error) {
            console.error('WASM processing failed:', error);
            throw error;
        }
    }

    fibonacci(n) {
        if (!this.initialized) {
            throw new Error('WASM module not initialized');
        }
        return fibonacci(n);
    }
}

export default WasmProcessor;
```

### 🐛 Common WebAssembly Errors and Solutions

**Error 1: "Target wasm32-unknown-unknown not found"**
```bash
# Problem: WASM target not installed
# Solution: Install the target
rustup target add wasm32-unknown-unknown
```

**Error 2: "wasm-bindgen import errors"**
```rust
// Problem: Incorrect bindings or missing attributes
// Solution: Ensure proper #[wasm_bindgen] usage

// ❌ Incorrect
pub fn my_function(input: String) -> String {
    format!("processed: {}", input)
}

// ✅ Correct
#[wasm_bindgen]
pub fn my_function(input: &str) -> String {
    format!("processed: {}", input)
}
```

**Error 3: "Bulk memory operations require bulk memory"**
```bash
# Problem: Outdated wasm-opt tool
# Solution: Update Binaryen or disable bulk memory

# Update Binaryen (recommended)
brew upgrade binaryen  # macOS
# or download from GitHub releases

# Temporary workaround
wasm-opt -O4 --enable-bulk-memory input.wasm -o output.wasm
```

**Error 4: "wasm is undefined" in browser**
```javascript
// Problem: Module not properly loaded or initialized
// Solution: Ensure proper async initialization

// ❌ Incorrect synchronous usage
import { greet } from './pkg/my_wasm_project.js';
greet('World'); // Fails if not initialized

// ✅ Correct async initialization
import init, { greet } from './pkg/my_wasm_project.js';

async function main() {
    await init();
    greet('World'); // Works properly
}

main().catch(console.error);
```

**Error 5: Memory exhaustion or stack overflow**
```rust
// Problem: Inefficient memory usage or recursion
// Solution: Optimize for WASM constraints

// ❌ Problematic large stack allocation
#[wasm_bindgen]
pub fn bad_function() {
    let huge_array = [0u8; 1_000_000]; // Stack overflow!
}

// ✅ Use heap allocation for large data
#[wasm_bindgen]
pub fn good_function() -> Vec<u8> {
    let huge_array = vec![0u8; 1_000_000]; // Heap allocation
    huge_array
}

// ❌ Deep recursion causing stack overflow
#[wasm_bindgen]
pub fn recursive_fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => recursive_fibonacci(n - 1) + recursive_fibonacci(n - 2),
    }
}

// ✅ Iterative solution
#[wasm_bindgen]
pub fn iterative_fibonacci(n: u64) -> u64 {
    let (mut a, mut b) = (0, 1);
    for _ in 0..n {
        let temp = a + b;
        a = b;
        b = temp;
    }
    a
}
```

### 🔍 Debugging Techniques

**Browser DevTools Integration:**
```rust
use wasm_bindgen::prelude::*;
use web_sys::console;

// Console logging from Rust
#[wasm_bindgen]
pub fn debug_function(value: f64) {
    console::log_1(&format!("Debug: value = {}", value).into());

    // Conditional logging for debugging
    #[cfg(debug_assertions)]
    console::warn_1(&"This is a debug warning".into());
}

// Performance measurement
#[wasm_bindgen]
pub fn measured_operation() {
    let start = web_sys::window()
        .unwrap()
        .performance()
        .unwrap()
        .now();

    // ... perform operation ...

    let end = web_sys::window()
        .unwrap()
        .performance()
        .unwrap()
        .now();

    console::log_1(&format!("Operation took {}ms", end - start).into());
}
```

**Source Map Configuration:**
```bash
# Build with debug information
wasm-pack build --dev

# Generate source maps
wasm-pack build --debug
```

**Testing in WASM Environment:**
```bash
# Headless browser testing
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox

# Node.js testing
wasm-pack test --node
```

### 🎯 Advanced Topics

**Web Workers Integration:**
```javascript
// worker.js
import init, { heavy_computation } from './pkg/my_wasm_project.js';

let wasmInitialized = false;

async function initializeWasm() {
    if (!wasmInitialized) {
        await init();
        wasmInitialized = true;
    }
}

self.onmessage = async (e) => {
    await initializeWasm();

    const result = heavy_computation(e.data);
    self.postMessage({
        type: 'result',
        data: result
    });
};
```

**Streaming Compilation for Large Modules:**
```javascript
async function loadWasmStreaming() {
    const response = await fetch('pkg/my_wasm_project_bg.wasm');
    const bytes = await response.arrayBuffer();
    const results = await WebAssembly.instantiateStreaming(bytes);

    // Use the compiled module
    return results.instance;
}
```

---

## Performance Optimization

### 🏗️ Build Performance

**Workspace-Level Optimization:**
```toml
# .cargo/config.toml
[build]
# Use all available CPU cores
jobs = num_cpus::get()

# Enable pipelined compilation
pipelining = true

# Use sccache for shared compilation cache
[build.rustc]
wrapper = "sccache"
```

**Profile Optimization:**
```toml
# Cargo.toml
[profile.dev]
opt-level = 1          # Better than 0 for dev builds
debug = true
incremental = true
codegen-units = 256    # Faster builds, slower runtime

[profile.release]
opt-level = 3
lto = true              # Link-time optimization
codegen-units = 1      # Maximum optimization
panic = "abort"        # Smaller binaries
strip = true           # Remove debug symbols
```

### 🚀 Runtime Performance

**Memory Management:**
```rust
// ✅ Use stack allocation where possible
let buffer = [0u8; 1024];  // Stack allocation

// ✅ Arena allocation for many objects
use bumpalo::Bump;
let bump = Bump::new();
let strings: Vec<&str> = vec!["hello", "world"].into_iter()
    .map(|s| bump.alloc_str(s))
    .collect();

// ❌ Avoid excessive heap allocations
let strings: Vec<String> = vec!["hello".to_string(), "world".to_string()];
```

**SIMD Optimization:**
```rust
// ✅ Use explicit SIMD when beneficial
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
fn fast_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    let (chunks_a, chunks_b, chunks_result) = unsafe {
        (
            std::slice::from_raw_parts(a.as_ptr() as *const __m256, a.len() / 8),
            std::slice::from_raw_parts(b.as_ptr() as *const __m256, b.len() / 8),
            std::slice::from_raw_parts_mut(result.as_mut_ptr() as *mut __m256, result.len() / 8)
        )
    };

    for (i, (&a_vec, &b_vec)) in chunks_a.iter().zip(chunks_b.iter()).enumerate() {
        unsafe {
            chunks_result[i] = _mm256_add_ps(a_vec, b_vec);
        }
    }
}
```

### 📊 Profiling and Monitoring

**Built-in Profiling:**
```bash
# CPU profiling
cargo build --release
perf record --call-graph=dwarf ./target/release/myapp
perf report

# Memory profiling
valgrind --tool=massif ./target/release/myapp
```

**Windows-Specific Profiling:**
```bash
# Use Windows Performance Toolkit
wpr -start GeneralProfile.wpr
./target/release/myapp
wpr -stop GeneralProfile.wpr
wpa GeneralProfile.etl
```

---

## Security Best Practices

### 🔒 Code Security

**Memory Safety:**
```rust
// ✅ Safe by default
fn process_data(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::with_capacity(data.len());
    buffer.extend_from_slice(data);
    Ok(buffer)
}

// ❌ Avoid unsafe unless necessary
unsafe fn process_data_unsafe(data: *const u8, len: usize) -> Vec<u8> {
    std::slice::from_raw_parts(data, len).to_vec()
}
```

**Input Validation:**
```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
struct UserInput {
    #[validate(length(min = 1, max = 100))]
    username: String,

    #[validate(email)]
    email: String,

    #[validate(range(min = 13, max = 120))]
    age: u8,
}

fn process_user_input(input: UserInput) -> Result<(), Error> {
    input.validate()?;
    // Process validated input
    Ok(())
}
```

### 🛡️ Dependency Security

**Regular Auditing:**
```bash
# Check for known vulnerabilities
cargo audit

# Check license compliance
cargo deny check licenses

# Check for outdated dependencies
cargo outdated
```

**Dependency Configuration:**
```toml
# .cargo/config.toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"

# For offline builds or supply chain security
```

### 🔐 Environment Security

**Secure Configuration:**
```rust
use secrecy::{Secret, ExposeSecret};
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
struct SecretKey {
    key: Secret<String>,
}

impl SecretKey {
    fn new(key: String) -> Self {
        Self { key: Secret::new(key) }
    }

    fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        // Use secret.key.expose_secret() for actual encryption
        vec![] // Implementation
    }
}
```

**Environment Variables:**
```rust
use std::env;

fn get_database_url() -> Result<String, Error> {
    env::var("DATABASE_URL")
        .map_err(|_| Error::EnvironmentVariableMissing("DATABASE_URL"))
}

// Use dotenv for development (never in production!)
#[cfg(debug_assertions)]
dotenv::dotenv().ok();
```

---

## Quick Reference Checklist

### ✅ Environment Setup
- [ ] Install Rust with `rustup` (MSVC toolchain)
- [ ] Install Visual Studio Build Tools
- [ ] Configure VS Code with rust-analyzer
- [ ] Set up `.cargo/config.toml` for performance
- [ ] Configure `.vscode/settings.json`

### ✅ WebAssembly Setup
- [ ] Add WASM target: `rustup target add wasm32-unknown-unknown`
- [ ] Install wasm-pack: `cargo install wasm-pack`
- [ ] Install wasm-opt (Binaryen) for optimization
- [ ] Configure WASM-specific dependencies in Cargo.toml
- [ ] Set up proper crate-type: `["cdylib"]`

### ✅ Project Structure
- [ ] Use workspace configuration
- [ ] Organize by domain (libs/, apps/, tools/)
- [ ] Set up workspace dependencies
- [ ] Configure build profiles
- [ ] Add dependency checking with cargo-deny

### ✅ Development Workflow
- [ ] Enable `rust-analyzer` features
- [ ] Set up pre-commit hooks (cargo fmt, cargo clippy)
- [ ] Configure path-based CI triggers
- [ ] Add property-based tests
- [ ] Set up cargo-watch for development
- [ ] Configure WASM build pipeline: `wasm-pack build --target web`
- [ ] Set up WASM testing: `wasm-pack test --headless --chrome`
- [ ] Configure WASM optimization with wasm-opt

### ✅ Security & Maintenance
- [ ] Regular dependency audits
- [ ] Use cargo-audit and cargo-deny
- [ ] Enable security scanning in CI
- [ ] Keep dependencies updated
- [ ] Review and minimize unsafe code

---

## Resources and Tools

### 🛠️ Essential Tools
```bash
# Development tools
cargo install cargo-watch cargo-expand cargo-audit
cargo install cargo-deny cargo-outdated cargo-edit

# Performance tools
cargo install cargo-flamegraph cargo-criterion

# Security tools
cargo install cargo-audit cargo-deny

# WebAssembly tools
cargo install wasm-pack
# wasm-opt from Binaryen: https://github.com/WebAssembly/binaryen/releases
npm install -g http-server  # For local WASM testing
```

### 📚 Learning Resources
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)
- [Rust Compiler Performance Guide](https://rust-lang.github.io/compiler-performance-guide/)
- [Rust WebAssembly Book](https://rustwasm.github.io/book/)
- [WebAssembly.org](https://webassembly.org/)

### 🌐 Community
- [Rust Users Forum](https://users.rust-lang.org/)
- [Rust Discord](https://discord.gg/rust-lang)
- [Reddit r/rust](https://reddit.com/r/rust)

---

*This guide is maintained for Rust 2025 development on Windows 11. Last updated: January 2025*
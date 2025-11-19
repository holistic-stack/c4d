const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function run(command, options = {}) {
	execSync(command, { stdio: 'inherit', ...options });
}

const root = __dirname;
const crateName = 'openscad-wasm';
const targetDir = path.join(root, 'target', 'wasm32-unknown-unknown', 'release');
const outDir = path.join(root, 'playground', 'src', 'lib', 'wasm', 'pkg');

function ensureWasmBindgen() {
  try {
    run('wasm-bindgen --version', { cwd: root });
    return;
  } catch {
    run('cargo install --locked wasm-bindgen-cli', { cwd: root });
  }
}

function hasWasmOpt() {
  try {
    run('wasm-opt --version', { cwd: root });
    return true;
  } catch {
    const binCandidates = [
      path.join(root, 'playground', 'node_modules', '.bin', process.platform === 'win32' ? 'wasm-opt.cmd' : 'wasm-opt'),
      path.join(root, 'node_modules', '.bin', process.platform === 'win32' ? 'wasm-opt.cmd' : 'wasm-opt')
    ];
    for (const candidate of binCandidates) {
      if (fs.existsSync(candidate)) {
        try {
          run(`"${candidate}" --version`, { cwd: root });
          return true;
        } catch {}
      }
    }
    return false;
  }
}

function wasmOptCmd() {
  if (process.platform === 'win32') {
    const local = path.join(root, 'playground', 'node_modules', '.bin', 'wasm-opt.cmd');
    if (fs.existsSync(local)) return `"${local}"`;
  } else {
    const local = path.join(root, 'playground', 'node_modules', '.bin', 'wasm-opt');
    if (fs.existsSync(local)) return `"${local}"`;
  }
  return 'wasm-opt';
}

ensureWasmBindgen();

const rustflags = [
  '-C opt-level=z',
  '-C codegen-units=1',
  '-C strip=symbols'
].join(' ');

// Get the Rust sysroot for WASI
const rustSysroot = 'C:\\Users\\luciano\\scoop\\persist\\rustup-gnu\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc';
const wasiSysroot = path.join(rustSysroot, 'lib', 'rustlib', 'wasm32-wasip1');

// Use wasm32-unknown-unknown target but with WASI sysroot for Zig
run(`cargo build --release --package ${crateName} --target wasm32-unknown-unknown`, { 
  cwd: root, 
  env: { 
    ...process.env, 
    RUSTFLAGS: rustflags,
    CC_wasm32_unknown_unknown: 'zig cc -target wasm32-wasi',
    AR_wasm32_unknown_unknown: 'zig ar',
    CFLAGS_wasm32_unknown_unknown: `-target wasm32-wasi --sysroot="${wasiSysroot}" -O3 -ffunction-sections -fdata-sections -fno-exceptions`,
    CRATE_CC_NO_DEFAULTS: '1',
    WASI_SYSROOT: wasiSysroot
  } 
});

fs.mkdirSync(outDir, { recursive: true });

const wasmPath = path.join(targetDir, 'openscad_wasm.wasm');
run(`wasm-bindgen "${wasmPath}" --out-dir "${outDir}" --target web --typescript`, { cwd: root });

const bgWasm = path.join(outDir, 'openscad_wasm_bg.wasm');
if (fs.existsSync(bgWasm)) {
  const before = fs.statSync(bgWasm).size;
  if (hasWasmOpt()) {
    const optimized = path.join(outDir, 'openscad_wasm_bg.opt.wasm');
    run(`${wasmOptCmd()} -Oz --strip-debug --strip-producers --enable-bulk-memory --vacuum "${bgWasm}" -o "${optimized}"`, { cwd: root });
    if (fs.existsSync(optimized)) {
      fs.copyFileSync(optimized, bgWasm);
      fs.unlinkSync(optimized);
    }
  } else {
    try {
      run('pnpm add -D binaryen', { cwd: path.join(root, 'playground') });
      if (hasWasmOpt()) {
        const optimized = path.join(outDir, 'openscad_wasm_bg.opt.wasm');
        run(`${wasmOptCmd()} -Oz --strip-debug --strip-producers --enable-bulk-memory --vacuum "${bgWasm}" -o "${optimized}"`, { cwd: root });
        if (fs.existsSync(optimized)) {
          fs.copyFileSync(optimized, bgWasm);
          fs.unlinkSync(optimized);
        }
      }
    } catch {}
  }
  const after = fs.statSync(bgWasm).size;
  const glueJs = path.join(outDir, 'openscad_wasm.js');
  const glueSize = fs.existsSync(glueJs) ? fs.statSync(glueJs).size : 0;
  const kb = (n) => (n / 1024).toFixed(2) + ' KB';
  console.log(`WASM size: ${kb(before)} -> ${kb(after)} | Glue: ${kb(glueSize)}`);
}
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

run(`cargo build --release --package ${crateName} --target wasm32-unknown-unknown`, { cwd: root, env: { ...process.env, RUSTFLAGS: rustflags } });

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

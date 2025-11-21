import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const WASM_TARGET = 'wasm32-unknown-unknown';
const ROOT = __dirname;
const WASM_CRATE = path.join(ROOT, 'libs', 'wasm');
const OUTPUT_DIR = path.join(WASM_CRATE, 'pkg');
const WASM_BIN = path.join(ROOT, 'target', WASM_TARGET, 'release', 'wasm.wasm');
const WASM_BINDGEN_VERSION = '0.2.105';

function run(command, options = {}) {
    const mergedEnv = {
        ...process.env,
        ...options.env,
    };
    console.log(`> ${command}`);
    execSync(command, {
        stdio: 'inherit',
        cwd: options.cwd ?? ROOT,
        env: mergedEnv,
    });
}

function ensureWasmBindgen() {
    try {
        const version = execSync('wasm-bindgen --version', { stdio: 'pipe' })
            .toString()
            .trim();
        if (!version.includes(WASM_BINDGEN_VERSION)) {
            console.warn(
                `wasm-bindgen-cli version mismatch (expected ${WASM_BINDGEN_VERSION}). Found: ${version}`
            );
        }
    } catch (error) {
        throw new Error(
            `wasm-bindgen-cli not found. Install it via \`cargo install -f wasm-bindgen-cli --version ${WASM_BINDGEN_VERSION}\`.`
        );
    }
}

function buildEnv() {
    const env = { ...process.env };
    const wasiPath = env.WASI_SDK_PATH;

    if (wasiPath) {
        const sysroot = env.WASI_SYSROOT ?? path.join(wasiPath, 'share', 'wasi-sysroot');
        env.CC = env.CC ?? path.join(wasiPath, 'bin', 'clang');
        env.CXX = env.CXX ?? path.join(wasiPath, 'bin', 'clang++');
        env.AR = env.AR ?? path.join(wasiPath, 'bin', 'llvm-ar');
        env.CC_wasm32_unknown_unknown = env.CC_wasm32_unknown_unknown ?? env.CC;
        env.AR_wasm32_unknown_unknown = env.AR_wasm32_unknown_unknown ?? env.AR;
        env.CFLAGS_wasm32_unknown_unknown = env.CFLAGS_wasm32_unknown_unknown ?? `--sysroot=${sysroot}`;
    } else {
        console.warn(
            'WASI_SDK_PATH is not set. Falling back to the host toolchain; ensure clang/llvm can target wasm32.'
        );
    }

    return env;
}

function ensureOutputDir() {
    if (fs.existsSync(OUTPUT_DIR)) {
        fs.rmSync(OUTPUT_DIR, { recursive: true, force: true });
    }
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

try {
    console.log('Ensuring wasm32 target is installed...');
    run(`rustup target add ${WASM_TARGET}`);

    console.log('Checking wasm-bindgen-cli...');
    ensureWasmBindgen();

    const env = buildEnv();

    console.log('Building Rust crate for wasm...');
    run(`cargo build --release -p wasm --target ${WASM_TARGET}` , { env });

    if (!fs.existsSync(WASM_BIN)) {
        throw new Error(`Expected wasm artifact not found at ${WASM_BIN}`);
    }

    console.log('Preparing output directory...');
    ensureOutputDir();

    console.log('Running wasm-bindgen...');
    run(
        `wasm-bindgen --target web --out-dir "${OUTPUT_DIR}" --out-name wasm "${WASM_BIN}"`,
        { env }
    );

    console.log('WASM build successful!');
} catch (error) {
    console.error('Build failed:', error.message ?? error);
    process.exit(1);
}

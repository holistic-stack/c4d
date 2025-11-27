#!/usr/bin/env node
/**
 * Simple helper script to build the WASM package and copy the outputs into the playground app.
 */
const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const ROOT_DIR = __dirname;
const WASM_CRATE_DIR = path.join(ROOT_DIR, 'libs', 'wasm');
const WASM_PKG_DIR = path.join(WASM_CRATE_DIR, 'pkg');
const PLAYGROUND_WASM_DIR = path.join(
    ROOT_DIR,
    'apps',
    'playground',
    'src',
    'lib',
    'wasm',
    'pkg'
);

function log(message) {
    console.log(`\x1b[36m[buil-wasm]\x1b[0m ${message}`);
}

function run(command, args, options = {}) {
    log(`$ ${command} ${args.join(' ')}`);
    const result = spawnSync(command, args, {
        stdio: 'inherit',
        ...options
    });

    if (result.status !== 0) {
        throw new Error(`Command failed: ${command}`);
    }
}

function ensureCommand(command) {
    const checker = process.platform === 'win32' ? 'where' : 'which';
    const result = spawnSync(checker, [command], { stdio: 'ignore' });

    if (result.status !== 0) {
        throw new Error(`Required command not found: ${command}`);
    }
}

function copyArtifacts() {
    if (!fs.existsSync(WASM_PKG_DIR)) {
        throw new Error(`Expected WASM output directory missing: ${WASM_PKG_DIR}`);
    }

    fs.rmSync(PLAYGROUND_WASM_DIR, { recursive: true, force: true });
    fs.mkdirSync(PLAYGROUND_WASM_DIR, { recursive: true });

    for (const file of fs.readdirSync(WASM_PKG_DIR)) {
        const src = path.join(WASM_PKG_DIR, file);
        const dest = path.join(PLAYGROUND_WASM_DIR, file);
        fs.copyFileSync(src, dest);
    }

    log(`Copied artifacts to ${PLAYGROUND_WASM_DIR}`);
}

function main() {
    ensureCommand('wasm-pack');

    run('wasm-pack', ['build', '--release', '--target', 'web', '--out-dir', 'pkg'], {
        cwd: WASM_CRATE_DIR
    });

    copyArtifacts();
    log('Build complete.');
}

try {
    main();
} catch (err) {
    console.error(`\x1b[31m[buil-wasm]\x1b[0m ${err.message}`);
    process.exit(1);
}

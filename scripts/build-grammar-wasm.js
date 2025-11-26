#!/usr/bin/env node
/**
 * # Build OpenSCAD Grammar to WASM
 *
 * Builds the tree-sitter-openscad-parser grammar to a WASM file
 * for use with web-tree-sitter in the browser.
 *
 * ## Prerequisites
 *
 * One of the following must be available:
 * - Emscripten SDK (emcc)
 * - Docker
 * - Podman
 *
 * ## Usage
 *
 * ```bash
 * node scripts/build-grammar-wasm.js
 * ```
 *
 * ## Output
 *
 * Creates `tree-sitter-openscad_parser.wasm` in `apps/playground/static/`
 */

const { execSync, spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configuration
const ROOT_DIR = path.resolve(__dirname, '..');
const GRAMMAR_DIR = path.join(ROOT_DIR, 'libs', 'openscad-parser');
const OUTPUT_DIR = path.join(ROOT_DIR, 'apps', 'playground', 'static');
const OUTPUT_FILE = 'tree-sitter-openscad_parser.wasm';

/**
 * Logs a message with a prefix.
 * @param {string} message - Message to log
 */
function log(message) {
    console.log(`\x1b[34m[grammar-wasm]\x1b[0m ${message}`);
}

/**
 * Logs an error message.
 * @param {string} message - Error message
 */
function error(message) {
    console.error(`\x1b[31m[grammar-wasm]\x1b[0m ${message}`);
}

/**
 * Checks if a command exists.
 * @param {string} command - Command to check
 * @returns {boolean} True if command exists
 */
function commandExists(command) {
    try {
        const result = spawnSync(process.platform === 'win32' ? 'where' : 'which', [command], {
            stdio: 'pipe'
        });
        return result.status === 0;
    } catch {
        return false;
    }
}

/**
 * Checks if Docker is running.
 * @returns {boolean} True if Docker is running
 */
function isDockerRunning() {
    try {
        const result = spawnSync('docker', ['ps'], {
            stdio: 'pipe',
            timeout: 5000
        });
        return result.status === 0;
    } catch {
        return false;
    }
}

/**
 * Main build function.
 */
async function main() {
    log('Building OpenSCAD grammar to WASM...');

    // Check prerequisites
    const hasEmcc = commandExists('emcc');
    const hasDocker = commandExists('docker') && isDockerRunning();
    const hasPodman = commandExists('podman');

    if (!hasEmcc && !hasDocker && !hasPodman) {
        error('No build tool available!');
        error('Please install one of the following:');
        error('  - Emscripten SDK: https://emscripten.org/docs/getting_started/downloads.html');
        error('  - Docker Desktop: https://www.docker.com/products/docker-desktop/');
        error('  - Podman: https://podman.io/');
        error('');
        error('If Docker is installed, make sure Docker Desktop is running.');
        process.exit(1);
    }

    // Ensure output directory exists
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });

    // Build the grammar
    log(`Grammar directory: ${GRAMMAR_DIR}`);
    log(`Output directory: ${OUTPUT_DIR}`);

    try {
        execSync(`npx tree-sitter build --wasm "${GRAMMAR_DIR}"`, {
            cwd: GRAMMAR_DIR,
            stdio: 'inherit'
        });

        // Move the output file to the correct location
        const generatedFile = path.join(GRAMMAR_DIR, OUTPUT_FILE);
        const targetFile = path.join(OUTPUT_DIR, OUTPUT_FILE);

        if (fs.existsSync(generatedFile)) {
            fs.copyFileSync(generatedFile, targetFile);
            fs.unlinkSync(generatedFile);
            log(`Grammar WASM built successfully: ${targetFile}`);
        } else {
            // Check if it was generated with a different name
            const files = fs.readdirSync(GRAMMAR_DIR).filter(f => f.endsWith('.wasm'));
            if (files.length > 0) {
                const srcFile = path.join(GRAMMAR_DIR, files[0]);
                fs.copyFileSync(srcFile, targetFile);
                fs.unlinkSync(srcFile);
                log(`Grammar WASM built successfully: ${targetFile}`);
            } else {
                error('WASM file not found after build');
                process.exit(1);
            }
        }
    } catch (err) {
        error(`Build failed: ${err.message}`);
        process.exit(1);
    }
}

main().catch(err => {
    error(err.message);
    process.exit(1);
});

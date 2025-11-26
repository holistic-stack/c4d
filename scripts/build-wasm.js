#!/usr/bin/env node
/**
 * # WASM Build Script (Cross-Platform)
 *
 * Builds the OpenSCAD WASM package for browser use.
 * Works on Windows, macOS, and Linux.
 *
 * ## Prerequisites
 *
 * - Rust toolchain with wasm32-unknown-unknown target
 * - wasm-bindgen-cli (cargo install wasm-bindgen-cli)
 * - Node.js 18+
 *
 * ## Usage
 *
 * ```bash
 * node scripts/build-wasm.js
 * ```
 */

const { execSync, spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configuration
const ROOT_DIR = path.resolve(__dirname, '..');
const WASM_TARGET = 'wasm32-unknown-unknown';
const WASM_CRATE = 'openscad-wasm';
const WASM_CRATE_DIR = path.join(ROOT_DIR, 'libs', 'wasm');
const OUTPUT_DIR = path.join(WASM_CRATE_DIR, 'pkg');
const WASM_BIN = path.join(ROOT_DIR, 'target', WASM_TARGET, 'release', 'openscad_wasm.wasm');
const WASM_BINDGEN_VERSION = '0.2.105';

/**
 * Logs a message with a prefix.
 * @param {string} message - Message to log
 */
function log(message) {
    console.log(`\x1b[34m[wasm]\x1b[0m ${message}`);
}

/**
 * Logs an error message.
 * @param {string} message - Error message
 */
function error(message) {
    console.error(`\x1b[31m[wasm]\x1b[0m ${message}`);
}

/**
 * Runs a command and returns the output.
 * @param {string} command - Command to run
 * @param {object} options - Spawn options
 * @returns {string} Command output
 */
function run(command, options = {}) {
    log(`Running: ${command}`);
    try {
        return execSync(command, {
            cwd: ROOT_DIR,
            stdio: 'inherit',
            ...options
        });
    } catch (err) {
        error(`Command failed: ${command}`);
        throw err;
    }
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
 * Ensures the WASM target is installed.
 */
function ensureWasmTarget() {
    log('Checking WASM target...');
    const result = spawnSync('rustup', ['target', 'list', '--installed'], {
        stdio: 'pipe',
        encoding: 'utf-8'
    });

    if (!result.stdout.includes(WASM_TARGET)) {
        log(`Installing ${WASM_TARGET} target...`);
        run(`rustup target add ${WASM_TARGET}`);
    } else {
        log(`${WASM_TARGET} target already installed.`);
    }
}

/**
 * Ensures wasm-bindgen-cli is installed.
 */
function ensureWasmBindgen() {
    log('Checking wasm-bindgen-cli...');

    if (!commandExists('wasm-bindgen')) {
        error('wasm-bindgen-cli not found. Install it with:');
        error(`  cargo install wasm-bindgen-cli --version ${WASM_BINDGEN_VERSION}`);
        process.exit(1);
    }

    const result = spawnSync('wasm-bindgen', ['--version'], {
        stdio: 'pipe',
        encoding: 'utf-8'
    });

    if (!result.stdout.includes(WASM_BINDGEN_VERSION)) {
        log(`Warning: Expected wasm-bindgen ${WASM_BINDGEN_VERSION}, found ${result.stdout.trim()}`);
    }
}

/**
 * Creates dummy C headers for tree-sitter WASM compilation.
 * Tree-sitter has C dependencies that need stub headers for wasm32-unknown-unknown.
 */
function createDummyHeaders() {
    const includeDir = path.join(ROOT_DIR, 'target', 'wasm-include');
    fs.mkdirSync(includeDir, { recursive: true });

    const headers = {
        'stdio.h': `
#ifndef _STDIO_H
#define _STDIO_H
#include <stddef.h>
#include <stdarg.h>
typedef struct {} FILE;
#define stderr ((FILE*)0)
#define stdout ((FILE*)0)
#define stdin  ((FILE*)0)
int printf(const char *format, ...);
int fprintf(FILE *stream, const char *format, ...);
int snprintf(char *str, size_t size, const char *format, ...);
int sprintf(char *str, const char *format, ...);
int vfprintf(FILE *stream, const char *format, void *ap);
int vsnprintf(char *str, size_t size, const char *format, va_list ap);
int fputc(int c, FILE *stream);
int fputs(const char *s, FILE *stream);
int fclose(FILE *stream);
FILE *fdopen(int fd, const char *mode);
#endif
`,
        'stdlib.h': `
#ifndef _STDLIB_H
#define _STDLIB_H
#include <stddef.h>
void *malloc(size_t size);
void free(void *ptr);
void *realloc(void *ptr, size_t size);
void *calloc(size_t nmemb, size_t size);
void exit(int status);
char *getenv(const char *name);
int abs(int j);
void abort(void);
#endif
`,
        'string.h': `
#ifndef _STRING_H
#define _STRING_H
#include <stddef.h>
void *memcpy(void *dest, const void *src, size_t n);
void *memmove(void *dest, const void *src, size_t n);
void *memset(void *s, int c, size_t n);
int memcmp(const void *s1, const void *s2, size_t n);
size_t strlen(const char *s);
char *strcpy(char *dest, const char *src);
char *strncpy(char *dest, const char *src, size_t n);
int strcmp(const char *s1, const char *s2);
int strncmp(const char *s1, const char *s2, size_t n);
char *strdup(const char *s);
#endif
`,
        'ctype.h': `
#ifndef _CTYPE_H
#define _CTYPE_H
int isspace(int c);
int isdigit(int c);
int isalpha(int c);
int isalnum(int c);
int isprint(int c);
#endif
`,
        'unistd.h': `
#ifndef _UNISTD_H
#define _UNISTD_H
#include <stddef.h>
typedef int ssize_t;
int close(int fd);
ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);
int dup(int oldfd);
#endif
`,
        'inttypes.h': `
#ifndef _INTTYPES_H
#define _INTTYPES_H
#include <stdint.h>
#define PRId8 "d"
#define PRId16 "d"
#define PRId32 "d"
#define PRId64 "lld"
#define PRIu8 "u"
#define PRIu16 "u"
#define PRIu32 "u"
#define PRIu64 "llu"
#define PRIx8 "x"
#define PRIx16 "x"
#define PRIx32 "x"
#define PRIx64 "llx"
#define PRIX8 "X"
#define PRIX16 "X"
#define PRIX32 "X"
#define PRIX64 "llX"
#endif
`,
        'wctype.h': `
#ifndef _WCTYPE_H
#define _WCTYPE_H
#include <ctype.h>
typedef int wint_t;
typedef int wctype_t;
#define WEOF (-1)
int iswspace(wint_t wc);
int iswalpha(wint_t wc);
int iswalnum(wint_t wc);
int iswdigit(wint_t wc);
wint_t towlower(wint_t wc);
wint_t towupper(wint_t wc);
#endif
`,
        'time.h': `
#ifndef _TIME_H
#define _TIME_H
typedef long time_t;
typedef long clock_t;
#define CLOCKS_PER_SEC 1000000
clock_t clock(void);
time_t time(time_t *tloc);
#endif
`,
        'endian.h': `
#ifndef _ENDIAN_H
#define _ENDIAN_H
#include <stdint.h>
#define le16toh(x) (x)
#define be16toh(x) __builtin_bswap16(x)
#define le32toh(x) (x)
#define be32toh(x) __builtin_bswap32(x)
#define htole16(x) (x)
#define htobe16(x) __builtin_bswap16(x)
#define htole32(x) (x)
#define htobe32(x) __builtin_bswap32(x)
#endif
`,
        'assert.h': `
#ifndef _ASSERT_H
#define _ASSERT_H
#define assert(ignore) ((void)0)
#endif
`
    };

    for (const [filename, content] of Object.entries(headers)) {
        fs.writeFileSync(path.join(includeDir, filename), content.trim() + '\n');
    }

    log(`Created dummy headers in ${includeDir}`);
    return includeDir;
}

/**
 * Prepares the output directory.
 */
function prepareOutputDir() {
    if (fs.existsSync(OUTPUT_DIR)) {
        fs.rmSync(OUTPUT_DIR, { recursive: true });
    }
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
    log(`Prepared output directory: ${OUTPUT_DIR}`);
}

/**
 * Main build function.
 */
async function main() {
    log('Starting WASM build...');

    // Check prerequisites
    ensureWasmTarget();
    ensureWasmBindgen();

    // Create dummy headers for tree-sitter
    const includeDir = createDummyHeaders();

    // Set environment variables for C compilation
    const env = {
        ...process.env,
        CFLAGS_wasm32_unknown_unknown: `-I${includeDir}`,
        CC_wasm32_unknown_unknown: 'clang',
        AR_wasm32_unknown_unknown: 'llvm-ar'
    };

    // Build the WASM crate
    log(`Building ${WASM_CRATE} for ${WASM_TARGET}...`);
    try {
        execSync(`cargo build --release -p ${WASM_CRATE} --target ${WASM_TARGET}`, {
            cwd: ROOT_DIR,
            stdio: 'inherit',
            env
        });
    } catch (err) {
        error('Cargo build failed.');
        error('If you see C header errors, you may need to install clang/LLVM.');
        error('On Windows: Install LLVM from https://releases.llvm.org/');
        error('On macOS: xcode-select --install');
        error('On Linux: apt install clang llvm');
        process.exit(1);
    }

    // Check if WASM binary was created
    if (!fs.existsSync(WASM_BIN)) {
        error(`Expected WASM binary not found at ${WASM_BIN}`);
        process.exit(1);
    }

    // Prepare output directory
    prepareOutputDir();

    // Run wasm-bindgen
    // Note: --out-name must match what's imported in loader.ts (openscad_wasm)
    log('Running wasm-bindgen...');
    run(`wasm-bindgen --target web --out-dir "${OUTPUT_DIR}" --out-name openscad_wasm "${WASM_BIN}"`);

    log(`WASM build complete! Artifacts in ${OUTPUT_DIR}`);
    log('Files:');
    fs.readdirSync(OUTPUT_DIR).forEach(file => {
        const stat = fs.statSync(path.join(OUTPUT_DIR, file));
        log(`  ${file} (${(stat.size / 1024).toFixed(1)} KB)`);
    });

    // Copy WASM files to playground
    const playgroundWasmDir = path.join(ROOT_DIR, 'apps', 'playground', 'src', 'lib', 'wasm', 'pkg');
    if (fs.existsSync(path.dirname(playgroundWasmDir))) {
        log(`Copying WASM files to playground...`);
        fs.mkdirSync(playgroundWasmDir, { recursive: true });
        fs.readdirSync(OUTPUT_DIR).forEach(file => {
            const src = path.join(OUTPUT_DIR, file);
            const dest = path.join(playgroundWasmDir, file);
            fs.copyFileSync(src, dest);
        });
        log(`Copied to ${playgroundWasmDir}`);
    }
}

main().catch(err => {
    error(err.message);
    process.exit(1);
});

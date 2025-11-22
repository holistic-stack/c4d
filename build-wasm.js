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

    if (!wasiPath) {
        const includeDir = path.join(ROOT, 'target', 'wasm-include');
        if (!fs.existsSync(includeDir)) {
            fs.mkdirSync(includeDir, { recursive: true });
        }

        const headers = {
            'stdio.h': `#ifndef _STDIO_H
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
#endif`,
            'stdlib.h': `#ifndef _STDLIB_H
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
#endif`,
            'string.h': `#ifndef _STRING_H
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
#endif`,
            'ctype.h': `#ifndef _CTYPE_H
#define _CTYPE_H
int isspace(int c);
int isdigit(int c);
int isalpha(int c);
int isalnum(int c);
int isprint(int c);
#endif`,
            'unistd.h': `#ifndef _UNISTD_H
#define _UNISTD_H
#include <stddef.h>
typedef int ssize_t;
int close(int fd);
ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);
int dup(int oldfd);
#endif`,
            'inttypes.h': `#ifndef _INTTYPES_H
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
#endif`,
            'wctype.h': `#ifndef _WCTYPE_H
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
#endif`,
            'time.h': `#ifndef _TIME_H
#define _TIME_H
typedef long time_t;
typedef long clock_t;
#define CLOCKS_PER_SEC 1000000
clock_t clock(void);
time_t time(time_t *tloc);
#endif`,
            'endian.h': `#ifndef _ENDIAN_H
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
#endif`,
            'assert.h': `#ifndef _ASSERT_H
#define _ASSERT_H
#define assert(ignore) ((void)0)
#endif`
        };

        for (const [filename, content] of Object.entries(headers)) {
            fs.writeFileSync(path.join(includeDir, filename), content);
        }

        env.CFLAGS_wasm32_unknown_unknown = (env.CFLAGS_wasm32_unknown_unknown || '') + ` -I${includeDir}`;
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
    run(`cargo build --release -p wasm --target ${WASM_TARGET}`, { env });

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

#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WASM_TARGET="wasm32-unknown-unknown"
WASM_CRATE="wasm"
WASM_CRATE_DIR="${ROOT_DIR}/libs/${WASM_CRATE}"
OUTPUT_DIR="${WASM_CRATE_DIR}/pkg"
WASM_BIN="${ROOT_DIR}/target/${WASM_TARGET}/release/${WASM_CRATE}.wasm"
WASM_BINDGEN_VERSION="0.2.105"

log() {
    echo -e "\033[1;34m[wasm]\033[0m $*"
}

ensure_wasm_target() {
    if ! rustup target list --installed | grep -q "${WASM_TARGET}"; then
        log "Installing ${WASM_TARGET} target via rustup..."
        rustup target add "${WASM_TARGET}"
    fi
}

ensure_wasm_bindgen() {
    if ! command -v wasm-bindgen >/dev/null 2>&1; then
        cat <<EOF
wasm-bindgen-cli not found. Install it with:
    cargo install -f wasm-bindgen-cli --version ${WASM_BINDGEN_VERSION}
EOF
        exit 1
    fi

    local version
    version=$(wasm-bindgen --version)
    if [[ "${version}" != *"${WASM_BINDGEN_VERSION}"* ]]; then
        log "Warning: expected wasm-bindgen ${WASM_BINDGEN_VERSION}, but found ${version}."
    fi
}

configure_wasi_env() {
    if [[ -n "${WASI_SDK_PATH:-}" ]]; then
        local sysroot
        sysroot="${WASI_SYSROOT:-${WASI_SDK_PATH}/share/wasi-sysroot}"
        export CC="${CC:-${WASI_SDK_PATH}/bin/clang}"
        export CXX="${CXX:-${WASI_SDK_PATH}/bin/clang++}"
        export AR="${AR:-${WASI_SDK_PATH}/bin/llvm-ar}"
        export CC_wasm32_unknown_unknown="${CC_wasm32_unknown_unknown:-${CC}}"
        export AR_wasm32_unknown_unknown="${AR_wasm32_unknown_unknown:-${AR}}"
        export CFLAGS_wasm32_unknown_unknown="${CFLAGS_wasm32_unknown_unknown:---sysroot=${sysroot}}"
        log "Configured WASI toolchain from ${WASI_SDK_PATH}."
    fi
}

prepare_output_dir() {
    rm -rf "${OUTPUT_DIR}"
    mkdir -p "${OUTPUT_DIR}"
}

run() {
    log "$*"
    "$@"
}

main() {
    ensure_wasm_target
    ensure_wasm_bindgen
    configure_wasi_env

    configure_wasi_env

    # Generate dummy headers for wasm32-unknown-unknown if not using WASI SDK
    if [[ -z "${WASI_SDK_PATH:-}" ]]; then
        local include_dir="${ROOT_DIR}/target/wasm-include"
        mkdir -p "${include_dir}"
        
        # Generate stdio.h
        cat <<EOF > "${include_dir}/stdio.h"
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
EOF

        # Generate stdlib.h
        cat <<EOF > "${include_dir}/stdlib.h"
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
EOF

        # Generate string.h
        cat <<EOF > "${include_dir}/string.h"
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
EOF

        # Generate ctype.h
        cat <<EOF > "${include_dir}/ctype.h"
#ifndef _CTYPE_H
#define _CTYPE_H
int isspace(int c);
int isdigit(int c);
int isalpha(int c);
int isalnum(int c);
int isprint(int c);
#endif
EOF

        # Generate unistd.h
        cat <<EOF > "${include_dir}/unistd.h"
#ifndef _UNISTD_H
#define _UNISTD_H
#include <stddef.h>
typedef int ssize_t;
int close(int fd);
ssize_t read(int fd, void *buf, size_t count);
ssize_t write(int fd, const void *buf, size_t count);
int dup(int oldfd);
#endif
EOF

        # Generate inttypes.h
        cat <<EOF > "${include_dir}/inttypes.h"
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
EOF

        # Generate wctype.h
        cat <<EOF > "${include_dir}/wctype.h"
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
EOF

        # Generate time.h
        cat <<EOF > "${include_dir}/time.h"
#ifndef _TIME_H
#define _TIME_H
typedef long time_t;
typedef long clock_t;
#define CLOCKS_PER_SEC 1000000
clock_t clock(void);
time_t time(time_t *tloc);
#endif
EOF

        # Generate endian.h
        cat <<EOF > "${include_dir}/endian.h"
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
EOF

        # Generate assert.h
        cat <<EOF > "${include_dir}/assert.h"
#ifndef _ASSERT_H
#define _ASSERT_H
#define assert(ignore) ((void)0)
#endif
EOF

        export CFLAGS_wasm32_unknown_unknown="${CFLAGS_wasm32_unknown_unknown:-} -I${include_dir}"
    fi

    log "Building Rust crate for ${WASM_TARGET}..."
    run cargo build --release -p "${WASM_CRATE}" --target "${WASM_TARGET}"

    if [[ ! -f "${WASM_BIN}" ]]; then
        echo "Expected artifact not found at ${WASM_BIN}" >&2
        exit 1
    fi

    log "Preparing output directory at ${OUTPUT_DIR}..."
    prepare_output_dir

    log "Running wasm-bindgen..."
    run wasm-bindgen \
        --target web \
        --out-dir "${OUTPUT_DIR}" \
        --out-name wasm \
        "${WASM_BIN}"

    log "WASM build complete. Artifacts are in ${OUTPUT_DIR}."
}

main "$@"

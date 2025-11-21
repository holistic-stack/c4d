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

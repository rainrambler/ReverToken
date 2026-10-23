#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

CLANGXX="${CLANGXX:-clang++}"
SYSROOT="$(rustc --print sysroot)"
RUST_LLD="${RUST_LLD:-$(find "$SYSROOT" -type f -name rust-lld | head -n 1)}"

if [[ -z "${RUST_LLD}" || ! -x "${RUST_LLD}" ]]; then
    echo "error: rust-lld not found; set RUST_LLD=/path/to/rust-lld" >&2
    exit 1
fi

TMPDIR_FIXTURE="$(mktemp -d)"
trap 'rm -rf "${TMPDIR_FIXTURE}"' EXIT

# ELF shared object -----------------------------------------------------------
"${CLANGXX}" --target=x86_64-unknown-linux-gnu -std=c++17 -fPIC \
    -fvisibility=default -c src/elf.cpp -o "${TMPDIR_FIXTURE}/elf.o"
"${RUST_LLD}" -flavor gnu -shared -o libdemo.so "${TMPDIR_FIXTURE}/elf.o"

# PE DLL ----------------------------------------------------------------------
"${CLANGXX}" --target=x86_64-pc-windows-msvc -std=c++17 -fno-exceptions \
    -fno-rtti -c src/pe.cpp -o "${TMPDIR_FIXTURE}/pe.obj"
"${RUST_LLD}" -flavor link /dll /noentry /nodefaultlib /machine:x64 \
    /implib:"${TMPDIR_FIXTURE}/demo.lib" /out:demo.dll "${TMPDIR_FIXTURE}/pe.obj"

# Mach-O dylib (requires Apple's toolchain) ----------------------------------
if [[ "$(uname -s)" == "Darwin" ]]; then
    "${CLANGXX}" -std=c++17 -dynamiclib -fvisibility=default \
        -o libdemo.dylib src/macho.cpp
fi

echo "fixtures rebuilt"

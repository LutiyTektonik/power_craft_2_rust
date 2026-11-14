#!/usr/bin/env bash
# Source this file; keep tools and caches in the writable project directory.
PC2_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -d "$PC2_ROOT/.tools/cargo" ]]; then
    export CARGO_HOME="$PC2_ROOT/.tools/cargo"
    export RUSTUP_HOME="$PC2_ROOT/.tools/rustup"
    export PATH="$CARGO_HOME/bin:$PATH"
fi
if [[ -d "$PC2_ROOT/.tools/usr/bin" ]]; then
    export PATH="$PC2_ROOT/.tools/usr/bin:$PATH"
fi
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export CARGO_INCREMENTAL=0

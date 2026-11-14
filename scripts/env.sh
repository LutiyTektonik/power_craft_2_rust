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
if [[ -d "$PC2_ROOT/.tools/usr/lib/x86_64-linux-gnu" ]]; then
    export LD_LIBRARY_PATH="$PC2_ROOT/.tools/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
fi
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export CARGO_INCREMENTAL=0

pc2_require_horizon_space() {
    python3 - "$PC2_ROOT" <<'PY'
import shutil
import sys
free = shutil.disk_usage(sys.argv[1]).free
minimum = 6 * 1024**3
if free < minimum:
    sys.exit('Для полной проверки Horizon оставьте не менее 6 ГиБ свободного диска; сборка не начата.')
PY
}

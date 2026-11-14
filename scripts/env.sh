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
    python3 - "$PC2_ROOT" "${1:-build}" <<'PY'
import shutil
import sys
from pathlib import Path
free = shutil.disk_usage(sys.argv[1]).free
warm_check = sys.argv[2] == 'check' and (Path(sys.argv[1]) / 'native/target/cargo/debug/deps').is_dir()
minimum_gib = 3 if warm_check else 6
minimum = minimum_gib * 1024**3
if free < minimum:
    sys.exit(f'Для Horizon требуется не менее {minimum_gib} ГиБ свободного диска; операция не начата.')
PY
}

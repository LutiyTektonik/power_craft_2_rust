#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"
cd "$PC2_ROOT"
mode="${1:-}"
[[ $# -le 1 && ( -z "$mode" || "$mode" == --static || "$mode" == --horizon ) ]] || {
    echo 'Использование: scripts/check.sh [--static|--horizon]' >&2; exit 2;
}
python3 scripts/fetch-tools.py
python3 scripts/audit.py
python3 -m unittest discover -s tests -v
if [[ "$mode" == --static ]]; then
    echo 'Структурные проверки завершены. Rust и интеграция с игрой этой командой не проверяются.'
    exit 0
fi
command -v cargo >/dev/null || { echo 'Cargo недоступен: Rust-проверки не выполнены.' >&2; exit 1; }
cargo fmt --all --check
rustfmt --edition 2024 --check native/powercraft2/mixins/powercraft2/{listen,item_tick,placement}.rs
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
if [[ "$mode" == --horizon ]]; then
    command -v rsync >/dev/null || { echo 'Для native.py check нужен rsync' >&2; exit 1; }
    command -v cargo-horizon-mod >/dev/null || { echo 'Для проверки .hmod нужен cargo-horizon-mod' >&2; exit 1; }
    pc2_require_horizon_space
    # Downloads pinned source, stages with official Mixins, checks the real game.
    python3 native/tools/native.py check powercraft2
    python3 scripts/package.py
    cargo horizon-mod check dist/powercraft2-0.1.0-content.hmod
    cargo horizon-mod check native/target/powercraft2-0.1.0.hnmod
fi

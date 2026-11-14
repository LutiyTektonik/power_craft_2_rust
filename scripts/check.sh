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
rustfmt --edition 2024 --check native/powercraft2/mixins/powercraft2/*.rs
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
if [[ "$mode" == --horizon ]]; then
    command -v rsync >/dev/null || { echo 'Для native.py check нужен rsync' >&2; exit 1; }
    command -v cargo-horizon-mod >/dev/null || { echo 'Для проверки .hmod нужен cargo-horizon-mod' >&2; exit 1; }
    pc2_require_horizon_space check
    # Downloads pinned source, stages with official Mixins, checks the real game.
    python3 native/tools/native.py check powercraft2
    python3 native/tools/native.py package powercraft2
    python3 scripts/prepare-windows.py --native-only
    (cd content && cargo horizon-mod build)
    mkdir -p dist
    content_version="$(python3 -c 'import tomllib; print(tomllib.load(open("content/horizon-mod.toml", "rb"))["version"])')"
    cp content/target/horizon-mod/powercraft2.hmod "dist/powercraft2-${content_version}-content.hmod"
    cargo horizon-mod check "dist/powercraft2-${content_version}-content.hmod"
    native_version="$(python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["package"]["version"])')"
    cargo horizon-mod check "dist/powercraft2-${native_version}.hnmod"
fi

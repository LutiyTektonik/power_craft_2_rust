#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"
cd "$PC2_ROOT"
mode="${1:-}"
[[ $# -le 1 && ( -z "$mode" || "$mode" == --source-only || "$mode" == --horizon ) ]] || {
    echo 'Использование: scripts/build.sh [--source-only|--horizon]' >&2; exit 2;
}
python3 scripts/fetch-tools.py
python3 scripts/audit.py
native_version="$(python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["package"]["version"])')"
python3 scripts/package.py
python3 native/tools/native.py package powercraft2
python3 scripts/prepare-windows.py --native-only
if [[ "$mode" == --source-only ]]; then
    echo 'Созданы архивы исходников/контента. Это не проверка компиляции или запуска игры.'
    exit 0
fi
command -v cargo >/dev/null || { echo 'Cargo недоступен: исходные архивы созданы, Rust не собран.' >&2; exit 1; }
cargo build --release --locked --offline
command -v cargo-horizon-mod >/dev/null || { echo 'Нет официального cargo-horizon-mod' >&2; exit 1; }
(cd content && cargo horizon-mod build)
cp content/target/horizon-mod/powercraft2.hmod dist/powercraft2-0.1.0-content.hmod
cargo horizon-mod check dist/powercraft2-0.1.0-content.hmod
cargo horizon-mod check "dist/powercraft2-${native_version}.hnmod"
if [[ "$mode" == --horizon ]]; then
    pc2_require_horizon_space
    python3 native/tools/native.py build powercraft2
fi

#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"
cd "$PC2_ROOT"
install=0
[[ "${1:-}" == --install ]] && install=1
if [[ $# -gt 1 || ( $# -eq 1 && "$1" != --install ) ]]; then
    echo 'Использование: scripts/bootstrap.sh [--install]' >&2
    exit 2
fi
for tool in git python3 curl cc ar make; do
    command -v "$tool" >/dev/null || { echo "Нет инструмента: $tool" >&2; exit 1; }
done
python3 -c 'import sys; assert sys.version_info >= (3, 11), "Нужен Python >= 3.11"'
if [[ "$install" == 1 ]]; then
    python3 scripts/fetch-tools.py --download
else
    python3 scripts/fetch-tools.py
fi
if ! command -v cargo >/dev/null || ! command -v rustc >/dev/null; then
    if [[ "$install" == 0 ]]; then
        echo 'Rust отсутствует. Для разрешённой установки: scripts/bootstrap.sh --install' >&2
        exit 1
    fi
    mkdir -p .tools/cargo .tools/rustup
    export CARGO_HOME="$PC2_ROOT/.tools/cargo"
    export RUSTUP_HOME="$PC2_ROOT/.tools/rustup"
    export PATH="$CARGO_HOME/bin:$PATH"
    # Official rustup only. Keep inherited proxies/CA settings and TLS verification.
    curl --proto '=https' --tlsv1.2 -fsSL --connect-timeout 5 --max-time 30 \
        https://sh.rustup.rs -o .tools/rustup-init.sh
    sh .tools/rustup-init.sh -y --no-modify-path --profile minimal \
        --default-toolchain 1.95.0 --component rustfmt --component clippy
fi
if command -v rustup >/dev/null; then
    if ! rustup run 1.95.0 rustc --version >/dev/null 2>&1; then
        [[ "$install" == 1 ]] || { echo 'Нет закреплённого stable Rust 1.95.0' >&2; exit 1; }
        rustup toolchain install 1.95.0 --profile minimal --component rustfmt --component clippy
    fi
    if [[ "$install" == 1 ]]; then
        if ! cargo fmt --version >/dev/null 2>&1; then
            rustup component add --toolchain 1.95.0 rustfmt
        fi
        if ! cargo clippy --version >/dev/null 2>&1; then
            rustup component add --toolchain 1.95.0 clippy
        fi
    fi
fi
rustc --version
cargo --version
cargo fmt --version
cargo clippy --version
if ! command -v rsync >/dev/null; then
    if [[ "$install" == 1 && -x /usr/local/bin/cloud-agent-packages && "$(id -u)" == 0 ]]; then
        /usr/local/bin/cloud-agent-packages update
        /usr/local/bin/cloud-agent-packages install rsync
    elif [[ "$install" == 1 && "$(id -u)" == 0 ]]; then
        apt-get update
        apt-get install --no-install-recommends -y rsync
    elif [[ "$install" == 1 ]] && command -v apt-get >/dev/null && command -v dpkg-deb >/dev/null; then
        # Use the configured official APT sources and a private cache. Extract
        # only the required Debian tools under the project, without sudo.
        mkdir -p .cache/apt/lists/partial .cache/apt/archives/partial .cache/system-packages
        apt_args=(-o "Dir::State::lists=$PC2_ROOT/.cache/apt/lists"
                  -o "Dir::Cache=$PC2_ROOT/.cache/apt"
                  -o Dir::State::status=/var/lib/dpkg/status)
        apt-get "${apt_args[@]}" update
        (cd .cache/system-packages && apt-get "${apt_args[@]}" download rsync libpopt0)
        for package in .cache/system-packages/*.deb; do
            dpkg-deb -x "$package" .tools
        done
        source scripts/env.sh
        rsync --version
    else
        echo 'rsync отсутствует; нужен для полной native.py check/build. Системные права не запрашивались.' >&2
    fi
fi
if [[ "$install" == 1 ]] && ! command -v cargo-horizon-mod >/dev/null; then
    revision="$(python3 -c 'import json; print(json.load(open("upstream.lock.json"))["horizon"]["revision"])')"
    # Dedicated Cargo target prevents a tools install from bloating the source tree.
    CARGO_TARGET_DIR="$PC2_ROOT/.cache/tool-build" cargo install --locked \
        --git https://github.com/horizon-rust/horizon --rev "$revision" cargo-horizon-mod
    rm -rf -- "$PC2_ROOT/.cache/tool-build"
fi
echo 'Базовые инструменты доступны. Скрипт не загружает ресурсы Minecraft и не собирает игру.'

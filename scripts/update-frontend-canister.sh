#!/usr/bin/env bash
set -euo pipefail

SCRIPT=$(readlink -f "${0}")
SCRIPT_DIR=$(dirname "${SCRIPT}")
cd "${SCRIPT_DIR}/.."

if ! command -v rustc &> /dev/null; then
    echo "Must have Rust installed" >&2
    exit 1
fi

rust_version=$(rustc --version | cut -f 2 -d ' ') # fetches from rust-toolchain.toml

if [ -d "${CARGO_HOME:-"$HOME/.cargo"}/registry/index" ]; then
    registry_flag="--build-context=registry=${CARGO_HOME:-"$HOME/.cargo"}/registry/index"
fi

docker buildx build . -f "$SCRIPT_DIR/update-frontend-canister.Dockerfile" -o src/distributed \
    --build-arg=RUST_VERSION="$rust_version" ${registry_flag:+"$registry_flag"}

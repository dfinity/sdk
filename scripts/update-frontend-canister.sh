#!/usr/bin/env bash
set -euo pipefail

SCRIPT=$(readlink -f "${0}")
SCRIPT_DIR=$(dirname "${SCRIPT}")
cd ${SCRIPT_DIR}/..

if ! command -v rustc &> /dev/null; then
    echo "Must have Rust installed" >&2
    exit 1
fi

rust_version=$(rustc --version | cut -wf 2) # fetches from rust-toolchain.toml

DOCKER_BUILDKIT=1 docker build . -f "$SCRIPT_DIR/update-frontend-canister.Dockerfile" -o src/distributed --build-arg=RUST_VERSION=$rust_version

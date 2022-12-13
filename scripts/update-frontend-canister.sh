#!/usr/bin/env bash
set -euo pipefail

help()
{
   # display help
   echo "Build frontend canister wasm. Copies the wasm build artifact"
   echo "and corresponding candid file (src/canisters/frontend/ic-certified-assets/assets.did)"
   echo "to src/distributed/assetstorage.did."
   echo
   echo "Options:"
   echo "  -d, --development-build    build canister using cargo"
   echo "  -r, --release-build        build canister using linux/amd64 docker image"
   echo "  -h, --help                 print this help message"
   echo
}

SCRIPT=$(readlink -f "${0}")
SCRIPT_DIR=$(dirname "${SCRIPT}")
cd "${SCRIPT_DIR}/.."

if ! command -v rustc &> /dev/null; then
    echo "Must have Rust installed" >&2
    exit 1
fi

rust_version=$(rustc --version | cut -f 2 -d ' ') # fetches from rust-toolchain.toml

case ${1---help} in

  --development-build | -d)
    cargo --version >/dev/null || die "Must have cargo installed."
    ic-wasm --version >/dev/null || die "Must have ic-wasm installed."

    BUILD_DIR="target/wasm32-unknown-unknown/release"

    cargo build -p ic-frontend-canister --release --target wasm32-unknown-unknown

    ic-wasm --output $BUILD_DIR/ic_frontend_canister.wasm $BUILD_DIR/ic_frontend_canister.wasm metadata --file src/canisters/frontend/ic-certified-assets/assets.did --visibility public candid:service
    ic-wasm --output $BUILD_DIR/ic_frontend_canister.wasm $BUILD_DIR/ic_frontend_canister.wasm shrink
    gzip --best --keep --force --no-name $BUILD_DIR/ic_frontend_canister.wasm

    cp -f $BUILD_DIR/ic_frontend_canister.wasm.gz src/distributed/assetstorage.wasm.gz
    cp -f src/canisters/frontend/ic-certified-assets/assets.did src/distributed/assetstorage.did
    ;;

  --release-build | -r)
    if [ -d "${CARGO_HOME:-"$HOME/.cargo"}/registry/index" ]; then
        registry_flag="--build-context=registry=${CARGO_HOME:-"$HOME/.cargo"}/registry/index"
    fi

    docker --version >/dev/null || die "Must have docker installed."

    docker buildx build  . \
        -f "$SCRIPT_DIR/update-frontend-canister.Dockerfile" -o src/distributed \
        --build-arg=RUST_VERSION="$rust_version" ${registry_flag:+"$registry_flag"} \
        --platform linux/amd64 \
        --progress plain
    ;;

  --help | -h)
    help
    ;;
esac


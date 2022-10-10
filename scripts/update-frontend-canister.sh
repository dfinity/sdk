#!/usr/bin/env bash

set -e
die() {
  echo "$1" >&2
  exit 1
}

cargo --version >/dev/null || die "Must have cargo installed."

SCRIPT=$(readlink -f "${0}")
SCRIPT_DIR=$(dirname "${SCRIPT}")
cd ${SCRIPT_DIR}/..

if [ -z "${CARGO_HOME}" ]
then
  export CARGO_HOME="${HOME}/.cargo"
fi

export RUSTFLAGS="--remap-path-prefix $(readlink -f ${SCRIPT_DIR}/..)=/build --remap-path-prefix ${CARGO_HOME}/bin=/cargo/bin --remap-path-prefix ${CARGO_HOME}/git=/cargo/git"
for l in $(ls ${CARGO_HOME}/registry/src/)
do
  export RUSTFLAGS="--remap-path-prefix ${CARGO_HOME}/registry/src/${l}=/cargo/registry/src/github ${RUSTFLAGS}"
done
cargo build -p ic-frontend-canister --release --target wasm32-unknown-unknown

BUILD_DIR="target/wasm32-unknown-unknown/release"
ic-wasm --output $BUILD_DIR/ic_frontend_canister.wasm $BUILD_DIR/ic_frontend_canister.wasm metadata --file src/canisters/frontend/ic-certified-assets/assets.did --visibility public candid:service
ic-wasm --output $BUILD_DIR/ic_frontend_canister.wasm $BUILD_DIR/ic_frontend_canister.wasm shrink
gzip --best --keep --force --no-name $BUILD_DIR/ic_frontend_canister.wasm

cp -f $BUILD_DIR/ic_frontend_canister.wasm.gz src/distributed/assetstorage.wasm.gz
cp -f src/canisters/frontend/ic-certified-assets/assets.did src/distributed/assetstorage.did

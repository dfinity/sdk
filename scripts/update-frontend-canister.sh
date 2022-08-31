#!/usr/bin/env bash

set -e
die() {
  echo "$1" >&2
  exit 1
}

cargo --version >/dev/null || die "Must have cargo installed."

export RUSTFLAGS="--remap-path-prefix=\"${PWD}\"=./ --remap-path-prefix=\"${HOME}\"=_/"
cargo build -p ic-assets-canister --release --target wasm32-unknown-unknown

BUILD_DIR="target/wasm32-unknown-unknown/release"
ic-wasm --output $BUILD_DIR/ic_assets_canister.wasm $BUILD_DIR/ic_assets_canister.wasm metadata --file src/canisters/frontend/ic-certified-assets/assets.did --visibility public candid:service
ic-wasm --output $BUILD_DIR/ic_assets_canister.wasm $BUILD_DIR/ic_assets_canister.wasm shrink
gzip --best --keep --force --no-name $BUILD_DIR/ic_assets_canister.wasm

cp -f $BUILD_DIR/ic_assets_canister.wasm.gz src/distributed/assetstorage.wasm.gz
cp -f src/canisters/frontend/ic-certified-assets/assets.did src/distributed/assetstorage.did

#!/usr/bin/env bash

die() {
  echo "$1" >&2
  exit 1
}

cargo --version >/dev/null || die "Must have cargo installed."

export RUSTFLAGS="--remap-path-prefix=\"${PWD}\"=./ --remap-path-prefix=\"${HOME}\"=_/"
cargo build -p certified-assets --release --target wasm32-unknown-unknown

BUILD_DIR="target/wasm32-unknown-unknown/release"
ic-wasm --output $BUILD_DIR/certified_assets.wasm $BUILD_DIR/certified_assets.wasm metadata --file src/canisters/frontend/ic-certified-assets/assets.did --visibility public candid:service
ic-wasm --output $BUILD_DIR/certified_assets.wasm $BUILD_DIR/certified_assets.wasm shrink
gzip --best --keep --force --no-name $BUILD_DIR/certified_assets.wasm

cp -f $BUILD_DIR/certified_assets.wasm.gz src/distributed/assetstorage.wasm
cp -f src/canisters/frontend/ic-certified-assets/assets.did src/distributed/assetstorage.did

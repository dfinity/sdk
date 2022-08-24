#!/usr/bin/env bash

die() {
  echo "$1" >&2
  exit 1
}

cargo --version >/dev/null || die "Must have cargo installed."

BUILD_DIR="../../../target/wasm32-unknown-unknown/release"
export RUSTFLAGS="--remap-path-prefix=\"${PWD}\"=./ --remap-path-prefix=\"${HOME}\"=_/"
cargo build --release --target wasm32-unknown-unknown -p certified-assets
ic-wasm --output $BUILD_DIR/certified_assets.wasm $BUILD_DIR/certified_assets.wasm metadata --file ../ic-certified-assets/assets.did --visibility public candid:service
ic-wasm --output $BUILD_DIR/certified_assets.wasm $BUILD_DIR/certified_assets.wasm shrink
gzip --best --keep --force $BUILD_DIR/certified_assets.wasm

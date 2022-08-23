#!/usr/bin/env bash

die() {
  echo "$1" >&2
  exit 1
}

cargo --version >/dev/null || die "Must have cargo installed."

export RUSTFLAGS="--remap-path-prefix=\"${PWD}\"=./ --remap-path-prefix=\"${HOME}\"=_/"
cargo build --release --target wasm32-unknown-unknown

if cargo install ic-cdk-optimizer --root target; then
  target/bin/ic-cdk-optimizer \
    target/wasm32-unknown-unknown/release/certified_assets.wasm \
    -o target/wasm32-unknown-unknown/release/certified_assets-opt.wasm
else
  die "Could not install ic-cdk-optimizer (see above)."
fi

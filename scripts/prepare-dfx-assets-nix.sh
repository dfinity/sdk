#!/usr/bin/env bash

set -e

SDK_ROOT_DIR="$( cd -- "$(dirname -- "$( dirname -- "${BASH_SOURCE[0]}" )" )" &> /dev/null && pwd )"
DFX_ASSETS_DIR="${1?'Must specify a destination directory'}"

BUILT_DFX_ASSETS_DIR="$(nix-build $SDK_ROOT_DIR/assets.nix)"

rm -rf "$DFX_ASSETS_DIR"
mkdir -p "$DFX_ASSETS_DIR"
cp -R "$BUILT_DFX_ASSETS_DIR/" "$DFX_ASSETS_DIR/"

echo "Created $DFX_ASSETS_DIR"

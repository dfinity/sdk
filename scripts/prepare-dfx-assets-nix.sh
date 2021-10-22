#!/usr/bin/env bash

set -e

SDK_ROOT_DIR="$( cd -- "$(dirname -- "$( dirname -- "${BASH_SOURCE[0]}" )" )" &> /dev/null && pwd )"
DFX_ASSETS_DIR="$SDK_ROOT_DIR/.dfx-assets"

rm -rf "$DFX_ASSETS_DIR"
mkdir -p "$DFX_ASSETS_DIR"

(
    cd "$SDK_ROOT_DIR"

    BUILT_DFX_ASSETS_DIR="$(nix-build assets.nix)"
    cp -R "$BUILT_DFX_ASSETS_DIR/" "$DFX_ASSETS_DIR/"
)

echo "Created $DFX_ASSETS_DIR"

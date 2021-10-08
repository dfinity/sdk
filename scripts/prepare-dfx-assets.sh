#!/usr/bin/env bash

set -ex

which -s jq || ( echo "Please install jq in order to run this script." ; exit 1 )

SDK_ROOT_DIR="$( cd -- "$(dirname -- "$( dirname -- "${BASH_SOURCE[0]}" )" )" &> /dev/null && pwd )"
DFX_ASSETS_DIR="$SDK_ROOT_DIR/.dfx-assets2"
BINARY_CACHE_TEMP_DIR=$(mktemp -d)
DOWNLOAD_TEMP_DIR=$(mktemp -d)

MACHINE=$(uname -m) # ex: x86_64
case "$OSTYPE" in
  darwin*)  PLATFORM="darwin" ;;
  linux*)   PLATFORM="linux" ;;
  *)        echo "Unsupported OS type: $OSTYPE"  ; exit 1;;
esac

if [[ -L "$DFX_ASSETS_DIR" ]]; then
    rm "$DFX_ASSETS_DIR"
else
    rm -f -v "$DFX_ASSETS_DIR/*.tgz"
fi

mkdir -p "$DFX_ASSETS_DIR"

tar -czf "$DFX_ASSETS_DIR"/assetstorage_canister.tgz -C "$SDK_ROOT_DIR"/src/distributed ./assetstorage.did ./assetstorage.wasm
tar -czf "$DFX_ASSETS_DIR"/wallet_canister.tgz -C "$SDK_ROOT_DIR"/src/distributed ./wallet.did ./wallet.wasm
tar -czf "$DFX_ASSETS_DIR"/ui_canister.tgz -C "$SDK_ROOT_DIR"/src/distributed ./ui.did ./ui.wasm

download_binary() {
    NAME="$1"
    KEY="replica-$MACHINE-$PLATFORM"
    URL=$(jq -r .'"'"$KEY"'".url' nix/sources.json)
    EXPECTED_SHA256=$(jq -r .'"'"$KEY"'".sha256' nix/sources.json)
    echo "replica URL is $URL with sha256 $SHA256"
    curl -o "$BINARY_CACHE_TEMP_DIR/$NAME.gz" "$URL"

    # This doesn't work, because...
    ACTUAL_SHA256=$(shasum -a 256 "$BINARY_CACHE_TEMP_DIR/$NAME.gz")

    # ... Nix uses a nonstandard base32 hash format.  What now?
    ACTUAL_SHA256=$(nix-hash --flat --base32 --type sha256 "$BINARY_CACHE_TEMP_DIR/$NAME.gz")

    if [ "$EXPECTED_SHA256" != "$ACTUAL_SHA256" ]; then
        echo "SHA256 mismatch for $URL: expected $EXPECTED_SHA256, got $ACTUAL_SHA256"
        exit 1
    fi

    gunzip "$BINARY_CACHE_TEMP_DIR/$NAME.gz"
    chmod 0444 "$BINARY_CACHE_TEMP_DIR/$NAME"
}

build_icx_proxy() {
    REV="$(jq -r .\"agent-rs\".rev nix/sources.json)"
    REPO="$(jq -r .\"agent-rs\".repo nix/sources.json)"
    echo "repo $REPO rev $REV"
    TMPDIR="$(mktemp -d)"
    (
        cd "$TMPDIR"
        git clone "$REPO"
        (
            cd agent-rs
            git checkout "$REV"
            cargo build --release -p icx-proxy
            cp target/release/icx-proxy "$BINARY_CACHE_TEMP_DIR/icx-proxy"
            chmod 0444 "$BINARY_CACHE_TEMP_DIR/icx-proxy"
        )
    )
    rm -rf "$TMPDIR"
}

download_binary "replica"
download_binary "ic-starter"
build_icx_proxy

tar -czf "$DFX_ASSETS_DIR"/binary_cache.tgz -C "$BINARY_CACHE_TEMP_DIR" .

echo "Binary cache temp dir:"
ls -l "$BINARY_CACHE_TEMP_DIR"

echo "download temp dir:"
ls -l "$DOWNLOAD_TEMP_DIR"

echo
rm -rf "$DOWNLOAD_TEMP_DIR" "$BINARY_CACHE_TEMP_DIR"

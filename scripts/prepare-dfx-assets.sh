#!/usr/bin/env bash

set -e

echo >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
echo "Starting... $(date)" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
echo "output path=$1" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt

which -s jq || ( echo "Please install jq in order to run this script." ; exit 1 )

export >/Users/ericswanson/trouble.txt


SDK_ROOT_DIR="$( cd -- "$(dirname -- "$( dirname -- "${BASH_SOURCE[0]}" )" )" &> /dev/null && pwd )"
NIX_SOURCES_JSON="$SDK_ROOT_DIR/nix/sources.json"

DFX_ASSETS_FINAL_DIR=${1:-"$SDK_ROOT_DIR/.dfx-assets"}
echo "DFX_ASSETS_FINAL_DIR=$DFX_ASSETS_FINAL_DIR" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt

DFX_ASSETS_TEMP_DIR=$(mktemp -d)
BINARY_CACHE_TEMP_DIR=$(mktemp -d)
DOWNLOAD_TEMP_DIR=$(mktemp -d)

echo "DFX_ASSETS_TEMP_DIR=$DFX_ASSETS_TEMP_DIR" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
echo "BINARY_CACHE_TEMP_DIR=$BINARY_CACHE_TEMP_DIR" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
echo "DOWNLOAD_TEMP_DIR=$DOWNLOAD_TEMP_DIR" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt

function cleanup {
    rm -rf "$DFX_ASSETS_TEMP_DIR" "$BINARY_CACHE_TEMP_DIR" "$DOWNLOAD_TEMP_DIR"
}
trap cleanup EXIT

MACHINE=$(uname -m) # ex: x86_64
case "$OSTYPE" in
    darwin*)  PLATFORM="darwin" ;;
    linux*)   PLATFORM="linux" ;;
    *)        echo "Unsupported OS type: $OSTYPE"  ; exit 1;;
esac

add_canisters() {
    echo "add_canisters" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
    tar -czf "$DFX_ASSETS_TEMP_DIR"/assetstorage_canister.tgz -C "$SDK_ROOT_DIR"/src/distributed ./assetstorage.did ./assetstorage.wasm
    tar -czf "$DFX_ASSETS_TEMP_DIR"/wallet_canister.tgz -C "$SDK_ROOT_DIR"/src/distributed ./wallet.did ./wallet.wasm
    tar -czf "$DFX_ASSETS_TEMP_DIR"/ui_canister.tgz -C "$SDK_ROOT_DIR"/src/distributed ./ui.did ./ui.wasm
}

download_url_and_check_sha() {
    URL="$1"
    EXPECTED_SHA256="$2"
    LOCAL_PATH="$3"

    echo "Downloading $URL with expected sha256=$EXPECTED_SHA256 to $LOCAL_PATH"

    curl --fail --location --output "$LOCAL_PATH" "$URL"

    # This doesn't work, because...
    ACTUAL_SHA256=$(shasum -a 256 "$LOCAL_PATH" | cut -f 1 -d ' ')

    # # ... Nix uses a nonstandard base32 hash format.  What now?
    # ACTUAL_SHA256=$(nix-hash --flat --base32 --type sha256 "$BINARY_CACHE_TEMP_DIR/$NAME.gz")

    if [ "$EXPECTED_SHA256" != "$ACTUAL_SHA256" ]; then
        echo "SHA256 mismatch for $URL: expected $EXPECTED_SHA256, got $ACTUAL_SHA256"
        exit 1
    fi
}

download_url_from_nix_sources_and_check_sha() {
    NAME="$1"
    LOCAL_PATH="$2"
    KEY="$NAME-$MACHINE-$PLATFORM"

    URL=$(jq -r .'"'"$KEY"'".url' "$NIX_SOURCES_JSON")

    EXPECTED_SHA256_BASE32=$(jq -r .'"'"$KEY"'".sha256' "$NIX_SOURCES_JSON")

    # ... Nix uses a nonstandard base32 hash format.  So...
    EXPECTED_SHA256=$(nix to-base16 --type sha256 "$EXPECTED_SHA256_BASE32")

    download_url_and_check_sha "$URL" "$EXPECTED_SHA256" "$LOCAL_PATH"
}

download_binary() {
    echo download_binary "$1" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
    NAME="$1"

    DOWNLOAD_PATH="$DOWNLOAD_TEMP_DIR/$NAME.gz"
    BINARY_CACHE_PATH="$BINARY_CACHE_TEMP_DIR/$NAME"

    download_url_from_nix_sources_and_check_sha "$NAME" "$DOWNLOAD_PATH"

    gunzip -c "$DOWNLOAD_PATH" >"$BINARY_CACHE_PATH"
    chmod 0500 "$BINARY_CACHE_PATH"
}

download_ic_ref() {
    echo download_ic_ref >>/Users/ericswanson/prepare-dfx-assets-invocations.txt

    NAME="ic-ref"

    DOWNLOAD_PATH="$DOWNLOAD_TEMP_DIR/$NAME.tar.gz"

    download_url_from_nix_sources_and_check_sha "$NAME" "$DOWNLOAD_PATH"

    tar -xvf "$DOWNLOAD_PATH" -C "$BINARY_CACHE_TEMP_DIR"
}

download_motoko_binaries() {
    echo download_motoko_binaries >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
    NAME="motoko"

    DOWNLOAD_PATH="$DOWNLOAD_TEMP_DIR/$NAME.tar.gz"

    download_url_from_nix_sources_and_check_sha "$NAME" "$DOWNLOAD_PATH"

    tar -xvf "$DOWNLOAD_PATH" -C "$BINARY_CACHE_TEMP_DIR"

    for a in mo-doc mo-ide moc;
    do
        chmod 0500 "$BINARY_CACHE_TEMP_DIR/$a"
    done
}

download_motoko_base() {
    echo download_motoko_base >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
    # not used, because the sha256 doesn't match the tarball itself, rather, it's a nix hash of over the contents.
    NAME="motoko-base"

    KEY="$NAME"
    LOCAL_PATH="$DOWNLOAD_TEMP_DIR/$NAME.tar.gz"

    URL=$(jq -r .'"'"$KEY"'".url' "$NIX_SOURCES_JSON")

    EXPECTED_SHA256_BASE32=$(jq -r .'"'"$KEY"'".sha256' "$NIX_SOURCES_JSON")

    # ... Nix uses a nonstandard base32 hash format.  So...
    EXPECTED_SHA256=$(nix to-base16 --type sha256 "$EXPECTED_SHA256_BASE32")

    download_url_and_check_sha "$URL" "$EXPECTED_SHA256" "$LOCAL_PATH"
}

copy_motoko_base() {
    echo copy_motoko_base >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
    # the sha256 in nix/sources.json doesn't match the sha256 of the archive, it's a nix hash of the contents
    # so we will clone the repo

    KEY="motoko-base"

    REV=$(jq -r .'"'"$KEY"'".rev' "$NIX_SOURCES_JSON")
    BRANCH=$(jq -r .'"'"$KEY"'".branch' "$NIX_SOURCES_JSON")

    (
        cd "$DOWNLOAD_TEMP_DIR" # ok technically we are not downloading

        git clone -b "$BRANCH" --single-branch https://github.com/dfinity/motoko-base.git
        (
            cd motoko-base
            git checkout "$REV"
            cp -R src/ "$BINARY_CACHE_TEMP_DIR/base"
        )
    )
}

build_icx_proxy() {
    echo build_icx_proxy >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
    BRANCH="$(jq -r .\"agent-rs\".branch "$NIX_SOURCES_JSON")"
    REV="$(jq -r .\"agent-rs\".rev "$NIX_SOURCES_JSON")"
    REPO="$(jq -r .\"agent-rs\".repo "$NIX_SOURCES_JSON")"
    echo "repo $REPO rev $REV" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
    TEMP_BUILD_DIR="$(mktemp -d)"
    (
        cd "$TEMP_BUILD_DIR"
        echo clone... >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
        git clone -b "$BRANCH" --single-branch "$REPO"

        (
            cd agent-rs
            echo checkout... >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
            git checkout "$REV"
            echo cargo build in $(pwd)... >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
            cargo build --release -p icx-proxy 2>&1 >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
            cp target/release/icx-proxy "$BINARY_CACHE_TEMP_DIR/icx-proxy"
            chmod 0500 "$BINARY_CACHE_TEMP_DIR/icx-proxy"
        )
    )
    rm -rf "$TEMP_BUILD_DIR"
}

add_binary_cache() {
    echo "add_binary_cache" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt
    download_binary "replica"
    download_binary "ic-starter"
    download_ic_ref
    download_motoko_binaries
    #download_motoko_base
    copy_motoko_base

    build_icx_proxy

    tar -czf "$DFX_ASSETS_TEMP_DIR"/binary_cache.tgz -C "$BINARY_CACHE_TEMP_DIR" .
}

echo "Building $DFX_ASSETS_FINAL_DIR"

add_canisters
add_binary_cache

rm -rf "$DFX_ASSETS_FINAL_DIR"
mv "$DFX_ASSETS_TEMP_DIR" "$DFX_ASSETS_FINAL_DIR"

echo "...Finished $(date)" >>/Users/ericswanson/prepare-dfx-assets-invocations.txt


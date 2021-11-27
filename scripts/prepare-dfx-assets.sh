#!/usr/bin/env bash

set -e

SDK_ROOT_DIR="$( cd -- "$(dirname -- "$( dirname -- "${BASH_SOURCE[0]}" )" )" &> /dev/null && pwd )"

# shellcheck disable=SC1090
source "$SDK_ROOT_DIR/scripts/dfx-asset-sources.sh"

DFX_ASSETS_FINAL_DIR=${1?'Must specify a destination directory.'}

DFX_ASSETS_TEMP_DIR=$(mktemp -d)
BINARY_CACHE_TEMP_DIR=$(mktemp -d)
DOWNLOAD_TEMP_DIR=$(mktemp -d)

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

    ACTUAL_SHA256=$(shasum -a 256 "$LOCAL_PATH" | cut -f 1 -d ' ')

    if [ "$EXPECTED_SHA256" != "$ACTUAL_SHA256" ]; then
        echo "SHA256 mismatch for $URL: expected $EXPECTED_SHA256, got $ACTUAL_SHA256"
        exit 1
    fi
}

get_variable() {
    NAME="$1"
    PART="$2"

    VAR_NAME=$(echo "${NAME}_${MACHINE}_${PLATFORM}_${PART}" | tr '[:lower:]-' '[:upper:]_')
    VAR_VALUE=${!VAR_NAME}

    echo "$VAR_VALUE"
}

download_binary() {
    NAME="$1"
    SHA256=$(get_variable "$NAME" "SHA256")
    URL=$(get_variable "$NAME" "URL")

    DOWNLOAD_PATH="$DOWNLOAD_TEMP_DIR/$NAME.gz"
    BINARY_CACHE_PATH="$BINARY_CACHE_TEMP_DIR/$NAME"

    download_url_and_check_sha "$URL" "$SHA256" "$DOWNLOAD_PATH"

    gunzip -c "$DOWNLOAD_PATH" >"$BINARY_CACHE_PATH"
    chmod 0500 "$BINARY_CACHE_PATH"
}

download_tarball() {
    NAME="$1"

    SHA256=$(get_variable "$NAME" "SHA256")
    URL=$(get_variable "$NAME" "URL")
    DOWNLOAD_PATH="$DOWNLOAD_TEMP_DIR/$NAME.tar.gz"

    download_url_and_check_sha "$URL" "$SHA256" "$DOWNLOAD_PATH"

    # -k: some archives contain r-x ".", and on linux the default behavior is to overwrite the
    # metadata.  We only want to extract new files anyway.
    tar -xkvf "$DOWNLOAD_PATH" -C "$BINARY_CACHE_TEMP_DIR"
}

download_ic_ref() {
    download_tarball "ic-ref"
    chmod 0500 "$BINARY_CACHE_TEMP_DIR/ic-ref"
}

download_motoko_binaries() {
    download_tarball "motoko"

    for a in mo-doc mo-ide moc;
    do
        chmod 0500 "$BINARY_CACHE_TEMP_DIR/$a"
    done
}

copy_motoko_base_from_clone() {
    REV=$MOTOKO_BASE_REV
    BRANCH=$MOTOKO_BASE_BRANCH

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
    BRANCH="$ICX_PROXY_BRANCH"
    REV="$ICX_PROXY_REV"
    REPO="$ICX_PROXY_REPO"

    (
        cd "$DOWNLOAD_TEMP_DIR" # ok technically we are not downloading

        git clone -b "$BRANCH" --single-branch "$REPO"
        (
            cd icx-proxy
            git checkout "$REV"
            cargo build --release -p icx-proxy
            cp target/release/icx-proxy "$BINARY_CACHE_TEMP_DIR/icx-proxy"
            chmod 0500 "$BINARY_CACHE_TEMP_DIR/icx-proxy"
        )
    )
}

add_binary_cache() {
    download_binary "replica"
    download_binary "ic-starter"
    download_ic_ref
    download_motoko_binaries
    copy_motoko_base_from_clone

    build_icx_proxy

    tar -czf "$DFX_ASSETS_TEMP_DIR"/binary_cache.tgz -C "$BINARY_CACHE_TEMP_DIR" .
}

echo "Building $DFX_ASSETS_FINAL_DIR"

add_canisters
add_binary_cache

if [ -d "$DFX_ASSETS_FINAL_DIR" ]
then
    (
        cd "$DFX_ASSETS_FINAL_DIR"
        rm -f binary_cache.tgz assetstorage_canister.tgz wallet_canister.tgz ui_canister.tgz
    )
    rmdir "$DFX_ASSETS_FINAL_DIR"
fi
mv "$DFX_ASSETS_TEMP_DIR" "$DFX_ASSETS_FINAL_DIR"

echo "Built $DFX_ASSETS_FINAL_DIR"

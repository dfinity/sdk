#!/usr/bin/env bash

set -euo pipefail

#   $1 not set   ||   not running in repo root
if [ -z ${1+x} ] || [ ! "$0" -ef ./scripts/update-motoko.sh ]; then
    echo "Usage: run ./scripts/update-motoko.sh <version-to-update-to> in repo root"
    exit 1
fi

for cmd in curl jq sponge; do
    if ! command -v "$cmd" &>/dev/null; then
        echo "'$cmd' was not found."
        echo "This script requires curl, jq, and moreutils to be installed"
        exit 1
    fi
done

urlencode() {
    printf '%s' "$1" | jq -sRr @uri
}

sources="src/dfx/assets/dfx-asset-sources.json"
version=$1
echo "Updating sources to version ${version}"

motoko_base_url=$(printf 'https://github.com/dfinity/motoko/releases/download/%s/motoko-base-library.tar.gz' "$(urlencode "$version")")
motoko_base_sha=$(curl --proto '=https' --tlsv1.2 -sSfL "$motoko_base_url" | sha256sum | head -c 64)
jq '.common."motoko-base" = {url: $url, sha256: $sha256, version: $version}' --arg version "$version" \
    --arg url "$motoko_base_url" --arg sha256 "$motoko_base_sha" "$sources" | sponge "$sources"

declare -A variants=([x86_64-darwin]=Darwin-x86_64 [x86_64-linux]=Linux-x86_64 [arm64-darwin]=Darwin-arm64 [arm64-linux]=Linux-aarch64)
for platform in "${!variants[@]}"; do
    motoko_url=$(printf 'https://github.com/dfinity/motoko/releases/download/%s/motoko-%s-%s.tar.gz' \
        "$(urlencode "$version")" "$(urlencode "${variants[$platform]}")" "$(urlencode "$version")")
    motoko_sha=$(curl --proto '=https' --tlsv1.2 -sSfL "$motoko_url" | sha256sum | head -c 64)
    jq '.[$platform].motoko = {url: $url, sha256: $sha256, version: $version}' --arg platform "$platform" --arg version "$version" \
        --arg url "$motoko_url" --arg sha256 "$motoko_sha" "$sources" | sponge "$sources"
done

echo "Done. Don't forget to update CHANGELOG.md"

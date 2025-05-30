#!/usr/bin/env bash
set -euo pipefail

#   $1 not set   ||   not running in repo root
if [ -z ${1+x} ] || [ ! "$0" -ef ./scripts/update-btc-canister.sh ]; then
    echo "Usage: run ./scripts/update-btc-canister.sh <version-to-update-to> in repo root"
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
echo "Updating sources to version $version"
btc_canister_url=$(printf 'https://github.com/dfinity/bitcoin-canister/releases/download/%s/ic-btc-canister.wasm.gz' "$(urlencode "$version")")
btc_canister_sha=$(curl --proto '=https' --tlsv1.2 -sSfL "$btc_canister_url" | sha256sum | head -c 64)
jq '.common."ic-btc-canister" = {url: $url, sha256: $sha256, version: $version}' --arg version "$version" \
    --arg url "$btc_canister_url" --arg sha256 "$btc_canister_sha" "$sources" | sponge "$sources"

echo "Done. Don't forget to update CHANGELOG.md"

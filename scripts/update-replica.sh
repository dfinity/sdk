#!/usr/bin/env bash

set -euo pipefail

#   $1 not set   ||   not running in repo root
if [ -z ${1+x} ] || [ ! "$0" -ef ./scripts/update-replica.sh ]; then
    echo "Usage: run ./scripts/update-replica.sh <SHA-to-update-to> in repo root"
    exit 1
fi

for cmd in curl jq sponge; do
    if ! command -v "$cmd" &>/dev/null; then
        echo "'$cmd' was not found."
        echo "This script requires curl, jq, and moreutils to be installed"
        exit 1
    fi
done

sources="src/dfx/assets/dfx-asset-sources.json"

rev=$1
echo "Updating sources to rev ${rev}"
jq '."replica-rev" = $rev' --arg rev "$rev" "$sources" | sponge "$sources"

declare -A variants=([x86_64-darwin]=pocket-ic.gz [x86_64-linux]=pocket-ic.gz [arm64-darwin]=pocket-ic-server-arm64-darwin)
for platform in "${!variants[@]}"; do
    pocketic_url=$(printf 'https://download.dfinity.systems/ic/%s/binaries/%s/%s' "$rev" "$platform" "${variants[$platform]}")
    pocketic_sha=$(curl --proto '=https' --tlsv1.2 -sSfL "$pocketic_url" | sha256sum | head -c 64)
    jq '.[$platform]."pocket-ic" = {url: $url, sha256: $sha256, rev: $rev}' --arg platform "$platform" --arg rev "$rev" \
        --arg url "$pocketic_url" --arg sha256 "$pocketic_sha" "$sources" | sponge "$sources"
done

# pocket-ic client needs to be upgraded to the same rev as the pocket-ic server
perl -i.bak -pe "s/(pocket-ic = {[^}]*rev = \")[a-f0-9]+(\")/\${1}$rev\${2}/" src/dfx/Cargo.toml
cargo update -p pocket-ic # refresh the lock file

echo "Done. Don't forget to update CHANGELOG.md"

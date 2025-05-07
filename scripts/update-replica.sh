#!/usr/bin/env bash

set -euo pipefail

#   $1 not set   ||   not running in repo root
if [ -z ${1+x} ] || [ ! "$0" -ef ./scripts/update-replica.sh ]; then
    echo "Usage: run ./scripts/update-replica.sh <SHA-to-update-to> in repo root"
    exit 1
fi
if ! command -v jq sponge curl cargo &>/dev/null; then
    echo "This script requires Rust, jq, moreutils, and curl to be installed"
    exit 1
fi
sources="src/dfx/assets/dfx-asset-sources.json"

rev=$1
echo "Updating sources to rev ${rev}"
jq '."replica-rev" = $rev' --arg rev "$rev" "$sources" | sponge "$sources"
for platform in x86_64-darwin x86_64-linux; do
    pocketic_url=$(printf 'https://download.dfinity.systems/ic/%s/binaries/%s/pocket-ic.gz' "$rev" "$platform")
    pocketic_sha=$(curl --proto '=https' --tlsv1.2 -sSfL "$pocketic_url" | sha256sum | head -c 64)
    jq '.[$platform]."pocket-ic" = {url: $url, sha256: $sha256, rev: $rev}' --arg platform "$platform" --arg rev "$rev" \
        --arg url "$pocketic_url" --arg sha256 "$pocketic_sha" "$sources" | sponge "$sources"
done

# pocket-ic client needs to be upgraded to the same rev as the pocket-ic server
perl -i.bak -pe "s/(pocket-ic = {[^}]*rev = \")[a-f0-9]+(\")/\${1}$rev\${2}/" src/dfx/Cargo.toml
cargo update -p pocket-ic # refresh the lock file

echo "Done. Don't forget to update CHANGELOG.md"

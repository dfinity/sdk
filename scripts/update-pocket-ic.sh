#!/usr/bin/env bash

set -euo pipefail

#   $1 not set   ||   not running in repo root
if [ -z ${1+x} ] || [ ! "$0" -ef ./scripts/update-pocket-ic.sh ]; then
    echo "Usage: run ./scripts/update-pocket-ic.sh <version-to-update-to> in repo root"
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

version=$1
echo "Updating sources to version ${version}"

for platform in x86_64-darwin x86_64-linux; do
    pocketic_url=$(printf 'https://github.com/dfinity/pocketic/releases/download/%s/pocket-ic-%s.gz' "$version" "$platform")
    pocketic_sha=$(curl --proto '=https' --tlsv1.2 -sSfL "$pocketic_url" | sha256sum | head -c 64)
    jq '.[$platform]."pocket-ic" = {url: $url, sha256: $sha256, version: $version}' --arg platform "$platform" --arg version "$version" \
        --arg url "$pocketic_url" --arg sha256 "$pocketic_sha" "$sources" | sponge "$sources"
done

# TODO: pocket-ic client needs to be upgraded to the compatible version as the pocket-ic server
# perl -i.bak -pe "s/(pocket-ic = {[^}]*rev = \")[a-f0-9]+(\")/\${1}$rev\${2}/" src/dfx/Cargo.toml
# cargo update -p pocket-ic # refresh the lock file

echo "Done. Don't forget to update CHANGELOG.md"

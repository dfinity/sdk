#!/usr/bin/env bash

set -e

die () {
    echo >&2 "$@"
    exit 1
}

[ "$#" -eq 1 ] || die "Usage: $0 <n.n.n>"
echo $1 | grep -E -q '^[0-9]+\.[0-9]+\.[0-9]+$' || die "'$1' is not a valid semantic version"

export NEW_DFX_VERSION=$1

echo "Building release: $NEW_DFX_VERSION"
nix-shell --option extra-binary-caches https://cache.dfinity.systems --command "$(envsubst < scripts/release-nix-command.sh)"
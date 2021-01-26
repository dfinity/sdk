#!/usr/bin/env bash

set -ex

die () {
    echo >&2 "$@"
    exit 1
}

[ "$#" -eq 1 ] || die "Usage: $0 <n.n.n>"
echo $1 | grep -E -q '^[0-9]+\.[0-9]+\.[0-9]+$' || die "'$1' is not a valid semantic version"

export NEW_DFX_VERSION=$1

echo "Building release: $NEW_DFX_VERSION"

! IFS='' read -r -d '' Command <<EOFF
    #git switch -c $USER/release-$NEW_DFX_VERSION

    # update first version in src/dfx/Cargo.toml to be NEW_DFX_VERSION

    sed -i '0,/^version = ".*"/s//version = "$NEW_DFX_VERSION"/' src/dfx/Cargo.toml

    #cargo build
EOFF

#echo "$Command"

nix-shell --option extra-binary-caches https://cache.dfinity.systems --command "$Command"

echo nix-shell --option extra-binary-caches https://cache.dfinity.systems --command '
    #git switch -c $USER/release-$NEW_DFX_VERSION

    # update first version in src/dfx/Cargo.toml to be NEW_DFX_VERSION

    sed -i '"'"'0,/^version = ".*"/s//version = "$NEW_DFX_VERSION"/'"'"' src/dfx/Cargo.toml

    #cargo build

'
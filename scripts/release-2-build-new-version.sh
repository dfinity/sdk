#!/usr/bin/env bash

set -e

die () {
    echo >&2 "$@"
    exit 1
}

[ "$#" -eq 1 ] || die "Usage: $0 <n.n.n>"
echo $1 | grep -E -q '^[0-9]+\.[0-9]+\.[0-9]+$' || die "'$1' is not a valid semantic version"

export NEW_DFX_VERSION=$1
export BRANCH=$USER/release-$NEW_DFX_VERSION

echo "Building release: $NEW_DFX_VERSION"
NIX_COMMAND=$(envsubst <<"EOF"
    set -e

    echo "Switching to branch: $BRANCH"
    echo git switch -c $BRANCH

    echo "Updating version in src/dfx/Cargo.toml"
    # update first version in src/dfx/Cargo.toml to be NEW_DFX_VERSION
    sed -i '0,/^version = ".*"/s//version = "$NEW_DFX_VERSION"/' src/dfx/Cargo.toml

    echo "Building dfx with cargo."
    cargo build

    echo "Appending version to public/manifest.json"
    # Append the new version to `public/manifest.json` by appending it to the `versions` list.
    cat <<<$(jq --indent 4 '.versions += ["$NEW_DFX_VERSION"]' public/manifest.json) >public/manifest.json

    echo "Creating release branch: $BRANCH"
    echo git add --all
    echo git commit --signoff --message "chore: Release $NEW_DFX_VERSION"
    echo git push origin $BRANCH

    echo "Please open a pull request, review and approve it, and then label automerge-squash."
EOF
)
echo "$NIX_COMMAND"
echo "Starting nix-shell."
nix-shell --option extra-binary-caches https://cache.dfinity.systems --command "$NIX_COMMAND"

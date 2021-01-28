#!/usr/bin/env bash

set -e

nix-shell --option extra-binary-caches https://cache.dfinity.systems --command '
    set -e

    export NEW_DFX_VERSION=$(jq -r .versions[-1] public/manifest.json)
    export EXPECTED_BRANCH_NAME=$USER/release-$NEW_DFX_VERSION
    if [[ "$(git branch --show-current)" != "$EXPECTED_BRANCH_NAME" ]]; then
        echo "This script must be run from the release branch $EXPECTED_BRANCH_NAME in order to release $NEW_DFX_VERSION"
        exit 1
    fi

    echo "NEW_DFX_VERSION is $NEW_DFX_VERSION"

    echo "Switching to the stable branch."
    echo git switch stable

    echo "Pulling the remove stable branch into the local stable branch."
    echo git pull origin stable

    echo "Pulling the merged changes into the stable branch."
    echo git pull origin master --ff-only

    echo "Creating a new tag $NEW_DFX_VERSION"
    echo git tag --annotate $NEW_DFX_VERSION --message "Release: $NEW_DFX_VERSION"

    echo "Displaying tags"
    git log -1
    git describe --always

    echo "Pushing tag $NEW_DFX_VERSION"
    echo git push origin NEW_DFX_VERSION

    echo "Updating the stable branch."
    echo git push origin stable
'

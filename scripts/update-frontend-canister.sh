#!/usr/bin/env bash
set -euo pipefail

help()
{
   # display help
   echo "Build frontend canister wasm. Copies the wasm build artifact"
   echo "and corresponding candid file (src/canisters/frontend/ic-certified-assets/assets.did)"
   echo "to src/distributed/assetstorage.did."
   echo
   echo "Options:"
   echo "  -d, --development-build    build canister using cargo"
   echo "  -r, --release-build        build canister using linux/amd64 docker image"
   echo "  -c, --changelog            update CHANGELOG.md with latest release"
   echo "  -h, --help                 print this help message"
   echo
}

update_changelog()
{
    REPO_ROOT=$(git rev-parse --show-toplevel)
    CHANGELOG_PATH="$REPO_ROOT/CHANGELOG.md"
    CHANGELOG_PATH_BACKUP="$REPO_ROOT/CHANGELOG.md.bak"
    UNRELEASED_LOC=$(grep -nE "# UNRELEASED" "$CHANGELOG_PATH" | head -n 1 | cut -f1 -d:)
    DEPENDENCIES_LOC=$(grep -nE "## Dependencies" "$CHANGELOG_PATH" | head -n 1 | cut -f1 -d:)
    FRONTEND_CANISTER_LOC=$(grep -nE "### Frontend canister" "$CHANGELOG_PATH" | head -n 1 | cut -f1 -d:)
    MODULE_HASH_LOC=$(grep -nE "\- Module hash: [a-f0-9]{64}" "$CHANGELOG_PATH" | head -n 1 | cut -f1 -d:)
    LATEST_RELEASE_LOC=$(grep -nE "# \d+\.\d+\.\d+" "$CHANGELOG_PATH" | head -n 1 | cut -f1 -d:)
    # shellcheck disable=SC2004
    LINE_ABOVE_LAST_RELEASE=$(($LATEST_RELEASE_LOC - 1))
    NEW_WASM_CHECKSUM=$(shasum -a 256 "$REPO_ROOT/src/distributed/assetstorage.wasm.gz" | awk '{print $1}')

    if ! command -v gh &> /dev/null
    then
        echo "gh could not be found (brew install gh && gh auth login)"
        exit
    fi
    PR_NUMBER=$(gh pr view --json number --jq '.number')
    if [ -z "$PR_NUMBER" ]; then
        echo "Could not find PR number. We will help you create a new PR."
        echo "! Please make sure you are on the correct branch."
        echo "! Please make sure you've made at least one commit on this branch."
        gh pr create --base "dfinity/sdk"
    fi

    PR_NUMBER=$(gh pr view --json number --jq '.number')
    LINK_TO_PR="https://github.com/dfinity/sdk/pull/$PR_NUMBER"

    if [ -z "$UNRELEASED_LOC" ] || [ "$UNRELEASED_LOC" -gt "$LATEST_RELEASE_LOC" ]; then
        echo "No \"# Unreleased\" section found in changelog, or \"# Unreleased\" section is not at the top of the changelog"
        exit 1
    fi

    cp "$CHANGELOG_PATH" "$CHANGELOG_PATH_BACKUP"

    if [ "$DEPENDENCIES_LOC" -lt "$LATEST_RELEASE_LOC" ]; then
        # Dependencies section is present just above the latest release
        if [ "$FRONTEND_CANISTER_LOC" -lt "$LATEST_RELEASE_LOC" ]; then
            # Frontend canister section is present in the Dependencies section.
            # Adding the new wasm checksum.
            awk 'NR==loc && $0~filter && $0~target{gsub(target,replacement)}1' \
                loc="${MODULE_HASH_LOC}" \
                filter="- Module hash: " \
                target=": [a-f0-9]{64}" \
                replacement=": ${NEW_WASM_CHECKSUM}" \
                "$CHANGELOG_PATH_BACKUP" | sponge "$CHANGELOG_PATH_BACKUP"
            # read line below MODULE_HASH_LOC and check if it contains LINK_TO_PR, if not, add it
            if ! grep -q "$LINK_TO_PR" <(sed -n "$((MODULE_HASH_LOC+1))p" "$CHANGELOG_PATH_BACKUP"); then
                awk 'NR==loc{print replacement}1' \
                    loc=$((MODULE_HASH_LOC+1)) \
                    replacement="- ${LINK_TO_PR}" \
                    "$CHANGELOG_PATH_BACKUP" | sponge "$CHANGELOG_PATH_BACKUP"
            fi

        else
            # Frontend canister section is not present in the Dependencies section.
            # It needs to be added together with the new wasm checksum and the link to the PR
            CONTENT="\n### Frontend canister\n\n- Module hash: ${NEW_WASM_CHECKSUM}\n- ${LINK_TO_PR}"
            awk 'NR==loc {print content}1' \
                content="$CONTENT" \
                loc="$LINE_ABOVE_LAST_RELEASE" \
                "$CHANGELOG_PATH_BACKUP" | sponge "$CHANGELOG_PATH_BACKUP"
        fi
    else
        # Dependencies section is not present under Unreleased section.
        # It needs to be added together with the Frontend canister section
        # and the new wasm checksum and the link to the PR.
        CONTENT="\n## Dependencies\n\n### Frontend canister\n\n- Module hash: ${NEW_WASM_CHECKSUM}\n- ${LINK_TO_PR}"
        awk 'NR==loc {print content}1' \
            content="$CONTENT" \
            loc="$LINE_ABOVE_LAST_RELEASE" \
            "$CHANGELOG_PATH_BACKUP" | sponge "$CHANGELOG_PATH_BACKUP"
    fi

    # Using git diff to get nice colorful output across all OSes.
    if diff -q "$CHANGELOG_PATH" "$CHANGELOG_PATH_BACKUP" &>/dev/null; then
        echo "No changes to the changelog"
        rm "$CHANGELOG_PATH_BACKUP"
        exit 1
    fi

    echo "Suggested Changelog updates:"
    git diff -U8 --no-index "$CHANGELOG_PATH" "$CHANGELOG_PATH_BACKUP" && echo
    echo
    echo "Please review the suggested changes before applying them to CHANGELOG.md"
    read -r -p "Do you want to apply these changes to the changelog? [y/N] " response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]
    then
        mv "$CHANGELOG_PATH_BACKUP" "$CHANGELOG_PATH"
        echo "Changelog updated"
    else
        echo "Aborting"
        rm "$CHANGELOG_PATH_BACKUP"
    fi
}



SCRIPT=$(readlink -f "${0}")
SCRIPT_DIR=$(dirname "${SCRIPT}")
cd "${SCRIPT_DIR}/.."

if ! command -v rustc &> /dev/null; then
    echo "Must have Rust installed" >&2
    exit 1
fi

rust_version=$(rustc --version | cut -f 2 -d ' ') # fetches from rust-toolchain.toml

case ${1---help} in

  --development-build | -d)
    cargo --version >/dev/null || die "Must have cargo installed."
    ic-wasm --version >/dev/null || die "Must have ic-wasm installed."

    BUILD_DIR="target/wasm32-unknown-unknown/release"

    cargo build -p ic-frontend-canister --release --target wasm32-unknown-unknown

    ic-wasm --output $BUILD_DIR/ic_frontend_canister.wasm $BUILD_DIR/ic_frontend_canister.wasm metadata --file src/canisters/frontend/ic-certified-assets/assets.did --visibility public candid:service
    ic-wasm --output $BUILD_DIR/ic_frontend_canister.wasm $BUILD_DIR/ic_frontend_canister.wasm shrink
    gzip --best --keep --force --no-name $BUILD_DIR/ic_frontend_canister.wasm

    cp -f $BUILD_DIR/ic_frontend_canister.wasm.gz src/distributed/assetstorage.wasm.gz
    cp -f src/canisters/frontend/ic-certified-assets/assets.did src/distributed/assetstorage.did
    ;;

  --release-build | -r)
    if [ -d "${CARGO_HOME:-"$HOME/.cargo"}/registry/index" ]; then
        registry_flag="--build-context=registry=${CARGO_HOME:-"$HOME/.cargo"}/registry/index"
    fi

    docker --version >/dev/null || die "Must have docker installed."

    docker buildx build  . \
        -f "$SCRIPT_DIR/update-frontend-canister.Dockerfile" -o src/distributed \
        --build-arg=RUST_VERSION="$rust_version" ${registry_flag:+"$registry_flag"} \
        --platform linux/amd64 \
        --progress plain
    # check if its being run inside a github CI
    # if no, then run the changelog update script
    if [[ -z ${CI+x} ]]; then
        update_changelog
    fi
    ;;

  --changelog | -c)
    update_changelog
    ;;

  --help | -h)
    help
    ;;
esac


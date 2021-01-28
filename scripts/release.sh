#!/usr/bin/env bash

set -e

die () {
    echo >&2 "$@"
    exit 1
}

announce() {
    term_green
    echo
    echo "======================================="
    echo "= $1"
    echo "======================================="
    echo
    term_reset
}

term_green() {
  tput setaf 2
}

term_reset() {
  tput sgr0
}

get_parameters() {
    [ "$#" -eq 1 ] || die "Usage: $0 <n.n.n>"
    echo $1 | grep -E -q '^[0-9]+\.[0-9]+\.[0-9]+$' || die "'$1' is not a valid semantic version"

    export NEW_DFX_VERSION=$1
}

pre_release_check() {
    announce "Ensuring dfx and replica are not running."
    if [[ $(ps -ef | grep -v grep | grep -E 'replica|dfx') ]] ; then
        echo "dfx and replica cannot still be running.  kill them and try again."
        exit 1
    fi
}

build_release_candidate() {
    announce "Building dfx release candidate."
    x="$(nix-build ./dfx.nix -A build --option extra-binary-caches https://cache.dfinity.systems)"
    echo $x
    export sdk_rc=$x
    echo $sdk_rc

    export dfx_rc="$sdk_rc/bin/dfx"

    echo "Checking for dfx release candidate."
    test -x $dfx_rc

    echo "Deleting existing dfx cache to make sure not to use a stale binary."
    $dfx_rc cache delete

    echo "Building the JavaScript agent."
    export agent_js_rc="$(nix-build . -A agent-js --option extra-binary-caches https://cache.dfinity.systems)"
    export agent_js_rc_npm_packed="$(sh -c 'echo "$1"' sh $agent_js_rc/dfinity-agent-*.tgz)"

    echo "Checking for the packed JS agent."
    test -f $agent_js_rc_npm_packed
}

wait_for_response() {
    expected="$1"
    while true; do
        echo "All good?  Type '$expected' to continue."
        read answer
        if [ "$answer" == "$expected" ]; then
            break
        fi
    done
}

validate_default_project() {
    announce "Validating default project."
    PROJECTDIR=$(mktemp -d -t dfx-release-XXXXXXXX)
    trap 'rm -rf -- "$PROJECTDIR"' EXIT

    (
        cd $PROJECTDIR

        echo "Creating new project."
        $dfx_rc new hello_world
        cd hello_world

        echo "Installing the locally-build JavaScript agent."
        npm install "$agent_js_rc_npm_packed"

        echo "Starting the local 'replica' as a background process."
        $dfx_rc start --background

        echo "Deploying canisters."
        $dfx_rc deploy

        echo "Calling the canister."
        $dfx_rc canister call hello_world greet everyone

        export hello_world_assets_url="http://localhost:8000/?canisterId=$($dfx_rc canister id hello_world_assets)"
        export hello_world_candid_url="http://localhost:8000/candid?canisterId=$($dfx_rc canister id hello_world)"

        echo
        echo "=================================================="
        echo "dfx project directory: $(pwd)"
        echo "assets URL: $hello_world_assets_url"
        echo "candid URL: $hello_world_candid_url"
        echo "=================================================="
        echo
        echo "[1/4] Verify 'hello' functionality in a browser."
        echo "  - Open this URL in your web browser with empty cache or 'Private Browsing' mode"
        echo "  - Type a name and verify the response."
        echo
        echo "  $hello_world_assets_url"
        echo
        wait_for_response 'assets UI passes'
        echo
        echo "[2/4] Verify there are no errors in the console by opening the Developer Tools."
        echo
        wait_for_response 'no errors on console'
        echo
        echo "[3/4] Verify the Candid UI."
        echo
        echo "  - Open this URL in your web browser with empty cache or 'Private Browsing' mode"
        echo "  - Verify UI loads, then test the greet function by entering text and clicking *Call* or clicking *Lucky*"
        echo
        echo "  $hello_world_candid_url"
        echo
        wait_for_response 'candid UI passes'
        echo
        echo "[4/4] Verify there are no errors in the console by opening the Developer Tools."
        echo
        wait_for_response 'no errors on console'
        echo

        $dfx_rc stop
    )
}

build_release_branch() {
    export BRANCH=$USER/release-$NEW_DFX_VERSION

    announce "Building branch $BRANCH for release $NEW_DFX_VERSION"
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
    # echo "$NIX_COMMAND"
    echo "Starting nix-shell."
    nix-shell --option extra-binary-caches https://cache.dfinity.systems --command "$NIX_COMMAND"

    wait_for_response 'PR approved'
}

get_parameters $*
pre_release_check
build_release_candidate
# validate_default_project
build_release_branch

echo "All done!"
exit 0

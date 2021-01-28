#!/usr/bin/env bash

set -e

pre_release_check()
build_release_candidate()
validate_default_project()

pre_release_check() {
    echo "Ensuring dfx and replica are not running."
    if [[ $(ps -ef | grep -v grep | grep -E 'replica|dfx') ]] ; then
        echo "dfx and replica cannot still be running.  kill them and try again."
        exit 1
    fi
}

build_release_candidate() {
    echo "Building dfx release candidate."
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
        echo "All good? type '$expected' if it worked."
        read answer
        if [ "$answer" == "$expected" ]; then
            break
        fi
    done
}

validate_default_project() {
    echo "Validating default project."
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
echo "All done!"
#!/usr/bin/env bash

set -e

die () {
    echo >&2 "$@"
    exit 1
}

announce() {
    term_green
    echo
    echo "===================================================================="
    echo "= $1"
    echo "===================================================================="
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
    export BRANCH=$USER/release-$NEW_DFX_VERSION

    if [[ "$DRY_RUN" == '' ]]; then
        export DRY_RUN_ECHO=''
        dry_run_explain=''
    else
        export DRY_RUN_ECHO='echo DRY RUN: '
        dry_run_explain=' (dry run)'
    fi

    announce "Building release $NEW_DFX_VERSION as branch $BRANCH $dry_run_explain"
}

pre_release_check() {
    announce "Ensuring dfx and replica are not running."
    if [[ $(ps -ef | grep -v grep | grep -E 'replica|dfx') ]] ; then
        echo "dfx and replica cannot still be running.  kill them and try again."
        exit 1
    fi
}

#
# build the release candidate and export these environment variables:
#    sdk_rc                  SDK release candidate
#    dfx_rc                    - dfx executable within
#    agent_js_rc             JavaScript agent release candidate
#    agent_js_rc_npm_packed    - npm unpacked
#
build_release_candidate() {
    announce "Building dfx release candidate."
    x="$(nix-build ./dfx.nix -A build --option extra-binary-caches https://cache.dfinity.systems)"
    export sdk_rc=$x
    export dfx_rc="$sdk_rc/bin/dfx"

    echo "Checking for dfx release candidate at $dfx_rc"
    test -x $dfx_rc

    echo "Deleting existing dfx cache to make sure not to use a stale binary."
    $dfx_rc cache delete

    echo "Building the JavaScript agent."
    x="$(nix-build . -A agent-js --option extra-binary-caches https://cache.dfinity.systems)"
    export agent_js_rc=$x

    x="$(sh -c 'echo "$1"' sh $agent_js_rc/dfinity-agent-*.tgz)"
    export agent_js_rc_npm_packed=$x

    echo "Checking for the packed JS agent at $agent_js_rc_npm_packed"
    test -f $agent_js_rc_npm_packed
}

wait_for_response() {
    expected="$1"
    while true; do
        echo
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

        echo "Installing webpack and webpack-cli"
        npm install webpack webpack-cli
        npm install terser-webpack-plugin

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

    announce "Building branch $BRANCH for release $NEW_DFX_VERSION"
    NIX_COMMAND=$(envsubst <<"EOF"
        set -e

        echo "Switching to branch: $BRANCH"
        $DRY_RUN_ECHO git switch -c $BRANCH

        echo "Updating version in src/dfx/Cargo.toml"
        # update first version in src/dfx/Cargo.toml to be NEW_DFX_VERSION
        sed -i '0,/^version = ".*"/s//version = "$NEW_DFX_VERSION"/' src/dfx/Cargo.toml

        echo "Building dfx with cargo."
        cargo build

        echo "Appending version to public/manifest.json"
        # Append the new version to `public/manifest.json` by appending it to the `versions` list.
        cat <<<$(jq --indent 4 '.versions += ["$NEW_DFX_VERSION"]' public/manifest.json) >public/manifest.json

        echo "Creating release branch: $BRANCH"
        $DRY_RUN_ECHO git add --all
        $DRY_RUN_ECHO git commit --signoff --message "chore: Release $NEW_DFX_VERSION"
        $DRY_RUN_ECHO git push origin $BRANCH

        echo "Please open a pull request, review and approve it, and then label automerge-squash."
EOF
    )
    # echo "$NIX_COMMAND"
    echo "Starting nix-shell."
    nix-shell --option extra-binary-caches https://cache.dfinity.systems --command "$NIX_COMMAND"

    wait_for_response 'PR approved'
}

update_stable_branch() {
    announce 'Updating stable branch'

    NIX_COMMAND=$(envsubst <<"EOF"
        set -e

        echo "Switching to the stable branch."
        $DRY_RUN_ECHO git switch stable

        echo "Pulling the remove stable branch into the local stable branch."
        $DRY_RUN_ECHO git pull origin stable

        # This seems like a race condition in our release process.
        # A PR could have been merged to master.
        echo "Pulling the merged changes into the stable branch."
        $DRY_RUN_ECHO git pull origin master --ff-only

        echo "Creating a new tag $NEW_DFX_VERSION"
        $DRY_RUN_ECHO git tag --annotate $NEW_DFX_VERSION --message "Release: $NEW_DFX_VERSION"

        echo "Displaying tags"
        git log -1
        git describe --always

        echo "Pushing tag $NEW_DFX_VERSION"
        $DRY_RUN_ECHO git push origin $NEW_DFX_VERSION

        echo "Updating the stable branch."
        $DRY_RUN_ECHO git push origin stable
EOF
)

    nix-shell --option extra-binary-caches https://cache.dfinity.systems --command "$NIX_COMMAND"
}

publish_javascript_agent() {
    announce 'publishing JavaScript agent'

    # The 'confirmation' variable will be set by read, so make sure that
    # 'envsubst' does not substitute it with an empty string:
    export Q='$'

    NIX_COMMAND=$(envsubst <<"EOF"
        set -e

        (
            cd $(mktemp -d)
            tar -xvf "$agent_js_rc_npm_packed"
            cd package

            npm version $NEW_DFX_VERSION

            echo "check that every .js file has a .d.ts assigned and that every .js and .d.ts file has a source file that is not a test:"
            if diff <(find types src \( -name \*.d.ts -o -name \*.js \) -a \! -name \*.test.\* | sort) <(npm publish --dry-run 2>&1 | egrep 'npm notice [0-9.]*k?B' | awk '{ print $4 }' | grep -v package.json | grep -v README.md | sort) ; then
                echo "  - No discrepancies to report."
            else
                while true; do
                    echo "  - There were differences.  Type 'continue anyway' to ignore them or 'stop' to stop:"
                    read confirmation
                    if [[ "${Q}confirmation" == "continue anyway" ]]; then
                        echo "Onward!"
                        break
                    elif [[ "${Q}confirmation" == "stop" ]]; then
                        echo "Stopping."
                        exit 1
                    fi
                done
            fi

            echo "Logging in to npm"
            until $DRY_RUN_ECHO npm login ; do
                echo "Failed to log in to npm.  Try again, or Ctrl-C if you give up."
            done

            echo "Publishing to npm"
            until $DRY_RUN_ECHO npm publish ; do
                echo "Failed to publish to npm.  Press enter to try again, or Ctrl-C to stop."
                read
            done

            echo "Logging out of npm"
            $DRY_RUN_ECHO npm logout
        )
EOF
)

    nix-shell --option extra-binary-caches https://cache.dfinity.systems --command "$NIX_COMMAND"
}

get_parameters $*
pre_release_check
build_release_candidate
validate_default_project
build_release_branch
update_stable_branch
publish_javascript_agent

echo "All done!"


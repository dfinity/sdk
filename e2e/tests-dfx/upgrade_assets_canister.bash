#!/usr/bin/env bats

load ../utils/_
load ../utils/releases

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

# The e2e matrix should ensure that this test is not run on ic-ref, as the Wasm does
# heavy computation
@test "asset canister can be upgraded from the latest release version when storing a lot of data" {
    # As a starting point for the load, we looked at OpenChat's usage.
    # As of 2023-02-10, they had 40MB of assets spread over 135 files.
    # We'll use a bigger example (~3x in number of files, ~20x in total size) to add a safety margin.
    local -r total_files=400
    local file_size
    if [ "$(uname)" == "Darwin" ]; then
        file_size="2m"
    else
        file_size="2M"
    fi

    local -r file_size="2MB"
    local -r canister_name=e2e_project_frontend

    local -r asset_dir="src/${canister_name}/assets"
    for a in $(seq 1 $total_files); do
        dd if=/dev/urandom of="${asset_dir}/large-asset-${a}.bin" bs="$file_size" count=1 1>/dev/null
    done

    dfx_start
    dfx canister create --all
    dfx build

    # Install the canister using the Wasm from the latest release
    local -r release_asset_wasm_dir=$(mktemp -d)
    get_from_latest_release_tarball src/distributed/assetstorage.wasm.gz "$release_asset_wasm_dir"
    export DFX_ASSETS_WASM="${release_asset_wasm_dir}/assetstorage.wasm.gz"
    assert_command dfx canister install $canister_name

    use_default_asset_wasm
    assert_command dfx deploy $canister_name --upgrade-unchanged
}

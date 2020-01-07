#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
    dfx stop

    # Verify that processes are killed.
    ! ( ps | grep \ dfx\ start )
}

@test "build fails on invalid motoko" {
    install_asset invalid_mo
    assert_command_fail dfx build
    assert_match "syntax error"
}

@test "build succeeds on default project" {
    assert_command dfx build
}

@test "build outputs the canister ID" {
    assert_command dfx build
    [[ -f canisters/e2e_project/_canister.id ]]
}

@test "build insert the assets in the cannister" {
    dfx_new_frontend
    assert_command dfx build
    dfx_start
    assert_command dfx canister install --all
    assert_command dfx canister call --query e2e_project __dfx_asset_path --type=string index.js
    assert_match "Attempt to hash a value of unsupported type"
}

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
    assert_match "Building hello..."
}

@test "build outputs the canister ID" {
    assert_command dfx build
    [[ -f canisters/hello/_canister.id ]]
}

@test "build can take a single argument" {
    assert_command dfx build hello
    assert_match "Building hello..."

    assert_command_fail dfx build unknown_canister
    assert_match "Could not find.*unknown_canister.*"
}

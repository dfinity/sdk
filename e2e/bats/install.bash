#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
    dfx_stop
}

@test "install fails if no argument is provided" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    dfx_start
    assert_command_fail dfx canister install
    assert_match "required arguments were not provided"
    assert_match "--all"
}

@test "install succeeds when --all is provided" {
    dfx_start
    dfx build

    assert_command dfx canister install --all

    assert_match "Installing code for canister e2e_project"
}

@test "install succeeds with provider URL" {
    dfx_start
    dfx build

    assert_command dfx canister --provider http://127.0.0.1:8000 install --all

    assert_match "Installing code for canister e2e_project"
}

@test "install fails with incorrect provider URL" {
    dfx_start
    dfx build

    assert_command_fail dfx canister --provider http://127.0.0.1:8765 install --all

    assert_match "Installing code for canister e2e_project"
}

@test "install succeeds with network name" {
    dfx_start
    dfx build

    assert_command dfx canister --network local install --all

    assert_match "Installing code for canister e2e_project"
}

@test "install fails with network name that is not in dfx.json" {
    dfx_start
    dfx build

    assert_command_fail dfx canister --network nosuch install --all

    assert_match "ComputeNetworkNotFound.*nosuch"
}

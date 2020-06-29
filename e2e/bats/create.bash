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

@test "create succeeds on default project" {
    dfx_start
    assert_command dfx canister create --all
}

@test "build fails without create" {
    dfx_start
    assert_command_fail dfx build
    assert_match 'Failed to find canister manifest'
}

@test "build fails if all canisters in project are not created" {
    dfx_start
    assert_command dfx canister create e2e_project
    assert_command_fail dfx build
    assert_match 'Failed to find canister id for e2e_project_assets'
}
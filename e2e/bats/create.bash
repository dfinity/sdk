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

@test "create fails with incorrect provider URL default project" {
    dfx_start
    assert_command_fail dfx canister --provider http://127.0.0.1:8765 create --all
    assert_match "ConnectionRefused"
}

@test "create succeeds with network parameter" {
    dfx_start
    assert_command dfx canister --network local create --all
}

@test "create fails with incorrect network" {
    dfx_start
    assert_command_fail dfx canister --network nosuch create --all
    assert_match "ComputeNetworkNotFound"
}

@test "create succeeds when requested network is configured" {
    dfx_start

    assert_command dfx config networks.tungsten.providers '[ "http://127.0.0.1:8000" ]'
    assert_command dfx canister --network tungsten create --all
}

@test "create fails if selected network exists but has no providers" {
    dfx_start

    assert_command dfx config networks.tungsten.providers '[  ]'
    assert_command_fail dfx canister --network tungsten create --all
    assert_match "ComputeNetworkHasNoProviders"
}

@test "create fails with network parameter when network does not exist" {
    dfx_start
    assert_command dfx config networks.tungsten.providers '[ "http://not-real.nowhere.systems" ]'
    assert_command_fail dfx canister --network tungsten create --all
    assert_match "ConnectError"
}

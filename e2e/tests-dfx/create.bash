#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit
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

@test "create generates the canister_ids.json" {
    dfx_start
    assert_command dfx canister create --all
    [[ -f .dfx/local/canister_ids.json ]]
}

@test "build fails without create" {
    dfx_start
    assert_command_fail dfx build
    assert_match "Cannot find canister id."
}

@test "build fails if all canisters in project are not created" {
    dfx_start
    assert_command dfx canister create e2e_project
    assert_command_fail dfx build
    assert_match "Cannot find canister id. Please issue 'dfx canister create e2e_project_assets'"
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

    setup_actuallylocal_network
    assert_command dfx canister --network actuallylocal create --all
}

@test "create with wallet succeeds when requested network is configured" {
    dfx_start
    setup_actuallylocal_network

    assert_command dfx_set_wallet
    assert_command dfx canister --network actuallylocal create --all
}

@test "create fails if selected network exists but has no providers" {
    dfx_start

    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.actuallylocal.providers=[]' dfx.json)" >dfx.json
    assert_command_fail dfx canister --network actuallylocal create --all
    assert_match "Cannot find providers for network"
}

@test "create fails with network parameter when network does not exist" {
    dfx_start
    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.actuallylocal.providers=["http://not-real.nowhere.test."]' dfx.json)" >dfx.json
    assert_command_fail dfx canister --network actuallylocal create --all
    assert_match "dns error: failed to lookup address information"
}

#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "canister install --upgrade-unchanged upgrades even if the .wasm did not change" {
    dfx_start
    dfx canister create --all
    dfx build

    assert_command dfx canister install --all

    assert_command dfx canister install --all --mode upgrade
    assert_match "Module hash.*is already installed"

    assert_command dfx canister install --all --mode upgrade --upgrade-unchanged
    assert_not_match "Module hash.*is already installed"
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
    dfx canister create --all
    dfx build

    assert_command dfx canister install --all

    assert_match "Installing code for canister e2e_project"
}

@test "install succeeds with network name" {
    dfx_start
    dfx canister create --all
    dfx build

    assert_command dfx canister --network local install --all

    assert_match "Installing code for canister e2e_project"
}

@test "install fails with network name that is not in dfx.json" {
    dfx_start
    dfx canister create --all
    dfx build

    assert_command_fail dfx canister --network nosuch install --all

    assert_match "ComputeNetworkNotFound.*nosuch"
}

@test "install succeeds with arbitrary wasm" {
    dfx_start
    dfx canister create --all
    wallet="${archive:?}/wallet/0.10.0/wallet.wasm"
    assert_command dfx canister install e2e_project --wasm "$wallet"
    assert_command dfx canister info e2e_project
    assert_match "Module hash: 0x$(sha2sum "$wallet" | head -c 64)"
}

@test "install --all fails with arbitrary wasm" {
    dfx_start
    dfx canister create --all
    assert_command_fail dfx canister install --all --wasm "${archive:?}/wallet/0.10.0/wallet.wasm"
}

@test "install runs post-install tasks" {
    dfx_start
    # shellcheck disable=SC2094
    cat <<<"$(jq '.canisters.e2e_project.post_install="sh -c \"echo hello\""' dfx.json)" >dfx.json

    assert_command dfx canister create --all
    assert_command dfx build

    assert_command dfx canister install --all
    assert_match hello
    
    # shellcheck disable=SC2094
    cat <<<"$(jq '.canisters.e2e_project.post_install=["sh -c \"echo hello\"", "sh -c \"return 1\""]' dfx.json)" >dfx.json
    assert_command_fail dfx canister install --all --mode upgrade
    assert_match hello
}

@test "post-install tasks receive environment variables" {
    dfx_start
    # shellcheck disable=SC2094
    cat <<<"$(jq '.canisters.e2e_project.post_install="sh -c \"echo hello $CANISTER_ID\""' dfx.json)" >dfx.json

    assert_command dfx canister create --all
    assert_command dfx build
    id=$(dfx canister id e2e_project)

    assert_command dfx canister install --all
    assert_match "hello $id"
    assert_command dfx canister install e2e_project --mode upgrade
    assert_match "hello $id"

    assert_command dfx deploy
    assert_match "hello $id"
    assert_command dfx deploy e2e_project
    assert_match "hello $id"
}

@test "post-install tasks discover dependencies" {
    dfx_start
    # shellcheck disable=SC2094
    cat <<<"$(jq '.canisters.e2e_project_assets.post_install="sh -c \"echo hello $CANISTER_ID_e2e_project\""' dfx.json)" >dfx.json

    assert_command dfx canister create --all
    assert_command dfx build
    id=$(dfx canister id e2e_project)
    
    assert_command dfx canister install --all
    assert_match "hello $id"
}
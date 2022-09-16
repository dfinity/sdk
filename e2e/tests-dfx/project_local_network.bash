#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "dfx start starts a local network if dfx.json defines one" {
    dfx_new hello
    cat dfx.json
    define_project_network

    dfx_start
    dfx deploy

    assert_directory_not_exists "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY"
    assert_file_exists .dfx/network/local/pid
}

@test "can run more than one project network at the same time" {
    mkdir a
    cd a
    dfx_new hello
    install_asset counter
    define_project_network
    dfx_start
    dfx deploy
    dfx canister call hello_backend inc
    dfx canister call hello_backend inc

    cd ../..

    mkdir b
    cd b
    dfx_new hello
    install_asset counter
    define_project_network
    dfx_start
    dfx deploy
    dfx canister call hello_backend write '(6: nat)'

    cd ../..

    (
        cd a/hello
        assert_command dfx canister call hello_backend read
        assert_eq "(2 : nat)"
    )

    (
        cd b/hello
        assert_command dfx canister call hello_backend read
        assert_eq "(6 : nat)"
    )

    # the above would work even with a shared network.
    # So here's the real trick: they will have the same canister ids, because
    # each project has its own replica.
    HELLO_BACKEND_ID_A="$(cd a/hello ; dfx canister id hello_backend)"
    HELLO_BACKEND_ID_B="$(cd b/hello ; dfx canister id hello_backend)"
    assert_eq "$HELLO_BACKEND_ID_A" "$HELLO_BACKEND_ID_B"

    (cd a/hello ; dfx stop)
    (cd b/hello ; dfx stop)
}

@test "upgrade a wallet in a project-specific network" {
    [ "$USE_IC_REF" ] && skip "wallet upgrade with emulator times out often under CI"

    dfx_new hello
    define_project_network

    dfx_start
    use_wallet_wasm 0.8.2

    dfx identity get-wallet

    assert_command dfx canister info "$(dfx identity get-wallet)"
    assert_match "Module hash: 0x$(wallet_sha 0.8.2)"

    use_wallet_wasm 0.10.0

    dfx wallet upgrade
    assert_command dfx canister info "$(dfx identity get-wallet)"
    assert_match "Module hash: 0x$(wallet_sha 0.10.0)"
}

@test "with a project-specific network, wallet id is stored local to the project" {
    dfx_new hello
    define_project_network

    dfx_start

    assert_file_not_exists .dfx/local/wallets.json

    WALLET_ID="$(dfx identity get-wallet)"

    assert_command jq -r .identities.default.local .dfx/local/wallets.json
    assert_eq "$WALLET_ID"
    assert_file_not_exists "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/wallets.json"
}

@test "with a shared network, wallet id is stored in the shared location" {
    dfx_start

    assert_file_not_exists "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/wallets.json"

    WALLET_ID="$(dfx identity get-wallet)"

    assert_command jq -r .identities.default.local "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/wallets.json"
    assert_eq "$WALLET_ID"
    assert_file_not_exists .dfx/local/wallets.json
}

@test "for project-specific network create stores canister ids for default-ephemeral local networks in .dfx/{network}/canister_ids.json" {
    dfx_new hello
    define_project_network

    dfx_start

    dfx canister create --all --network local

    # canister creates writes to a spinner (stderr), not stdout
    assert_command dfx canister id hello_backend --network local
    assert_match "$(jq -r .hello_backend.local <.dfx/local/canister_ids.json)"
}

@test "for project-specific network, create with wallet stores canister ids for configured-ephemeral networks in canister_ids.json" {
    dfx_new hello
    define_project_network
    dfx_start

    setup_actuallylocal_project_network
    jq '.networks.actuallylocal.type="ephemeral"' dfx.json | sponge dfx.json
    dfx_set_wallet

    dfx canister create --all --network actuallylocal

    # canister creates writes to a spinner (stderr), not stdout
    assert_command dfx canister id hello_backend --network actuallylocal
    assert_match "$(jq -r .hello_backend.actuallylocal <.dfx/actuallylocal/canister_ids.json)"
}


@test "dfx start and stop take into account dfx 0.11.x pid files" {
    dfx_new hello
    define_project_network
    dfx_start

    mv .dfx/network/local/pid .dfx/pid

    assert_command_fail dfx start
    assert_match 'dfx is already running'

    assert_command dfx stop
    assert_file_not_exists .dfx/pid
    assert_not_match "Nothing to do"
    assert_command dfx stop
    assert_match "Nothing to do"
}

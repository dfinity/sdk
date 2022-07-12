#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx identity new --disable-encryption test_id
    dfx identity use test_id
    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "create with wallet stores canister ids for default-persistent networks in canister_ids.json" {
    dfx_start
    setup_actuallylocal_network
    dfx_set_wallet
    assert_command dfx_set_wallet

    assert_command dfx canister --network actuallylocal create --all

    # canister creates writes to a spinner (stderr), not stdout
    assert_command dfx canister --network actuallylocal id e2e_project_backend
    assert_match "$(jq -r .e2e_project_backend.actuallylocal <canister_ids.json)"
}

@test "create with wallet stores canister ids for configured-ephemeral networks in canister_ids.json" {
    dfx_start

    setup_actuallylocal_network
    # shellcheck disable=SC2094
    cat <<<"$(jq .networks.actuallylocal.type=\"ephemeral\" dfx.json)" >dfx.json
    assert_command dfx_set_wallet

    assert_command dfx canister --network actuallylocal create --all

    # canister creates writes to a spinner (stderr), not stdout
    assert_command dfx canister --network actuallylocal id e2e_project_backend
    assert_match "$(jq -r .e2e_project_backend.actuallylocal <.dfx/actuallylocal/canister_ids.json)"
}

@test "create stores canister ids for default-ephemeral local networks in .dfx/{network}canister_ids.json" {
    dfx_start

    assert_command dfx canister --network local create --all

    # canister creates writes to a spinner (stderr), not stdout
    assert_command dfx canister --network local id e2e_project_backend
    assert_match "$(jq -r .e2e_project_backend.local <.dfx/local/canister_ids.json)"
}


@test "create stores canister ids for configured-persistent local networks in canister_ids.json" {
    dfx_start

    # shellcheck disable=SC2094
    cat <<<"$(jq .networks.local.type=\"persistent\" dfx.json)" >dfx.json

    assert_command dfx canister --network local create --all

    # canister creates writes to a spinner (stderr), not stdout
    assert_command dfx canister --network local id e2e_project_backend
    assert_match "$(jq -r .e2e_project_backend.local <canister_ids.json)"
}

@test "failure message does not include network if for local network" {
    dfx_start
    assert_command_fail dfx build --network local
    assert_match "Cannot find canister id. Please issue 'dfx canister create e2e_project_frontend"
}

@test "failure message does include network if for non-local network" {
    dfx_start

    setup_actuallylocal_network

    assert_command_fail dfx build --network actuallylocal
    assert_match "Cannot find canister id. Please issue 'dfx canister --network actuallylocal create e2e_project"
}

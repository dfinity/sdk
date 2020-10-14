#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    run dfx identity new test_id
    dfx identity use test_id
    dfx_new
}

teardown() {
    dfx_stop
}

@test "create stores canister ids for default-persistent networks in canister_ids.json" {
    dfx_start
    dfx_set_wallet

    webserver_port=$(cat .dfx/webserver-port)
    assert_command dfx config networks.ic.providers '[ "http://127.0.0.1:'$webserver_port'" ]'

    assert_command dfx canister --network ic create --all

    # canister creates writes to a spinner (stderr), not stdout
    assert_command dfx canister --network ic id e2e_project
    assert_match $(cat canister_ids.json | jq -r .e2e_project.ic)
}

@test "create stores canister ids for configured-ephemeral networks in canister_ids.json" {
    dfx_start
    dfx_set_wallet

    webserver_port=$(cat .dfx/webserver-port)
    assert_command dfx config networks.ic.providers '[ "http://127.0.0.1:'$webserver_port'" ]'
    cat <<<$(jq .networks.ic.type=\"ephemeral\" dfx.json) >dfx.json

    assert_command dfx canister --network ic create --all

    # canister creates writes to a spinner (stderr), not stdout
    assert_command dfx canister --network ic id e2e_project
    assert_match $(cat .dfx/ic/canister_ids.json | jq -r .e2e_project.ic)
}

@test "create stores canister ids for default-ephemeral local networks in .dfx/{network}canister_ids.json" {
    dfx_start

    assert_command dfx canister --network local create --all

    # canister creates writes to a spinner (stderr), not stdout
    assert_command dfx canister --network local id e2e_project
    assert_match $(cat .dfx/local/canister_ids.json | jq -r .e2e_project.local)
}


@test "create stores canister ids for configured-persistent local networks in canister_ids.json" {
    dfx_start

    cat <<<$(jq .networks.local.type=\"persistent\" dfx.json) >dfx.json

    assert_command dfx canister --network local create --all

    # canister creates writes to a spinner (stderr), not stdout
    assert_command dfx canister --network local id e2e_project
    assert_match $(cat canister_ids.json | jq -r .e2e_project.local)
}

@test "failure message does not include network if for local network" {
    dfx_start
    assert_command_fail dfx build --network local
    assert_match "Cannot find canister id. Please issue 'dfx canister create e2e_project"
}

@test "failure message does include network if for non-local network" {
    dfx_start

    webserver_port=$(cat .dfx/webserver-port)
    assert_command dfx config networks.ic.providers '[ "http://127.0.0.1:'$webserver_port'" ]'

    assert_command_fail dfx build --network ic
    assert_match "Cannot find canister id. Please issue 'dfx canister --network ic create e2e_project"
}

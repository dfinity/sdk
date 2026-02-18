#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
    dfx_new hello
}

teardown() {
    dfx_stop
    standard_teardown
}

@test "canister migrate canister id" {
    dfx_start --system-canisters
    install_asset counter
    
    # Update dfx.json: rename hello_backend -> migrated, and add replaced canister
    jq '.canisters.migrated = .canisters.hello_backend | del(.canisters.hello_backend)' dfx.json | sponge dfx.json
    jq '.canisters.replaced = { "main": "counter.mo", "type": "motoko" }' dfx.json | sponge dfx.json

    # Deploy the migrated canister to the application subnet.
    dfx deploy migrated

    # Create the replaced canister on the fiduciary subnet.
    dfx canister create replaced --subnet-type fiduciary

    dfx canister stop migrated
    dfx canister stop replaced

    # Make sure the migrated canister has enough cycles to do the migration.
    dfx ledger fabricate-cycles --canister migrated --cycles 10000000000000

    # The migration will take a few minutes to complete.
    assert_command dfx canister migrate-id migrated --replace replaced --yes
    assert_contains "Migration succeeded"

    assert_command dfx canister status migrated
    assert_command_fail dfx canister status replaced
}

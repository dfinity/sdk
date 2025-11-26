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
    
    # Update dfx.json: rename hello_backend -> source, and add target canister
    jq '.canisters.source = .canisters.hello_backend | del(.canisters.hello_backend)' dfx.json | sponge dfx.json
    jq '.canisters.target = { "main": "counter.mo", "type": "motoko" }' dfx.json | sponge dfx.json

    # Deploy the source to the application subnet.
    dfx deploy source

    # Create the target canister on the fiduciary subnet.
    dfx canister create target --subnet-type fiduciary

    dfx canister stop source
    dfx canister stop target

    # Make sure the source has enough cycles to do the migration.
    dfx ledger fabricate-cycles --canister source --cycles 10000000000000

    # The migration will take a few minutes to complete.
    assert_command dfx canister migrate-id source --replace target --yes
    assert_contains "Migration succeeded"

    assert_command dfx canister status source
    assert_command_fail dfx canister status target
}

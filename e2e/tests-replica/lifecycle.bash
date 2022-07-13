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

@test "test canister lifecycle" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello_backend
    assert_command dfx canister status hello_backend
    assert_match "Status: Running."

    # Stop
    assert_command dfx canister stop hello_backend
    assert_command dfx canister status hello_backend
    assert_match "Status: Stopped."
    assert_command_fail dfx canister call "$(dfx canister id hello_backend)" greet '("Names are difficult")'
    assert_match "is stopped"

    # Start
    assert_command dfx canister start hello_backend
    assert_command dfx canister status hello_backend
    assert_match "Status: Running."

    # Call
    assert_command dfx canister call "$(dfx canister id hello_backend)" greet '("Names are difficult")'
    assert_match '("Hello, Names are difficult!")'

    # Id
    assert_command dfx canister id hello_backend
    assert_match "$(jq -r .hello_backend.local < .dfx/local/canister_ids.json)"
    x="$(dfx canister id hello_backend)"
    local old_id="$x"

    # Delete
    assert_command_fail dfx canister delete hello_backend
    assert_command dfx canister stop hello_backend
    assert_command dfx canister delete hello_backend
    assert_command_fail dfx canister status hello_backend
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_backend'."

    # Create again
    assert_command dfx canister create hello_backend
    assert_command dfx canister id hello_backend
    assert_neq "$old_id"
}

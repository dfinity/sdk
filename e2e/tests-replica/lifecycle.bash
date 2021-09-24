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
    dfx canister install hello
    assert_command dfx canister status hello
    assert_match "Status: Running."

    # Stop
    assert_command dfx canister stop hello
    assert_command dfx canister status hello
    assert_match "Status: Stopped."
    assert_command_fail dfx canister call "$(dfx canister id hello)" greet '("Names are difficult")'
    assert_match "is stopped"

    # Start
    assert_command dfx canister start hello
    assert_command dfx canister status hello
    assert_match "Status: Running."

    # Call
    assert_command dfx canister call "$(dfx canister id hello)" greet '("Names are difficult")'
    assert_match '("Hello, Names are difficult!")'

    # Id
    assert_command dfx canister id hello
    assert_match "$(jq -r .hello.local < .dfx/local/canister_ids.json)"
    x="$(dfx canister id hello)"
    local old_id="$x"

    # Delete
    assert_command_fail dfx canister delete hello
    assert_command dfx canister stop hello
    assert_command dfx canister delete hello
    assert_command_fail dfx canister status hello
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello'."

    # Create again
    assert_command dfx canister create hello
    assert_command dfx canister id hello
    assert_neq "$old_id"
}

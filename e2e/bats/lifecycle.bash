#!/usr/bin/env bats

load utils/_

setup() {
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    dfx_new hello
}

teardown() {
    dfx_stop
}

@test "test canister lifecycle" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello
    assert_command dfx canister status hello
    assert_match "Canister hello's status is Running."

    # Stop
    assert_command dfx canister stop hello
    assert_command dfx canister status hello
    assert_match "Canister hello's status is Stopped."
    assert_command_fail dfx canister call $(dfx canister id hello) greet '("Names are difficult")'
    assert_match "is stopped and cannot accept ingress messages"
    
    # Start
    assert_command dfx canister start hello
    assert_command dfx canister status hello
    assert_match "Canister hello's status is Running."

    # Call
    assert_command dfx canister call $(dfx canister id hello) greet '("Names are difficult")'
    assert_match '("Hello, Names are difficult!")'

    # Id
    assert_command dfx canister id hello
    assert_match $(cat .dfx/local/canister_ids.json | jq -r .hello.local)
    local old_id=$(dfx canister id hello)

    # Delete
    assert_command_fail dfx canister delete hello
    assert_command dfx canister stop hello
    assert_command dfx canister delete hello
    assert_command_fail dfx canister status hello
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello'."

    # Create again
    assert_command dfx canister create hello
    assert_command dfx canister id hello
    assert_neq $old_id
}

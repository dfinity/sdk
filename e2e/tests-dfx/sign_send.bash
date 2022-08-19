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

@test "sign + send" {
    install_asset counter
    dfx_start
    dfx deploy

    assert_command dfx canister sign --query hello_backend read
    assert_eq "Query message generated at [message.json]"

    sleep 10
    echo y | assert_command dfx canister send message.json

    assert_command_fail dfx canister send message.json --status
    assert_eq "Error: Can only check request_status on update calls."

    assert_command_fail dfx canister sign --query hello_backend read
    assert_eq "Error: [message.json] already exists, please specify a different output file name."

    assert_command dfx canister sign --update hello_backend inc --file message-inc.json
    assert_eq "Update message generated at [message-inc.json]
Signed request_status append to update message in [message-inc.json]"

    sleep 10
    echo y | assert_command dfx canister send message-inc.json
    assert_command dfx canister send message-inc.json --status
    assert_match "To see the content of response, copy-paste the encoded string into cbor.me."
}

@test "sign outside of a dfx project" {
    cd "$E2E_TEMP_DIR"
    mkdir not-a-project-dir
    cd not-a-project-dir

    assert_command dfx canister sign --query rwlgt-iiaaa-aaaaa-aaaaa-cai read --network ic
    assert_match "Query message generated at \[message.json\]"
}

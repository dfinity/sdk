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
    dfx deploy --no-wallet

    assert_command dfx canister --no-wallet sign --query hello read
    assert_eq "Query message generated at [message.json]"

    sleep 10
    echo y | assert_command dfx canister --no-wallet send message.json

    assert_command_fail dfx canister --no-wallet send message.json --status
    assert_eq "Can only check request_status on update calls."

    assert_command_fail dfx canister --no-wallet sign --query hello read
    assert_eq "[message.json] already exists, please specify a different output file name."

    assert_command dfx canister --no-wallet sign --update hello inc --file message-inc.json
    assert_eq "Update message generated at [message-inc.json] Signed request_status append to update message in [message-inc.json]"

    sleep 10
    echo y | assert_command dfx canister --no-wallet send message-inc.json
    assert_command dfx canister --no-wallet send message-inc.json --status
    assert_match "To see the content of response, copy-paste the encoded string into cbor.me."
}

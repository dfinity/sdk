#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
    dfx_stop
}

@test "Can use an identity PEM file to send requests" {
    install_asset identity_mo
    dfx build
    dfx start --background
    dfx canister install --all

    assert_command dfx canister -p id_ed25519.pem call e2e_project get_id
    assert_match "0"
}


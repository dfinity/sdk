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

@test "Provides same principal identifier as sender" {
    install_asset pricipal_id_mo
    dfx build
    dfx start --background
    dfx canister install --all

    assert_command dfx canister -p call e2e_project get_principal_id
    assert_match "0"
    # Second time must be the same.
    assert_command dfx canister -p call e2e_project get_principal_id
    assert_match "0"
}

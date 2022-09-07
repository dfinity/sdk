#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "--identity and --network are stil accepted as prefix" {
    install_asset whoami
    dfx_start
    dfx deploy
    dfx identity new alice --disable-encryption
    assert_command dfx --identity alice canister --network local call whoami whoami
    assert_match "$(dfx --identity alice identity get-principal)"
    assert_match "$(dfx identity get-principal --identity alice)"
}

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

@test "build uses default build args" {
    install_asset rust

    rustup default stable
    rustup target add wasm32-unknown-unknown

    dfx_start
    dfx canister --no-wallet create --all
    assert_command dfx build print
    assert_match "Finished"
    assert_command dfx canister --no-wallet install print
}
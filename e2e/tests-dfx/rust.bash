#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "rust starter project can build and call" {
    dfx_new_rust print

    dfx_start
    dfx canister --no-wallet create --all
    assert_command dfx build print
    assert_match "Finished"
    assert_command dfx canister --no-wallet install print
    assert_command dfx canister --no-wallet call print print
}
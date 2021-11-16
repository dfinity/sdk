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
    dfx_new_rust hello

    dfx_start
    dfx canister --no-wallet create --all
    assert_command dfx build hello
    assert_match "`ic-cdk-optimizer` not installed"
    cargo install ic-cdk-optimizer
    assert_command dfx build hello
    assert_match "Executing: ic-cdk-optimizer"
    assert_command dfx canister --no-wallet install hello
    assert_command dfx canister --no-wallet call hello greet dfinity
    assert_match '("Hello, dfinity!")'
}

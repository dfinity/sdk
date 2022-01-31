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
    dfx canister create --all
    assert_command dfx build hello
    assert_match "ic-cdk-optimizer not installed"
    cargo install ic-cdk-optimizer
    # shellcheck disable=SC2030
    export PATH="$HOME/.cargo/bin/:$PATH"
    assert_command dfx build hello
    assert_match "Executing: ic-cdk-optimizer"
    assert_command dfx canister install hello
    assert_command dfx canister call hello greet dfinity
    assert_match '("Hello, dfinity!")'
}

@test "rust canister can resolve dependencies" {
    dfx_new_rust rust_deps
    install_asset rust_deps

    dfx_start
    assert_command dfx deploy
    assert_command dfx canister call multiply_deps read
    assert_match '(1 : nat)'
    assert_command dfx canister call multiply_deps mul '(3)'
    assert_match '(9 : nat)'
    assert_command dfx canister call rust_deps read
    assert_match '(9 : nat)'
}

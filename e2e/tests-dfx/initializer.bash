#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit

    # Each test gets its own home directory in order to have its own identities.
    x=$(pwd)/home-for-test
    mkdir "$x"
    export HOME="$x"

    dfx_new
}

teardown() {
    dfx_stop
    x=$(pwd)/home-for-test
    rm -rf "$x"
}

@test "test access control flow via dfx call" {
    install_asset initializer
    dfx_start
    assert_command dfx identity new alice
    assert_command dfx identity use alice

    dfx canister create --all
    assert_command dfx build
    assert_command dfx canister install --all

    # The wallet is the initializer
    assert_command dfx canister call e2e_project test
    assert_eq '(true)'

    # The user Identity's principal is not the initializer
    assert_command dfx canister --no-wallet call e2e_project test
    assert_eq '(false)'
}

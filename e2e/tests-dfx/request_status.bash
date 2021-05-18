#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit
    export RUST_BACKTRACE=1

    dfx_new hello
}

teardown() {
  dfx_stop
}

@test "request-status output raw" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build

    dfx canister install hello

    assert_command dfx canister call --async hello greet Bob

    # shellcheck disable=SC2154
    assert_command dfx canister request-status --output raw "$stdout" "$(dfx canister id hello)"
    assert_eq '4449444c0001710b48656c6c6f2c20426f6221'

}

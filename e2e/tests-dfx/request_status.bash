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

@test "request-status output raw" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build

    dfx canister install hello_backend

    assert_command dfx canister call --async hello_backend greet Bob

    # shellcheck disable=SC2154
    assert_command dfx canister request-status --output raw "$stdout" "$(dfx canister id hello_backend)"
    assert_eq '4449444c0001710b48656c6c6f2c20426f6221'

}

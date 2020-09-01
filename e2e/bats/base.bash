#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    dfx_new
}

teardown() {
    dfx_stop
}

@test "provides base library location by default" {
    install_asset base

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project

    assert_command dfx canister call --query e2e_project is_digit '("5")'
    assert_eq '(true)'

    assert_command dfx canister call --query e2e_project is_digit '("w")'
    assert_eq '(false)'
}

@test "does not provide base library if there is a packtool" {
    install_asset base
    dfx config defaults/build/packtool "echo"

    dfx_start
    dfx canister create --all
    assert_command_fail dfx build
    assert_match 'import error, package "base" not defined'
}

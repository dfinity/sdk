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

@test "provides base library location by default" {
    install_asset base

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project_backend

    assert_command dfx canister call --query e2e_project_backend is_digit '("5")'
    assert_eq '(true)'

    assert_command dfx canister call --query e2e_project_backend is_digit '("w")'
    assert_eq '(false)'
}

@test "does not provide base library if there is a packtool" {
    install_asset base
    jq '.defaults.build.packtool="echo"' dfx.json | sponge dfx.json

    dfx_start
    dfx canister create --all
    assert_command_fail dfx build
    assert_match 'import error \[M0010\], package "base" not defined'
}

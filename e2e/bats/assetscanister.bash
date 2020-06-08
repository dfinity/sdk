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

@test "can store and retrieve assets by key" {
    install_asset assetscanister

    dfx_start
    dfx build
    dfx canister install e2e_project

    assert_command dfx canister call --update e2e_project store '("AA", "xxx")'
    assert_eq '()'
    assert_command dfx canister call --update e2e_project store '("B", "yyyy")'
    assert_eq '()'

    assert_command dfx canister call --query e2e_project retrieve '("B")'
    assert_eq '("yyyy")'

    assert_command dfx canister call --query e2e_project retrieve '("AA")'
    assert_eq '("xxx")'

    assert_command dfx canister call --query e2e_project retrieve '("B")'
    assert_eq '("yyyy")'

    assert_command_fail dfx canister call --query e2e_project retrieve '("C")'
}


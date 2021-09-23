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

@test "print_mo" {
    [ "$USE_IC_REF" ] && skip "printing from mo not specified"

    install_asset print
    dfx_start 2>stderr.txt
    dfx canister create --all
    dfx build
    dfx canister install e2e_project
    dfx canister call e2e_project hello
    run tail -2 stderr.txt
    assert_match "Hello, World! from DFINITY"
}

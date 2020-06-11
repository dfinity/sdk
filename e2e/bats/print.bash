#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1
    dfx_new
}

teardown() {
    dfx_stop
}

@test "print_mo" {
    skip "Don't run on CI for now"
    [ "$USE_IC_REF" ] && skip "printing from mo not specified"

    install_asset print
    dfx_start 2>stderr.txt
    dfx build
    dfx canister install e2e_project
    dfx canister call e2e_project hello
    run cat stderr.txt
    assert_match "debug.print: Hello, World! from DFINITY"
}

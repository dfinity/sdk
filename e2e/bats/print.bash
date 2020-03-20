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
    if [ "$USE_IC_REF" = "true" ]; then skip "printing from mo not specified"; fi

    install_asset print_mo
    dfx_start 2>stderr.txt
    dfx build
    dfx canister install e2e_project
    dfx canister call e2e_project hello
    run cat stderr.txt
    assert_match "debug.print: Hello, World! from DFINITY"
}

#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1
    dfx_new
}

teardown() {
    # Kill the node manager, the dfx and the client. Ignore errors (ie. if processes aren't
    # running).
    killall dfx nodemanager client |& sed 's/^/killall: /' || true
}

@test "print_mo" {
    install_asset print_mo
    dfx_start 2>stderr.txt
    dfx build
    dfx canister install 1 canisters/print.wasm --wait
    dfx canister call 1 hello --wait
    run cat stderr.txt
    assert_match "debug.print: Hello, World! from DFINITY"
}

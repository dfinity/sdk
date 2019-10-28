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

@test "print_as" {
    install_asset print_as
    dfx_start
    dfx build
    dfx canister install 1 canisters/print.wasm --wait
    assert_eq "FIXME"
}

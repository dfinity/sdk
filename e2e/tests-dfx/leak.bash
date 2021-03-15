#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
    dfx_stop
}

@test "repeated install wasm" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    install_asset custom_canister
    dfx_start
    dfx deploy
    for i in {1..50}
    do
      dfx canister install --all --mode=reinstall
    done
}

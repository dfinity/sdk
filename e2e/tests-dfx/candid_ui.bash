#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit
    export RUST_BACKTRACE=1
    dfx_new hello
}

teardown() {
  dfx_stop
}

@test "build + install + call + request-status -- greet_mo" {
    dfx_start
    dfx deploy
    assert_command dfx canister call hello greet '("Banzai")'
    assert_eq '("Hello, Banzai!")'
    ID=$(dfx canister id __Candid_UI)
    PORT=$(cat .dfx/webserver-port)
    assert_command curl http://localhost:"$PORT"/?canisterId="$ID"
    assert_match "Candid UI"
}

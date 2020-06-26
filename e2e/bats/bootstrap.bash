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

@test "bootstrap fetches candid file" {
    dfx_start
    dfx build
    dfx canister install hello
    ID=$(dfx canister id hello)

    assert_command curl http://localhost:8000/_/candid?canisterId="$ID"
    assert_lines_match '"greet": (text) -> (text);' 1
}

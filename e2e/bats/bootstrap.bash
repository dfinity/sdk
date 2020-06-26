#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    dfx_new hello
}

teardown() {
    dfx_stop
}

@test "bootstrap fetches candid file" {
    dfx_start
    dfx build
    dfx canister install hello
    ID=$(dfx canister id hello)

    assert_command curl http://localhost:8000/_/candid?canisterId="$ID" -o ./web.txt
    assert_command diff canisters/hello/hello.did ./web.txt
    assert_command curl http://localhost:8000/_/candid?canisterId="$ID"\&format=js -o ./web.txt
    assert_command diff canisters/hello/hello.did.js ./web.txt
}

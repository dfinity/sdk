#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    dfx_stop_replica_and_bootstrap

    standard_teardown
}

@test "bootstrap fetches candid file" {

    dfx_start_replica_and_bootstrap

    dfx canister create --all
    dfx build
    dfx canister install hello

    ID=$(dfx canister id hello)
    PORT=$(cat .dfx/webserver-port)
    assert_command curl http://localhost:"$PORT"/_/candid?canisterId="$ID" -o ./web.txt
    assert_command diff .dfx/local/canisters/hello/hello.did ./web.txt
    assert_command curl http://localhost:"$PORT"/_/candid?canisterId="$ID"\&format=js -o ./web.txt
    # Relax diff as it's produced by two different compilers.
    assert_command diff --ignore-all-space --ignore-blank-lines .dfx/local/canisters/hello/hello.did.js ./web.txt

}

@test "forbid starting webserver with a forwarded port" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    assert_command_fail dfx bootstrap --port 8000
    assert_match "Cannot forward API calls to the same bootstrap server"
}

@test "bootstrap supports http requests" {
    dfx_start_replica_and_bootstrap

    dfx canister create --all
    dfx build
    dfx canister install hello_assets

    ID=$(dfx canister id hello_assets)
    PORT=$(cat .dfx/webserver-port)
    assert_command curl http://localhost:"$PORT"/sample-asset.txt?canisterId="$ID"
    # shellcheck disable=SC2154
    assert_eq "This is a sample asset!" "$stdout"
}

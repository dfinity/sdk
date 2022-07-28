#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    stop_dfx_replica
    stop_dfx_bootstrap

    standard_teardown
}

@test "bootstrap fetches candid file" {

    dfx_replica
    dfx_bootstrap

    dfx canister create --all
    dfx build
    dfx canister install hello_backend

    ID=$(dfx canister id hello_backend)
    PORT=$(get_webserver_port)
    assert_command curl http://localhost:"$PORT"/_/candid?canisterId="$ID" -o ./web.txt --max-time 60
    assert_command diff .dfx/local/canisters/hello_backend/hello_backend.did ./web.txt
    assert_command curl http://localhost:"$PORT"/_/candid?canisterId="$ID"\&format=js -o ./web.txt --max-time 60
    # Relax diff as it's produced by two different compilers.
    assert_command diff --ignore-all-space --ignore-blank-lines .dfx/local/canisters/hello_backend/hello_backend.did.js ./web.txt

}

@test "forbid starting webserver with a forwarded port" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    assert_command_fail dfx bootstrap --port 8000
    assert_match "Cannot forward API calls to the same bootstrap server"
}

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

@test "forbid starting webserver with a forwarded port" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    assert_command_fail dfx bootstrap --port 4943
    assert_match "Cannot forward API calls to the same bootstrap server"
}

@test "bootstrap supports http requests" {
    dfx_replica
    dfx_bootstrap

    dfx canister create --all
    dfx build
    dfx canister install hello_frontend

    ID=$(dfx canister id hello_frontend)
    PORT=$(get_webserver_port)
    assert_command curl http://localhost:"$PORT"/sample-asset.txt?canisterId="$ID" --max-time 60
    # shellcheck disable=SC2154
    assert_eq "This is a sample asset!" "$stdout"
}

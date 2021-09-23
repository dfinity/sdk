#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new_frontend
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "dfx start serves a frontend with static assets" {
    skip "Need a build of @dfinity/agent that works with HTTP Query"
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    ID=$(dfx canister id e2e_project_assets)
    PORT=$(cat .dfx/webserver-port)
    assert_command curl http://localhost:"$PORT"/?canisterId="$ID"
    assert_match "logo.png"
}

@test "dfx start serves a frontend on a port" {
    [ "$USE_IC_REF" ] && skip "dfx start cannot serve frontend when using ic-ref"
    skip "Need a build of @dfinity/agent that works with HTTP Query"

    dfx_start --host 127.0.0.1:12345

    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.local.bind="127.0.0.1:12345"' dfx.json)" >dfx.json

    dfx canister create --all
    dfx build
    dfx canister install --all

    ID=$(dfx canister id e2e_project_assets)
    assert_command curl http://localhost:12345/?canisterId="$ID"
    assert_match "<html>"

    assert_command_fail curl http://localhost:8000
    assert_match "Connection refused"
}

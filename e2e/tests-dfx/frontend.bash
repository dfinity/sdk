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

@test "dfx deploy shows a url for the frontend and for the candid interface" {
    dfx_start
    PORT=$(get_webserver_port)

    assert_command dfx deploy
    CANDID_UI_ID=$(dfx canister id __Candid_UI)
    APP_ID=$(dfx canister id e2e_project_backend)
    ASSETS_ID=$(dfx canister id e2e_project_frontend)
    assert_match "e2e_project_backend: http://127.0.0.1:$PORT/\?canisterId=$CANDID_UI_ID&id=$APP_ID"
    assert_match "e2e_project_frontend: http://127.0.0.1:$PORT/\?canisterId=$ASSETS_ID"

    # the urls are a little nicer if the bind address is localhost:8000 rather than 127.0.0.1:8000
    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.local.bind="localhost:'"$PORT"'"' dfx.json)" >dfx.json

    assert_command dfx deploy
    assert_match "e2e_project_backend: http://$CANDID_UI_ID.localhost:$PORT/\?id=$APP_ID"
    assert_match "e2e_project_frontend: http://$ASSETS_ID.localhost:$PORT/"
}

@test "dfx start serves a frontend with static assets" {
    skip "Need a build of @dfinity/agent that works with HTTP Query"
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    ID=$(dfx canister id e2e_project_frontend)
    PORT=$(get_webserver_port)
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

    ID=$(dfx canister id e2e_project_frontend)
    assert_command curl http://localhost:12345/?canisterId="$ID"
    assert_match "<html>"

    assert_command_fail curl http://localhost:8000
    assert_match "Connection refused"
}

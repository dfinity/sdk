#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    dfx_new_frontend
}

teardown() {
    dfx_stop
}

@test "dfx start serves a frontend" {
    dfx_start
    dfx canister create --all
    dfx build --skip-frontend

    sleep 1
    PORT=$(cat .dfx/webserver-port)
    assert_command curl http://localhost:"$PORT"
    assert_match "<html>"
}

@test "dfx start serves a frontend on a port" {
    [ "$USE_IC_REF" ] && skip "dfx start cannot serve frontend when using ic-ref"

    dfx_start --host 127.0.0.1:12345

    cat <<<$(jq '.networks.local.bind="127.0.0.1:12345"' dfx.json) >dfx.json

    dfx canister create --all
    dfx build --skip-frontend

    assert_command curl http://localhost:12345
    assert_match "<html>"

    assert_command_fail curl http://localhost:8000
    assert_match "Connection refused"
}

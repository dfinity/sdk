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
    dfx build --skip-frontend
    dfx_start

    sleep 1
    assert_command curl http://localhost:8000 # 8000 = default port.
    assert_match "<html>"
}

@test "dfx start serves a frontend on a port" {
    if [ "$USE_IC_REF" = "true" ]; then skip "dfx start cannot serve frontent when using ic-ref"; fi

    dfx build --skip-frontend
    dfx_start --host 127.0.0.1:12345

    assert_command curl http://localhost:12345 # 8000 = default port.
    assert_match "<html>"

    assert_command_fail curl http://localhost:8000
    assert_match "Connection refused"
}

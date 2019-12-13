#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    dfx_new_frontend
}

teardown() {
    dfx stop

    # Verify that processes are killed.
    ! ( ps | grep \ dfx\ start )
}

@test "dfx start serves a frontend" {
    dfx build
    dfx_start

    assert_command curl http://localhost:8000 # 8000 = default port.
    assert_match "<html>"
}

@test "dfx start serves a frontend on a port" {
    dfx build
    dfx_start --host 127.0.0.1:12345

    assert_command curl http://localhost:12345 # 8000 = default port.
    assert_match "<html>"

    assert_command_fail curl http://localhost:8000
    assert_match "Connection refused"
}

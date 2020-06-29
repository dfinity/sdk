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

@test "dfx ping fails if replica not running" {
    assert_command_fail dfx ping
}

@test "dfx start succeeds" {
    dfx_start
}

@test "dfx ping succeeds if replica is running" {
    dfx_start
    assert_command dfx ping

    assert_match "{ \"ic_api_version\": .* }"
}

@test "dfx ping succeeds by specific host:post" {
    dfx_start
    assert_command dfx ping http://127.0.0.1:8000

    assert_match "{ \"ic_api_version\": .* }"
}

@test "dfx ping succeeds by network name" {
    dfx_start
    assert_command dfx ping local

    assert_match "{ \"ic_api_version\": .* }"
}

@test "dfx ping succeeds by network name if network bind address is host:port format" {
    dfx_start
    assert_command dfx config networks.local.bind '"127.0.0.1:8000"'
    assert_command dfx ping local

    assert_match "{ \"ic_api_version\": .* }"
}

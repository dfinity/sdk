#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "dfx ping fails if replica not running" {
    assert_command_fail dfx ping
}


@test "dfx ping succeeds if replica is running" {
    dfx_start
    assert_command dfx ping

    assert_match "\"ic_api_version\""
}

@test "dfx ping succeeds by specific host:post" {
    dfx_start
    webserver_port=$(cat .dfx/webserver-port)
    assert_command dfx ping http://127.0.0.1:"$webserver_port"

    assert_match "\"ic_api_version\""
}

@test "dfx ping succeeds by network name" {
    dfx_start
    assert_command dfx ping local

    assert_match "\"ic_api_version\""
}

@test "dfx ping succeeds by network name if network bind address is host:port format" {
    dfx_start
    webserver_port=$(cat .dfx/webserver-port)
    assert_command dfx config networks.local.bind '"127.0.0.1:'"$webserver_port"'"'
    assert_command dfx ping local

    assert_match "\"ic_api_version\""
}

@test "dfx ping succeeds by arbitrary network name to a nonstandard port" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    dfx_start --host 127.0.0.1:12345
    # dfx_start overwrites local bind with provided port arg, set it back to default
    assert_command dfx config networks.local.bind '"127.0.0.1:8000"'
    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.arbitrary.providers=["http://127.0.0.1:12345"]' dfx.json)" >dfx.json

    assert_command dfx ping arbitrary
    assert_match "\"ic_api_version\""

    assert_command_fail dfx ping
    # this port won't match the ephemeral port that the ic ref picked
    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.arbitrary.providers=["127.0.0.1:22113"]' dfx.json)" >dfx.json
    assert_command_fail dfx ping arbitrary
}

@test "dfx ping can have a URL for network to ping" {
    dfx_start
    webserver_port=$(cat .dfx/webserver-port)
    assert_command dfx ping "http://127.0.0.1:$webserver_port"
    assert_match "\"ic_api_version\""
}

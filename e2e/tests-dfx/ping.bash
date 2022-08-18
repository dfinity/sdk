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
    webserver_port=$(get_webserver_port)
    assert_command dfx ping http://127.0.0.1:"$webserver_port"

    assert_match "\"ic_api_version\""
}

@test "dfx ping does not require dfx.json" {
    dfx_start
    webserver_port=$(get_webserver_port)

    mkdir "$E2E_TEMP_DIR/not-a-project"
    (
        cd "$E2E_TEMP_DIR/not-a-project"

        assert_command dfx ping http://127.0.0.1:"$webserver_port"
        assert_match "\"ic_api_version\""
    )
}

@test "dfx ping succeeds by network name" {
    dfx_start
    assert_command dfx ping local

    assert_match "\"ic_api_version\""
}

@test "dfx ping succeeds by network name if network bind address is host:port format" {
    dfx_start
    webserver_port=$(get_webserver_port)
    jq '.networks.nnn.bind="127.0.0.1:'"$webserver_port"'"' dfx.json | sponge dfx.json
    assert_command dfx ping nnn

    assert_match "\"ic_api_version\""
}

@test "dfx ping succeeds by arbitrary network name to a nonstandard port" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    dfx_start --host 127.0.0.1:12345

    # Make dfx use the port from configuration:
    rm "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/webserver-port"

    jq '.networks.arbitrary.providers=["http://127.0.0.1:12345"]' dfx.json | sponge dfx.json

    assert_command dfx ping arbitrary
    assert_match "\"ic_api_version\""

    assert_command_fail dfx ping
    # this port won't match the ephemeral port that the replica picked
    jq '.networks.arbitrary.providers=["127.0.0.1:22113"]' dfx.json | sponge dfx.json
    assert_command_fail dfx ping arbitrary
}

@test "dfx ping can have a URL for network to ping" {
    dfx_start
    webserver_port=$(get_webserver_port)
    assert_command dfx ping "http://127.0.0.1:$webserver_port"
    assert_match "\"ic_api_version\""
}

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

    assert_match "\"ic_api_version\""
}

@test "dfx ping succeeds by specific host:post" {
    dfx_start
    assert_command dfx ping http://127.0.0.1:8000

    assert_match "\"ic_api_version\""
}

@test "dfx ping succeeds by network name" {
    dfx_start
    assert_command dfx ping local

    assert_match "\"ic_api_version\""
}

@test "dfx ping succeeds by network name if network bind address is host:port format" {
    dfx_start
    assert_command dfx config networks.local.bind '"127.0.0.1:8000"'
    assert_command dfx ping local

    assert_match "\"ic_api_version\""
}

@test "dfx ping succeeds by arbitrary network name to a nonstandard port" {
    if [ "$USE_IC_REF" ]; then
        dfx_start
        local_bind=$(jq -r .networks.local.bind dfx.json)
        cat <<<$(jq .networks.arbitrary.providers=[\"http://${local_bind}\"] dfx.json) >dfx.json
        # without this, `dfx ping` below will succeed:
        cat <<<$(jq .networks.local.bind=\"127.0.0.1:22113\" dfx.json) >dfx.json
    else
        dfx_start --host 127.0.0.1:12345
        cat <<<$(jq .networks.arbitrary.providers=[\"http://127.0.0.1:12345\"] dfx.json) >dfx.json
    fi

    assert_command dfx ping arbitrary
    assert_match "\"ic_api_version\""

    if [ "$USE_IC_REF" ]; then
        # calling out that this works too (and connects to the bootstrap).
        # This is the reason for repointing the local.bind address above.
        assert_command dfx ping http://127.0.0.1:8000
    fi

    assert_command_fail dfx ping
    # this port won't match the ephemeral port that the ic ref picked
    cat <<<$(jq .networks.arbitrary.providers=[\"127.0.0.1:22113\"] dfx.json) >dfx.json
    assert_command_fail dfx ping arbitrary
}

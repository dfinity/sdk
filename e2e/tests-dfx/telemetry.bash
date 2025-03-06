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

# Be sure to only set telemetry to 'local', not 'enabled'.

@test "telemetry is enabled by default" {
    cfg=$(dfx info config-json-path)
    assert_command jq -r .telemetry "$cfg"
    assert_eq on
}

@test "telemetry can be disabled" {
    dfx config telemetry off
    assert_command env DFX_TELEMETRY= dfx config telemetry
    assert_eq off
    dfx config telemetry local
    assert_command env DFX_TELEMETRY= dfx config telemetry
    assert_eq local
    cfg=$(dfx info config-json-path)
    jq '.telemetry = "off"' "$cfg" | sponge "$cfg"
    assert_command env DFX_TELEMETRY= dfx config telemetry
    assert_eq off
}

@test "environment variables override the config" {
    # dfx var overrides both ways
    dfx config telemetry local
    assert_command env DFX_TELEMETRY=off dfx config telemetry
    assert_eq off
    dfx config telemetry off
    assert_command env DFX_TELEMETRY=local dfx config telemetry
    assert_eq local
    # generic var only overrides to false
    dfx config telemetry local
    assert_command env NO_TELEMETRY=1 dfx config telemetry
    assert_eq off
    dfx config telemetry off
    assert_command env NO_TELEMETRY=0 dfx config telemetry
    assert_eq off
}

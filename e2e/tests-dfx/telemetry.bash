#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx_new
  unset DFX_TELEMETRY_ENABLED
  export __DFX_CI_DISABLE_TELEMETRY_BUT_MISREPORT_SETTING=1
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "telemetry is enabled by default" {
    cfg=$(dfx info config-json-path)
    assert_command jq -r .telemetry "$cfg"
    assert_eq true
    assert_command dfx info is-telemetry-enabled
    assert_eq true
}

@test "telemetry can be disabled" {
    dfx config --telemetry disabled
    assert_command dfx info is-telemetry-enabled
    assert_eq false
}

@test "environment variables override the config" {
    # dfx var overrides both ways
    dfx config --telemetry enabled
    assert_command env DFX_TELEMETRY_ENABLED=false dfx info is-telemetry-enabled
    assert_eq false
    dfx config --telemetry disabled
    assert_command env DFX_TELEMETRY_ENABLED=true dfx info is-telemetry-enabled
    assert_eq true
    # generic var only overrides to false
    dfx config --telemetry enabled
    assert_command env NO_TELEMETRY=1 dfx info is-telemetry-enabled
    assert_eq false
    dfx config --telemetry disabled
    assert_command env NO_TELEMETRY=0 dfx info is-telemetry-enabled
    assert_eq false
}

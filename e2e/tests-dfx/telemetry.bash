#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx_new
  
  export DFX_TELEMETRY=local
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
}

@test "telemetry is collected" {
    local expected_platform log n
    case "$(uname)" in
    Darwin) expected_platform=macos;;
    Linux) expected_platform=linux;;
    *) fail 'unknown platform';;
    esac
    log=$(dfx info telemetry-log-path)
    assert_command dfx identity get-principal
    assert_command jq -se 'last | .command == "identity get-principal" and .platform == "'"$expected_platform"'"
        and .exit_code == 0 and (.parameters | length == 0)' "$log"
    n=$(jq -sr length "$log")
    assert_command_fail env DFX_NETWORK=ic dfx identity get-platypus
    assert_command jq -se "length == $n" "$log"
    assert_command_fail env DFX_NETWORK=ic dfx identity get-principal --identity platypus
    assert_command jq -se 'length == '$((n+1))' and (last | .command == "identity get-principal" and .exit_code == 255 and
        (.parameters | any(.name == "network" and .source == "environment")
            and any(.name == "identity" and .source == "command-line")))' "$log"
}

@test "telemetry reprocesses extension commands" {
    local log
    log=$(dfx info telemetry-log-path)
    assert_command dfx extension install nns --version 0.3.1
    assert_command dfx nns import
    assert_command jq -se 'last | .command == "extension run" and (.parameters | any(.name == "name"))' "$log"
}

@test "concurrent commands do not corrupt the log file" {
    local log
    log=$(dfx info telemetry-log-path)
    dfx identity get-principal # initialize it first
    for _ in {0..100}; do
        assert_command dfx identity get-principal &
    done
    wait
    assert_command jq -se '.[-101:-1] | all(.command == "identity get-principal") and length == 100' "$log"
}

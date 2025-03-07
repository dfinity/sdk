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

@test "telemetry is collected" {
    local expected_platform log
    case "$(uname)" in
    Darwin) expected_platform=macos;;
    Linux) expected_platform=linux;;
    *) fail 'unknown platform';;
    esac
    log=$(dfx info telemetry-log-path)
    assert_command env DFX_TELEMETRY=local dfx identity get-principal
    assert_command jq -se '.[0] | .command == "identity get-principal" and .platform == "'"$expected_platform"'"
        and .exit_code == 0 and (.parameters | length == 0)' "$log"
    assert_command_fail env DFX_TELEMETRY=local DFX_NETWORK=ic dfx identity get-platypus
    assert_command jq -se 'length == 0' "$log"
    assert_command_fail env DFX_TELEMETRY=local DFX_NETWORK=ic dfx identity get-principal --identity platypus
    assert_command jq -se 'length == 1 and .[1] | .command == "identity get-principal" and .exit_code == 255 and
        (.parameters | any(.name == "network" and .source == "environment")
            and any(.name == "identity" and source == "command_line"))' "$log"
}

@test "telemetry reprocesses extension commands" {
    local log
    log=$(dfx info telemetry-log-path)
    assert_command dfx extension install nns --version 0.3.1
    assert_command env DFX_TELEMETRY=local dfx nns import
    assert_command jq -se 'last | .command == "extension run" and (.parameters | any(.name == "name"))' "$log"
}

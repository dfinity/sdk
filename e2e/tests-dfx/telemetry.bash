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

@test "command-line args are collected" {
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

@test "the last replica error is collected" {
    dfx_new_assets
    local log wallet
    log=$(dfx info telemetry-log-path)
    # explicit call, known canister
    dfx_start
    assert_command_fail dfx canister call ryjl3-tyaaa-aaaaa-aaaba-cai name
    assert_command jq -se 'last | .replica_error_call_site == "name" and .replica_error_code == "IC0301"' "$log"
    # implicit call, wallet canister
    wallet=$(dfx identity get-wallet)
    dfx canister stop "$wallet"
    dfx canister delete "$wallet" --no-withdrawal -y
    assert_command_fail dfx canister create e2e_project_backend
    assert_command jq -se 'last | .replica_error_call_site == "wallet_api_version" and .replica_error_code == "IC0301"' "$log"
    # call to unknown canister
    dfx canister create --all --no-wallet
    assert_command_fail dfx canister call e2e_project_backend greet
    assert_command jq -se 'last | .replica_error_call_site == "<user-specified canister method>" and .replica_error_code == "IC0537"' "$log"
    # call to assets canister
    install_asset wasm
    dfx build e2e_project_frontend
    dfx canister install "$(dfx canister id e2e_project_frontend)" --wasm identity/main.wasm
    assert_command_fail dfx canister install e2e_project_frontend --mode upgrade --no-asset-upgrade -y
    assert_command jq -se 'last | .replica_error_call_site == "list" and .replica_error_code == "IC0536"' "$log"
}

@test "network information is collected" {
    local log
    log=$(dfx info telemetry-log-path)
    dfx_start
    dfx identity get-wallet
    assert_command jq -se 'last.network_type == "local-shared"' "$log"
    assert_command_fail dfx identity get-wallet --ic
    assert_command jq -se 'last.network_type == "ic"' "$log"
    setup_actuallylocal_project_network
    dfx identity get-wallet --network actuallylocal
    assert_command jq -se 'last.network_type == "unknown-configured"' "$log"
    setup_ephemeral_project_network
    dfx identity get-wallet --network ephemeral
    assert_command jq -se 'last.network_type == "project-local"' "$log"
    assert_command_fail dfx identity get-wallet --playground
    assert_command jq -se 'last.network_type == "playground"' "$log"
    assert_command_fail dfx identity get-wallet --network "https://example.com"
    assert_command jq -se 'last.network_type == "unknown-url"' "$log"
}

@test "project structure is collected" {
    local log
    log=$(dfx info telemetry-log-path)
    dfx_start
    dfx deploy
    assert_command jq -se 'last.project_canisters == [{type: "motoko"}]' "$log" 
    dfx_new_frontend
    dfx deploy
    assert_command jq -se 'last.project_canisters | sort_by(.type) == [{type: "assets"}, {type: "motoko"}]' "$log"
    install_asset deps/app
    dfx deploy || true
    assert_command jq -se 'last.project_canisters | sort_by(.type) == [{type: "motoko"}, {type: "pull"}, {type: "pull"}]' "$log"
}

@test "sender information is collected" {
    local log
    log=$(dfx info telemetry-log-path)
    dfx_start
    dfx canister create --all
    assert_command jq -se 'last | .cycles_host == "cycles-wallet" and .identity_type == "plaintext"' "$log"
    dfx canister delete --all -y
    (
        export DFX_CI_MOCK_KEYRING_LOCATION="$MOCK_KEYRING_LOCATION";
        dfx identity new alice
        dfx canister create --all --no-wallet --identity alice
    )
    assert_command jq -se 'last | .cycles_host == null and .identity_type == "keyring"' "$log"
    dfx cycles balance --ic --identity anonymous
    assert_command jq -se 'last | .cycles_host == "cycles-ledger" and .identity_type == "anonymous"' "$log"
}

#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
}

teardown() {
  dfx_stop
  standard_teardown
}

set_project_default_canister_http_enabled() {
  jq ".defaults.canister_http.enabled=${1:-true}" dfx.json | sponge dfx.json
}

set_project_local_network_canister_http_enabled() {
  jq ".networks.local.canister_http.enabled=${1:-true}" dfx.json | sponge dfx.json
}

set_shared_local_network_canister_http_enabled() {
  create_networks_json

  jq ".local.canister_http.enabled=${1:-true}" "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
}

set_shared_local_network_canister_http_empty() {
  create_networks_json

  jq '.local.canister_http={}' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
}

@test "canister http feature is enabled by default" {
  dfx_start

  assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-https-outcalls-adapter-pid"
}

@test "canister http feature is enabled by default with empty json element" {
  set_shared_local_network_canister_http_empty

  dfx_start

  assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-https-outcalls-adapter-pid"
}

@test "dfx restarts replica when ic-https-outcalls-adapter restarts" {
  dfx_new_assets hello
  dfx_start

  install_asset greet
  assert_command dfx deploy
  assert_command dfx canister call hello_backend greet '("Alpha")'
  assert_eq '("Hello, Alpha!")'

  REPLICA_PID=$(get_replica_pid)
  CANISTER_HTTP_ADAPTER_PID=$(get_canister_http_adapter_pid)

  echo "replica pid is $REPLICA_PID"
  echo "ic-https-outcalls-adapter pid is $CANISTER_HTTP_ADAPTER_PID"

  kill -KILL "$CANISTER_HTTP_ADAPTER_PID"
  assert_process_exits "$CANISTER_HTTP_ADAPTER_PID" 15s
  assert_process_exits "$REPLICA_PID" 15s

  timeout 30s sh -c \
    'until dfx ping; do echo waiting for replica to restart; sleep 1; done' \
    || (echo "replica did not restart" && ps aux && exit 1)
  wait_until_replica_healthy

  # Sometimes initially get an error like:
  #     IC0537: Attempt to execute a message on canister <>> which contains no Wasm module
  # but the condition clears.
  timeout 30s sh -c \
    "until dfx canister call hello_backend greet '(\"wait 1\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
    || (echo "canister call did not succeed") # but continue, for better error reporting

  # Even so, after that passes, sometimes this happens:
  #     IC0208: Certified state is not available yet. Please try again...
  sleep 10
  timeout 30s sh -c \
    "until dfx canister call hello_backend greet '(\"wait 2\")'; do echo waiting for any canister call to succeed; sleep 1; done" \
    || (echo "canister call did not succeed") # but continue, for better error reporting

  assert_command dfx canister call hello_backend greet '("Omega")'
  assert_eq '("Hello, Omega!")'

  ID=$(dfx canister id hello_frontend)

  timeout 15s sh -c \
    "until curl --fail http://localhost:\$(cat \"$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/webserver-port\")/sample-asset.txt?canisterId=$ID; do echo waiting for pocket-ic proxy to restart; sleep 1; done" \
    || (echo "pocket-ic proxy did not restart" && ps aux && exit 1)

  assert_command curl --fail http://localhost:"$(get_webserver_port)"/sample-asset.txt?canisterId="$ID"
}

@test "dfx start --enable-canister-http with no other configuration succeeds" {
  dfx_new hello

  dfx_start --enable-canister-http

  assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-https-outcalls-adapter-pid"
}

@test "can enable http through project default configuration - dfx start" {
  dfx_new hello
  define_project_network
  set_project_default_canister_http_enabled

  dfx_start

  assert_file_not_empty .dfx/network/local/ic-https-outcalls-adapter-pid
}

@test "can disable http through project default configuration - dfx start" {
  dfx_new hello
  define_project_network
  set_project_default_canister_http_enabled false

  dfx_start

  assert_file_empty .dfx/network/local/ic-https-outcalls-adapter-pid
}

@test "can enable http through project local network - dfx start" {
  dfx_new hello
  define_project_network
  set_project_local_network_canister_http_enabled

  dfx_start

  assert_file_not_empty .dfx/network/local/ic-https-outcalls-adapter-pid
}

@test "can disable http through project local network - dfx start" {
  dfx_new hello
  define_project_network
  set_project_local_network_canister_http_enabled false

  dfx_start

  assert_file_empty .dfx/network/local/ic-https-outcalls-adapter-pid
}

@test "can enable http through shared local network - dfx start" {
  dfx_new hello
  set_shared_local_network_canister_http_enabled

  dfx_start

  assert_file_not_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-https-outcalls-adapter-pid"
}

@test "can disable http through shared local network - dfx start" {
  dfx_new hello
  set_shared_local_network_canister_http_enabled false

  dfx_start

  assert_file_empty "$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY/ic-https-outcalls-adapter-pid"
}

@test "dfx starts http adapter with correct log level - project defaults" {
  dfx_new
  jq '.defaults.canister_http.log_level="warning"' dfx.json | sponge dfx.json
  define_project_network

  assert_command dfx start --background --verbose
  assert_match "log level: Warning"
  assert_command dfx stop

  jq '.defaults.canister_http.log_level="critical"' dfx.json | sponge dfx.json
  assert_command dfx start --background --verbose
  assert_match "log level: Critical"
}

@test "dfx starts http adapter with correct log level - local network" {
  dfx_new
  jq '.networks.local.canister_http.log_level="warning"' dfx.json | sponge dfx.json
  define_project_network

  assert_command dfx start --background --verbose
  assert_match "log level: Warning"
  assert_command dfx stop

  jq '.networks.local.canister_http.log_level="critical"' dfx.json | sponge dfx.json
  assert_command dfx start --background --verbose --clean
  assert_match "log level: Critical"
}

@test "dfx starts http adapter with correct log level - shared network" {
  dfx_new
  create_networks_json
  jq '.local.canister_http.log_level="warning"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  assert_command dfx start --background --verbose
  assert_match "log level: Warning"
  assert_command dfx stop

  jq '.local.canister_http.log_level="critical"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  assert_command dfx start --background --verbose
  assert_match "log level: Critical"
}

@test "can query a website" {
  dfx_start

  dfx_new
  install_asset canister_http

  dfx deploy

  assert_command dfx canister call e2e_project_backend get_url '("www.githubstatus.com:443","https://www.githubstatus.com:443")'
  assert_contains "Git Operations"
  assert_contains "API Requests"
}

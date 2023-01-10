#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    dfx_stop

    standard_teardown
}

setup_playground() {
  mv dfx.json dfx.json.previous
  install_asset playground_backend
  dfx_start
  SKIP_WASM=true dfx deploy
  dfx ledger fabricate-cycles --t 9999999 --canister backend
  export PLAYGROUND_CANISTER_ID=$(dfx canister id backend)
  echo "PLAYGROUND_CANISTER_ID is $PLAYGROUND_CANISTER_ID"
  create_networks_json
  jq '.local.replica.subnet_type=system' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON" # required until DTS enabled on dfx's replica. Should happen no later than 0.12.2
  WEBSERVER_PORT=$(get_webserver_port)
  jq '.playground.bind="127.0.0.1:'$WEBSERVER_PORT'"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  jq '.playground.playground.playground_cid="'$PLAYGROUND_CANISTER_ID'"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  rm dfx.json
  rm .dfx/local/canister_ids.json
  mv dfx.json.previous dfx.json
}

@test "--playground aliases to --network playground" {
  dfx_start
  create_networks_json
  webserver_port=$(get_webserver_port)
  jq '.playground.bind="127.0.0.1:'$webserver_port'"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  jq '.playground.playground_cid="aaaaa-aa"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  # --playground aliases to --network playground, including all its settings
  assert_command dfx deploy hello_backend --playground
  assert_command dfx canister id hello_backend --playground
}

@test "canister lifecycle" {
  setup_playground
  echo "trying to call canister $PLAYGROUND_CANISTER_ID stats"
  dfx canister call "$PLAYGROUND_CANISTER_ID" getStats '()' --query
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "Reserved canister 'hello_backend'"
  echo "trying to call canister $PLAYGROUND_CANISTER_ID stats"
  dfx canister call "$PLAYGROUND_CANISTER_ID" getStats '()' --query
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "hello_backend canister was already created" "$stderr"
  sleep 10
  jq '.playground.playground.timeout="5"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "Canister 'hello_backend' has timed out."
  assert_match "Reserved canister 'hello_backend'"

  # TODO: install wasm, test calling it. Blocked by .env file change
}
#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
    setup_playground
    dfx_new hello
}

teardown() {
    dfx_stop

    standard_teardown
}

setup_playground() {
  dfx identity new --storage-mode plaintext playground-setup
  dfx identity use playground-setup
  dfx_new hello
  create_networks_json
  mv dfx.json dfx.json.previous
  install_asset playground_backend
  # TODO: remove once discussion with Yan is resolved
  jq '.local.replica.subnet_type="system"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  dfx_start
  dfx deploy backend
  dfx ledger fabricate-cycles --t 9999999 --canister backend
  PLAYGROUND_CANISTER_ID=$(dfx canister id backend)
  export PLAYGROUND_CANISTER_ID
  echo "PLAYGROUND_CANISTER_ID is $PLAYGROUND_CANISTER_ID"
  WEBSERVER_PORT=$(get_webserver_port)
  jq '.playground.bind="127.0.0.1:'"$WEBSERVER_PORT"'"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  jq '.playground.playground.playground_cid="'"$PLAYGROUND_CANISTER_ID"'"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  cd ..
  rm -rf hello
  dfx identity use default
}

@test "--playground aliases to --network playground" {
  assert_command dfx canister create hello_backend --playground -vv
  NETWORK_PLAYGROUND_ID=$(dfx canister id hello_backend --network playground)
  assert_command dfx canister id hello_backend --playground
  assert_match "${NETWORK_PLAYGROUND_ID}"
}

@test "canister lifecycle" {
  assert_command dfx deploy --playground
  assert_command dfx canister --playground call hello_backend greet '("player")'
  assert_match "Hello, player!"

  CANISTER=$(dfx canister --playground id hello_backend)
  assert_command_fail dfx canister --playground stop hello_backend
  assert_match "Canisters borrowed from a playground cannot be stopped."
  assert_command_fail dfx canister stop "${CANISTER}"
  assert_match "403 Forbidden"

  sed -i '' 's/Hello/Goodbye/g' src/hello_backend/main.mo
  assert_command dfx deploy --playground
  assert_command dfx canister --playground call hello_backend greet '("player")'
  assert_match "Goodbye, player!"

  assert_command dfx canister --playground delete hello_backend
  assert_command_fail dfx canister --playground info hello_backend
  # canister is not actually deleted - the playground would have to do that
  assert_command dfx canister --playground info "$CANISTER"
}

@test "timeout" {
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "Reserved canister 'hello_backend'"
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "hello_backend canister was already created"
  sleep 10
  jq '.playground.playground.timeout="5"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "Canister 'hello_backend' has timed out."
  assert_match "Reserved canister 'hello_backend'"
}

@test "Can deploy to mainnet playground" {
  rm "$E2E_NETWORKS_JSON"
  assert_command dfx deploy --playground
  assert_command dfx canister --playground call hello_backend greet '("player")'
  assert_match "Hello, player!"
}

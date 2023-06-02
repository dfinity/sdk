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
  dfx_new hello
  create_networks_json
  jq '.local.replica.subnet_type="system"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON" # use system subnet for local tests because current dfx (0.13.1) has an older replica that doesn't have DTS enabled yet - tested manually against mainnet
  mv dfx.json dfx.json.previous
  install_asset playground_backend
  dfx_start
  echo "STARTED"
  dfx deploy backend
  dfx ledger fabricate-cycles --t 9999999 --canister backend
  export PLAYGROUND_CANISTER_ID=$(dfx canister id backend)
  echo "PLAYGROUND_CANISTER_ID is $PLAYGROUND_CANISTER_ID"
  WEBSERVER_PORT=$(get_webserver_port)
  jq '.playground.bind="127.0.0.1:'$WEBSERVER_PORT'"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  jq '.playground.playground.playground_cid="'$PLAYGROUND_CANISTER_ID'"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  cd ..
  rm -rf hello
}

@test "--playground aliases to --network playground" {
  assert_command dfx canister create hello_backend --playground -vv
  assert_command dfx canister id hello_backend --playground
  CANISTER_ID=$(dfx canister id hello_backend --network playground)
  assert_match "${CANISTER_ID}"
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

  find .
  sed -i '' 's/Hello/Goodbye/g' src/hello_backend/main.mo
  assert_command dfx deploy --playground
  assert_command dfx canister --playground call hello_backend greet '("player")'
  assert_match "Goodbye, player!"

  assert_command dfx canister --playground delete hello_backend
  assert_command_fail dfx canister --playground info hello_backend
  assert_command dfx canister --playground info "$CANISTER"
}

@test "timeout" {
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "Reserved canister 'hello_backend'"
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "hello_backend canister was already created" "$stderr"
  sleep 10
  jq '.playground.playground.timeout="5"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "Canister 'hello_backend' has timed out."
  assert_match "Reserved canister 'hello_backend'"
}
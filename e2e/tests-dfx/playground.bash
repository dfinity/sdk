#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
  setup_playground
  dfx_new_assets hello
}

teardown() {
  dfx_stop
  standard_teardown
}

setup_playground() {
  if ! command -v ic-mops &> /dev/null
  then
    npm i -g ic-mops
  fi
  dfx_new hello
  create_networks_json
  install_asset playground_backend
  touch "$HOME/.bashrc" # required by following mops command
  mops toolchain init   # install the pinned moc version defined in mops.toml
  export DFX_MOC_PATH=moc-wrapper # use the moc-wrapper installed by mops
  dfx_start
  dfx deploy backend
  dfx ledger fabricate-cycles --t 9999999 --canister backend
  PLAYGROUND_CANISTER_ID=$(dfx canister id backend)
  export PLAYGROUND_CANISTER_ID
  echo "PLAYGROUND_CANISTER_ID is $PLAYGROUND_CANISTER_ID"
  WEBSERVER_PORT=$(get_webserver_port)
  jq '.playground.bind="127.0.0.1:'"$WEBSERVER_PORT"'"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  jq '.playground.playground.playground_canister="'"$PLAYGROUND_CANISTER_ID"'"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  cd ..
  rm -rf hello
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
  assert_match "The principal you are using to call a management function is not part of the controllers."

  if [ "$(uname)" == "Darwin" ]; then
    sed -i '' 's/Hello/Goodbye/g' src/hello_backend/main.mo
  elif [ "$(uname)" == "Linux" ]; then
    sed -i 's/Hello/Goodbye/g' src/hello_backend/main.mo
  fi
  
  assert_command dfx deploy --playground
  assert_command dfx canister --playground call hello_backend greet '("player")'
  assert_match "Goodbye, player!"

  assert_command dfx canister --playground delete hello_backend
  assert_command_fail dfx canister --playground info hello_backend
  # canister is not actually deleted - the playground would have to do that
  assert_command dfx canister --playground info "$CANISTER"
}

@test "deploy fresh project to playground" {
  cd ..
  rm -rf hello
  dfx_new_frontend hello

  [[ "$USE_POCKETIC" ]] && assert_command dfx canister create --all --playground
  [[ "$USE_POCKETIC" ]] && assert_command dfx ledger fabricate-cycles --t 9999999 --canister hello_backend --playground
  [[ "$USE_POCKETIC" ]] && assert_command dfx ledger fabricate-cycles --t 9999999 --canister hello_frontend --playground

  assert_command dfx deploy --playground
  assert_command dfx canister --playground call hello_backend greet '("player")'
  assert_match "Hello, player!"
}

@test "Handle timeout correctly" {
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "Reserved canister 'hello_backend'"
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "hello_backend canister was already created"
  sleep 10
  jq '.playground.playground.timeout_seconds=5' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  assert_command dfx canister create hello_backend --playground -vv
  assert_match "Canister 'hello_backend' has timed out."
  assert_match "Reserved canister 'hello_backend'"
}

# This is important for whitelisting wasm hashes in the playground.
# If the hashes didn't match then the playground would attempt to
# instrument the asset canister during upload which would run into execution limits.
@test "playground-installed asset canister is same wasm as normal asset canister" {
  assert_command dfx deploy --playground
  PLAYGROUND_HASH=$(dfx canister --playground info hello_frontend | grep hash)
  echo "PLAYGROUND_HASH: ${PLAYGROUND_HASH}"
  assert_command dfx deploy
  assert_command bash -c 'dfx canister info hello_frontend | grep hash'
  assert_match "${PLAYGROUND_HASH}"
}

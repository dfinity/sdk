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
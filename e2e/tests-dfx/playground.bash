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

@test "specify playground in network" {
  create_networks_json
  jq '.playground.playground-cid="aaaaa-aa"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  # --playground aliases to --network playground, including all its settings
  assert_command dfx deploy hello_backend --playground
}
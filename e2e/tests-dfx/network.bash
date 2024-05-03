#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx identity new --storage-mode plaintext test_id
  dfx identity use test_id
  dfx_new
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "create with wallet stores canister ids for default-persistent networks in canister_ids.json" {
  dfx_start
  setup_actuallylocal_shared_network
  dfx_set_wallet
  dfx_set_wallet

  dfx canister create --all --network actuallylocal

  # canister creates writes to a spinner (stderr), not stdout
  assert_command dfx canister id e2e_project_backend --network actuallylocal
  assert_match "$(jq -r .e2e_project_backend.actuallylocal <canister_ids.json)"
}

@test "create with wallet stores canister ids for configured-ephemeral networks in canister_ids.json" {
  dfx_start

  setup_actuallylocal_shared_network
  jq '.actuallylocal.type="ephemeral"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  dfx_set_wallet

  dfx canister create --all --network actuallylocal

  # canister creates writes to a spinner (stderr), not stdout
  assert_command dfx canister id e2e_project_backend --network actuallylocal
  assert_match "$(jq -r .e2e_project_backend.actuallylocal .dfx/actuallylocal/canister_ids.json)"
}

@test "create stores canister ids for default-ephemeral local networks in .dfx/{network}canister_ids.json" {
  dfx_start

  assert_command dfx canister create --all --network local

  # canister creates writes to a spinner (stderr), not stdout
  assert_command dfx canister id e2e_project_backend --network local
  assert_match "$(jq -r .e2e_project_backend.local <.dfx/local/canister_ids.json)"
}

@test "create stores canister ids for configured-persistent local networks in canister_ids.json" {
  dfx_start

  webserver_port=$(get_webserver_port)

  create_networks_json

  jq '.local.bind="127.0.0.1:'"$webserver_port"'"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  jq '.local.type="persistent"' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"

  assert_command dfx canister create --all --network local

  # canister creates writes to a spinner (stderr), not stdout
  assert_command dfx canister id e2e_project_backend --network local
  assert_match "$(jq -r .e2e_project_backend.local <canister_ids.json)"
}

@test "failure message does not include network if for local network" {
  dfx_start
  assert_command_fail dfx build --network local
  assert_match "Cannot find canister id."
  assert_not_contains "--network local"
}

@test "failure message does include network if for non-local network" {
  dfx_start

  setup_actuallylocal_shared_network

  assert_command_fail dfx build --network actuallylocal
  assert_match "Cannot find canister id."
  assert_match "--network actuallylocal"
}

@test "network 'playground' has a default definition" {
  # if network is unknown dfx fails with `Network not found: <network name>`
  assert_command_fail dfx canister id hello_backend --network playground
  assert_contains "Cannot find canister id"
}

@test "equivalent: --network ic and --ic" {
  dfx_start
  dfx identity get-wallet

  assert_command_fail dfx diagnose --network ic
  assert_contains "The test_id identity is not stored securely."
  assert_contains "use it in mainnet-facing commands"
  assert_contains "No wallet found; nothing to do"

  assert_command_fail dfx diagnose --ic
  assert_contains "The test_id identity is not stored securely."
  assert_contains "use it in mainnet-facing commands"
  assert_contains "No wallet found; nothing to do"

  assert_command dfx diagnose
  assert_not_contains "identity is not stored securely"
  assert_eq "No problems found"
}

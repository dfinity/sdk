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

@test "id subcommand prints valid canister identifier" {
  install_asset id
  dfx_start
  dfx canister create --all
  dfx build
  assert_command dfx canister id e2e_project_backend
  assert_match "$(jq -r .e2e_project_backend.local < .dfx/local/canister_ids.json)"
}

@test "id subcommand does not display warning about plaintext keys" {
  install_asset id
  dfx identity get-principal
  echo "{}" | jq '.e2e_project_backend.ic = "bd3sg-teaaa-aaaaa-qaaba-cai"' >canister_ids.json
  assert_command dfx canister id e2e_project_backend --ic
  assert_eq "bd3sg-teaaa-aaaaa-qaaba-cai"
}

@test "id subcommand works from a subdirectory of the project - ephemeral id" {
  install_asset id
  dfx_start
  dfx canister create --all
  ID=$(dfx canister id e2e_project_backend)
  echo "canister id is $ID"

  (
    cd src
    dfx canister id e2e_project_backend
    assert_command dfx canister id e2e_project_backend
    assert_eq "$ID"
  )
}

@test "id subcommand works from a subdirectory of the project - persistent id" {
  install_asset id

  jq '.networks.local.type="persistent"' dfx.json | sponge dfx.json
  dfx_start
  dfx canister create --all
  ID=$(dfx canister id e2e_project_backend)
  echo "canister id is $ID"
  (
    cd src
    dfx canister id e2e_project_backend
    assert_command dfx canister id e2e_project_backend
    assert_eq "$ID"
  )
}

@test "id subcommand uses default network for remotes only" {
  install_asset id
  install_shared_asset subnet_type/shared_network_settings/application
  # Add a remote canister with a specific ID for one network and a different default for other networks.
  jq '.canisters.external_canister = {
  "build": "",
  "candid": "candid/external_canister.did",
  "remote": {
    "id": {
      "namedremote": "va76m-bqaaa-aaaaa-aaayq-cai",
      "__default": "rkp4c-7iaaa-aaaaa-aaaca-cai"
    }
  },
  "type": "custom",
  "wasm": ""
  }' dfx.json | sponge dfx.json
  # We need to define the networks we are going to use:
  jq '.namedremote= {"type": "persistent", "providers": ["http://namedremote"]}' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  jq '.somethingelse= {"type": "persistent", "providers": ["http://somethingelse"]}' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
  cat dfx.json
  cat "$E2E_NETWORKS_JSON"
  # Ok, start:
  dfx_start || true
  dfx canister create --all
  # The local dfx canister ID should not be affected:
  assert_command dfx canister id e2e_project_backend
  assert_match "$(jq -r .e2e_project_backend.local < .dfx/local/canister_ids.json)"
  # Named remotes should be unaffected:
  assert_command dfx canister --network namedremote id external_canister
  assert_match "va76m-bqaaa-aaaaa-aaayq-cai"
  # Other remotes should use the default entry:
  assert_command dfx canister --network somethingelse id external_canister
  assert_match "rkp4c-7iaaa-aaaaa-aaaca-cai"
}

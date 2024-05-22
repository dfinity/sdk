#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx_new_assets hello
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "url subcommand prints valid frontend canister urls on local" {
  dfx_start
  dfx canister create --all
  frontend_id=$(dfx canister id hello_frontend)
  
  assert_command dfx canister url hello_frontend
  assert_match "http://127.0.0.1.+${frontend_id}"
  assert_match "${frontend_id}.localhost"

  assert_command dfx canister url $frontend_id
  assert_match "http://127.0.0.1.+${frontend_id}"
  assert_match "${frontend_id}.localhost"
}

@test "url subcommand prints valid backend canister urls on local" {
  dfx_start
  dfx canister create --all
  jq '.__Candid_UI.local="br5f7-7uaaa-aaaaa-qaaca-cai"' .dfx/local/canister_ids.json | sponge .dfx/local/canister_ids.json

  backend_id=$(dfx canister id hello_backend)
  assert_command dfx canister url hello_backend
  assert_match "canisterId=br5f7-7uaaa-aaaaa-qaaca-cai&id=${backend_id}"

  backend_id=$(dfx canister id hello_backend)
  assert_command dfx canister url $backend_id
  assert_match "canisterId=br5f7-7uaaa-aaaaa-qaaca-cai&id=${backend_id}"
}

@test "url subcommand prints valid frontend canister urls from a subdirectory" {
  dfx_start
  dfx canister create --all
  frontend_id=$(dfx canister id hello_frontend)

  cd src
  assert_command dfx canister url hello_frontend
  assert_match "http://127.0.0.1.+${frontend_id}"
  assert_match "${frontend_id}.localhost"
}

@test "url subcommand prints valid frontend canister urls on mainnet" {
  dfx_start
  echo "{}" > canister_ids.json
  jq '.hello_frontend.ic = "qsgof-4qaaa-aaaan-qekqq-cai"' canister_ids.json | sponge canister_ids.json
  frontend_id=$(dfx canister id hello_frontend --ic)
  
  assert_command dfx canister url hello_frontend --ic
  assert_match "https://${frontend_id}.icp0.io"

  assert_command dfx canister url $frontend_id --ic
  assert_match "https://${frontend_id}.icp0.io"
}

@test "url subcommand prints valid backend canister urls on mainnet" {
  dfx_start
  echo "{}" > canister_ids.json
  jq '.hello_backend.ic = "qvhir-riaaa-aaaan-qekqa-cai"' canister_ids.json | sponge canister_ids.json
  backend_id=$(dfx canister id hello_backend --ic)

  assert_command dfx canister url hello_backend --ic
  assert_contains "https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=${backend_id}"

  assert_command dfx canister url $backend_id --ic
  assert_contains "https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=${backend_id}"
}

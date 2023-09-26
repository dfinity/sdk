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

@test "delete can be used to delete a canister" {
  dfx_start
  dfx deploy e2e_project_backend
  id=$(dfx canister id e2e_project_backend)
  dfx canister stop e2e_project_backend
  assert_command dfx canister delete e2e_project_backend
  assert_command_fail dfx canister info e2e_project_backend
  assert_contains "Cannot find canister id. Please issue 'dfx canister create e2e_project_backend'."
  assert_command_fail dfx canister status "$id"
  assert_contains "Canister $id not found"
}

@test "delete requires confirmation if the canister is not stopped" {
  dfx_start
  dfx deploy e2e_project_backend
  id=$(dfx canister id e2e_project_backend)
  assert_command_fail timeout -s9 20s dfx canister delete e2e_project_backend
  assert_command dfx canister info e2e_project_backend
  assert_command dfx canister delete e2e_project_backend -y
  assert_command_fail dfx canister info e2e_project_backend
  assert_contains "Cannot find canister id. Please issue 'dfx canister create e2e_project_backend'."
  assert_command_fail dfx canister status "$id"
  assert_contains "Canister $id not found"
}

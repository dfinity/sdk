#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx_new_assets
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "get canister status" {
  dfx_start
  assert_command dfx canister create e2e_project_frontend --no-wallet

  assert_command dfx canister status e2e_project_frontend
  assert_contains "Memory allocation: "
  assert_contains "Compute allocation: "
  assert_contains "Balance: "
}

dfx_canister_status_grep_memory_allocation() {
  dfx canister status e2e_project_frontend | grep "Memory allocation"
}

dfx_canister_status_grep_compute_allocation() {
  dfx canister status e2e_project_frontend | grep "Compute allocation"
}

dfx_canister_status_grep_balance() {
  dfx canister status e2e_project_frontend | grep "Balance"
}

@test "canister status output is grep compatible" {
  dfx_start
  assert_command dfx canister create e2e_project_frontend --no-wallet

  assert_command dfx_canister_status_grep_memory_allocation
  assert_contains "Memory allocation: "
  assert_not_contains "Compute allocation: "
  assert_not_contains "Balance: "

  assert_command dfx_canister_status_grep_compute_allocation
  assert_not_contains "Memory allocation: "
  assert_contains "Compute allocation: "
  assert_not_contains "Balance: "

  assert_command dfx_canister_status_grep_balance
  assert_not_contains "Memory allocation: "
  assert_not_contains "Compute allocation: "
  assert_contains "Balance: "
}

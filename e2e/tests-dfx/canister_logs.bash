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

@test "get canister logs" {
  install_asset logs
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install e2e_project
  dfx canister call e2e_project hello Alice
  dfx canister call e2e_project hello Bob
  sleep 2
  assert_command dfx canister logs e2e_project
  assert_contains "Hello, Alice!"
  assert_contains "Hello, Bob!"
}

dfx_canister_logs_grep_Alice() {
  dfx canister logs e2e_project | grep Alice
}

@test "canister logs output is grep compatible" {
  install_asset logs
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install e2e_project
  dfx canister call e2e_project hello Alice
  dfx canister call e2e_project hello Bob
  sleep 2
  assert_command dfx_canister_logs_grep_Alice
  assert_contains "Alice"
  assert_not_contains "Bob"
}

dfx_canister_logs_tail_n_1() {
  # Extra echo is necessary to simulate file input for tail.
  # shellcheck disable=SC2005
  echo "$(dfx canister logs e2e_project)" | tail -n 1
}

@test "canister logs output is tail compatible" {
  install_asset logs
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install e2e_project
  dfx canister call e2e_project hello Alice
  dfx canister call e2e_project hello Bob
  sleep 2
  assert_command dfx_canister_logs_tail_n_1
  assert_not_contains "Alice"
  assert_contains "Bob"
}

@test "canister logs only visible to allowed viewers." {
  install_asset logs
  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install e2e_project
  dfx canister call e2e_project hello Alice
  sleep 2

  assert_command dfx canister logs e2e_project
  assert_contains "Hello, Alice!"

  # Create identity for viewers.
  assert_command dfx identity new --storage-mode plaintext alice
  ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
  assert_command dfx canister update-settings --add-log-viewer="${ALICE_PRINCIPAL}" e2e_project
  assert_command dfx canister status e2e_project
  assert_contains "${ALICE_PRINCIPAL}"

  assert_command dfx canister logs e2e_project --identity alice
  assert_contains "Hello, Alice!"
}

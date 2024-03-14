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

@test "fetching canister logs" {
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

@test "fetching canister logs output is grep-able" {
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
  echo "$(dfx canister logs e2e_project)" | tail -n 1
}

@test "fetching canister logs output is tail-able" {
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

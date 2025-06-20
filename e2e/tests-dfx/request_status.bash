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

@test "request-status output raw" {
  install_asset greet
  dfx_start --artificial-delay 10000
  dfx canister create hello_backend
  dfx build hello_backend

  dfx canister install hello_backend

  assert_command dfx canister call --async hello_backend greet Bob

  # shellcheck disable=SC2154
  assert_command dfx canister request-status --output raw "$stdout" "$(dfx canister id hello_backend)"
  assert_eq '4449444c0001710b48656c6c6f2c20426f6221'

}

@test "request-status requires same identity" {
  install_asset greet
  dfx_start --artificial-delay 10000

  dfx canister create hello_backend
  dfx build hello_backend
  dfx canister install hello_backend

  assert_command dfx canister call --async hello_backend greet Bob

  # shellcheck disable=SC2154
  REQUEST_ID="$stdout"

  assert_command_fail dfx canister request-status "$REQUEST_ID" "$(dfx canister id hello_backend)" --identity anonymous
  assert_contains "The user tries to access Request ID not signed by the caller"

  assert_command dfx canister request-status "$REQUEST_ID" "$(dfx canister id hello_backend)"
  assert_eq '("Hello, Bob!")'
}

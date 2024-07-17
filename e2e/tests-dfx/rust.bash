#!/usr/bin/env bats
# shellcheck disable=SC2030,SC2031

load ../utils/_

setup() {
  standard_setup
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "rust starter project can build and call" {
  dfx_new_rust hello

  dfx_start
  dfx canister create --all
  assert_command dfx build hello_backend -vvv
  assert_match "Shrinking Wasm"
  assert_command dfx canister install hello_backend
  assert_command dfx canister call hello_backend greet dfinity
  assert_match '("Hello, dfinity!")'

  # dfx sets the candid:service metadata
  dfx canister metadata hello_backend candid:service >installed.did
  assert_command diff src/hello_backend/hello_backend.did installed.did
}

@test "rust canister can resolve dependencies" {
  dfx_new_rust rust_deps
  install_asset rust_deps
  dfx_start
  assert_command dfx deploy
  assert_command dfx canister call multiply_deps read
  assert_match '(1 : nat)'
  assert_command dfx canister call multiply_deps mul '(3)'
  assert_match '(9 : nat)'
  assert_command dfx canister call rust_deps read
  assert_match '(9 : nat)'
}

@test "rust canister can have nonstandard target dir location" {
  dfx_new_rust
  # We used to set CARGO_TARGET_DIR="$(echo -ne '\x81')"
  # But since rust 1.69, `cargo metadata` returns
  #   error: path contains invalid UTF-8 characters
  CARGO_TARGET_DIR="custom-target"
  export CARGO_TARGET_DIR
  dfx_start
  assert_command dfx deploy
  assert_command dfx canister call e2e_project_backend greet dfinity
}

@test "rust canister fails to build with missing lockfile" {
  dfx_new_rust
  rm -f ./Cargo.lock
  dfx_start
  assert_command_fail dfx deploy
  cargo update
  assert_command dfx deploy
}

@test "rust canisters support complex package layout" {
  dfx_new_rust rust_complex
  install_asset rust_complex
  dfx_start
  assert_command dfx deploy
  assert_command dfx canister call can1 id
  assert_contains '(1 : nat32)'
  assert_command dfx canister call can2 id
  assert_contains '(2 : nat32)'
  assert_command dfx canister call can3 id
  assert_contains '(3 : nat32)'
}

#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
}

teardown() {
  standard_teardown
}

@test "remote generate-binding succeeds for --all" {
  install_asset remote_generate_binding/basic

  assert_command dfx remote generate-binding --all

  assert_file_exists remote.mo
  assert_file_exists remote.rs
  assert_file_exists remote.ts
  assert_file_exists remote.js
}

@test "remote generate-binding --overwrite succeeds for --all" {
  install_asset remote_generate_binding/basic
  echo "to_overwrite" > remote.mo
  echo "to_overwrite" > remote.rs

  assert_command dfx remote generate-binding --overwrite --all

  # should generate if not present
  assert_file_exists remote.js
  assert_file_exists remote.ts

  # should overwrite if already present
  assert_neq "to_overwrite" "$(cat remote.mo)"
  assert_neq "to_overwrite" "$(cat remote.rs)"
}

@test "remote generate-binding does not overwrite if file already present" {
  install_asset remote_generate_binding/basic
  echo "to_overwrite" > remote.mo
  echo "to_overwrite" > remote.rs

  assert_command dfx remote generate-binding --all

  # should generate if not present
  assert_file_exists remote.ts
  assert_file_exists remote.js

  # should not overwrite if already present
  assert_eq "to_overwrite" "$(cat remote.mo)"
  assert_eq "to_overwrite" "$(cat remote.rs)"
}

@test "remote generate-binding succeeds for specific rust canister" {
  install_asset remote_generate_binding/basic

  assert_command dfx remote generate-binding remote-rust

  assert_file_exists remote.rs
  assert_file_not_exists remote.mo
  assert_file_not_exists remote.ts
  assert_file_not_exists remote.js
}

@test "remote generate-binding succeeds for specific motoko canister" {
  install_asset remote_generate_binding/basic

  assert_command dfx remote generate-binding remote-motoko

  assert_file_exists remote.mo
  assert_file_not_exists remote.rs
  assert_file_not_exists remote.ts
  assert_file_not_exists remote.js
}

@test "remote generate-binding succeeds for specific javascript canister" {
  install_asset remote_generate_binding/basic

  assert_command dfx remote generate-binding remote-javascript

  assert_file_exists remote.js
  assert_file_not_exists remote.mo
  assert_file_not_exists remote.rs
  assert_file_not_exists remote.ts
}

@test "remote generate-binding succeeds for specific typescript canister" {
  install_asset remote_generate_binding/basic

  assert_command dfx remote generate-binding remote-typescript

  assert_file_exists remote.ts
  assert_file_not_exists remote.mo
  assert_file_not_exists remote.rs
  assert_file_not_exists remote.js
}

@test "remote generate-binding --overwrite succeeds for specific canister" {
  install_asset remote_generate_binding/basic
  echo "to_overwrite" > remote.mo

  # should not overwrite without --overwrite
  assert_command dfx remote generate-binding remote-motoko
  assert_match 'already exists'
  assert_eq "to_overwrite" "$(cat remote.mo)"

  # should overwrite with --overwrite
  assert_command dfx remote generate-binding --overwrite remote-motoko
  assert_neq "to_overwrite" "$(cat remote.mo)"
}

@test "remote generate-binding incomplete command rejected" {
  assert_command_fail dfx remote generate-binding
}

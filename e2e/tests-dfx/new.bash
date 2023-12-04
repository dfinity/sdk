#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
}

teardown() {
  standard_teardown
}

@test "dfx new - good names" {
  dfx new --no-frontend a_good_name_
  dfx new --no-frontend A
  dfx new --no-frontend b
  dfx new --no-frontend a_
  dfx new --no-frontend a_1
  dfx new --no-frontend a1
  dfx new --no-frontend a1a
  dfx new --no-frontend a-b-c
}

@test "dfx new - bad names" {
  assert_command_fail dfx new _a_good_name_
  assert_command_fail dfx new __also_good
  assert_command_fail dfx new _1
  assert_command_fail dfx new _a
  assert_command_fail dfx new 1
  assert_command_fail dfx new 1_
  assert_command_fail dfx new -
  assert_command_fail dfx new _
  assert_command_fail dfx new '🕹'
  assert_command_fail dfx new '不好'
  assert_command_fail dfx new 'a:b'
}

@test "dfx new readmes contain appropriate links" {
  assert_command dfx new --type rust e2e_rust --no-frontend
  assert_command grep "https://docs.rs/ic-cdk" e2e_rust/README.md
  assert_command dfx new --type motoko e2e_motoko --no-frontend
  assert_command grep "https://internetcomputer.org/docs/current/motoko/main/language-manual" e2e_motoko/README.md
}

@test "dfx new emits projects of the correct type" {
  assert_command dfx new --type rust e2e_rust --no-frontend
  assert_command jq -r '.canisters.e2e_rust_backend.type' e2e_rust/dfx.json
  assert_eq "rust"
  assert_command dfx new --type motoko e2e_motoko --no-frontend
  assert_command jq -r '.canisters.e2e_motoko_backend.type' e2e_motoko/dfx.json
  assert_eq "motoko"
}

@test "dfx new always emits sample-asset.txt" {
  assert_command dfx new e2e_frontend --frontend
  assert_file_exists e2e_frontend/src/e2e_frontend_frontend/assets/sample-asset.txt
  assert_command dfx new e2e_no_frontend --no-frontend
  assert_file_exists e2e_no_frontend/src/e2e_no_frontend_frontend/assets/sample-asset.txt
}

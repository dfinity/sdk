#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
}

teardown() {
  dfx_stop
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
  assert_command_fail dfx new a-b-c
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

@test "frontend templates apply successfully" {
  for frontend in sveltekit vue react vanilla simple-assets none; do
    assert_command dfx new e2e_${frontend/-/_} --frontend $frontend
  done
  assert_file_not_exists e2e_none/src/e2e_none_frontend
}

@test "frontend templates pass the frontend tests" {
  dfx_start
  for frontend in sveltekit vue react vanilla; do
    assert_command dfx new e2e_$frontend --frontend $frontend --extras frontend-tests
    pushd e2e_$frontend
    assert_command dfx deploy
    assert_command npm test --workspaces
    popd
  done
}

@test "backend templates" {
  for backend in motoko rust kybra azle; do
    assert_command dfx new e2e_$backend --type $backend --no-frontend
  done
}

@test "interactive template selection" {
  assert_command "${BATS_TEST_DIRNAME}/../assets/expect_scripts/rust_svelte_with_tests_and_ii.exp"
  assert_file_exists e2e_project/Cargo.toml
  assert_file_exists e2e_project/src/e2e_project_frontend/src/routes/+page.svelte
  assert_file_exists e2e_project/src/e2e_project_frontend/src/setupTests.js
  assert_command jq .canisters.internet_identity e2e_project/dfx.json
}

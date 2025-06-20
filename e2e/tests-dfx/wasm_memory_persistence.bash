#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx_new test
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "migrate Motoko from classical persistence to classical persistence" {
  install_asset wasm_memory_persistence
  dfx_start
  dfx deploy
  dfx deploy --upgrade-unchanged
  assert_command dfx canister call test getVersion '()'
  assert_match "(2 : nat)"
}

@test "migrate Motoko from classical persistence to enhanced orthogonal persistence" {
  install_asset wasm_memory_persistence
  dfx_start
  dfx deploy
  jq '.canisters.test.wasm="enhanced-actor.wasm"' dfx.json | sponge dfx.json
  dfx deploy
  assert_command dfx canister call test getVersion '()'
  assert_match "(2 : nat)"
}

@test "migrate Motoko from enhanced orthogonal persistence to enhanced orthogonal persistence" {
  install_asset wasm_memory_persistence
  dfx_start
  jq '.canisters.test.wasm="enhanced-actor.wasm"' dfx.json | sponge dfx.json
  dfx deploy
  jq '.canisters.test.wasm="enhanced-actor.wasm"' dfx.json | sponge dfx.json
  dfx deploy --upgrade-unchanged
  assert_command dfx canister call test getVersion '()'
  assert_match "(2 : nat)"
}

@test "failing Motoko downgrade from enhanced orthogonal persistence to classical persistence" {
  install_asset wasm_memory_persistence
  dfx_start
  jq '.canisters.test.wasm="enhanced-actor.wasm"' dfx.json | sponge dfx.json
  dfx deploy
  jq '.canisters.test.wasm="classical-actor.wasm"' dfx.json | sponge dfx.json
  assert_command_fail dfx deploy
  assert_match "The \`wasm_memory_persistence: opt Keep\` upgrade option requires that the new canister module supports enhanced orthogonal persistence."
}

@test "re-install Motoko enhanced orthogonal persistence with classical persistence" {
  install_asset wasm_memory_persistence
  dfx_start
  jq '.canisters.test.wasm="enhanced-actor.wasm"' dfx.json | sponge dfx.json
  dfx deploy
  jq '.canisters.test.wasm="classical-actor.wasm"' dfx.json | sponge dfx.json
  echo yes | dfx canister install test --mode=reinstall
  assert_command dfx canister call test getVersion '()'
  assert_match "(1 : nat)"
}

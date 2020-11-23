#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1

    dfx_new
}

teardown() {
    dfx_stop
}

@test "build fails on invalid motoko" {
    install_asset invalid
    dfx_start
    dfx canister create --all
    assert_command_fail dfx build
    assert_match "syntax error"
}

@test "build supports relative imports" {
    install_asset import
    dfx_start
    dfx canister create --all
    assert_command dfx build
    dfx canister install --all
    assert_command dfx canister call e2e_project greet World
    assert_match "10World"
}

@test "build succeeds on default project" {
    dfx_start
    dfx canister create --all
    assert_command dfx build
}

# TODO: Before Tungsten, we need to update this test for code with inter-canister calls.
# Currently due to new canister ids, the wasm binary will be different for inter-canister calls.
@test "build twice produces the same wasm binary" {
  dfx_start
  dfx canister create --all
  assert_command dfx build
  cp .dfx/local/canisters/e2e_project/e2e_project.wasm ./old.wasm
  assert_command dfx build
  assert_command diff .dfx/local/canisters/e2e_project/e2e_project.wasm ./old.wasm
}

@test "build outputs warning" {
    install_asset warning
    dfx_start
    dfx canister create --all
    assert_command dfx build
    assert_match "warning, this pattern of type"
}

@test "build fails on unknown imports" {
    install_asset import_error
    dfx_start
    dfx canister create --all
    assert_command_fail dfx build
    assert_match 'import error, canister alias "random" not defined'
}

@test "build fails if canister type is not supported" {
  dfx_start
  dfx config canisters.e2e_project.type unknown_canister_type
  dfx canister create --all
  assert_command_fail dfx build
  assert_match "Cannot find builder for canister"
}

@test "can build a custom canister type" {
  install_asset custom_canister
  dfx_start
  dfx canister create --all
  assert_command dfx build
  assert_match "CUSTOM_CANISTER_BUILD_DONE"

  dfx canister install --all
  assert_command dfx canister call custom fromQuery
}

@test "build succeeds with network parameter" {
    dfx_start
    dfx canister --network local create --all
    assert_command dfx build --network local
}

@test "build succeeds when requested network is configured" {
    dfx_start

    webserver_port=$(cat .dfx/webserver-port)
    assert_command dfx config networks.ic.providers '[ "http://127.0.0.1:'$webserver_port'" ]'
    assert_command dfx canister --network ic create --all
    assert_command dfx build --network ic
}

@test "build output for local network is in expected directory" {
  dfx_start
  dfx canister create --all
  assert_command dfx build
  assert_command ls .dfx/local/canisters/e2e_project/
  assert_command ls .dfx/local/canisters/e2e_project/e2e_project.wasm
}

@test "build output for non-local network is in expected directory" {
  dfx_start
  webserver_port=$(cat .dfx/webserver-port)
  assert_command dfx config networks.ic.providers '[ "http://127.0.0.1:'$webserver_port'" ]'
  dfx canister --network ic create --all
  assert_command dfx build --network ic
  assert_command ls .dfx/ic/canisters/e2e_project/
  assert_command ls .dfx/ic/canisters/e2e_project/e2e_project.wasm
}


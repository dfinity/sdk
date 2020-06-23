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
    assert_command_fail dfx build
    assert_match "syntax error"
}

@test "build supports relative imports" {
    install_asset import
    dfx_start
    assert_command dfx build
    dfx canister install --all
    assert_command dfx canister call e2e_project greet World
    assert_match "10World"
}

@test "build succeeds on default project" {
    dfx_start
    assert_command dfx build
}

# TODO: Before Tungsten, we need to update this test for code with inter-canister calls.
# Currently due to new canister ids, the wasm binary will be different for inter-canister calls.
@test "build twice produces the same wasm binary" {
  dfx_start
  assert_command dfx build
  cp canisters/e2e_project/main.wasm ./old.wasm
  assert_command dfx build
  assert_command diff canisters/e2e_project/main.wasm ./old.wasm
}

@test "build outputs the canister manifest" {
    dfx_start
    assert_command dfx build
    [[ -f canisters/canister_manifest.json ]]
}

@test "build outputs warning" {
    install_asset warning
    dfx_start
    assert_command dfx build
    assert_match "warning, this pattern consuming type"
}

@test "build fails on unknown imports" {
    install_asset import_error
    dfx_start
    assert_command_fail dfx build
    assert_match 'import error, canister alias "random" not defined'
}

@test "build fails if canister type is not supported" {
  dfx_start
  dfx config canisters.e2e_project.type unknown_canister_type
  assert_command_fail dfx build
  assert_match "CouldNotFindBuilderForCanister"
}

@test "can build a custom canister type" {
  dfx_start
  install_asset custom_canister
  assert_command dfx build
  assert_match "CUSTOM_CANISTER_BUILD_DONE"

  dfx canister install --all
  assert_command dfx canister call custom hashFromQuery
}

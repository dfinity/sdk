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
    install_asset invalid_mo
    assert_command_fail dfx build
    assert_match "syntax error"
}

@test "build supports relative imports" {
    install_asset import_mo
    assert_command dfx build
    dfx_start
    dfx canister install --all
    assert_command dfx canister call e2e_project greet --type=string World
    assert_match "10World"
}

@test "build succeeds on default project" {
    assert_command dfx build
}

# TODO: Before Tungsten, we need to update this test for code with inter-canister calls.
# Currently due to new canister ids, the wasm binary will be different for inter-canister calls.
@test "build twice produces the same wasm binary" {
  assert_command dfx build
  cp canisters/e2e_project/main.wasm ./old.wasm
  assert_command dfx build
  assert_command diff canisters/e2e_project/main.wasm ./old.wasm
}

@test "build outputs the canister ID" {
    assert_command dfx build
    [[ -f canisters/e2e_project/_canister.id ]]
}

@test "build outputs warning" {
    install_asset warning_mo
    assert_command dfx build
    assert_match "warning, this pattern consuming type"
}

@test "build fails on unknown imports" {
    install_asset import_error_mo
    assert_command_fail dfx build
    assert_match "Cannot find canister random"
}
